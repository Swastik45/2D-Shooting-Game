use bevy::prelude::*;
use crate::player::{Gun, Weapon};
use crate::world::{TILE_SIZE, MAP_W, is_walkable_position, LAYER_PLAYER};
use crate::game_ui::GameEntity;

const ENEMY_FRAME_W: u32 = 384;
const ENEMY_FRAME_H: u32 = 1024;
const ENEMY_FRAME_COUNT: u32 = 1; // Single frame for now

const ENEMY_SPEED: f32 = 120.0;
const ENEMY_SIZE: f32 = TILE_SIZE * 1.2;
const ENEMY_HEALTH: f32 = 30.0;
const SPAWN_COOLDOWN: f32 = 2.0;
const MIN_PLAYER_DISTANCE: f32 = 80.0;
const ENEMY_SEPARATION_DISTANCE: f32 = ENEMY_SIZE * 1.2;
const MIN_SPAWN_DISTANCE_FROM_PLAYER: f32 = 120.0;

#[derive(Component)]
#[allow(dead_code)]
pub struct Enemy {
    pub health: f32,
    pub max_health: f32,
    pub gun: Gun,
    pub facing_left: bool,
}

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

pub fn spawn_enemy_spawner(mut commands: Commands) {
    commands.insert_resource(EnemySpawner::default());
}

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
        let base_angle = spawner.count as f32 * std::f32::consts::TAU / spawner.max_enemies as f32;
        let spawn_angle = base_angle + elapsed * 0.8;
        let spawn_radius = (MAP_W as f32 * TILE_SIZE) / 2.5;
        let spawn_x = spawn_angle.cos() * spawn_radius;
        let spawn_y = spawn_angle.sin() * spawn_radius;
        let spawn_pos = Vec3::new(spawn_x, spawn_y, 1.0);

        let too_close_to_player = spawn_pos.distance(player_transform.translation) < MIN_SPAWN_DISTANCE_FROM_PLAYER;
        let too_close_to_enemy = enemy_query
            .iter()
            .any(|other| other.translation.distance(spawn_pos) < ENEMY_SEPARATION_DISTANCE);

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

            let weapon_texture = asset_server.load("weapon_sprite.png");

            let enemy_entity = commands.spawn((
                Sprite {
                    image: texture,
                    texture_atlas: Some(TextureAtlas {
                        layout: layout_handle,
                        index: 0,
                    }),
                    custom_size: Some(Vec2::new(ENEMY_SIZE, ENEMY_SIZE * 2.0)), // Adjust size
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
                },
                GameEntity,
            )).id();

            // Spawn weapon as child
            commands.entity(enemy_entity).with_children(|parent| {
                parent.spawn((
                    Sprite {
                        image: weapon_texture.clone(),
                        custom_size: Some(Vec2::new(20.0, 10.0)), // Adjust size
                        ..default()
                    },
                    Transform::from_xyz(15.0, 0.0, 0.1), // Offset from enemy
                    Weapon,
                ));
            });

            spawner.count += 1;

            if spawner.count >= spawner.max_enemies {
                spawner.wave_active = false;
            }
        }
    }
}

pub fn move_enemies_toward_player(
    time: Res<Time>,
    player_query: Query<&Transform, (With<crate::player::Player>, Without<Enemy>)>,
    mut enemy_query: Query<(&mut Transform, &mut Enemy)>,
) {
    let Ok(player_transform) = player_query.single() else {
        return;
    };

    let enemy_positions: Vec<Vec3> = enemy_query
        .iter()
        .map(|(transform, _)| transform.translation)
        .collect();

    for (mut enemy_transform, mut enemy) in &mut enemy_query {
        let to_player = player_transform.translation - enemy_transform.translation;
        let distance_to_player = to_player.length();

        if distance_to_player <= MIN_PLAYER_DISTANCE {
            continue;
        }

        enemy.facing_left = to_player.x < 0.0; // Face towards player

        let mut direction = to_player / distance_to_player;
        let mut separation = Vec3::ZERO;

        for other_pos in &enemy_positions {
            let separation_vector = enemy_transform.translation - *other_pos;
            let dist = separation_vector.length();
            if dist > 0.0 && dist < ENEMY_SEPARATION_DISTANCE {
                separation += separation_vector.normalize()
                    * ((ENEMY_SEPARATION_DISTANCE - dist) / ENEMY_SEPARATION_DISTANCE);
            }
        }

        if separation.length_squared() > 0.0 {
            direction = (direction + separation * 0.6).normalize();
        }

        let new_pos = enemy_transform.translation + direction * ENEMY_SPEED * time.delta_secs();
        if new_pos.distance(player_transform.translation) < MIN_PLAYER_DISTANCE {
            continue;
        }

        if is_walkable_position(new_pos) {
            enemy_transform.translation = new_pos;
        }
    }
}

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

            let direction = (player_transform.translation - enemy_transform.translation).normalize();
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

#[derive(Component)]
pub struct EnemyBullet;