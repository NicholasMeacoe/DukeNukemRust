#![allow(dead_code)]

use bevy::prelude::*;
use bevy_rapier3d::prelude::*;
use crate::combat::types::*;
use crate::player::types::PlayerController;
use crate::scripting::{ConActor, ConScriptEngine, VmActorContext, move_flags, getincangle};

pub fn update_con_actors(
    time: Res<Time>,
    script_engine: Option<Res<ConScriptEngine>>,
    mut commands: Commands,
    mut actors: Query<(
        Entity,
        &mut Transform,
        &mut ConActor,
        Option<&mut EnemyActor>,
        Option<&mut crate::animation::AnimatedTileMaterial>,
        Option<&mut FlyingActor>,
        Option<&mut SituationalSpawn>,
    ), (Without<PlayerController>, Without<crate::combat::types::Projectile>)>,
    player_query: Query<(&Transform, &PlayerController), (Without<ConActor>, Without<crate::combat::types::Projectile>)>,
    projectiles: Query<&Transform, (With<crate::combat::types::Projectile>, Without<ConActor>, Without<PlayerController>)>,
    rapier_context: Option<Res<RapierContext>>,
    mut projectile_events: EventWriter<SpawnProjectileEvent>,
    mut sound_events: EventWriter<crate::audio::PlaySoundEvent>,
    mut duke_voice_events: EventWriter<crate::audio::PlayDukeVoiceEvent>,
    mut explosion_events: EventWriter<crate::interactivity::ExplosionDamageEvent>,
    mut gib_events: EventWriter<GibEvent>,
) {
    let Some(engine) = script_engine else { return; };
    let Ok((player_trans, player_ctrl)) = player_query.get_single() else { return; };

    let dt = time.delta_seconds();
    let p_pos = player_trans.translation;

    for (entity, mut trans, mut actor, mut enemy_opt, anim_opt, flying_opt, mut sit_opt) in actors.iter_mut() {
        if let Some(ref mut enemy) = enemy_opt {
            if enemy.is_frozen {
                enemy.freeze_timer -= dt;
                if enemy.freeze_timer <= 0.0 {
                    enemy.is_frozen = false;
                }
                continue;
            }
            if enemy.is_shrunk {
                enemy.shrink_timer -= dt;
                if enemy.shrink_timer <= 0.0 {
                    enemy.is_shrunk = false;
                }
            }
        }

        let dist_to_player = trans.translation.distance(p_pos);
        let dist_build = (dist_to_player * 1024.0) as i32;
        let dir_to_player = (p_pos - trans.translation).normalize_or_zero();

        // Line of sight raycast check
        let mut can_see = false;
        if dist_to_player <= 40.0 {
            can_see = true;
            if let Some(ref rapier) = rapier_context {
                let ray_origin = trans.translation + Vec3::Y * 0.5;
                let filter = QueryFilter::exclude_kinematic();
                if let Some((_hit_entity, toi)) = rapier.cast_ray(ray_origin, dir_to_player, dist_to_player, true, filter) {
                    if toi < dist_to_player - 0.5 {
                        can_see = false;
                    }
                }
            }
        }

        // Situational Spawn Dormancy check
        if let Some(ref mut sit) = sit_opt {
            if sit.is_dormant {
                if can_see || actor.last_hit_weapon != 0 || dist_to_player < 10.0 {
                    sit.is_dormant = false;
                } else {
                    continue;
                }
            }
        }

        // 3D Vertical Flight & Swimming Tracking
        if flying_opt.is_some() || matches!(enemy_opt.as_ref().map(|e| e.kind), Some(EnemyKind::Octabrain | EnemyKind::SentryDrone | EnemyKind::AssaultCommander | EnemyKind::Shark | EnemyKind::ReconCar)) {
            let target_y = p_pos.y + 0.3;
            let diff_y = target_y - trans.translation.y;
            trans.translation.y += diff_y.clamp(-3.5 * dt, 3.5 * dt);
        }

        // Convert world pos to Build units
        let mut sprite_x = (trans.translation.x * 1024.0) as i32;
        let mut sprite_y = (trans.translation.z * 1024.0) as i32;
        let mut sprite_z = -(trans.translation.y * 1024.0 * 16.0) as i32;

        let player_build_ang = ((-player_ctrl.yaw / std::f32::consts::TAU) * 2048.0) as i16;
        let actor_to_player_angle = ((-dir_to_player.z.atan2(dir_to_player.x) / std::f32::consts::TAU) * 2048.0) as i16;

        let player_facing_actor = getincangle(player_build_ang, actor_to_player_angle).abs() < 128;

        let ConActor {
            ref mut registers,
            ref mut ang,
            ref mut xvel,
            ref mut zvel,
            ref mut extra,
            ref mut picnum,
            ref mut sectnum,
            ref mut cstat,
            ref mut pal,
            ref mut xrepeat,
            ref mut yrepeat,
            ref mut clipdist,
            ref mut lotag,
            ref mut hitag,
            ref mut last_hit_weapon,
            spawned_by_picnum,
        } = *actor;

        let last_hit = *last_hit_weapon;

        let mut bullet_near = false;
        let actor_pos = trans.translation;
        for proj_trans in projectiles.iter() {
            let p = proj_trans.translation;
            if (p.x - actor_pos.x).abs() < 5.0 && (p.z - actor_pos.z).abs() < 5.0 && (p.y - actor_pos.y).abs() < 5.0 {
                if actor_pos.distance_squared(p) < 25.0 {
                    bullet_near = true;
                    break;
                }
            }
        }
        let can_shoot_target = can_see && dist_to_player <= 30.0;
        let not_moving = *xvel == 0 && *zvel == 0;

        let mut ctx = VmActorContext {
            sprite_idx: entity.index() as usize,
            player_idx: 0,
            dist_to_player: dist_build,
            can_see_player: can_see,
            hit_by_weapon: last_hit != 0,
            registers,
            sprite_x: &mut sprite_x,
            sprite_y: &mut sprite_y,
            sprite_z: &mut sprite_z,
            sprite_ang: ang,
            sprite_xvel: xvel,
            sprite_zvel: zvel,
            sprite_extra: extra,
            sprite_picnum: picnum,
            sprite_sectnum: sectnum,
            sprite_cstat: cstat,
            sprite_pal: pal,
            sprite_xrepeat: xrepeat,
            sprite_yrepeat: yrepeat,
            sprite_clipdist: clipdist,
            sprite_lotag: lotag,
            sprite_hitag: hitag,
            killit_flag: false,
            spawned_sprites: Vec::new(),
            sound_events: Vec::new(),
            quotes_displayed: Vec::new(),
            pal_flashes: Vec::new(),
            player_health_delta: 0,
            player_ammo_deltas: Vec::new(),
            player_inventory_deltas: Vec::new(),
            debris_events: Vec::new(),
            hitradius_events: Vec::new(),
            player_health: player_ctrl.health,
            player_ang: player_build_ang,
            player_on_ground: player_ctrl.movement_mode != crate::player::types::PlayerMovementMode::JetpackFlying,
            player_jumping_counter: 0,
            player_posz_velocity: 0,
            player_crouching: player_ctrl.movement_mode == crate::player::types::PlayerMovementMode::Crouching,
            player_xvel: player_ctrl.speed as i32,
            player_running: player_ctrl.speed > 12.0,
            player_quick_kick: if player_ctrl.quick_kick_timer > 0.0 { 1 } else { 0 },
            player_shrunk: player_ctrl.shrink_timer > 0.0,
            player_jetpack_on: player_ctrl.inventory.jetpack_active,
            player_steroids_active: player_ctrl.inventory.steroids_active,
            player_dead: player_ctrl.health <= 0,
            player_weapon: player_ctrl.current_weapon as i32,
            player_kickback: 0,
            player_facing_actor,
            player_steroids_amount: player_ctrl.inventory.steroids_amount,
            player_shield_amount: player_ctrl.armor,
            player_scuba_amount: player_ctrl.inventory.scuba_amount,
            player_holoduke_amount: player_ctrl.inventory.holoduke_amount,
            player_jetpack_amount: player_ctrl.inventory.jetpack_amount,
            player_heat_amount: player_ctrl.inventory.nightvision_amount,
            player_firstaid_amount: player_ctrl.inventory.medkit_amount,
            player_boot_amount: player_ctrl.inventory.boots_amount,
            player_got_access: 0,
            sector_lotag: 0,
            sector_ceilingstat: 0,
            is_multiplayer: false,
            hit_space_pressed: false,
            spawned_by_picnum,
            last_hit_weapon: last_hit,
            can_shoot_target,
            bullet_near,
            not_moving,
            shoot_events: Vec::new(),
            end_of_game: None,
        };

        // Execute VM Tick!
        engine.vm.execute(&mut ctx);

        // Reset hit weapon trigger after tick
        *last_hit_weapon = 0;

        let killit = ctx.killit_flag;
        let shoot_events = std::mem::take(&mut ctx.shoot_events);
        let sound_events_out = std::mem::take(&mut ctx.sound_events);
        let debris_events = std::mem::take(&mut ctx.debris_events);
        let hitradius_events = std::mem::take(&mut ctx.hitradius_events);
        let end_of_game = ctx.end_of_game;
        drop(ctx);

        if let Some(_delay) = end_of_game {
            duke_voice_events.send(crate::audio::PlayDukeVoiceEvent { name: None });
            commands.add(|world: &mut World| {
                world.send_event(crate::game_flow::LevelCompletedEvent);
            });
        }

        // Apply death / killit
        if killit {
            commands.entity(entity).despawn_recursive();
            continue;
        }

        // Apply health sync & specialized death mechanics
        if let Some(ref mut enemy) = enemy_opt {
            let was_alive = enemy.health > 0;
            enemy.health = *extra as i32;

            if was_alive && enemy.health <= 0 {
                // Recon car pilot ejection
                if enemy.kind == EnemyKind::ReconCar {
                    let eject_pos = trans.translation + Vec3::Y * 0.5;
                    commands.spawn((
                        EnemyActor::new_pigcop(),
                        ConActor::new(2000, *sectnum, *ang, 100),
                        TransformBundle::from_transform(Transform::from_translation(eject_pos)),
                        crate::game_flow::LevelEntity,
                    ));
                }
            }

            // Lethal Boss Stomp
            if matches!(enemy.kind, EnemyKind::Boss1Battlelord | EnemyKind::Boss1Mini | EnemyKind::Boss2Overlord | EnemyKind::Boss3Cycloid | EnemyKind::Boss4Queen) {
                if dist_to_player <= 1.25 && player_ctrl.health > 0 {
                    projectile_events.send(SpawnProjectileEvent {
                        projectile_type: ProjectileType::MightyBoot,
                        origin: trans.translation,
                        direction: dir_to_player,
                        velocity: 10.0,
                        damage: 1000,
                        is_player_source: false,
                    });
                }
            }

            // Slimer facehugger attack
            if enemy.kind == EnemyKind::ProtozoidSlimer && dist_to_player <= 0.8 && player_ctrl.health > 0 {
                enemy.attack_timer += dt;
                if enemy.attack_timer >= 0.5 {
                    enemy.attack_timer = 0.0;
                    projectile_events.send(SpawnProjectileEvent {
                        projectile_type: ProjectileType::MightyBoot,
                        origin: trans.translation,
                        direction: dir_to_player,
                        velocity: 5.0,
                        damage: 5,
                        is_player_source: false,
                    });
                }
            }

            // Rat scampering behavior (flees away from player within 5.0m)
            if enemy.kind == EnemyKind::ScamperingRat && dist_to_player <= 5.0 {
                let flee_dir = -dir_to_player.with_y(0.0).normalize_or_zero();
                trans.translation += flee_dir * (enemy.speed * dt);
            }

            // Turret tracking and continuous firing
            if enemy.kind == EnemyKind::Turret && can_see && dist_to_player <= 30.0 {
                enemy.attack_timer += dt;
                if enemy.attack_timer >= enemy.attack_cooldown {
                    enemy.attack_timer = 0.0;
                    projectile_events.send(SpawnProjectileEvent {
                        projectile_type: ProjectileType::AlienBlaster,
                        origin: trans.translation + Vec3::NEG_Y * 0.3 + dir_to_player * 0.4,
                        direction: dir_to_player,
                        velocity: 35.0,
                        damage: 15,
                        is_player_source: false,
                    });
                    sound_events.send(crate::audio::PlaySoundEvent { sound_id: 110 });
                }
            }
        }

        // Handle Shoot Events
        for (tile, _, _, _, ang_shot) in shoot_events {
            let shoot_ang_rad = -(ang_shot as f32 / 2048.0) * std::f32::consts::TAU;
            let fire_dir = Vec3::new(shoot_ang_rad.cos(), 0.0, shoot_ang_rad.sin()).normalize_or_zero();
            let fire_origin = trans.translation + Vec3::Y * 0.4 + fire_dir * 0.4;

            let (proj_type, vel, dmg) = map_tile_to_projectile(tile);
            if proj_type == ProjectileType::ShotgunPellet {
                for _ in 0..7 {
                    let spread = Vec3::new(
                        (rand::random::<f32>() - 0.5) * 0.08,
                        (rand::random::<f32>() - 0.5) * 0.08,
                        (rand::random::<f32>() - 0.5) * 0.08,
                    );
                    projectile_events.send(SpawnProjectileEvent {
                        projectile_type: proj_type,
                        origin: fire_origin,
                        direction: (fire_dir + spread).normalize_or_zero(),
                        velocity: vel,
                        damage: dmg,
                        is_player_source: false,
                    });
                }
            } else {
                projectile_events.send(SpawnProjectileEvent {
                    projectile_type: proj_type,
                    origin: fire_origin,
                    direction: fire_dir,
                    velocity: vel,
                    damage: dmg,
                    is_player_source: false,
                });
            }
        }

        // Handle Sound Events
        for (sound_id, _once) in sound_events_out {
            sound_events.send(crate::audio::PlaySoundEvent { sound_id });
        }

        // Handle Gib / Debris Events
        for (_tile, count) in debris_events {
            gib_events.send(GibEvent {
                origin: trans.translation,
                gib_count: (count as usize).clamp(1, 8),
            });
        }

        // Handle Hitradius Events
        for (radius, dmg, _, _, _) in hitradius_events {
            explosion_events.send(crate::interactivity::ExplosionDamageEvent {
                origin: trans.translation,
                radius: radius as f32 / 1024.0,
                damage: dmg,
            });
        }

        // Movement application from active MoveDef & AI flags
        if let Some(move_ptr) = registers.move_ptr {
            if move_ptr + 1 < engine.compiled.bytecode.len() {
                let hvel = engine.compiled.bytecode[move_ptr];
                let flags = *hitag as i32;

                if (flags & move_flags::FACE_PLAYER) != 0 || (flags & move_flags::SEEK_PLAYER) != 0 {
                    *ang = actor_to_player_angle;
                    let look_dir = dir_to_player.with_y(0.0);
                    if look_dir.length_squared() > 0.001 {
                        trans.look_to(look_dir, Vec3::Y);
                    }
                }

                if hvel > 0 {
                    let move_speed = (hvel as f32) / 8.0;
                    let ang_rad = -(*ang as f32 / 2048.0) * std::f32::consts::TAU;
                    let move_dir = Vec3::new(ang_rad.cos(), 0.0, ang_rad.sin());
                    let step_vec = move_dir * move_speed * dt;
                    let step_dist = step_vec.length();

                    let mut can_step = true;
                    let mut actual_step = step_vec;

                    if let Some(ref rapier) = rapier_context {
                        if step_dist > 0.001 {
                            let ray_dir = step_vec / step_dist;
                            let right_vec = Vec3::new(-ray_dir.z, 0.0, ray_dir.x) * 0.25;
                            let ray_center = trans.translation + Vec3::Y * 0.4;
                            let origins = [ray_center, ray_center + right_vec, ray_center - right_vec];
                            let filter = QueryFilter::only_fixed();

                            for ray_origin in origins {
                                if let Some((_hit_entity, intersection)) = rapier.cast_ray_and_get_normal(
                                    ray_origin,
                                    ray_dir,
                                    step_dist + 0.35,
                                    true,
                                    filter,
                                ) {
                                    let n = intersection.normal.with_y(0.0).normalize_or_zero();
                                    if n.length_squared() > 0.01 {
                                        let tangent_step = step_vec - step_vec.dot(n) * n;
                                        actual_step = tangent_step;
                                    } else {
                                        can_step = false;
                                    }
                                    break;
                                }
                            }
                        }
                    }

                    if can_step {
                        trans.translation += actual_step;
                    }
                }
            }
        }

        // Sync frame animation offset if AnimatedTileMaterial is present
        if let Some(mut anim) = anim_opt {
            anim.current_offset = registers.frame_offset;
        }
    }
}

pub fn map_tile_to_projectile(tile: i16) -> (ProjectileType, f32, i32) {
    match tile {
        1625 => (ProjectileType::AlienBlaster, 50.0, 7),     // FIRELASER
        1636 => (ProjectileType::Spit, 35.0, 8),             // SPIT (Enforcer venom)
        1641 => (ProjectileType::FreezeShard, 45.0, 20),     // FREEZEBLAST
        1646 | 2556 => (ProjectileType::ShrinkRay, 40.0, 0), // SHRINKSPARK / SHRINKER
        1650 => (ProjectileType::Mortar, 30.0, 50),          // MORTER (Battlelord / Tank artillery)
        2595 => (ProjectileType::HitscanBullet, 150.0, 9),   // SHOTSPARK1 (Chaingun / Enforcer / Battlelord)
        2605 => (ProjectileType::Rocket, 45.0, 140),         // RPG (Commander / Overlord / Cycloid)
        2613 => (ProjectileType::ShotgunPellet, 80.0, 10),   // SHOTGUN (Pigcop)
        1360 => (ProjectileType::PsiBlast, 30.0, 38),        // COOLEXPLOSION1 (Octabrain)
        _ => (ProjectileType::HitscanBullet, 100.0, 10),
    }
}

