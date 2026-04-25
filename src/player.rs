use bevy::prelude::*;
use bevy::window::PrimaryWindow;

// This function spawns the player
pub fn spawn_player(mut commands: Commands) {
    commands.spawn((
        Sprite {
            color: Color::srgb(0.9, 0.3, 0.3),
            custom_size: Some(Vec2::new(50.0, 50.0)),
            ..default()
        },
        Transform::from_xyz(0.0, 0.0, 0.0),
    ));
}

// This handles the movement and borders
pub fn move_player(
    keyboard_input: Res<ButtonInput<KeyCode>>,
    window_query: Query<&Window, With<PrimaryWindow>>,
    mut query: Query<&mut Transform, With<Sprite>>,
) {
    let window = window_query.single();
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
            let x_bound = (window_width / 2.0) - 25.0;
            let y_bound = (window_height / 2.0) - 25.0;

            let new_pos = transform.translation + direction * 7.0;
            transform.translation.x = new_pos.x.clamp(-x_bound, x_bound);
            transform.translation.y = new_pos.y.clamp(-y_bound, y_bound);
        }
    }
}