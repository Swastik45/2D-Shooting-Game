use bevy::prelude::*;

mod world;
mod player;
mod camera;
mod weapon;
mod enemy;
mod combat;
mod game_state;
mod game_ui;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins.set(ImagePlugin::default_nearest()))
        .init_state::<game_state::GameState>()
        .add_systems(Startup, (
            camera::spawn_camera,
            world::spawn_world,
            player::spawn_player,
            enemy::spawn_enemy_spawner,
            game_state::init_game_score,
            game_ui::spawn_ui,
        ))
        .add_systems(Update, (
            player::move_player,
            player::animate_player,
            weapon::fire_gun,
            weapon::move_bullets,
            weapon::update_muzzle_flashes,
            enemy::spawn_enemies,
            enemy::move_enemies_toward_player,
            enemy::enemy_fire_at_player,
            combat::check_bullet_collisions,
            combat::check_enemy_bullet_collisions,
            combat::remove_dead_enemies,
            game_state::check_game_over,
            game_ui::update_health_display,
            game_ui::update_score_display,
            camera::camera_follow,
        ).run_if(in_state(game_state::GameState::Playing)))
        .add_systems(OnEnter(game_state::GameState::GameOver), game_ui::spawn_game_over_ui)
        .add_systems(Update, game_ui::hide_ui_on_game_over.run_if(in_state(game_state::GameState::GameOver)))
        .run();
}