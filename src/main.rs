use bevy::prelude::*;

mod world;
mod player;
mod camera;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins.set(ImagePlugin::default_nearest()))
        .add_systems(Startup, (
            camera::spawn_camera,
            world::spawn_world,
            player::spawn_player,
        ))
        .add_systems(Update, (
            player::move_player,
            player::animate_player,
            camera::camera_follow,
        ))
        .run();
}