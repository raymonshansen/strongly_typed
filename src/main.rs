use bevy::{
  input::{
    keyboard::{Key, KeyboardInput},
    ButtonState,
  },
  prelude::*,
};
use rand::{distributions::Uniform, prelude::Distribution, seq::SliceRandom}; // 0.7.2

#[derive(Component)]
struct FallingWord;

#[derive(Component, Deref)]
struct FloatingAway(Vec2);

fn get_random_word() -> String {
  let vs = vec!["abc", "def", "ghi", "jkl", "mno"];
  let word = vs.choose(&mut rand::thread_rng());
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
    .add_systems(Startup, spawn_text)
    .add_systems(
      Update,
      (
        close_on_esc,
        update_position,
        float_away,
        listen_received_character_events,
      ),
    )
    .run();
}

fn create_new_word(window: &Query<&Window>, commands: &mut Commands, asset_server: &Res<AssetServer>) {
  let font = asset_server.load("fonts/MesloLGLNerdFont-Regular.ttf");

  let green_text = TextStyle {
    font: font.clone(),
    font_size: 60.0,
    color: Color::oklch(0.7, 0.141, 140.82).with_alpha(1.0),
  };

  let white_text = TextStyle {
    font: font.clone(),
    font_size: 60.0,
    color: Color::WHITE.with_alpha(1.0),
  };

  let word = get_random_word();

  let top = window.single().size().y / 2.0;
  let start_pos = Transform::from_translation(Vec3::ZERO.with_y(top));

  commands.spawn((
    Text2dBundle {
      text: Text {
        sections: vec![
          TextSection::new("", green_text),
          TextSection::new(word.clone(), white_text),
        ],
        ..default()
      },
      transform: start_pos,
      ..default()
    },
    FallingWord,
  ));
}

fn spawn_text(window: Query<&Window>, mut commands: Commands, asset_server: Res<AssetServer>) {
  commands.spawn(Camera2dBundle::default());

  create_new_word(&window, &mut commands, &asset_server);
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
      //text.sections[1].style.color.set_alpha(new_alpha);
    } else {
      commands.entity(entity).despawn();
    }
  }
}

fn listen_received_character_events(
  mut commands: Commands,
  mut events: EventReader<KeyboardInput>,
  mut edit_text: Query<
    (Entity, &mut Text, &mut Transform),
    (With<FallingWord>, Without<FloatingAway>),
  >,
  asset_server: Res<AssetServer>,
  window: Query<&Window>
) {
  let (entity, mut edit_text, mut pos) = edit_text.single_mut();

  for event in events.read() {
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

    if edit_text.sections[1].value.is_empty() {
      commands.entity(entity).remove::<FallingWord>();
      let vec2 = Vec2::new(get_random_float(), get_random_float()).normalize() * 50.;
      commands.entity(entity).insert(FloatingAway(vec2));
      create_new_word(&window, &mut commands, &asset_server);
    }
  }
}

fn close_on_esc(
  mut commands: Commands,
  focused_windows: Query<(Entity, &Window)>,
  input: Res<ButtonInput<KeyCode>>,
) {
  for (window, focus) in focused_windows.iter() {
    if !focus.focused {
      continue;
    }

    if input.just_pressed(KeyCode::Escape) {
      commands.entity(window).despawn();
    }
  }
}
