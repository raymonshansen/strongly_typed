use std::usize;

use bevy::{
  input::{ keyboard::{Key, KeyboardInput}, ButtonState },
  color::palettes::css::*,
  prelude::*
};
use rand::{distributions::Uniform, prelude::Distribution, seq::SliceRandom}; // 0.7.2


#[derive(Resource, Deref, DerefMut)]
struct PlayerLevel(usize);
#[derive(Component)]
struct LevelText;

#[derive(Component)]
struct FallingWord;

#[derive(Component)]
struct ScoreText;

#[derive(Resource, Deref, DerefMut)]
struct Score(usize);

#[derive(Component, Deref)]
struct FloatingAway(Vec2);

#[derive(Event)]
struct WordCompletedSuccesfully(Text);

fn get_random_word(level: usize) -> String {
  let words = vec![
    ["ape", "sko", "ball", "tog", "bil", "snø", "hus", "ake", "dag", "sol"],
    ["kake", "hest", "fisk", "gris", "vann", "bekk", "buss", "vott", "måke", "slott"]];
  let word = words[level - 1].choose(&mut rand::thread_rng());
  match word {
    Some(w) => w,
    None => "default",
  }
  .to_owned()
}

fn get_random_float() -> f32 {
  let rng = &mut rand::thread_rng();
  Uniform::from(-10f32..10f32).sample(rng)
}

fn main() {
  App::new()
    .add_plugins(DefaultPlugins)
    .insert_resource(PlayerLevel(1))
    .insert_resource(Score(0))
    .add_systems(Startup, setup)
    .add_event::<WordCompletedSuccesfully>()
    .add_systems(
      Update,
      (
        close_on_esc,
        update_position,
        float_away,
        listen_received_character_events,
        update_score,
      ),
    )
    .run();
}

fn create_info_ui(window: &Query<&Window>, commands: &mut Commands, asset_server: &Res<AssetServer>) {
  let win_width = window.single().size().x;
  let win_height = window.single().size().y;
  let score_x = win_width / -2. + win_width * 0.05;
  let score_y = win_height / 2. - 15.;
  let level_x = win_width / 2. - (win_width * 0.05);
  let level_y = win_height / 2. - 15.;
  let font = asset_server.load("fonts/MesloLGLNerdFont-Regular.ttf");
  let text_style = TextStyle {
    font: font.clone(),
    font_size: 30.0,
    color: bevy::prelude::Color::Srgba(MINT_CREAM),
  };
  commands.spawn((
    Text2dBundle {
      text: Text::from_sections([
        TextSection::new("Level: ", text_style.clone()),
        TextSection::new("1", text_style.clone())
        ]).with_justify(JustifyText::Left),
      transform: Transform::from_xyz(level_x, level_y, 0.0),
      ..default()
    },
    LevelText,
  ));
  commands.spawn((
    Text2dBundle {
      text: Text::from_sections([
        TextSection::new("Score: ", text_style.clone()),
        TextSection::new("0", text_style.clone())
        ]).with_justify(JustifyText::Left),
      transform: Transform::from_xyz(score_x, score_y, 0.0),
      ..default()
    },
    ScoreText));
}

fn create_new_word(window: &Query<&Window>, commands: &mut Commands, asset_server: &Res<AssetServer>, level: &Res<PlayerLevel>) {
  let font = asset_server.load("fonts/MesloLGLNerdFont-Regular.ttf");
  let word = get_random_word(***level);
  let top = window.single().size().y / 2.0;
  let green = TextStyle { font: font.clone(), font_size: 60.0, color: bevy::prelude::Color::Srgba(LIMEGREEN) };
  let white = TextStyle { font: font.clone(), font_size: 60.0, color: bevy::prelude::Color::Srgba(MINT_CREAM).with_alpha(1.0) };
  commands.spawn((
    Text2dBundle {
      text: Text::from_sections([TextSection::new("", green), TextSection::new(word.clone(), white)]),
      transform: Transform::from_translation(Vec3::ZERO.with_y(top)),
      ..default()
    },
    FallingWord
  ));
}

fn setup(window: Query<&Window>, mut commands: Commands, asset_server: Res<AssetServer>, level: Res<PlayerLevel>) {
  commands.spawn(Camera2dBundle::default());
  create_new_word(&window, &mut commands, &asset_server, &level);
  create_info_ui(&window, &mut commands, &asset_server);
}

fn update_position(time: Res<Time>, mut query: Query<&mut Transform, With<FallingWord>>) {
  for mut pos in query.iter_mut() {
    pos.translation.y -= 30. * time.delta_seconds()
  }
}

fn is_out_of_bounds(window: &Query<&Window>, pos: &Vec3) -> bool {
  let size = window.single().size();
  let right_bound = size.x / 2 as f32;
  let left_bound = -right_bound;
  let lower_bound = size.y / 2 as f32;
  let upper_bound = -lower_bound;

  return left_bound <= pos.x && pos.x <= right_bound && upper_bound <= pos.y && pos.y <= lower_bound;
}

fn float_away(
  mut commands: Commands,
  time: Res<Time>,
  window: Query<&Window>,
  mut query: Query<(Entity, &mut Transform, &mut Text, &FloatingAway), With<FloatingAway>>,
) {

  for entity in query.iter_mut() {
    let (entity, mut transform, mut text, float_direction) = entity;
    let pos = &mut transform.translation;
    let current_alpha = text.sections[0].style.color.alpha();
    if !is_out_of_bounds(&window, pos) || current_alpha > 0.0 {
      pos.x += float_direction.x * time.delta_seconds();
      pos.y += float_direction.y * time.delta_seconds();

      // Loose 3% oppasity every frame
      let new_alpha = current_alpha * 0.97;
      text.sections[0].style.color.set_alpha(new_alpha);
    } else {
      commands.entity(entity).despawn();
    }
  }
}

fn listen_received_character_events(
  mut commands: Commands,
  mut event_reader: EventReader<KeyboardInput>,
  mut event_writer: EventWriter<WordCompletedSuccesfully>,
  mut edit_text: Query<(Entity, &mut Text, &mut Transform), (With<FallingWord>, Without<FloatingAway>)>,
  window: Query<&Window>,
  asset_server: Res<AssetServer>,
  level: Res<PlayerLevel>
) {
  let (entity, mut edit_text, mut pos) = edit_text.single_mut();

  for event in event_reader.read() {
    if event.state == ButtonState::Released {
      continue;
    }

    let char = match &event.logical_key {
      Key::Character(c) => c,
      _ => continue,
    };

    if edit_text.sections[1].value.starts_with(char.as_str()) {
      let char = edit_text.sections[1].value.remove(0);
      edit_text.sections[0].value.push(char);
      pos.translation.y += 50.;
    }

    // If word is completed
    if edit_text.sections[1].value.is_empty() {
      commands.entity(entity).remove::<FallingWord>();
      let vec2 = Vec2::new(get_random_float(), get_random_float()).normalize() * 50.;
      commands.entity(entity).insert(FloatingAway(vec2));
      let immutable_text = &*edit_text;
      event_writer.send(WordCompletedSuccesfully(immutable_text.clone()));
      create_new_word(&window, &mut commands, &asset_server, &level);
    }
  }
}

fn update_score(mut score: ResMut<Score>, mut ev_word_completed: EventReader<WordCompletedSuccesfully>, mut query: Query<&mut Text, With<ScoreText>>){
  for completed_word in ev_word_completed.read(){
    let points = completed_word.0.sections[0].value.clone().len();
    **score += points * 10;
    let mut score_text = query.single_mut();
    score_text.sections[1].value = score.to_string();
  }
}

fn close_on_esc(mut commands: Commands, focused_windows: Query<(Entity, &Window)>, input: Res<ButtonInput<KeyCode>>,) {
  for (window, focus) in focused_windows.iter() {
    if !focus.focused {
      continue;
    }

    if input.just_pressed(KeyCode::Escape) {
      commands.entity(window).despawn();
    }
  }
}
