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
    mut enemies: Query<(&Transform, &mut EnemyActor), Without<Projectile>>,
    mut players: Query<(&Transform, &mut PlayerController), (Without<Projectile>, Without<EnemyActor>)>,
    mut explosion_events: EventWriter<ExplosionDamageEvent>,
    mut gib_events: EventWriter<GibEvent>,
) {
    let dt = time.delta_seconds();

    // Check walking squish on shrunk enemies by player
    for (player_trans, _) in players.iter() {
        for (enemy_trans, mut enemy) in enemies.iter_mut() {
            if enemy.is_shrunk && enemy.state != EnemyAiState::Gibbed && enemy.state != EnemyAiState::Dying {
                let dist = player_trans.translation.distance(enemy_trans.translation);
                if dist < 0.8 {
                    // Walking squish!
                    enemy.health = -50;
                    enemy.state = EnemyAiState::Gibbed;
                    gib_events.send(GibEvent {
                        origin: enemy_trans.translation,
                        gib_count: 6,
                    });
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

        // Check enemy collision if player source
        if proj.is_player_source {
            for (enemy_trans, mut enemy) in enemies.iter_mut() {
                if enemy.state == EnemyAiState::Dying || enemy.state == EnemyAiState::Gibbed {
                    continue;
                }

                let dist = trans.translation.distance(enemy_trans.translation);
                let hit_radius = if enemy.is_shrunk { 0.5 } else { 1.2 };

                if dist <= hit_radius {
                    // 1. Freezethrower Shatter Check
                    if enemy.state == EnemyAiState::Frozen {
                        enemy.health = -50;
                        enemy.state = EnemyAiState::Gibbed;
                        gib_events.send(GibEvent {
                            origin: enemy_trans.translation,
                            gib_count: 6,
                        });
                        proj.lifetime = 0.0;
                        break;
                    }

                    match proj.projectile_type {
                        ProjectileType::ShrinkRay => {
                            enemy.is_shrunk = true;
                            enemy.shrink_timer = 10.0;
                            enemy.speed *= 0.5;
                        }
                        ProjectileType::FreezeShard => {
                            enemy.health -= proj.damage;
                            if enemy.health <= 0 {
                                enemy.is_frozen = true;
                                enemy.freeze_timer = 15.0;
                                enemy.state = EnemyAiState::Frozen;
                            }
                        }
                        ProjectileType::MightyBoot => {
                            if enemy.is_shrunk {
                                // Instant squish kill!
                                enemy.health = -50;
                                enemy.state = EnemyAiState::Gibbed;
                                gib_events.send(GibEvent {
                                    origin: enemy_trans.translation,
                                    gib_count: 6,
                                });
                            } else {
                                enemy.health -= proj.damage;
                                enemy.state = EnemyAiState::Flinching;
                            }
                        }
                        _ => {
                            enemy.health -= proj.damage;
                            enemy.state = EnemyAiState::Flinching;
                        }
                    }

                    if proj.projectile_type == ProjectileType::Rocket || proj.projectile_type == ProjectileType::DevastatorMissile {
                        explosion_events.send(ExplosionDamageEvent {
                            origin: trans.translation,
                            radius: 5.0,
                            damage: proj.damage,
                        });
                    }

                    if enemy.health <= -40 {
                        enemy.state = EnemyAiState::Gibbed;
                        gib_events.send(GibEvent {
                            origin: enemy_trans.translation,
                            gib_count: 6,
                        });
                    } else if enemy.health <= 0 && enemy.state != EnemyAiState::Frozen {
                        enemy.state = EnemyAiState::Dying;
                    }

                    proj.lifetime = 0.0;
                    break;
                }
            }
        } else {
            // Enemy projectile hitting player
            for (player_trans, mut player) in players.iter_mut() {
                let dist = trans.translation.distance(player_trans.translation);
                if dist <= 1.0 {
                    player.health -= proj.damage;
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
