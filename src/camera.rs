use bevy::prelude::*;
use bevy::window::PrimaryWindow;
use crate::player::Player;
use crate::world::{TILE_SIZE, MAP_W, MAP_H};

pub fn spawn_camera(mut commands: Commands) {
    commands.spawn((
        Camera2d {
            ..default()
        },
        Transform::from_xyz(0.0, 0.0, 1000.0), // start centered on map
        GlobalTransform::default(),
    ));
}

pub fn camera_follow(
    player_query: Query<&Transform, With<Player>>,
    mut camera_query: Query<&mut Transform, (With<Camera2d>, Without<Player>)>,
    windows: Query<&Window, With<PrimaryWindow>>,
) {
    let Ok(player_transform) = player_query.single() else { return; };
    let Ok(mut camera_transform) = camera_query.single_mut() else { return; };
    let Ok(window) = windows.single() else { return; };

    let player_pos = player_transform.translation;

    // Snap camera target to tile grid
    let tile_x = (player_pos.x / TILE_SIZE).round() * TILE_SIZE;
    let tile_y = (player_pos.y / TILE_SIZE).round() * TILE_SIZE;

    // Clamp so camera never reveals black outside the map
    let half_win_x = window.width() / 2.0;
    let half_win_y = window.height() / 2.0;
    let map_half_x = (MAP_W as f32 * TILE_SIZE) / 2.0;
    let map_half_y = (MAP_H as f32 * TILE_SIZE) / 2.0;

    let clamped_x = tile_x.clamp(-map_half_x + half_win_x, map_half_x - half_win_x);
    let clamped_y = tile_y.clamp(-map_half_y + half_win_y, map_half_y - half_win_y);

    // Slight upward offset for taller sprite
    let target = Vec3::new(clamped_x, clamped_y + 30.0, camera_transform.translation.z);

    // Smooth follow
    camera_transform.translation = camera_transform.translation.lerp(target, 0.12);
}
