use bevy::prelude::*;
use crate::combat::Health;
use crate::player::Player;
use crate::game_state::{GameScore, GameState};

/// Tag every entity that should be wiped on restart
#[derive(Component)]
pub struct GameEntity;

#[derive(Component)]
pub struct HealthText;

#[derive(Component)]
pub struct ScoreText;

#[derive(Component)]
pub struct HighScoreText;

#[derive(Component)]
pub struct GameOverText;

pub fn spawn_ui(mut commands: Commands) {
    // Health
    commands.spawn((
        Text::new("HP: 100"),
        TextFont { font_size: 24.0, ..default() },
        TextColor(Color::WHITE),
        Node {
            position_type: PositionType::Absolute,
            top: Val::Px(10.0),
            left: Val::Px(10.0),
            ..default()
        },
        HealthText,
        GameEntity,  // ← tagged
    ));

    // Score
    commands.spawn((
        Text::new("Score: 0"),
        TextFont { font_size: 24.0, ..default() },
        TextColor(Color::srgb(0.0, 1.0, 0.0)),
        Node {
            position_type: PositionType::Absolute,
            top: Val::Px(40.0),
            left: Val::Px(10.0),
            ..default()
        },
        ScoreText,
        GameEntity,  // ← tagged
    ));

    // High Score
    commands.spawn((
        Text::new("Best: 0"),
        TextFont { font_size: 20.0, ..default() },
        TextColor(Color::srgb(1.0, 1.0, 0.0)),
        Node {
            position_type: PositionType::Absolute,
            top: Val::Px(70.0),
            left: Val::Px(10.0),
            ..default()
        },
        HighScoreText,
        GameEntity,  // ← tagged
    ));
}

pub fn spawn_game_over_ui(mut commands: Commands) {
    commands.spawn((
        Text::new("GAME OVER\nPress Enter to Restart"),
        TextFont { font_size: 72.0, ..default() },
        TextColor(Color::srgb(1.0, 0.0, 0.0)),
        Node {
            position_type: PositionType::Absolute,
            top: Val::Percent(40.0),
            left: Val::Percent(30.0),
            ..default()
        },
        GameOverText,
        GameEntity,  // ← tagged
    ));
}

pub fn update_health_display(
    player_query: Query<&Health, With<Player>>,
    mut text_query: Query<&mut Text, With<HealthText>>,
) {
    if let Ok(health) = player_query.single() {
        if let Ok(mut text) = text_query.single_mut() {
            text.0 = format!("HP: {:.0}", health.current.max(0.0));
        }
    }
}

pub fn update_score_display(
    score: Res<GameScore>,
    mut score_query: Query<&mut Text, (With<ScoreText>, Without<HighScoreText>)>,
    mut high_score_query: Query<&mut Text, (With<HighScoreText>, Without<ScoreText>)>,
) {
    if score.is_changed() {
        if let Ok(mut text) = score_query.single_mut() {
            text.0 = format!("Score: {}", score.current);
        }
        if let Ok(mut text) = high_score_query.single_mut() {
            text.0 = format!("Best: {}", score.high_score);
        }
    }
}

pub fn hide_ui_on_game_over(
    state: Res<State<GameState>>,
    mut texts: Query<&mut Visibility, (With<Text>, Without<GameOverText>)>,
) {
    if state.get() == &GameState::GameOver {
        for mut visibility in &mut texts {
            *visibility = Visibility::Hidden;
        }
    }
}

pub fn restart_game(
    keyboard: Res<ButtonInput<KeyCode>>,
    mut next_state: ResMut<NextState<GameState>>,
    mut commands: Commands,
    entities: Query<Entity, With<GameEntity>>,
    mut score: ResMut<GameScore>,
    mut spawner: ResMut<crate::enemy::EnemySpawner>,  // ← add this
) {
    if keyboard.just_pressed(KeyCode::Enter) {
        for e in &entities {
            commands.entity(e).despawn();
        }
        score.current = 0;
        *spawner = crate::enemy::EnemySpawner::default();  // ← reset count + timer
        next_state.set(GameState::Playing);
    }
}