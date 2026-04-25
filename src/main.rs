use bevy::prelude::*;
use bevy::window::PrimaryWindow; // Added this import for clarity

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_systems(Startup, setup)
        .add_systems(Update, move_player)
        .run();
}

fn setup(mut commands: Commands) {
    // 0.15 Standard: Spawn Camera and Player
    commands.spawn(Camera2d);

    commands.spawn((
        Sprite {
            color: Color::srgb(0.9, 0.3, 0.3),
            custom_size: Some(Vec2::new(50.0, 50.0)),
            ..default()
        },
        Transform::from_xyz(0.0, 0.0, 0.0),
    ));
}

fn move_player(
    keyboard_input: Res<ButtonInput<KeyCode>>,
    // Querying for the PrimaryWindow specifically
    window_query: Query<&Window, With<PrimaryWindow>>,
    mut query: Query<&mut Transform, With<Sprite>>,
) {
    // 1. Get the window safely. 
    // If it's not found, we just return and try again next frame.
    let Ok(window) = window_query.single() else {
        return;
    };

    // 2. Now 'window' is the actual Window object, so .width() works!
    let window_width = window.width();
    let window_height = window.height();

    for mut transform in &mut query {
        let mut direction = Vec3::ZERO;

        if keyboard_input.pressed(KeyCode::KeyW) { direction.y += 1.0; }
        if keyboard_input.pressed(KeyCode::KeyS) { direction.y -= 1.0; }
        if keyboard_input.pressed(KeyCode::KeyA) { direction.x -= 1.0; }
        if keyboard_input.pressed(KeyCode::KeyD) { direction.x += 1.0; }

        if direction != Vec3::ZERO {
            direction = direction.normalize();
            let move_speed = 7.0;
            let new_position = transform.translation + direction * move_speed;

            // --- THE BORDER CHECK (0,0 is center) ---
            let x_bound = (window_width / 2.0) - 25.0;
            let y_bound = (window_height / 2.0) - 25.0;

            // Clamping forces the value to stay within the window edges
            transform.translation.x = new_position.x.clamp(-x_bound, x_bound);
            transform.translation.y = new_position.y.clamp(-y_bound, y_bound);
        }
    }
}