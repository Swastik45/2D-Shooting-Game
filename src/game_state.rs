use bevy::prelude::*;
use crate::combat::Health;
use crate::player::Player;

#[derive(Resource, Clone)]
pub struct GameScore {
    pub current: u32,
    pub high_score: u32,
}

impl Default for GameScore {
    fn default() -> Self {
        Self {
            current: 0,
            high_score: 0,
        }
    }
}

#[derive(States, Default, Debug, Clone, PartialEq, Eq, Hash)]
pub enum GameState {
    #[default]
    Playing,
    GameOver,
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
