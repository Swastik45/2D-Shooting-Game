use bevy::prelude::*;
use crate::enemy::Enemy;
use crate::world::{TILE_SIZE, MAP_W, MAP_H, LAYER_PLAYER, is_walkable_position};
use crate::combat::Health;
use crate::game_ui::GameEntity;

const FRAME_W: u32 = 384;
const FRAME_H: u32 = 1024;
const FRAME_COUNT: u32 = 4;

const ANIMATION_SPEED: f32 = 0.15;
const PLAYER_SPEED: f32 = 200.0;

const SPRITE_W: f32 = TILE_SIZE * 1.5;
const SPRITE_H: f32 = TILE_SIZE * 4.0;
const FIRE_COOLDOWN: f32 = 0.25;

const FRAME_FRONT_IDLE: usize = 0;
const FRAME_FRONT_STEP: usize = 1;
const FRAME_SIDE: usize = 2;
const FRAME_BACK: usize = 3;

const MAP_BOUND_X: f32 = (MAP_W as f32 * TILE_SIZE) / 2.0 - TILE_SIZE;
const MAP_BOUND_Y: f32 = (MAP_H as f32 * TILE_SIZE) / 2.0 - TILE_SIZE;

const PLAYER_COLLISION_RADIUS: f32 = SPRITE_W / 2.0;
const ENEMY_COLLISION_RADIUS: f32 = TILE_SIZE * 0.6;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AnimationState {
    Idle,
    WalkingFront,
    WalkingSide,
    WalkingBack,
}

#[derive(Component)]
pub struct Player {
    animation_timer: Timer,
    pub animation_state: AnimationState,
    previous_state: AnimationState,
    pub facing_left: bool,
    frame_index: usize,
}

#[derive(Component)]
pub struct Gun {
    pub cooldown: Timer,
}

#[derive(Component)]
pub struct Weapon;

pub fn spawn_player(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    mut texture_atlas_layouts: ResMut<Assets<TextureAtlasLayout>>,
) {
    let texture = asset_server.load("player_sprite.png");

    let layout = TextureAtlasLayout::from_grid(
        UVec2::new(FRAME_W, FRAME_H),
        FRAME_COUNT,
        1,
        None,
        None,
    );
    let layout_handle = texture_atlas_layouts.add(layout);

    let weapon_texture = asset_server.load("weapon_sprite.png");

    let player_entity = commands.spawn((
        Sprite {
            image: texture,
            texture_atlas: Some(TextureAtlas {
                layout: layout_handle,
                index: FRAME_FRONT_IDLE,
            }),
            custom_size: Some(Vec2::new(SPRITE_W, SPRITE_H)),
            ..default()
        },
        Transform::from_xyz(0.0, 0.0, LAYER_PLAYER),
        Player {
            animation_timer: Timer::from_seconds(ANIMATION_SPEED, TimerMode::Repeating),
            animation_state: AnimationState::Idle,
            previous_state: AnimationState::Idle,
            facing_left: false,
            frame_index: FRAME_FRONT_IDLE,
        },
        Gun {
            cooldown: Timer::from_seconds(FIRE_COOLDOWN, TimerMode::Once),
        },
        Health::new(100.0),
        GameEntity,
    )).id();

    // Spawn weapon as child
    commands.entity(player_entity).with_children(|parent| {
        parent.spawn((
            Sprite {
                image: weapon_texture,
                custom_size: Some(Vec2::new(20.0, 10.0)), // Adjust size
                ..default()
            },
            Transform::from_xyz(15.0, 0.0, 0.1), // Offset from player
            Weapon,
        ));
    });
}

pub fn move_player(
    keyboard_input: Res<ButtonInput<KeyCode>>,
    time: Res<Time>,
    mut player_query: Query<(&mut Transform, &mut Player), Without<Enemy>>,
    enemy_query: Query<&Transform, With<Enemy>>,
) {
    for (mut transform, mut player) in &mut player_query {
        let up    = keyboard_input.pressed(KeyCode::KeyW);
        let down  = keyboard_input.pressed(KeyCode::KeyS);
        let left  = keyboard_input.pressed(KeyCode::KeyA);
        let right = keyboard_input.pressed(KeyCode::KeyD);

        let mut direction = Vec3::ZERO;
        if up    { direction.y += 1.0; }
        if down  { direction.y -= 1.0; }
        if left  { direction.x -= 1.0; }
        if right { direction.x += 1.0; }

        if !up && !down {
            if left  { player.facing_left = true; }
            if right { player.facing_left = false; }
        }

        if direction != Vec3::ZERO {
            player.animation_state = if up && !down {
                AnimationState::WalkingBack
            } else if down && !up {
                AnimationState::WalkingFront
            } else {
                AnimationState::WalkingSide
            };

            direction = direction.normalize();
            let new_position = transform.translation + direction * PLAYER_SPEED * time.delta_secs();

            if is_walkable_position(new_position)
                && !is_position_blocked_by_enemy(new_position, &enemy_query)
            {
                transform.translation = new_position;
                transform.translation.x = transform.translation.x.clamp(-MAP_BOUND_X, MAP_BOUND_X);
                transform.translation.y = transform.translation.y.clamp(-MAP_BOUND_Y, MAP_BOUND_Y);
            }
        } else {
            player.animation_state = AnimationState::Idle;
        }
    }
}

pub fn animate_player(
    time: Res<Time>,
    mut query: Query<(&mut Sprite, &mut Player)>,
) {
    for (mut sprite, mut player) in &mut query {
        let Some(atlas) = &mut sprite.texture_atlas else { continue; };

        player.animation_timer.tick(time.delta());

        if player.animation_state != player.previous_state {
            player.previous_state = player.animation_state;
            player.animation_timer.reset();
            player.frame_index = first_frame(player.animation_state);
        }

        if player.animation_timer.just_finished() {
            player.frame_index = match player.animation_state {
                AnimationState::WalkingFront => {
                    if player.frame_index == FRAME_FRONT_IDLE {
                        FRAME_FRONT_STEP
                    } else {
                        FRAME_FRONT_IDLE
                    }
                }
                _ => player.frame_index,
            };
        }

        atlas.index = player.frame_index;

        sprite.flip_x = match player.animation_state {
            AnimationState::WalkingSide => player.facing_left,
            AnimationState::WalkingBack => !player.facing_left,
            _                           => false,
        };
    }
}

pub fn update_weapon_positions(
    mut weapon_query: Query<(&mut Transform, &ChildOf), With<Weapon>>,
    player_query: Query<&Player>,
    enemy_query: Query<&Enemy>,
) {
    for (mut transform, parent) in &mut weapon_query {
        if let Ok(player) = player_query.get(parent.0) {
            transform.translation.x = if player.facing_left { -15.0 } else { 15.0 };
        } else if let Ok(enemy) = enemy_query.get(parent.0) {
            transform.translation.x = if enemy.facing_left { -15.0 } else { 15.0 };
        }
    }
}

fn first_frame(state: AnimationState) -> usize {
    match state {
        AnimationState::Idle         => FRAME_FRONT_IDLE,
        AnimationState::WalkingFront => FRAME_FRONT_IDLE,
        AnimationState::WalkingSide  => FRAME_SIDE,
        AnimationState::WalkingBack  => FRAME_BACK,
    }
}

fn is_position_blocked_by_enemy(
    position: Vec3,
    enemy_query: &Query<&Transform, With<Enemy>>,
) -> bool {
    let min_distance = PLAYER_COLLISION_RADIUS + ENEMY_COLLISION_RADIUS;
    enemy_query
        .iter()
        .any(|enemy_transform| enemy_transform.translation.distance(position) < min_distance)
}