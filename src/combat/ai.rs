#![allow(dead_code)]

use bevy::prelude::*;
use bevy_rapier3d::prelude::*;
use crate::combat::types::*;
use crate::player::types::PlayerController;

pub fn update_enemy_ai(
    time: Res<Time>,
    mut enemies: Query<(&mut Transform, &mut EnemyActor)>,
    player_query: Query<&Transform, (With<PlayerController>, Without<EnemyActor>)>,
    rapier_context: Option<Res<RapierContext>>,
    mut projectile_events: EventWriter<SpawnProjectileEvent>,
) {
    let dt = time.delta_seconds();
    let Ok(player_trans) = player_query.get_single() else { return; };
    let player_pos = player_trans.translation;

    for (mut trans, mut enemy) in enemies.iter_mut() {
        if enemy.is_frozen {
            enemy.freeze_timer -= dt;
            if enemy.freeze_timer <= 0.0 {
                enemy.is_frozen = false;
                enemy.state = EnemyAiState::Idle;
            }
            continue;
        }

        if enemy.is_shrunk {
            enemy.shrink_timer -= dt;
            if enemy.shrink_timer <= 0.0 {
                enemy.is_shrunk = false;
                enemy.speed *= 2.0; // Restore speed
            }
        }

        if enemy.state == EnemyAiState::Dying || enemy.state == EnemyAiState::Gibbed {
            continue;
        }

        let dist_to_player = trans.translation.distance(player_pos);
        let dir_to_player = (player_pos - trans.translation).normalize_or_zero();

        // Safe look rotation (prevents panic if directly vertical)
        let look_dir = dir_to_player.with_y(0.0);
        if look_dir.length_squared() > 0.001 {
            trans.look_to(look_dir, Vec3::Y);
        }

        // Line of sight raycast check
        let mut has_los = true;
        if let Some(ref rapier) = rapier_context {
            let ray_origin = trans.translation + Vec3::Y * 0.5;
            let ray_dir = dir_to_player;
            let max_toi = dist_to_player;
            let filter = QueryFilter::exclude_kinematic();

            if let Some((_entity, toi)) = rapier.cast_ray(ray_origin, ray_dir, max_toi, true, filter) {
                if toi < dist_to_player - 0.5 {
                    has_los = false; // Solid wall blocks line of sight
                }
            }
        }

        // Update attack timer
        if enemy.attack_timer > 0.0 {
            enemy.attack_timer -= dt;
        }

        match enemy.state {
            EnemyAiState::Idle => {
                if dist_to_player <= enemy.sight_radius && has_los {
                    enemy.state = EnemyAiState::Seeking;
                }
            }
            EnemyAiState::Patrol => {
                if dist_to_player <= enemy.sight_radius && has_los {
                    enemy.state = EnemyAiState::Seeking;
                }
            }
            EnemyAiState::Seeking => {
                if dist_to_player <= enemy.attack_range && has_los {
                    enemy.state = EnemyAiState::Attacking;
                } else if dist_to_player > enemy.sight_radius * 1.5 {
                    enemy.state = EnemyAiState::Idle;
                } else {
                    // Move towards player
                    trans.translation += dir_to_player * enemy.speed * dt;
                }
            }
            EnemyAiState::Attacking => {
                if !has_los {
                    enemy.state = EnemyAiState::Seeking;
                    continue;
                }

                if enemy.attack_timer <= 0.0 {
                    enemy.attack_timer = enemy.attack_cooldown;

                    let fire_origin = trans.translation + Vec3::Y * 0.4 + dir_to_player * 0.5;

                    match enemy.kind {
                        EnemyKind::Pigcop => {
                            // Pigcop fires authentic 7-pellet shotgun blast
                            for _ in 0..7 {
                                let spread = Vec3::new(
                                    (rand::random::<f32>() - 0.5) * 0.08,
                                    (rand::random::<f32>() - 0.5) * 0.08,
                                    (rand::random::<f32>() - 0.5) * 0.08,
                                );
                                projectile_events.send(SpawnProjectileEvent {
                                    projectile_type: ProjectileType::ShotgunPellet,
                                    origin: fire_origin,
                                    direction: (dir_to_player + spread).normalize_or_zero(),
                                    velocity: 80.0,
                                    damage: 8,
                                    is_player_source: false,
                                });
                            }
                        }
                        EnemyKind::Liztroop => {
                            // Trooper blaster
                            projectile_events.send(SpawnProjectileEvent {
                                projectile_type: ProjectileType::AlienBlaster,
                                origin: fire_origin,
                                direction: dir_to_player,
                                velocity: 40.0,
                                damage: 15,
                                is_player_source: false,
                            });
                        }
                        EnemyKind::Octabrain => {
                            // Psi-blast
                            projectile_events.send(SpawnProjectileEvent {
                                projectile_type: ProjectileType::PsiBlast,
                                origin: fire_origin,
                                direction: dir_to_player,
                                velocity: 25.0,
                                damage: 25,
                                is_player_source: false,
                            });
                        }
                        EnemyKind::Enforcer => {
                            // Chaingun stream
                            projectile_events.send(SpawnProjectileEvent {
                                projectile_type: ProjectileType::HitscanBullet,
                                origin: fire_origin,
                                direction: dir_to_player,
                                velocity: 120.0,
                                damage: 10,
                                is_player_source: false,
                            });
                        }
                        _ => {}
                    }
                }

                if dist_to_player > enemy.attack_range * 1.2 {
                    enemy.state = EnemyAiState::Seeking;
                }
            }
            EnemyAiState::Flinching => {
                if enemy.attack_timer <= 0.0 {
                    enemy.state = EnemyAiState::Seeking;
                }
            }
            _ => {}
        }
    }
}
