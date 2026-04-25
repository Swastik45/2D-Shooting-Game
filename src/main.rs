use bevy::prelude::*;

// Tell Rust to look for the player.rs file
mod player; 

fn main() {
    App::new()
        .add_plugins(DefaultPlugins.set(ImagePlugin::default_nearest()))
        .add_systems(Startup, (setup, player::spawn_player)) // Call from the module
        .add_systems(Update, (player::move_player, player::animate_player))           // Call from the module
    
        .run();
}

fn setup(mut commands: Commands) {
    commands.spawn(Camera2d);
}