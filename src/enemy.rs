use bevy::prelude::*;
use crate::player::Gun;
use crate::world::{TILE_SIZE, MAP_W, is_walkable_position, LAYER_PLAYER};
use crate::game_ui::GameEntity;

// ── Sprite sheet ──────────────────────────────────────────────────────────────
const ENEMY_FRAME_W: u32 = 384;
const ENEMY_FRAME_H: u32 = 1024;
const ENEMY_FRAME_COUNT: u32 = 4;

// Frame indices (must match the sprite sheet column order)
const FRAME_FRONT_IDLE: usize = 0;
const FRAME_FRONT_STEP: usize = 1;
const FRAME_SIDE: usize = 2;
const FRAME_BACK: usize = 3;

// ── Tuning constants ──────────────────────────────────────────────────────────
const ENEMY_SPEED: f32 = 120.0;
const ENEMY_SIZE: f32 = TILE_SIZE * 1.2;
const ENEMY_HEALTH: f32 = 30.0;
const SPAWN_COOLDOWN: f32 = 2.0;
const MIN_PLAYER_DISTANCE: f32 = 80.0;
const ENEMY_SEPARATION_DISTANCE: f32 = ENEMY_SIZE * 1.2;
const MIN_SPAWN_DISTANCE_FROM_PLAYER: f32 = 120.0;
const ANIMATION_SPEED: f32 = 0.2;

// ── Animation state ───────────────────────────────────────────────────────────
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EnemyAnimationState {
    Idle,
    WalkingFront,
    WalkingSide,
    WalkingBack,
}

// ── Components ────────────────────────────────────────────────────────────────
#[derive(Component)]
pub struct Enemy {
    pub health: f32,
    pub max_health: f32,
    pub gun: Gun,
    pub facing_left: bool,
    // animation
    animation_timer: Timer,
    animation_state: EnemyAnimationState,
    previous_state: EnemyAnimationState,
    frame_index: usize,
}

// ── Spawner resource ──────────────────────────────────────────────────────────
#[derive(Resource)]
pub struct EnemySpawner {
    pub cooldown: Timer,
    pub count: usize,
    pub max_enemies: usize,
    pub wave_active: bool,
}

impl Default for EnemySpawner {
    fn default() -> Self {
        Self {
            cooldown: Timer::from_seconds(SPAWN_COOLDOWN, TimerMode::Repeating),
            count: 0,
            max_enemies: 4,
            wave_active: true,
        }
    }
}

// ── Setup ─────────────────────────────────────────────────────────────────────
pub fn spawn_enemy_spawner(mut commands: Commands) {
    commands.insert_resource(EnemySpawner::default());
}

// ── Spawn system ──────────────────────────────────────────────────────────────
pub fn spawn_enemies(
    time: Res<Time>,
    mut spawner: ResMut<EnemySpawner>,
    player_query: Query<&Transform, With<crate::player::Player>>,
    enemy_query: Query<&Transform, With<Enemy>>,
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    mut texture_atlas_layouts: ResMut<Assets<TextureAtlasLayout>>,
) {
    spawner.cooldown.tick(time.delta());

    if !spawner.wave_active || spawner.count >= spawner.max_enemies {
        return;
    }

    if spawner.cooldown.just_finished() {
        let Ok(player_transform) = player_query.single() else {
            return;
        };

        let elapsed = time.elapsed_secs();
        let base_angle =
            spawner.count as f32 * std::f32::consts::TAU / spawner.max_enemies as f32;
        let spawn_angle = base_angle + elapsed * 0.8;
        let spawn_radius = (MAP_W as f32 * TILE_SIZE) / 2.5;
        let spawn_pos = Vec3::new(
            spawn_angle.cos() * spawn_radius,
            spawn_angle.sin() * spawn_radius,
            1.0,
        );

        let too_close_to_player =
            spawn_pos.distance(player_transform.translation) < MIN_SPAWN_DISTANCE_FROM_PLAYER;
        let too_close_to_enemy = enemy_query
            .iter()
            .any(|t| t.translation.distance(spawn_pos) < ENEMY_SEPARATION_DISTANCE);

        if is_walkable_position(spawn_pos) && !too_close_to_player && !too_close_to_enemy {
            let texture = asset_server.load("enemy_sprite.png");

            let layout = TextureAtlasLayout::from_grid(
                UVec2::new(ENEMY_FRAME_W, ENEMY_FRAME_H),
                ENEMY_FRAME_COUNT,
                1,
                None,
                None,
            );
            let layout_handle = texture_atlas_layouts.add(layout);

            commands.spawn((
                Sprite {
                    image: texture,
                    texture_atlas: Some(TextureAtlas {
                        layout: layout_handle,
                        index: FRAME_FRONT_IDLE,
                    }),
                    custom_size: Some(Vec2::new(ENEMY_SIZE, ENEMY_SIZE * 2.0)),
                    ..default()
                },
                Transform::from_xyz(spawn_pos.x, spawn_pos.y, LAYER_PLAYER),
                Enemy {
                    health: ENEMY_HEALTH,
                    max_health: ENEMY_HEALTH,
                    gun: Gun {
                        cooldown: Timer::from_seconds(1.5, TimerMode::Once),
                    },
                    facing_left: false,
                    animation_timer: Timer::from_seconds(ANIMATION_SPEED, TimerMode::Repeating),
                    animation_state: EnemyAnimationState::Idle,
                    previous_state: EnemyAnimationState::Idle,
                    frame_index: FRAME_FRONT_IDLE,
                },
                GameEntity,
            ));

            spawner.count += 1;

            if spawner.count >= spawner.max_enemies {
                spawner.wave_active = false;
            }
        }
    }
}

// ── Movement system ───────────────────────────────────────────────────────────
pub fn move_enemies_toward_player(
    time: Res<Time>,
    player_query: Query<&Transform, (With<crate::player::Player>, Without<Enemy>)>,
    mut enemy_query: Query<(&mut Transform, &mut Enemy)>,
) {
    let Ok(player_transform) = player_query.single() else {
        return;
    };

    // Snapshot positions for separation calculation
    let enemy_positions: Vec<Vec3> = enemy_query
        .iter()
        .map(|(t, _)| t.translation)
        .collect();

    for (mut enemy_transform, mut enemy) in &mut enemy_query {
        let to_player = player_transform.translation - enemy_transform.translation;
        let distance_to_player = to_player.length();

        // Stop when close enough
        if distance_to_player <= MIN_PLAYER_DISTANCE {
            enemy.animation_state = EnemyAnimationState::Idle;
            continue;
        }

        // Determine facing / animation state from movement direction
        enemy.facing_left = to_player.x < 0.0;

        let abs_x = to_player.x.abs();
        let abs_y = to_player.y.abs();

        enemy.animation_state = if abs_y > abs_x * 0.5 {
            if to_player.y > 0.0 {
                EnemyAnimationState::WalkingBack
            } else {
                EnemyAnimationState::WalkingFront
            }
        } else {
            EnemyAnimationState::WalkingSide
        };

        // Movement direction + separation steering
        let mut direction = to_player / distance_to_player;
        let mut separation = Vec3::ZERO;

        for other_pos in &enemy_positions {
            let sep = enemy_transform.translation - *other_pos;
            let dist = sep.length();
            if dist > 0.0 && dist < ENEMY_SEPARATION_DISTANCE {
                separation += sep.normalize()
                    * ((ENEMY_SEPARATION_DISTANCE - dist) / ENEMY_SEPARATION_DISTANCE);
            }
        }

        if separation.length_squared() > 0.0 {
            direction = (direction + separation * 0.6).normalize();
        }

        let new_pos =
            enemy_transform.translation + direction * ENEMY_SPEED * time.delta_secs();

        // Don't overshoot the stop radius
        if new_pos.distance(player_transform.translation) < MIN_PLAYER_DISTANCE {
            continue;
        }

        if is_walkable_position(new_pos) {
            enemy_transform.translation = new_pos;
        }
    }
}

// ── Animation system ──────────────────────────────────────────────────────────
pub fn animate_enemies(
    time: Res<Time>,
    mut query: Query<(&mut Sprite, &mut Enemy)>,
) {
    for (mut sprite, mut enemy) in &mut query {
        let Some(atlas) = &mut sprite.texture_atlas else { continue; };

        enemy.animation_timer.tick(time.delta());

        // On state change: reset timer and jump to first frame of new state
        if enemy.animation_state != enemy.previous_state {
            enemy.previous_state = enemy.animation_state;
            enemy.animation_timer.reset();
            enemy.frame_index = first_frame(enemy.animation_state);
        }

        // Advance frame on timer tick
        if enemy.animation_timer.just_finished() {
            enemy.frame_index = match enemy.animation_state {
                // Front walk: toggle between idle-pose and step-pose
                EnemyAnimationState::WalkingFront => {
                    if enemy.frame_index == FRAME_FRONT_IDLE {
                        FRAME_FRONT_STEP
                    } else {
                        FRAME_FRONT_IDLE
                    }
                }
                // All other states are single frames — no change needed
                _ => enemy.frame_index,
            };
        }

        atlas.index = enemy.frame_index;

        // Mirror sprite horizontally based on facing direction
        sprite.flip_x = match enemy.animation_state {
            EnemyAnimationState::WalkingSide => enemy.facing_left,
            EnemyAnimationState::WalkingBack => !enemy.facing_left,
            _ => false,
        };
    }
}

// ── Shooting system ───────────────────────────────────────────────────────────
pub fn enemy_fire_at_player(
    time: Res<Time>,
    player_query: Query<&Transform, (With<crate::player::Player>, Without<Enemy>)>,
    mut enemy_query: Query<(&Transform, &mut Enemy)>,
    mut commands: Commands,
) {
    let Ok(player_transform) = player_query.single() else {
        return;
    };

    for (enemy_transform, mut enemy) in &mut enemy_query {
        enemy.gun.cooldown.tick(time.delta());

        if enemy.gun.cooldown.is_finished() {
            enemy.gun.cooldown.reset();

            let direction =
                (player_transform.translation - enemy_transform.translation).normalize();
            let spawn_pos = enemy_transform.translation + direction * 15.0;
            let angle = direction.y.atan2(direction.x);

            commands.spawn((
                Sprite {
                    color: Color::srgb(1.0, 0.4, 0.4),
                    custom_size: Some(Vec2::new(10.0, 4.0)),
                    ..default()
                },
                Transform {
                    translation: Vec3::new(spawn_pos.x, spawn_pos.y, 1.1),
                    rotation: Quat::from_rotation_z(angle),
                    ..default()
                },
                crate::weapon::Bullet {
                    direction,
                    lifetime: Timer::from_seconds(1.2, TimerMode::Once),
                },
                EnemyBullet,
                GameEntity,
            ));
        }
    }
}

// ── Helper ────────────────────────────────────────────────────────────────────
fn first_frame(state: EnemyAnimationState) -> usize {
    match state {
        EnemyAnimationState::Idle         => FRAME_FRONT_IDLE,
        EnemyAnimationState::WalkingFront => FRAME_FRONT_IDLE,
        EnemyAnimationState::WalkingSide  => FRAME_SIDE,
        EnemyAnimationState::WalkingBack  => FRAME_BACK,
    }
}

// ── Marker component ──────────────────────────────────────────────────────────
#[derive(Component)]
pub struct EnemyBullet;