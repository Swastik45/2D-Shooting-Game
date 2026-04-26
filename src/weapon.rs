use bevy::prelude::*;
use crate::player::{AnimationState, Gun, Player};
use crate::world::{is_solid_tile, LAYER_PLAYER, world_tile_at_position};

const BULLET_SPEED: f32 = 420.0;
const BULLET_SIZE: Vec2 = Vec2::new(10.0, 4.0); // elongated — looks like a bullet
const BULLET_LIFETIME: f32 = 1.2;
const MUZZLE_FLASH_SIZE: f32 = 14.0;
const MUZZLE_FLASH_DURATION: f32 = 0.08;
const MUZZLE_OFFSET: f32 = 20.0;

#[derive(Component)]
pub struct Bullet {
    pub direction: Vec3,
    lifetime: Timer,
}

#[derive(Component)]
pub struct MuzzleFlash {
    timer: Timer,
}

pub fn fire_gun(
    mouse_input: Res<ButtonInput<MouseButton>>,
    keyboard_input: Res<ButtonInput<KeyCode>>,
    time: Res<Time>,
    mut commands: Commands,
    mut query: Query<(&Transform, &mut Gun, &Player)>,
) {
    let should_fire = mouse_input.just_pressed(MouseButton::Left)
        || keyboard_input.just_pressed(KeyCode::Space);

    for (transform, mut gun, player) in &mut query {
        gun.cooldown.tick(time.delta());

        if should_fire && gun.cooldown.is_finished() {
            gun.cooldown.reset();

            let direction = player_facing_direction(player);
            let angle = direction.y.atan2(direction.x); // rotation matching travel dir
            let spawn_pos = transform.translation + direction * MUZZLE_OFFSET;

            // Bullet — rotated rectangle so it looks like a projectile
            commands.spawn((
                Sprite {
                    color: Color::srgb(1.0, 0.95, 0.4),
                    custom_size: Some(BULLET_SIZE),
                    ..default()
                },
                Transform {
                    translation: Vec3::new(spawn_pos.x, spawn_pos.y, LAYER_PLAYER + 0.5),
                    rotation: Quat::from_rotation_z(angle),
                    ..default()
                },
                Bullet {
                    direction,
                    lifetime: Timer::from_seconds(BULLET_LIFETIME, TimerMode::Once),
                },
            ));

            // Muzzle flash — brief bright burst at barrel tip
            commands.spawn((
                Sprite {
                    color: Color::srgba(1.0, 0.6, 0.1, 0.9),
                    custom_size: Some(Vec2::splat(MUZZLE_FLASH_SIZE)),
                    ..default()
                },
                Transform::from_xyz(spawn_pos.x, spawn_pos.y, LAYER_PLAYER + 0.6),
                MuzzleFlash {
                    timer: Timer::from_seconds(MUZZLE_FLASH_DURATION, TimerMode::Once),
                },
            ));
        }
    }
}

pub fn move_bullets(
    time: Res<Time>,
    mut commands: Commands,
    mut query: Query<(Entity, &mut Transform, &mut Bullet)>,
) {
    for (entity, mut transform, mut bullet) in &mut query {
        transform.translation += bullet.direction * BULLET_SPEED * time.delta_secs();
        bullet.lifetime.tick(time.delta());

        let hit_wall = world_tile_at_position(transform.translation)
            .map_or(true, |tile_id| is_solid_tile(tile_id));

        if bullet.lifetime.is_finished() || hit_wall {
            commands.entity(entity).despawn();
        }
    }
}

pub fn update_muzzle_flashes(
    time: Res<Time>,
    mut commands: Commands,
    mut query: Query<(Entity, &mut Sprite, &mut MuzzleFlash)>,
) {
    for (entity, mut sprite, mut flash) in &mut query {
        flash.timer.tick(time.delta());

        // Fade out the flash over its lifetime
        let progress = flash.timer.fraction(); // 0.0 → 1.0
        sprite.color.set_alpha(1.0 - progress);

        if flash.timer.is_finished() {
            commands.entity(entity).despawn();
        }
    }
}

fn player_facing_direction(player: &Player) -> Vec3 {
    match player.animation_state {
        AnimationState::WalkingBack => Vec3::Y,
        AnimationState::WalkingFront => Vec3::NEG_Y,
        AnimationState::WalkingSide | AnimationState::Idle => {
            if player.facing_left { Vec3::NEG_X } else { Vec3::X }
        }
    }
}