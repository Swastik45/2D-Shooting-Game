use bevy::prelude::*;
use bevy::window::PrimaryWindow;

// Sprite sheet layout: 3 frames side by side (469x768 each)
// Index 0 = Front (idle)
// Index 1 = Side  (walking)
// Index 2 = Back  (moving away)

const FRAME_W: u32 = 469;
const FRAME_H: u32 = 768;

#[derive(Component)]
pub struct Player {
    pub animation_timer: Timer,
    pub is_moving: bool,
    pub facing_left: bool,
}

pub fn spawn_player(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    mut texture_atlas_layouts: ResMut<Assets<TextureAtlasLayout>>,
) {
    let texture = asset_server.load("player_sprite.png");

let layout = TextureAtlasLayout::from_grid(
    UVec2::new(320, 663), // exact frame size
    3, 1,                  // front | side | back
    None,
    None,
);

    let texture_atlas_layout = texture_atlas_layouts.add(layout);

    commands.spawn((
        Sprite {
            image: texture,
            texture_atlas: Some(TextureAtlas {
                layout: texture_atlas_layout,
                index: 0, // start on front-facing idle
            }),
            custom_size: Some(Vec2::new(60.0, 110.0)),
            ..default()
        },
        Transform::from_xyz(0.0, 0.0, 0.0),
        Player {
            animation_timer: Timer::from_seconds(0.15, TimerMode::Repeating),
            is_moving: false,
            facing_left: false,
        },
    ));
}

pub fn move_player(
    keyboard_input: Res<ButtonInput<KeyCode>>,
    window_query: Query<&Window, With<PrimaryWindow>>,
    mut query: Query<(&mut Transform, &mut Player)>,
) {
    let Ok(window) = window_query.single() else { return; };
    let window_width = window.width();
    let window_height = window.height();

    for (mut transform, mut player) in &mut query {
        let mut direction = Vec3::ZERO;
        if keyboard_input.pressed(KeyCode::KeyW) { direction.y += 1.0; }
        if keyboard_input.pressed(KeyCode::KeyS) { direction.y -= 1.0; }
        if keyboard_input.pressed(KeyCode::KeyA) { direction.x -= 1.0; }
        if keyboard_input.pressed(KeyCode::KeyD) { direction.x += 1.0; }

        player.is_moving = direction != Vec3::ZERO;

        if direction.x < 0.0 {
            player.facing_left = true;
        } else if direction.x > 0.0 {
            player.facing_left = false;
        }

        if direction != Vec3::ZERO {
            direction = direction.normalize();
            let x_bound = (window_width / 2.0) - 30.0;
            let y_bound = (window_height / 2.0) - 55.0;
            let new_pos = transform.translation + direction * 7.0;
            transform.translation.x = new_pos.x.clamp(-x_bound, x_bound);
            transform.translation.y = new_pos.y.clamp(-y_bound, y_bound);
        }
    }
}

pub fn animate_player(
    keyboard_input: Res<ButtonInput<KeyCode>>,
    mut query: Query<(&mut Sprite, &mut Player)>,
    time: Res<Time>,
) {
    for (mut sprite, mut player) in &mut query {
        let moving_up   = keyboard_input.pressed(KeyCode::KeyW);
        let moving_down = keyboard_input.pressed(KeyCode::KeyS);
        let moving_side = keyboard_input.pressed(KeyCode::KeyA)
                       || keyboard_input.pressed(KeyCode::KeyD);

        if let Some(atlas) = &mut sprite.texture_atlas {
            if player.is_moving {
                player.animation_timer.tick(time.delta());

                // Pick the right frame based on direction
                if moving_up {
                    atlas.index = 2; // back-facing when moving up
                } else if moving_down || moving_side {
                    // Animate between side frames (index 1 only here,
                    // swap to front briefly for a walk feel)
                    if player.animation_timer.just_finished() {
                        atlas.index = if atlas.index == 1 { 0 } else { 1 };
                    }
                }
            } else {
                atlas.index = 0; // idle = front-facing
            }
        }

        sprite.flip_x = player.facing_left;
    }
}