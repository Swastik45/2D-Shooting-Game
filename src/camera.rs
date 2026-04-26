use bevy::prelude::*;
use crate::player::Player;

pub fn spawn_camera(mut commands: Commands) {
    commands.spawn(Camera2d);
}

pub fn camera_follow(
    player_query: Query<&Transform, With<Player>>,
    mut camera_query: Query<&mut Transform, (With<Camera2d>, Without<Player>)>,
) {
    let Ok(player_transform) = player_query.single() else { return; };
    let Ok(mut camera_transform) = camera_query.single_mut() else { return; };

    let target = player_transform.translation;
    let current = camera_transform.translation;

    camera_transform.translation.x += (target.x - current.x) * 0.1;
    camera_transform.translation.y += (target.y - current.y) * 0.1;
}