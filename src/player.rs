use bevy::prelude::*;
use crate::world::{TILE_SIZE, MAP_W, MAP_H};

// Measured from the actual sprite PNG (1536x1024 total, 4 frames wide)
const FRAME_W: u32 = 384;
const FRAME_H: u32 = 1024;
const FRAME_COUNT: u32 = 4;

const ANIMATION_SPEED: f32 = 0.15;
const PLAYER_SPEED: f32 = 200.0;

// Display size preserving 384:1024 aspect ratio
const SPRITE_W: f32 = TILE_SIZE * 1.5;
const SPRITE_H: f32 = TILE_SIZE * 4.0;

// Frame indices — confirmed by pixel analysis of the sprite sheet:
//   0 = front idle  (faces forward)
//   1 = front step  (faces forward, one foot forward)
//   2 = side        (faces RIGHT natively)
//   3 = back        (faces LEFT natively)
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
        let up    = keyboard_input.pressed(KeyCode::KeyW);
        let down  = keyboard_input.pressed(KeyCode::KeyS);
        let left  = keyboard_input.pressed(KeyCode::KeyA);
        let right = keyboard_input.pressed(KeyCode::KeyD);

        let mut direction = Vec3::ZERO;
        if up    { direction.y += 1.0; }
        if down  { direction.y -= 1.0; }
        if left  { direction.x -= 1.0; }
        if right { direction.x += 1.0; }

        // Only update facing when purely moving horizontally (no vertical component).
        // This prevents the side-frame from flipping mid-diagonal when W+A -> W+D.
        if !up && !down {
            if left  { player.facing_left = true; }
            if right { player.facing_left = false; }
        }

        if direction != Vec3::ZERO {
            // Vertical movement takes priority over horizontal so back/front
            // animations are never overridden by a simultaneous side key.
            player.animation_state = if up && !down {
                AnimationState::WalkingBack
            } else if down && !up {
                AnimationState::WalkingFront
            } else {
                AnimationState::WalkingSide
            };

            direction = direction.normalize();
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

        // On state change: reset timer and snap to the first frame of the new state.
        if player.animation_state != player.previous_state {
            player.previous_state = player.animation_state;
            player.animation_timer.reset();
            player.frame_index = first_frame(player.animation_state);
        }

        // Advance animated states on each timer tick.
        if player.animation_timer.just_finished() {
            player.frame_index = match player.animation_state {
                // Toggle between idle-pose and step-pose for front walk.
                AnimationState::WalkingFront => {
                    if player.frame_index == FRAME_FRONT_IDLE {
                        FRAME_FRONT_STEP
                    } else {
                        FRAME_FRONT_IDLE
                    }
                }
                // All other states are single-frame.
                _ => player.frame_index,
            };
        }

        atlas.index = player.frame_index;

        // ── Flip logic ────────────────────────────────────────────────────────
        // Frame 2 (side) is drawn facing RIGHT  → flip_x when moving left.
        // Frame 3 (back) is drawn facing LEFT   → flip_x when moving right.
        // Frames 0/1 (front) face forward       → no flip needed.
        sprite.flip_x = match player.animation_state {
            AnimationState::WalkingSide => player.facing_left,   // native=right → flip for left
            AnimationState::WalkingBack => !player.facing_left,  // native=left  → flip for right
            _                           => false,
        };
    }
}

/// Returns the starting frame index for a given animation state.
fn first_frame(state: AnimationState) -> usize {
    match state {
        AnimationState::Idle         => FRAME_FRONT_IDLE,
        AnimationState::WalkingFront => FRAME_FRONT_IDLE,
        AnimationState::WalkingSide  => FRAME_SIDE,
        AnimationState::WalkingBack  => FRAME_BACK,
    }
}