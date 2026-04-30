use bevy::prelude::*;
use crate::player::{AnimationState, Gun, Player};
use crate::world::{is_solid_tile, LAYER_PLAYER, world_tile_at_position};

const BULLET_SPEED: f32 = 420.0;
const BULLET_SIZE: Vec2 = Vec2::new(10.0, 4.0);
const BULLET_LIFETIME: f32 = 1.2;
const MUZZLE_FLASH_SIZE: f32 = 14.0;
const MUZZLE_FLASH_DURATION: f32 = 0.08;
const MUZZLE_OFFSET: f32 = 20.0;

// Gun shape rendered as a plain coloured rectangle — no image needed.
// Barrel points RIGHT at 0°. flip_x / rotation handle all other directions.
const GUN_SIZE: Vec2 = Vec2::new(14.0, 5.0);
const GUN_COLOR: Color = Color::srgb(0.4, 0.4, 0.45);

#[derive(Component)]
pub struct Bullet {
    pub direction: Vec3,
    pub lifetime: Timer,
}

#[derive(Component)]
pub struct MuzzleFlash {
    timer: Timer,
}

/// Marks the small gun rectangle that is a child of the player entity.
#[derive(Component)]
pub struct WeaponSprite;

/// Spawns a gun rectangle and parents it to `owner`.
/// No image file required — just a coloured rect.
pub fn spawn_weapon_sprite(commands: &mut Commands, owner: Entity) {
    let gun = commands.spawn((
        Sprite {
            color: GUN_COLOR,
            custom_size: Some(GUN_SIZE),
            ..default()
        },
        Transform::from_xyz(10.0, -8.0, 0.1),
        WeaponSprite,
    )).id();
    commands.entity(owner).add_child(gun);
}

/// Repositions and rotates the gun child every frame to match the
/// player's facing direction and animation state.
/// This replaces `player::update_weapon_positions` entirely.
pub fn update_weapon_sprite_transform(
    player_query: Query<(&Player, &Children)>,
    mut gun_query: Query<(&mut Transform, &mut Sprite), With<WeaponSprite>>,
) {
    for (player, children) in &player_query {
        let facing_left = player.facing_left;

        // Local offset and rotation per animation state.
        // The gun sprite barrel points RIGHT, so:
        //   0°  → right,  180° → left,  90° → up,  270° → down
        let (local_x, local_y, angle_deg) = match player.animation_state {
            AnimationState::Idle | AnimationState::WalkingSide => {
                let x = if facing_left { -10.0 } else { 10.0 };
                let angle = if facing_left { 180.0_f32 } else { 0.0_f32 };
                (x, -8.0_f32, angle)
            }
            AnimationState::WalkingFront => {
                // Gun hangs slightly in front and below centre, barrel points down
                let x = if facing_left { -6.0 } else { 6.0 };
                (x, -14.0_f32, 270.0_f32)
            }
            AnimationState::WalkingBack => {
                // Gun raised, barrel points up
                let x = if facing_left { -6.0 } else { 6.0 };
                (x, -4.0_f32, 90.0_f32)
            }
        };

        for child in children.iter() {
            if let Ok((mut transform, mut sprite)) = gun_query.get_mut(child) {
                transform.translation.x = local_x;
                transform.translation.y = local_y;
                transform.rotation = Quat::from_rotation_z(angle_deg.to_radians());
                // No flip needed — rotation handles all four directions above
                sprite.flip_x = false;
            }
        }
    }
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
            let angle = direction.y.atan2(direction.x);
            let spawn_pos = transform.translation + direction * MUZZLE_OFFSET;

            commands.spawn((
                Sprite {
                    color: Color::srgb(1.0, 0.95, 0.4), // yellow
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
        sprite.color.set_alpha(1.0 - flash.timer.fraction());
        if flash.timer.is_finished() {
            commands.entity(entity).despawn();
        }
    }
}

pub fn player_facing_direction(player: &Player) -> Vec3 {
    match player.animation_state {
        AnimationState::WalkingBack  => Vec3::Y,
        AnimationState::WalkingFront => Vec3::NEG_Y,
        AnimationState::WalkingSide | AnimationState::Idle => {
            if player.facing_left { Vec3::NEG_X } else { Vec3::X }
        }
    }
}