use bevy::prelude::*;
use crate::weapon::Bullet;
use crate::enemy::{Enemy, EnemyBullet};
use crate::player::Player;


const BULLET_DAMAGE: f32 = 10.0;

#[derive(Component)]
#[allow(dead_code)]
pub struct Health {
    pub current: f32,
    pub max: f32,
}

impl Health {
    pub fn new(max: f32) -> Self {
        Self { current: max, max }
    }
}

pub fn check_bullet_collisions(
    mut commands: Commands,
    bullets: Query<(Entity, &Transform), With<Bullet>>,
    mut enemies: Query<(&Transform, &mut Enemy), Without<Bullet>>,
) {
    for (bullet_entity, bullet_transform) in &bullets {
        for (enemy_transform, mut enemy) in &mut enemies {
            let distance = bullet_transform.translation.distance(enemy_transform.translation);

            if distance < 20.0 {
                enemy.health -= BULLET_DAMAGE;
                commands.entity(bullet_entity).despawn();
                break; // important
            }
        }
    }
}

pub fn check_enemy_bullet_collisions(
    mut commands: Commands,
    enemy_bullets: Query<(Entity, &Transform), With<EnemyBullet>>,
    mut player_query: Query<(&Transform, &mut Health), With<Player>>,
) {
    let Ok((player_transform, mut health)) = player_query.single_mut() else {
        return;
    };

    for (bullet_entity, bullet_transform) in &enemy_bullets {
        let distance = bullet_transform.translation.distance(player_transform.translation);
        if distance < 25.0 {
            // Hit player!
            health.current -= BULLET_DAMAGE;
            commands.entity(bullet_entity).despawn();
        }
    }
}

pub fn remove_dead_enemies(
    mut commands: Commands,
    query: Query<(Entity, &Enemy), Changed<Enemy>>,
    mut score: ResMut<crate::game_state::GameScore>,
    mut spawner: ResMut<crate::enemy::EnemySpawner>,
) {
    for (entity, enemy) in &query {
        if enemy.health <= 0.0 {
            commands.entity(entity).despawn();
            score.current += 1;

            if score.current > score.high_score {
                score.high_score = score.current;
                crate::game_state::save_high_score(score.high_score);
            }

            if spawner.count > 0 {
                spawner.count -= 1;
            }
        }
    }
}
