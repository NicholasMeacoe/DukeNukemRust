#![allow(dead_code)]

use bevy::prelude::*;
use crate::combat::types::*;
use crate::player::types::PlayerController;
use crate::interactivity::types::ExplosionDamageEvent;

pub fn spawn_projectiles(
    mut events: EventReader<SpawnProjectileEvent>,
    mut commands: Commands,
) {
    for ev in events.read() {
        let vel = ev.direction.normalize_or_zero() * ev.velocity;
        let lifetime = match ev.projectile_type {
            ProjectileType::HitscanBullet | ProjectileType::ShotgunPellet | ProjectileType::MightyBoot => 0.05,
            ProjectileType::Rocket | ProjectileType::DevastatorMissile => 4.0,
            ProjectileType::Pipebomb => 8.0,
            _ => 3.0,
        };

        commands.spawn((
            Projectile {
                projectile_type: ev.projectile_type,
                velocity: vel,
                damage: ev.damage,
                is_player_source: ev.is_player_source,
                lifetime,
            },
            TransformBundle::from_transform(Transform::from_translation(ev.origin)),
        ));
    }
}

pub fn update_projectiles(
    time: Res<Time>,
    mut commands: Commands,
    mut projectiles: Query<(Entity, &mut Transform, &mut Projectile)>,
    mut enemies: Query<(Entity, &Transform, &mut EnemyActor), Without<Projectile>>,
    mut players: Query<(Entity, &Transform, &mut PlayerController), (Without<Projectile>, Without<EnemyActor>)>,
    mut damage_events: EventWriter<EntityDamageEvent>,
    mut explosion_events: EventWriter<ExplosionDamageEvent>,
) {
    let dt = time.delta_seconds();

    // Fast squared distance walking squish on shrunk enemies by player
    for (_, player_trans, _) in players.iter() {
        let p_pos = player_trans.translation;
        for (e_entity, enemy_trans, mut enemy) in enemies.iter_mut() {
            if enemy.is_shrunk && enemy.state != EnemyAiState::Gibbed && enemy.state != EnemyAiState::Dying {
                let dist_sq = p_pos.distance_squared(enemy_trans.translation);
                if dist_sq < 0.64 { // 0.8 * 0.8
                    damage_events.send(EntityDamageEvent {
                        target: e_entity,
                        amount: 100,
                        source: DamageSource::PlayerWeapon(ProjectileType::MightyBoot),
                        hit_origin: enemy_trans.translation,
                    });
                    enemy.is_shrunk = false;
                }
            }
        }
    }

    for (proj_entity, mut trans, mut proj) in projectiles.iter_mut() {
        proj.lifetime -= dt;

        // Pipebomb trajectory with gravity
        if proj.projectile_type == ProjectileType::Pipebomb {
            proj.velocity.y -= 15.0 * dt;
        }

        trans.translation += proj.velocity * dt;
        let proj_pos = trans.translation;

        // Check enemy collision if player source
        if proj.is_player_source {
            for (e_entity, enemy_trans, mut enemy) in enemies.iter_mut() {
                if enemy.state == EnemyAiState::Dying || enemy.state == EnemyAiState::Gibbed {
                    continue;
                }

                let dist_sq = proj_pos.distance_squared(enemy_trans.translation);
                let hit_radius_sq = if enemy.is_shrunk { 0.25 } else { 1.44 }; // 0.5^2 vs 1.2^2

                if dist_sq <= hit_radius_sq {
                    // Status effects
                    match proj.projectile_type {
                        ProjectileType::ShrinkRay => {
                            enemy.is_shrunk = true;
                            enemy.shrink_timer = 10.0;
                            enemy.speed *= 0.5;
                        }
                        ProjectileType::FreezeShard => {
                            if enemy.health - proj.damage <= 0 {
                                enemy.is_frozen = true;
                                enemy.freeze_timer = 15.0;
                                enemy.state = EnemyAiState::Frozen;
                            }
                        }
                        _ => {}
                    }

                    damage_events.send(EntityDamageEvent {
                        target: e_entity,
                        amount: proj.damage,
                        source: DamageSource::PlayerWeapon(proj.projectile_type),
                        hit_origin: proj_pos,
                    });

                    if proj.projectile_type == ProjectileType::Rocket || proj.projectile_type == ProjectileType::DevastatorMissile {
                        explosion_events.send(ExplosionDamageEvent {
                            origin: proj_pos,
                            radius: 5.0,
                            damage: proj.damage,
                        });
                    }

                    proj.lifetime = 0.0;
                    break;
                }
            }
        } else {
            // Enemy projectile hitting player
            for (p_entity, player_trans, _) in players.iter_mut() {
                let dist_sq = proj_pos.distance_squared(player_trans.translation);
                if dist_sq <= 1.0 {
                    damage_events.send(EntityDamageEvent {
                        target: p_entity,
                        amount: proj.damage,
                        source: DamageSource::EnemyWeapon(proj.projectile_type),
                        hit_origin: proj_pos,
                    });
                    proj.lifetime = 0.0;
                    break;
                }
            }
        }

        if proj.lifetime <= 0.0 {
            commands.entity(proj_entity).despawn_recursive();
        }
    }
}

pub fn apply_damage_events(
    mut damage_events: EventReader<EntityDamageEvent>,
    mut enemies: Query<(&Transform, &mut EnemyActor)>,
    mut players: Query<&mut PlayerController>,
    mut gib_events: EventWriter<GibEvent>,
) {
    for ev in damage_events.read() {
        if let Ok((enemy_trans, mut enemy)) = enemies.get_mut(ev.target) {
            if enemy.state == EnemyAiState::Frozen {
                enemy.health = -50;
                enemy.state = EnemyAiState::Gibbed;
                gib_events.send(GibEvent {
                    origin: enemy_trans.translation,
                    gib_count: 6,
                });
                continue;
            }

            enemy.health -= ev.amount;
            if enemy.health <= -40 {
                enemy.state = EnemyAiState::Gibbed;
                gib_events.send(GibEvent {
                    origin: enemy_trans.translation,
                    gib_count: 6,
                });
            } else if enemy.health <= 0 {
                enemy.state = EnemyAiState::Dying;
            } else {
                enemy.state = EnemyAiState::Flinching;
            }
        }

        if let Ok(mut player) = players.get_mut(ev.target) {
            player.health = player.health.saturating_sub(ev.amount);
        }
    }
}
