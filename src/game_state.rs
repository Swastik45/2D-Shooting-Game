use bevy::prelude::*;
use directories::ProjectDirs;
use std::fs;
use std::path::PathBuf;
use crate::combat::Health;
use crate::player::Player;

const HIGH_SCORE_FILE_NAME: &str = "high_score.txt";

#[derive(Resource, Clone)]
pub struct GameScore {
    pub current: u32,
    pub high_score: u32,
}

impl Default for GameScore {
    fn default() -> Self {
        Self {
            current: 0,
            high_score: load_high_score(),
        }
    }
}

#[derive(States, Default, Debug, Clone, PartialEq, Eq, Hash)]
pub enum GameState {
    #[default]
    Playing,
    GameOver,
}

fn score_save_path() -> Option<PathBuf> {
    ProjectDirs::from("com", "example", "my_bevy_game")
        .map(|dirs| dirs.config_dir().join(HIGH_SCORE_FILE_NAME))
}

fn load_high_score() -> u32 {
    let path = match score_save_path() {
        Some(path) => path,
        None => return 0,
    };

    if let Ok(contents) = fs::read_to_string(path) {
        if let Ok(value) = contents.trim().parse::<u32>() {
            return value;
        }
    }

    0
}

pub fn save_high_score(high_score: u32) {
    if let Some(path) = score_save_path() {
        if let Some(parent) = path.parent() {
            if let Err(err) = fs::create_dir_all(parent) {
                warn!("Failed to create high score directory: {err}");
                return;
            }
        }

        if let Err(err) = fs::write(&path, high_score.to_string()) {
            warn!("Failed to write high score file {path:?}: {err}");
        }
    }
}

pub fn init_game_score(mut commands: Commands) {
    commands.insert_resource(GameScore::default());
}

pub fn check_game_over(
    player_query: Query<&Health, With<Player>>,
    mut next_state: ResMut<NextState<GameState>>,
) {
    if let Ok(health) = player_query.single() {
        if health.current <= 0.0 {
            next_state.set(GameState::GameOver);
        }
    }
}
