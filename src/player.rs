use bevy::prelude::*;

const FRAME_W: u32 = 320;
const FRAME_H: u32 = 663;
const ANIMATION_SPEED: f32 = 0.15;
const PLAYER_SPEED: f32 = 200.0; // world units per second (was 7 pixels/frame)

const FRAME_FRONT_IDLE: usize = 0;
const FRAME_FRONT_STEP: usize = 1;
const FRAME_SIDE: usize = 2;
const FRAME_BACK: usize = 3;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AnimationState {
    Idle,
    WalkingFront,
    WalkingSide,
    WalkingBack,
}

#[derive(Component)]
pub struct Player {
    pub animation_timer: Timer,
    pub animation_state: AnimationState,
    pub previous_state: AnimationState,
    pub facing_left: bool,
    pub frame_index: usize,
}

pub fn spawn_player(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    mut texture_atlas_layouts: ResMut<Assets<TextureAtlasLayout>>,
) {
    let texture = asset_server.load("player_sprite.png");

    let layout = TextureAtlasLayout::from_grid(
        UVec2::new(FRAME_W, FRAME_H),
        4,
        1,
        None,
        None,
    );

    let texture_atlas_layout = texture_atlas_layouts.add(layout);

    commands.spawn((
        Sprite {
            image: texture,
            texture_atlas: Some(TextureAtlas {
                layout: texture_atlas_layout,
                index: FRAME_FRONT_IDLE,
            }),
            custom_size: Some(Vec2::new(120.0, 220.0)), // bigger character
            ..default()
        },
        Transform::from_xyz(0.0, 0.0, 0.0),
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

        if keyboard_input.pressed(KeyCode::KeyW) {
            direction.y += 1.0;
            moving_up = true;
        }
        if keyboard_input.pressed(KeyCode::KeyS) {
            direction.y -= 1.0;
            moving_down = true;
        }
        if keyboard_input.pressed(KeyCode::KeyA) {
            direction.x -= 1.0;
            moving_left = true;
        }
        if keyboard_input.pressed(KeyCode::KeyD) {
            direction.x += 1.0;
            moving_right = true;
        }

        if moving_left {
            player.facing_left = true;
        } else if moving_right {
            player.facing_left = false;
        }

        if direction != Vec3::ZERO {
            direction = direction.normalize();

            if moving_up && !moving_down {
                player.animation_state = AnimationState::WalkingBack;
            } else if moving_down && !moving_up {
                player.animation_state = AnimationState::WalkingFront;
            } else if moving_left || moving_right {
                player.animation_state = AnimationState::WalkingSide;
            }

            // Frame-rate independent movement — no clamping, world is infinite
            transform.translation += direction * PLAYER_SPEED * time.delta_secs();
        } else {
            player.animation_state = AnimationState::Idle;
        }
    }
}

pub fn animate_player(
    mut query: Query<(&mut Sprite, &mut Player)>,
    time: Res<Time>,
) {
    for (mut sprite, mut player) in &mut query {
        let Some(atlas) = &mut sprite.texture_atlas else { return; };

        player.animation_timer.tick(time.delta());

        // Reset on state change
        if player.animation_state != player.previous_state {
            player.previous_state = player.animation_state;
            player.animation_timer.reset();
            player.frame_index = match player.animation_state {
                AnimationState::Idle         => FRAME_FRONT_IDLE,
                AnimationState::WalkingFront => FRAME_FRONT_IDLE,
                AnimationState::WalkingSide  => FRAME_SIDE,
                AnimationState::WalkingBack  => FRAME_BACK,
            };
        }

        match player.animation_state {
            AnimationState::Idle => {
                player.frame_index = FRAME_FRONT_IDLE;
            }
            AnimationState::WalkingFront => {
                if player.animation_timer.just_finished() {
                    player.frame_index = if player.frame_index == FRAME_FRONT_IDLE {
                        FRAME_FRONT_STEP
                    } else {
                        FRAME_FRONT_IDLE
                    };
                }
            }
            AnimationState::WalkingSide => {
                player.frame_index = FRAME_SIDE;
            }
            AnimationState::WalkingBack => {
                player.frame_index = FRAME_BACK;
            }
        }

        atlas.index = player.frame_index;
        sprite.flip_x = player.facing_left;
    }
}