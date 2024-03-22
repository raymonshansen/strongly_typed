use bevy::prelude::*;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_systems(Startup, setup)
        .add_systems(Update, bevy::window::close_on_esc)
        .run();
}

fn setup(mut commands: Commands, asset_server: Res<AssetServer>) {
    let font = asset_server.load("fonts/MesloLGLNerdFont-Regular.ttf");
    let text_style = TextStyle {
        font: font.clone(),
        font_size: 60.0,
        color: Color::WHITE,
    };

    commands.spawn(Camera2dBundle::default());

    commands.spawn(Text2dBundle {
        text: Text {
            sections: vec![TextSection::new(
                format!("Hello, World!"),
                text_style,
            )],
            ..Default::default()
        },
        transform: Transform::from_translation(250. * Vec3::Y),
        ..default()
    });
}