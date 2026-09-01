#![allow(dead_code)]

use bevy::prelude::*;
use bevy_rapier3d::prelude::*;
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

        let bounces = match ev.projectile_type {
            ProjectileType::FreezeShard => 3,
            ProjectileType::Pipebomb => 4,
            _ => 0,
        };

        commands.spawn((
            Projectile {
                projectile_type: ev.projectile_type,
                velocity: vel,
                damage: ev.damage,
                is_player_source: ev.is_player_source,
                lifetime,
                bounces,
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
    mut sound_events: EventWriter<crate::audio::PlaySoundEvent>,
    mut decal_events: EventWriter<crate::combat::decals::SpawnDecalEvent>,
    rapier_context: Option<Res<RapierContext>>,
) {
    let dt = time.delta_seconds();

    // Fast squared distance walking squish on shrunk enemies by player
    for (_, player_trans, _) in players.iter() {
        let p_pos = player_trans.translation;
        for (e_entity, enemy_trans, mut enemy) in enemies.iter_mut() {
            if enemy.is_shrunk && enemy.state != EnemyAiState::Gibbed && enemy.state != EnemyAiState::Dying {
                let e_pos = enemy_trans.translation;
                let horizontal_dist_sq = (p_pos.x - e_pos.x).powi(2) + (p_pos.z - e_pos.z).powi(2);
                let vertical_dist = (p_pos.y - e_pos.y).abs();
                if horizontal_dist_sq < 0.64 && vertical_dist < 1.0 {
                    damage_events.send(EntityDamageEvent {
                        target: e_entity,
                        amount: 1000,
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

        let old_pos = trans.translation;
        let step_vec = proj.velocity * dt;
        let step_dist = step_vec.length();
        let mut hit_wall = false;
        let mut hit_point = old_pos + step_vec;
        let mut hit_normal = Vec3::Y;

        if let Some(ref rapier) = rapier_context {
            if step_dist > 0.001 {
                let ray_dir = step_vec / step_dist;
                let filter = QueryFilter::only_fixed();
                if let Some((_hit_entity, intersection)) = rapier.cast_ray_and_get_normal(
                    old_pos,
                    ray_dir,
                    step_dist,
                    true,
                    filter,
                ) {
                    hit_wall = true;
                    hit_point = intersection.point;
                    hit_normal = intersection.normal;
                }
            }
        }

        if hit_wall {
            trans.translation = hit_point;

            match proj.projectile_type {
                ProjectileType::Rocket | ProjectileType::DevastatorMissile | ProjectileType::Mortar => {
                    explosion_events.send(ExplosionDamageEvent {
                        origin: hit_point,
                        radius: 5.0,
                        damage: proj.damage,
                    });
                    decal_events.send(crate::combat::decals::SpawnDecalEvent {
                        origin: hit_point,
                        normal: hit_normal,
                        decal_type: crate::combat::decals::DecalType::ScorchMark,
                    });
                    sound_events.send(crate::audio::PlaySoundEvent { sound_id: 115 });
                    proj.lifetime = 0.0;
                }
                ProjectileType::FreezeShard => {
                    if proj.bounces > 0 {
                        proj.bounces -= 1;
                        let vel_dot = proj.velocity.dot(hit_normal);
                        proj.velocity = (proj.velocity - 2.0 * vel_dot * hit_normal) * 0.8;
                        trans.translation = hit_point + hit_normal * 0.05;
                    } else {
                        proj.lifetime = 0.0;
                        sound_events.send(crate::audio::PlaySoundEvent { sound_id: 118 });
                    }
                }
                ProjectileType::Pipebomb => {
                    if proj.bounces > 0 {
                        proj.bounces -= 1;
                        let vel_dot = proj.velocity.dot(hit_normal);
                        proj.velocity = (proj.velocity - 2.0 * vel_dot * hit_normal) * 0.6;
                        trans.translation = hit_point + hit_normal * 0.05;
                        sound_events.send(crate::audio::PlaySoundEvent { sound_id: 111 });
                    } else {
                        proj.velocity = Vec3::ZERO;
                    }
                }
                _ => {
                    decal_events.send(crate::combat::decals::SpawnDecalEvent {
                        origin: hit_point,
                        normal: hit_normal,
                        decal_type: crate::combat::decals::DecalType::BulletHole,
                    });
                    proj.lifetime = 0.0;
                    sound_events.send(crate::audio::PlaySoundEvent { sound_id: 110 });
                }
            }
        } else {
            trans.translation += step_vec;
        }

        let new_pos = trans.translation;

        // Check enemy continuous swept collision if player source
        if proj.is_player_source {
            for (e_entity, enemy_trans, mut enemy) in enemies.iter_mut() {
                if enemy.state == EnemyAiState::Dying || enemy.state == EnemyAiState::Gibbed {
                    continue;
                }

                let enemy_center = enemy_trans.translation + Vec3::Y * 0.4;
                let dist_sq = dist_sq_point_to_segment(enemy_center, old_pos, new_pos);
                let hit_radius_sq = if enemy.is_shrunk { 0.25 } else { 1.44 }; // 0.5^2 vs 1.2^2

                if dist_sq <= hit_radius_sq {
                    // Status effects
                    match proj.projectile_type {
                        ProjectileType::ShrinkRay => {
                            enemy.is_shrunk = true;
                            enemy.shrink_timer = 9.0;
                            enemy.speed *= 0.5;
                        }
                        ProjectileType::FreezeShard => {
                            if enemy.health - proj.damage <= 0 {
                                enemy.is_frozen = true;
                                enemy.freeze_timer = 4.6;
                                enemy.state = EnemyAiState::Frozen;
                            }
                        }
                        ProjectileType::ExpanderRay => {
                            if enemy.health - proj.damage <= 0 {
                                enemy.is_expanding = true;
                                enemy.expand_timer = 1.0;
                                enemy.state = EnemyAiState::Expanding;
                            }
                        }
                        _ => {}
                    }

                    damage_events.send(EntityDamageEvent {
                        target: e_entity,
                        amount: proj.damage,
                        source: DamageSource::PlayerWeapon(proj.projectile_type),
                        hit_origin: new_pos,
                    });

                    decal_events.send(crate::combat::decals::SpawnDecalEvent {
                        origin: new_pos,
                        normal: Vec3::Y,
                        decal_type: crate::combat::decals::DecalType::BloodSplatter,
                    });

                    if proj.projectile_type == ProjectileType::Rocket || proj.projectile_type == ProjectileType::DevastatorMissile {
                        explosion_events.send(ExplosionDamageEvent {
                            origin: new_pos,
                            radius: 5.0,
                            damage: proj.damage,
                        });
                    }

                    proj.lifetime = 0.0;
                    break;
                }
            }
        } else {
            // Enemy projectile continuous swept collision hitting player
            for (p_entity, player_trans, _) in players.iter_mut() {
                let player_center = player_trans.translation + Vec3::Y * 0.5;
                let dist_sq = dist_sq_point_to_segment(player_center, old_pos, new_pos);
                if dist_sq <= 1.0 {
                    damage_events.send(EntityDamageEvent {
                        target: p_entity,
                        amount: proj.damage,
                        source: DamageSource::EnemyWeapon(proj.projectile_type),
                        hit_origin: new_pos,
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

pub fn update_enemy_status_effects(
    time: Res<Time>,
    mut enemies: Query<(&mut Transform, &mut EnemyActor)>,
    mut gib_events: EventWriter<GibEvent>,
    mut explosion_events: EventWriter<ExplosionDamageEvent>,
    mut sound_events: EventWriter<crate::audio::PlaySoundEvent>,
) {
    let dt = time.delta_seconds();

    for (mut trans, mut enemy) in enemies.iter_mut() {
        if enemy.is_shrunk {
            enemy.shrink_timer -= dt;
            if enemy.shrink_timer > 0.0 {
                trans.scale = Vec3::splat(0.25);
            } else {
                enemy.is_shrunk = false;
                trans.scale = Vec3::splat(1.0);
            }
        }

        if enemy.is_frozen {
            enemy.freeze_timer -= dt;
            if enemy.freeze_timer <= 0.0 {
                enemy.is_frozen = false;
                enemy.state = EnemyAiState::Seeking;
                enemy.health = 1;
            }
        }

        if enemy.is_expanding {
            enemy.expand_timer -= dt;
            let progress = (1.0 - enemy.expand_timer).clamp(0.0, 1.0);
            trans.scale = Vec3::splat(1.0 + progress * 0.75); // Expands up to 1.75x

            if enemy.expand_timer <= 0.0 {
                enemy.is_expanding = false;
                enemy.state = EnemyAiState::Gibbed;
                enemy.health = -100;

                // Violent pop explosion
                sound_events.send(crate::audio::PlaySoundEvent { sound_id: 14 }); // PIPEBOMB_EXPLODE
                sound_events.send(crate::audio::PlaySoundEvent { sound_id: 69 }); // SQUISHED
                gib_events.send(GibEvent {
                    origin: trans.translation,
                    gib_count: 12,
                });
                explosion_events.send(ExplosionDamageEvent {
                    origin: trans.translation,
                    radius: 4.0,
                    damage: 80,
                });
            }
        }
    }
}

pub fn apply_damage_events(
    mut damage_events: EventReader<EntityDamageEvent>,
    mut enemies: Query<(&Transform, &mut EnemyActor)>,
    mut players: Query<&mut PlayerController>,
    mut gib_events: EventWriter<GibEvent>,
    mut sound_events: EventWriter<crate::audio::PlaySoundEvent>,
    mut duke_voice_events: EventWriter<crate::audio::PlayDukeVoiceEvent>,
) {
    for ev in damage_events.read() {
        if let Ok((enemy_trans, mut enemy)) = enemies.get_mut(ev.target) {
            if enemy.state == EnemyAiState::Gibbed {
                continue;
            }

            if enemy.state == EnemyAiState::Frozen {
                enemy.health = -50;
                enemy.state = EnemyAiState::Gibbed;
                sound_events.send(crate::audio::PlaySoundEvent { sound_id: 19 }); // GLASS_BREAKING
                gib_events.send(GibEvent {
                    origin: enemy_trans.translation,
                    gib_count: 8,
                });
                continue;
            }

            enemy.health -= ev.amount;
            if enemy.health <= -40 {
                enemy.state = EnemyAiState::Gibbed;
                sound_events.send(crate::audio::PlaySoundEvent { sound_id: 69 }); // SQUISHED
                gib_events.send(GibEvent {
                    origin: enemy_trans.translation,
                    gib_count: 6,
                });
                // 30% chance for Duke to drop a one-liner on gib kill
                if rand::random::<f32>() < 0.3 {
                    duke_voice_events.send(crate::audio::PlayDukeVoiceEvent { name: None });
                }
            } else if enemy.health <= 0 {
                enemy.state = EnemyAiState::Dying;
                let is_boss = matches!(
                    enemy.kind,
                    EnemyKind::Boss1Battlelord
                        | EnemyKind::Boss2Overlord
                        | EnemyKind::Boss3Cycloid
                        | EnemyKind::Boss4Queen
                );

                if is_boss {
                    duke_voice_events.send(crate::audio::PlayDukeVoiceEvent {
                        name: Some("REST_IN_PIECES".into()),
                    });
                } else if rand::random::<f32>() < 0.2 {
                    duke_voice_events.send(crate::audio::PlayDukeVoiceEvent { name: None });
                }

                let die_snd = match enemy.kind {
                    EnemyKind::Pigcop => 539,
                    EnemyKind::Liztroop => 512,
                    EnemyKind::Octabrain => 573,
                    _ => 512,
                };
                sound_events.send(crate::audio::PlaySoundEvent { sound_id: die_snd });
            } else {
                enemy.state = EnemyAiState::Flinching;
                let pain_snd = match enemy.kind {
                    EnemyKind::Pigcop => 538,
                    EnemyKind::Liztroop => 511,
                    EnemyKind::Octabrain => 572,
                    _ => 511,
                };
                sound_events.send(crate::audio::PlaySoundEvent { sound_id: pain_snd });
            }
        }

        if let Ok(mut player) = players.get_mut(ev.target) {
            player.health = player.health.saturating_sub(ev.amount);
            if player.health == 0 {
                sound_events.send(crate::audio::PlaySoundEvent { sound_id: 41 }); // DUKE_DEAD
            } else {
                sound_events.send(crate::audio::PlaySoundEvent { sound_id: 37 }); // DUKE_PAIN
            }
        }
    }
}

#[inline]
pub fn dist_sq_point_to_segment(point: Vec3, seg_a: Vec3, seg_b: Vec3) -> f32 {
    let ab = seg_b - seg_a;
    let ab_len_sq = ab.length_squared();
    if ab_len_sq < 0.00001 {
        return point.distance_squared(seg_a);
    }
    let t = ((point - seg_a).dot(ab) / ab_len_sq).clamp(0.0, 1.0);
    let closest = seg_a + ab * t;
    point.distance_squared(closest)
}
