use bevy::prelude::*;
use rand::seq::SliceRandom; // 0.7.2

#[derive(Component)]
struct FallingWord;

fn get_random_word() -> String {
  let vs = vec!["abc", "def", "ghi", "jkl", "mno"];
  let word = vs.choose(&mut rand::thread_rng());
  let var_name = match word {
    Some(w) => w,
    None => "default",
  };
  return format!("{}", var_name);
}

fn main() {
  App::new()
    .add_plugins(DefaultPlugins)
    .add_systems(Startup, spawn_text)
    .add_systems(
      Update,
      (bevy::window::close_on_esc, update_position, listen_received_character_events),
    )
    .run();
}

fn spawn_text(mut commands: Commands, asset_server: Res<AssetServer>) {
  let font = asset_server.load("fonts/MesloLGLNerdFont-Regular.ttf");
  let text_style = TextStyle {
    font: font.clone(),
    font_size: 60.0,
    color: Color::WHITE,
  };

  commands.spawn(Camera2dBundle::default());

  commands.spawn((
    Text2dBundle {
      text: Text {
        sections: vec![TextSection::new(
          get_random_word(),
          text_style,
        )],
        ..default()
      },
      transform: Transform::from_translation(200. * Vec3::Y),
      ..default()
    },
    FallingWord,
  ));
}

fn update_position(time: Res<Time>, mut query: Query<&mut Transform, With<FallingWord>>) {
  for mut pos in query.iter_mut() {
    pos.translation.y = 200.0 * time.elapsed_seconds().sin() - 30.0;
  }
}

fn listen_received_character_events(
  mut events: EventReader<ReceivedCharacter>,
  mut edit_text: Query<&mut Text>,
) {
  for event in events.read() {
    edit_text.single_mut().sections[0]
      .value
      .push_str(&event.char);
  }
}
