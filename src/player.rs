use bevy::prelude::*;
use crate::world::{TILE_SIZE, MAP_W, MAP_H};

const FRAME_W: u32 = 384;
const FRAME_H: u32 = 1024;
const FRAME_COUNT: u32 = 4;

const ANIMATION_SPEED: f32 = 0.15;
const PLAYER_SPEED: f32 = 200.0;

// Sprite display size — keep the 384:1024 aspect ratio scaled to fit the tile grid
const SPRITE_W: f32 = TILE_SIZE * 1.5;
const SPRITE_H: f32 = TILE_SIZE * 4.0;

const FRAME_FRONT_IDLE: usize = 0;
const FRAME_FRONT_STEP: usize = 1;
const FRAME_SIDE: usize = 2;
const FRAME_BACK: usize = 3;

const MAP_BOUND_X: f32 = (MAP_W as f32 * TILE_SIZE) / 2.0 - TILE_SIZE;
const MAP_BOUND_Y: f32 = (MAP_H as f32 * TILE_SIZE) / 2.0 - TILE_SIZE;

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
    animation_state: AnimationState,
    previous_state: AnimationState,
    facing_left: bool,
    frame_index: usize,
}

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

    commands.spawn((
        Sprite {
            image: texture,
            texture_atlas: Some(TextureAtlas {
                layout: layout_handle,
                index: FRAME_FRONT_IDLE,
            }),
            custom_size: Some(Vec2::new(SPRITE_W, SPRITE_H)),
            ..default()
        },
        Transform::from_xyz(0.0, 0.0, 1.0),
        Player {
            animation_timer: Timer::from_seconds(ANIMATION_SPEED, TimerMode::Repeating),
            animation_state: AnimationState::Idle,
            previous_state: AnimationState::Idle,
            facing_left: false,
            frame_index: FRAME_FRONT_IDLE,
        },
    ));
}

pub fn move_player(
    keyboard_input: Res<ButtonInput<KeyCode>>,
    time: Res<Time>,
    mut query: Query<(&mut Transform, &mut Player)>,
) {
    for (mut transform, mut player) in &mut query {
        let mut direction = Vec3::ZERO;
        let mut moving_up = false;
        let mut moving_down = false;
        let mut moving_left = false;
        let mut moving_right = false;

        if keyboard_input.pressed(KeyCode::KeyW) { direction.y += 1.0; moving_up = true; }
        if keyboard_input.pressed(KeyCode::KeyS) { direction.y -= 1.0; moving_down = true; }
        if keyboard_input.pressed(KeyCode::KeyA) { direction.x -= 1.0; moving_left = true; }
        if keyboard_input.pressed(KeyCode::KeyD) { direction.x += 1.0; moving_right = true; }

        if moving_left {
            player.facing_left = true;
        } else if moving_right {
            player.facing_left = false;
        }

        if direction != Vec3::ZERO {
            direction = direction.normalize();

            player.animation_state = if moving_up && !moving_down {
                AnimationState::WalkingBack
            } else if moving_down && !moving_up {
                AnimationState::WalkingFront
            } else {
                AnimationState::WalkingSide
            };

            transform.translation += direction * PLAYER_SPEED * time.delta_secs();
            transform.translation.x = transform.translation.x.clamp(-MAP_BOUND_X, MAP_BOUND_X);
            transform.translation.y = transform.translation.y.clamp(-MAP_BOUND_Y, MAP_BOUND_Y);
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

        // Reset timer and frame on state change
        if player.animation_state != player.previous_state {
            player.previous_state = player.animation_state;
            player.frame_index = match player.animation_state {
                AnimationState::Idle => FRAME_FRONT_IDLE,
                AnimationState::WalkingFront => FRAME_FRONT_IDLE,
                AnimationState::WalkingSide => FRAME_SIDE,
                AnimationState::WalkingBack => FRAME_BACK,
            };
            player.animation_timer.reset();
        }

        // Advance animation on tick
        if player.animation_timer.just_finished() {
            player.frame_index = match player.animation_state {
                AnimationState::Idle => FRAME_FRONT_IDLE,
                AnimationState::WalkingFront => {
                    if player.frame_index == FRAME_FRONT_IDLE { FRAME_FRONT_STEP } else { FRAME_FRONT_IDLE }
                }
                AnimationState::WalkingSide => FRAME_SIDE,
                AnimationState::WalkingBack => FRAME_BACK,
            };
        }

        atlas.index = player.frame_index;
        sprite.flip_x = player.facing_left;
    }
}