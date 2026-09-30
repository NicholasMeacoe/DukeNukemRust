use crate::combat::types::*;
use crate::interactivity::SafeDespawnExt;
use crate::player::types::PlayerController;
use crate::scripting::{getincangle, move_flags, ConActor, ConScriptEngine, VmActorContext};
use crate::names::*;
use bevy::prelude::*;
use bevy_rapier3d::prelude::*;

pub fn update_con_actors(
    time: Res<Time>,
    script_engine: Option<Res<ConScriptEngine>>,
    mut commands: Commands,
    mut actors: Query<
        (
            Entity,
            &mut Transform,
            &mut ConActor,
            Option<&mut EnemyActor>,
            Option<&mut crate::animation::AnimatedTileMaterial>,
            Option<&mut FlyingActor>,
            Option<&mut SituationalSpawn>,
            Option<&mut KinematicCharacterController>,
        ),
        (
            Without<PlayerController>,
            Without<crate::combat::types::Projectile>,
        ),
    >,
    player_query: Query<
        (&Transform, &PlayerController),
        (Without<ConActor>, Without<crate::combat::types::Projectile>),
    >,
    projectiles: Query<
        &Transform,
        (
            With<crate::combat::types::Projectile>,
            Without<ConActor>,
            Without<PlayerController>,
        ),
    >,
    rapier_context: Option<Res<RapierContext>>,
    (
        mut projectile_events,
        mut sound_events,
        mut duke_voice_events,
        mut explosion_events,
        mut gib_events,
    ): (
        EventWriter<SpawnProjectileEvent>,
        EventWriter<crate::audio::PlaySoundEvent>,
        EventWriter<crate::audio::PlayDukeVoiceEvent>,
        EventWriter<crate::interactivity::ExplosionDamageEvent>,
        EventWriter<GibEvent>,
    ),
    mut rng: ResMut<crate::net::DeterministicRng>,
    sector_map: Option<Res<crate::sector_map::SectorMap>>,
    mut camera_shake: Option<ResMut<crate::interactivity::EarthquakeCameraShake>>,
    level_progress: Option<Res<crate::game_flow::LevelProgress>>,
    mut screen_tint: Option<ResMut<crate::hud::ScreenTintState>>,
) {
    let Some(engine) = script_engine else {
        return;
    };
    let Ok((player_trans, player_ctrl)) = player_query.get_single() else {
        return;
    };

    let dt = time.delta_seconds();
    let p_pos = player_trans.translation;

    for (
        entity,
        mut trans,
        mut actor,
        mut enemy_opt,
        anim_opt,
        flying_opt,
        mut sit_opt,
        mut kcc_opt,
    ) in actors.iter_mut()
    {
        if let Some(ref mut enemy) = enemy_opt {
            if enemy.is_frozen || enemy.state == EnemyAiState::Frozen {
                enemy.freeze_timer -= dt;
                if enemy.freeze_timer <= 0.0 {
                    enemy.is_frozen = false;
                    enemy.state = EnemyAiState::Seeking;
                    enemy.health = 1;
                }
                continue;
            }
            if enemy.is_shrunk || enemy.state == EnemyAiState::Shrunk {
                enemy.shrink_timer -= dt;
                if enemy.shrink_timer <= 0.0 {
                    enemy.is_shrunk = false;
                    if enemy.state == EnemyAiState::Shrunk {
                        enemy.state = EnemyAiState::Seeking;
                    }
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
                let filter =
                    QueryFilter::new().groups(CollisionGroups::new(Group::ALL, Group::GROUP_1));
                if let Some((_hit_entity, toi)) =
                    rapier.cast_ray(ray_origin, dir_to_player, dist_to_player, true, filter)
                {
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
        let mut vertical_movement = 0.0;
        let is_flying = flying_opt.is_some()
            || matches!(
                enemy_opt.as_ref().map(|e| e.kind),
                Some(
                    EnemyKind::Octabrain
                        | EnemyKind::SentryDrone
                        | EnemyKind::AssaultCommander
                        | EnemyKind::Shark
                        | EnemyKind::ReconCar
                )
            );

        if is_flying {
            let target_y = p_pos.y + 0.3;
            let diff_y = target_y - trans.translation.y;
            vertical_movement = diff_y.clamp(-3.5 * dt, 3.5 * dt);
        } else if let Some(ref mut enemy) = enemy_opt {
            enemy.velocity.y -= 18.0 * dt;
            enemy.velocity.y = enemy.velocity.y.clamp(-14.0, 7.0);
            vertical_movement = enemy.velocity.y * dt;
        }

        // Convert world pos to Build units
        let mut sprite_x = (trans.translation.x * 1024.0) as i32;
        let mut sprite_y = (trans.translation.z * 1024.0) as i32;
        let mut sprite_z = -(trans.translation.y * 1024.0 * 16.0) as i32;

        let player_build_ang = ((-player_ctrl.yaw / std::f32::consts::TAU) * 2048.0) as i16;
        let actor_to_player_angle =
            ((-dir_to_player.z.atan2(dir_to_player.x) / std::f32::consts::TAU) * 2048.0) as i16;

        let player_facing_actor = getincangle(player_build_ang, actor_to_player_angle).abs() < 128;

        // Shrunk enemy boot-stomp / touch squish check
        if let Some(ref mut enemy) = enemy_opt {
            if (enemy.is_shrunk || enemy.state == EnemyAiState::Shrunk)
                && enemy.state != EnemyAiState::Gibbed
                && enemy.state != EnemyAiState::Dying
            {
                let is_touching = dist_to_player < 0.9;
                let is_kicking = (player_ctrl.quick_kick_timer > 0.0
                    || player_ctrl.current_weapon == crate::player::types::WeaponType::Knee)
                    && dist_to_player < 2.0
                    && player_facing_actor;

                if is_touching || is_kicking {
                    enemy.is_shrunk = false;
                    enemy.state = EnemyAiState::Gibbed;
                    enemy.health = -100;
                    sound_events.send(crate::audio::PlaySoundEvent { sound_id: 69 }); // SQUISHED
                    gib_events.send(GibEvent {
                        origin: trans.translation,
                        gib_count: 8,
                    });
                    continue;
                }
            }
        }

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
            if (p.x - actor_pos.x).abs() < 5.0
                && (p.z - actor_pos.z).abs() < 5.0
                && (p.y - actor_pos.y).abs() < 5.0
            {
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
            rng: &mut *rng,
            registers: registers,
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
            player_on_ground: player_ctrl.movement_mode
                != crate::player::types::PlayerMovementMode::JetpackFlying,
            player_jumping_counter: 0,
            player_posz_velocity: 0,
            player_crouching: player_ctrl.movement_mode
                == crate::player::types::PlayerMovementMode::Crouching,
            player_xvel: player_ctrl.speed as i32,
            player_running: player_ctrl.speed > 12.0,
            player_quick_kick: if player_ctrl.quick_kick_timer > 0.0 {
                1
            } else {
                0
            },
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
            sector_lotag: sector_map.as_ref()
                .and_then(|sm| sm.find_sector_world(trans.translation.x, trans.translation.z, None))
                .map(|si| sector_map.as_ref().unwrap().get_sector_lotag(si) as i32)
                .unwrap_or(0),
            sector_ceilingstat: sector_map.as_ref()
                .and_then(|sm| sm.find_sector_world(trans.translation.x, trans.translation.z, None))
                .map(|si| sector_map.as_ref().unwrap().get_sector_ceilingstat(si) as i32)
                .unwrap_or(0),
            is_multiplayer: false,
            hit_space_pressed: false,
            spawned_by_picnum,
            last_hit_weapon: last_hit,
            can_shoot_target,
            bullet_near,
            not_moving,
            away_from_wall: true,
            has_active_sound: false,
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
        let pal_flashes = std::mem::take(&mut ctx.pal_flashes);
        let end_of_game = ctx.end_of_game;
        drop(ctx);

        if let Some(_delay) = end_of_game {
            duke_voice_events.send(crate::audio::PlayDukeVoiceEvent { name: None });
            commands.add(|world: &mut World| {
                world.send_event(crate::game_flow::LevelCompletedEvent::default());
            });
        }

        // Apply death / killit
        if killit {
            commands.safe_despawn_recursive(entity);
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

                // Authentic Duke 3D Enemy Death Drops
                let drop_kind = match enemy.kind {
                    EnemyKind::Pigcop => {
                        if rng.next_f32() < 0.6 {
                            Some(crate::interactivity::PickupKind::ShotgunBox)
                        } else {
                            Some(crate::interactivity::PickupKind::ArmorVest)
                        }
                    }
                    EnemyKind::Liztroop | EnemyKind::AssaultCaptain => {
                        if rng.next_f32() < 0.5 {
                            Some(crate::interactivity::PickupKind::PistolClip)
                        } else {
                            Some(crate::interactivity::PickupKind::SmallMedkit)
                        }
                    }
                    EnemyKind::Enforcer => {
                        Some(crate::interactivity::PickupKind::ChaingunBox)
                    }
                    EnemyKind::Octabrain | EnemyKind::AssaultCommander => {
                        Some(crate::interactivity::PickupKind::RpgRocket)
                    }
                    EnemyKind::Boss1Battlelord
                    | EnemyKind::Boss1Mini
                    | EnemyKind::Boss2Overlord
                    | EnemyKind::Boss3Cycloid
                    | EnemyKind::Boss4Queen => {
                        Some(crate::interactivity::PickupKind::AtomicHealth)
                    }
                    _ => None,
                };

                if let Some(kind) = drop_kind {
                    let drop_pos = trans.translation + Vec3::Y * 0.2;
                    commands.spawn((
                        crate::interactivity::ItemPickup {
                            kind,
                            respawn_timer: None,
                        },
                        TransformBundle::from_transform(Transform::from_translation(drop_pos)),
                        crate::game_flow::LevelEntity,
                    ));
                }

                // Authentic boss death screams
                match enemy.kind {
                    EnemyKind::Boss1Battlelord | EnemyKind::Boss1Mini => {
                        sound_events.send(crate::audio::PlaySoundEvent {
                            sound_id: crate::audio::BOS1_DYING,
                        });
                    }
                    EnemyKind::Boss2Overlord => {
                        sound_events.send(crate::audio::PlaySoundEvent {
                            sound_id: crate::audio::BOS2_DYING,
                        });
                    }
                    EnemyKind::Boss3Cycloid => {
                        sound_events.send(crate::audio::PlaySoundEvent {
                            sound_id: crate::audio::BOS3_DYING,
                        });
                    }
                    EnemyKind::Boss4Queen => {
                        sound_events.send(crate::audio::PlaySoundEvent {
                            sound_id: crate::audio::BOS4_DYING,
                        });
                    }
                    _ => {}
                }

                // Climax victory trigger on Boss defeat in boss levels or episodes
                if matches!(
                    enemy.kind,
                    EnemyKind::Boss1Battlelord
                        | EnemyKind::Boss2Overlord
                        | EnemyKind::Boss3Cycloid
                        | EnemyKind::Boss4Queen
                ) {
                    let is_boss_level = match &level_progress {
                        Some(lp) => {
                            crate::campaign::episodes::is_boss_level(lp.current_episode, lp.current_level)
                                || (lp.current_episode == 1 && lp.current_level == 7)
                        }
                        None => true,
                    };
                    if is_boss_level {
                        duke_voice_events.send(crate::audio::PlayDukeVoiceEvent { name: None });
                        commands.add(|world: &mut World| {
                            world.send_event(crate::game_flow::LevelCompletedEvent {
                                is_secret: false,
                                is_boss_victory: true,
                            });
                        });
                    }
                }
            }

            // Lethal Boss Stomp
            if matches!(
                enemy.kind,
                EnemyKind::Boss1Battlelord
                    | EnemyKind::Boss1Mini
                    | EnemyKind::Boss2Overlord
                    | EnemyKind::Boss3Cycloid
                    | EnemyKind::Boss4Queen
            ) {
                if dist_to_player <= 1.25 && player_ctrl.health > 0 {
                    projectile_events.send(SpawnProjectileEvent {
                        projectile_type: ProjectileType::MightyBoot,
                        origin: trans.translation,
                        direction: dir_to_player,
                        velocity: 10.0,
                        damage: 1000,
                        is_player_source: false,
                        source_player_id: None,
                    });
                }
            }

            // Slimer facehugger attack
            if enemy.kind == EnemyKind::ProtozoidSlimer
                && dist_to_player <= 0.8
                && player_ctrl.health > 0
            {
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
                        source_player_id: None,
                    });
                }
            }

            // Rat scampering behavior (flees away from player within 5.0m, or ambient scurrying)
            if enemy.kind == EnemyKind::ScamperingRat {
                if dist_to_player <= 5.0 {
                    let flee_dir = -dir_to_player.with_y(0.0).normalize_or_zero();
                    trans.translation += flee_dir * (enemy.speed * dt);
                } else {
                    enemy.attack_timer += dt;
                    if enemy.attack_timer >= 1.5 {
                        enemy.attack_timer = 0.0;
                        let angle = rng.next_f32() * std::f32::consts::TAU;
                        enemy.velocity = Vec3::new(angle.cos(), 0.0, angle.sin()) * 2.0;
                    }
                    trans.translation += enemy.velocity * dt;
                }
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
                        source_player_id: None,
                    });
                    sound_events.send(crate::audio::PlaySoundEvent { sound_id: 110 });
                }
            }

            // Battlelord & Mini-Battlelord Combat Attacks: Minigun Barrage & Lobbed Mortar Artillery
            if matches!(enemy.kind, EnemyKind::Boss1Battlelord | EnemyKind::Boss1Mini)
                && can_see
                && dist_to_player <= 40.0
                && player_ctrl.health > 0
            {
                enemy.attack_timer += dt;
                if enemy.attack_timer <= 1.8 {
                    let sub_tick = (enemy.attack_timer / 0.1) as i32;
                    let prev_sub_tick = ((enemy.attack_timer - dt) / 0.1) as i32;
                    if sub_tick != prev_sub_tick || enemy.attack_timer == dt {
                        let right = dir_to_player.cross(Vec3::Y).normalize_or_zero();
                        let up = Vec3::Y;
                        let spread_x = (rng.next_f32() - 0.5) * 0.06;
                        let spread_y = (rng.next_f32() - 0.5) * 0.04;
                        let bullet_dir =
                            (dir_to_player + right * spread_x + up * spread_y).normalize_or_zero();
                        let gun_origin = trans.translation + Vec3::Y * 1.5 + dir_to_player * 0.8;
                        projectile_events.send(SpawnProjectileEvent {
                            projectile_type: ProjectileType::HitscanBullet,
                            origin: gun_origin,
                            direction: bullet_dir,
                            velocity: 150.0,
                            damage: 9,
                            is_player_source: false,
                            source_player_id: None,
                        });
                        sound_events.send(crate::audio::PlaySoundEvent { sound_id: 6 }); // CHAINGUN_FIRE
                    }
                } else if enemy.attack_timer >= 2.4 && (enemy.attack_timer - dt) < 2.4 {
                    let mortar_origin = trans.translation + Vec3::Y * 2.2 + dir_to_player * 0.8;
                    let mortar_dir = (dir_to_player + Vec3::Y * 0.35).normalize_or_zero();
                    projectile_events.send(SpawnProjectileEvent {
                        projectile_type: ProjectileType::Mortar,
                        origin: mortar_origin,
                        direction: mortar_dir,
                        velocity: 30.0,
                        damage: 60,
                        is_player_source: false,
                        source_player_id: None,
                    });
                    sound_events.send(crate::audio::PlaySoundEvent { sound_id: 112 }); // MORTAR
                } else if enemy.attack_timer >= 3.5 {
                    enemy.attack_timer = 0.0;
                }
            }

            // Boss 2: Overlord Combat Attacks: Dual Shoulder Rockets & Rapid Blasters
            if enemy.kind == EnemyKind::Boss2Overlord
                && can_see
                && dist_to_player <= 45.0
                && player_ctrl.health > 0
            {
                enemy.attack_timer += dt;
                let right = dir_to_player.cross(Vec3::Y).normalize_or_zero();

                // Phase 1 (0.0 to 1.5s): Dual Shoulder Rockets
                if enemy.attack_timer <= 1.5 {
                    let sub_tick = (enemy.attack_timer / 0.75) as i32;
                    let prev_sub_tick = ((enemy.attack_timer - dt) / 0.75) as i32;
                    if sub_tick != prev_sub_tick || enemy.attack_timer == dt {
                        sound_events.send(crate::audio::PlaySoundEvent {
                            sound_id: crate::audio::BOS2_ATTACK,
                        });

                        // Right shoulder rocket
                        let r_origin = trans.translation + Vec3::Y * 2.0 + right * 0.75 + dir_to_player * 0.6;
                        let r_dir = (dir_to_player + right * 0.05).normalize_or_zero();
                        projectile_events.send(SpawnProjectileEvent {
                            projectile_type: ProjectileType::Rocket,
                            origin: r_origin,
                            direction: r_dir,
                            velocity: 45.0,
                            damage: 70,
                            is_player_source: false,
                            source_player_id: None,
                        });

                        // Left shoulder rocket
                        let l_origin = trans.translation + Vec3::Y * 2.0 - right * 0.75 + dir_to_player * 0.6;
                        let l_dir = (dir_to_player - right * 0.05).normalize_or_zero();
                        projectile_events.send(SpawnProjectileEvent {
                            projectile_type: ProjectileType::Rocket,
                            origin: l_origin,
                            direction: l_dir,
                            velocity: 45.0,
                            damage: 70,
                            is_player_source: false,
                            source_player_id: None,
                        });
                        sound_events.send(crate::audio::PlaySoundEvent { sound_id: 7 }); // RPG_FIRE
                    }
                } else if enemy.attack_timer <= 2.8 {
                    // Phase 2 (1.6 to 2.8s): Rapid chest blasters
                    let sub_tick = (enemy.attack_timer / 0.15) as i32;
                    let prev_sub_tick = ((enemy.attack_timer - dt) / 0.15) as i32;
                    if sub_tick != prev_sub_tick {
                        let spread_x = (rng.next_f32() - 0.5) * 0.06;
                        let spread_y = (rng.next_f32() - 0.5) * 0.04;
                        let blaster_dir = (dir_to_player + right * spread_x + Vec3::Y * spread_y).normalize_or_zero();
                        let blaster_origin = trans.translation + Vec3::Y * 1.5 + dir_to_player * 0.8;
                        projectile_events.send(SpawnProjectileEvent {
                            projectile_type: ProjectileType::AlienBlaster,
                            origin: blaster_origin,
                            direction: blaster_dir,
                            velocity: 55.0,
                            damage: 15,
                            is_player_source: false,
                            source_player_id: None,
                        });
                        sound_events.send(crate::audio::PlaySoundEvent { sound_id: 110 }); // SOMETHINGFROZE / BLASTER
                    }
                } else if enemy.attack_timer >= 3.6 {
                    enemy.attack_timer = 0.0;
                }
            }

            // Boss 3: Cycloid Emperor Combat Attacks: Forehead Eye PsiBlast, Arm Rocket Salvos & Quake Shockwave
            if enemy.kind == EnemyKind::Boss3Cycloid
                && can_see
                && dist_to_player <= 50.0
                && player_ctrl.health > 0
            {
                enemy.attack_timer += dt;
                let right = dir_to_player.cross(Vec3::Y).normalize_or_zero();

                // Phase 1 (0.0 to 1.5s): Forehead Eye Psychic Blasts
                if enemy.attack_timer <= 1.5 {
                    let sub_tick = (enemy.attack_timer / 0.25) as i32;
                    let prev_sub_tick = ((enemy.attack_timer - dt) / 0.25) as i32;
                    if sub_tick != prev_sub_tick || enemy.attack_timer == dt {
                        if enemy.attack_timer == dt {
                            sound_events.send(crate::audio::PlaySoundEvent {
                                sound_id: crate::audio::BOS3_ATTACK,
                            });
                        }
                        let eye_origin = trans.translation + Vec3::Y * 3.2 + dir_to_player * 0.8;
                        let spread_x = (rng.next_f32() - 0.5) * 0.04;
                        let psi_dir = (dir_to_player + right * spread_x).normalize_or_zero();
                        projectile_events.send(SpawnProjectileEvent {
                            projectile_type: ProjectileType::PsiBlast,
                            origin: eye_origin,
                            direction: psi_dir,
                            velocity: 50.0,
                            damage: 25,
                            is_player_source: false,
                            source_player_id: None,
                        });
                        sound_events.send(crate::audio::PlaySoundEvent { sound_id: 110 });
                    }
                } else if enemy.attack_timer <= 2.8 {
                    // Phase 2 (1.6 to 2.8s): Arm Rocket Salvos
                    let sub_tick = (enemy.attack_timer / 0.3) as i32;
                    let prev_sub_tick = ((enemy.attack_timer - dt) / 0.3) as i32;
                    if sub_tick != prev_sub_tick || (enemy.attack_timer >= 1.6 && (enemy.attack_timer - dt) < 1.6) {
                        let arm_side = if sub_tick % 2 == 0 { 1.2 } else { -1.2 };
                        let arm_origin = trans.translation + Vec3::Y * 2.2 + right * arm_side + dir_to_player * 0.7;
                        projectile_events.send(SpawnProjectileEvent {
                            projectile_type: ProjectileType::Rocket,
                            origin: arm_origin,
                            direction: dir_to_player,
                            velocity: 45.0,
                            damage: 75,
                            is_player_source: false,
                            source_player_id: None,
                        });
                        sound_events.send(crate::audio::PlaySoundEvent { sound_id: 7 }); // RPG_FIRE
                    }
                } else if enemy.attack_timer >= 3.1 && (enemy.attack_timer - dt) < 3.1 {
                    // Phase 3 (3.1s): Ground Shockwave Stomp
                    if let Some(ref mut shake) = camera_shake {
                        shake.intensity = 0.5;
                        shake.offset = Vec3::new(
                            (rng.next_f32() - 0.5) * 0.3,
                            (rng.next_f32() - 0.5) * 0.3,
                            (rng.next_f32() - 0.5) * 0.3,
                        );
                    }
                    if dist_to_player <= 15.0 {
                        explosion_events.send(crate::interactivity::ExplosionDamageEvent {
                            origin: trans.translation,
                            radius: 15.0,
                            damage: 50,
                            attacker_id: None,
                            excluded_entity: None,
                        });
                    }
                    sound_events.send(crate::audio::PlaySoundEvent { sound_id: 112 }); // Quake stomp
                } else if enemy.attack_timer >= 4.0 {
                    enemy.attack_timer = 0.0;
                }
            }

            // Boss 4: Alien Queen Combat Attacks: Eye Lightning Blasts, Venom Spit Bursts & Tail Strike
            if enemy.kind == EnemyKind::Boss4Queen
                && can_see
                && dist_to_player <= 45.0
                && player_ctrl.health > 0
            {
                enemy.attack_timer += dt;
                let right = dir_to_player.cross(Vec3::Y).normalize_or_zero();

                // Phase 1 (0.0 to 1.5s): Triple Eye Lightning / Electrical Discharge
                if enemy.attack_timer <= 1.5 {
                    let sub_tick = (enemy.attack_timer / 0.2) as i32;
                    let prev_sub_tick = ((enemy.attack_timer - dt) / 0.2) as i32;
                    if sub_tick != prev_sub_tick || enemy.attack_timer == dt {
                        if enemy.attack_timer == dt {
                            sound_events.send(crate::audio::PlaySoundEvent {
                                sound_id: crate::audio::BOS4_ATTACK,
                            });
                        }
                        let eye_origin = trans.translation + Vec3::Y * 2.8 + dir_to_player * 0.7;
                        let spread_x = (rng.next_f32() - 0.5) * 0.05;
                        let lightning_dir = (dir_to_player + right * spread_x).normalize_or_zero();
                        projectile_events.send(SpawnProjectileEvent {
                            projectile_type: ProjectileType::AlienBlaster,
                            origin: eye_origin,
                            direction: lightning_dir,
                            velocity: 60.0,
                            damage: 20,
                            is_player_source: false,
                            source_player_id: None,
                        });
                        sound_events.send(crate::audio::PlaySoundEvent { sound_id: 110 });
                    }
                } else if enemy.attack_timer <= 2.8 {
                    // Phase 2 (1.6 to 2.8s): Spit / Venom Organic Barrage
                    let sub_tick = (enemy.attack_timer / 0.25) as i32;
                    let prev_sub_tick = ((enemy.attack_timer - dt) / 0.25) as i32;
                    if sub_tick != prev_sub_tick || (enemy.attack_timer >= 1.6 && (enemy.attack_timer - dt) < 1.6) {
                        let mouth_origin = trans.translation + Vec3::Y * 2.2 + dir_to_player * 0.8;
                        let spread_x = (rng.next_f32() - 0.5) * 0.08;
                        let spread_y = (rng.next_f32() - 0.5) * 0.04;
                        let spit_dir = (dir_to_player + right * spread_x + Vec3::Y * spread_y).normalize_or_zero();
                        projectile_events.send(SpawnProjectileEvent {
                            projectile_type: ProjectileType::Spit,
                            origin: mouth_origin,
                            direction: spit_dir,
                            velocity: 40.0,
                            damage: 30,
                            is_player_source: false,
                            source_player_id: None,
                        });
                        sound_events.send(crate::audio::PlaySoundEvent { sound_id: 570 }); // OCTA_ATTACK / SPIT
                    }
                } else if enemy.attack_timer >= 3.0 && (enemy.attack_timer - dt) < 3.0 {
                    // Phase 3 (3.0s): Close-Range Tail Strike
                    if dist_to_player <= 12.0 {
                        if let Some(ref mut shake) = camera_shake {
                            shake.intensity = 0.4;
                            shake.offset = Vec3::new(
                                (rng.next_f32() - 0.5) * 0.2,
                                (rng.next_f32() - 0.5) * 0.2,
                                (rng.next_f32() - 0.5) * 0.2,
                            );
                        }
                        explosion_events.send(crate::interactivity::ExplosionDamageEvent {
                            origin: trans.translation + dir_to_player * 2.0,
                            radius: 8.0,
                            damage: 60,
                            attacker_id: None,
                            excluded_entity: None,
                        });
                        sound_events.send(crate::audio::PlaySoundEvent { sound_id: 0 }); // KICK_HIT
                    }
                } else if enemy.attack_timer >= 3.8 {
                    enemy.attack_timer = 0.0;
                }
            }
        }

        // Handle Shoot Events
        for (tile, _, _, _, ang_shot) in shoot_events {
            let shoot_ang_rad = -(ang_shot as f32 / 2048.0) * std::f32::consts::TAU;
            let fire_dir =
                Vec3::new(shoot_ang_rad.cos(), 0.0, shoot_ang_rad.sin()).normalize_or_zero();
            let fire_origin = trans.translation + Vec3::Y * 0.4 + fire_dir * 0.4;

            let (proj_type, vel, dmg) = map_tile_to_projectile(tile);
            if proj_type == ProjectileType::ShotgunPellet {
                let right = fire_dir.cross(Vec3::Y).normalize_or_zero();
                let up = Vec3::Y;
                for _ in 0..7 {
                    let spread_x = (rng.next_f32() - 0.5) * 0.08;
                    let spread_y = (rng.next_f32() - 0.5) * 0.08;
                    let final_dir =
                        (fire_dir + right * spread_x + up * spread_y).normalize_or_zero();
                    projectile_events.send(SpawnProjectileEvent {
                        projectile_type: proj_type,
                        origin: fire_origin,
                        direction: final_dir,
                        velocity: vel,
                        damage: dmg,
                        is_player_source: false,
                        source_player_id: None,
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
                    source_player_id: None,
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
                attacker_id: None,
                excluded_entity: None,
            });
        }

        // Handle Palette Flash / Screen Tint Events from CON Script
        for (_duration, r, g, b) in pal_flashes {
            if let Some(ref mut tint) = screen_tint {
                let rf = (r as f32 / 64.0).clamp(0.0, 1.0);
                let gf = (g as f32 / 64.0).clamp(0.0, 1.0);
                let bf = (b as f32 / 64.0).clamp(0.0, 1.0);
                tint.target_color = Color::srgba(rf, gf, bf, 0.75);
            }
        }

        let mut final_movement = Vec3::new(0.0, vertical_movement, 0.0);

        // Movement application from active MoveDef & AI flags
        if let Some(move_ptr) = registers.move_ptr {
            if move_ptr + 1 < engine.compiled.bytecode.len() {
                let hvel = engine.compiled.bytecode[move_ptr];
                let flags = *hitag as i32;

                if (flags & move_flags::FACE_PLAYER) != 0 || (flags & move_flags::SEEK_PLAYER) != 0
                {
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
                    final_movement += move_dir * move_speed * dt;
                }
            }
        }

        if final_movement.length_squared() > 0.0 {
            if let Some(ref mut kcc) = kcc_opt {
                kcc.translation = Some(final_movement);
            } else {
                trans.translation += final_movement;
            }

            // Boss footstep screen shake and heavy walk sound
            if let Some(ref enemy) = enemy_opt {
                let is_massive_boss = matches!(
                    enemy.kind,
                    EnemyKind::Boss1Battlelord
                        | EnemyKind::Boss1Mini
                        | EnemyKind::Boss2Overlord
                        | EnemyKind::Boss3Cycloid
                        | EnemyKind::Boss4Queen
                );
                if is_massive_boss {
                    let step_cycle = ((time.elapsed_seconds() * 2.0) % 2.0) as i32;
                    let prev_step_cycle = (((time.elapsed_seconds() - dt) * 2.0) % 2.0) as i32;
                    if step_cycle != prev_step_cycle {
                        let dist_factor = (1.0 - (dist_to_player / 35.0)).clamp(0.0, 1.0);
                        if dist_factor > 0.0 {
                            if let Some(ref mut shake) = camera_shake {
                                shake.intensity = shake.intensity.max(0.06 * dist_factor);
                            }
                            sound_events.send(crate::audio::PlaySoundEvent { sound_id: 113 }); // BOSS_WALK
                        }
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
        FIRELASER | 1600 => (ProjectileType::AlienBlaster, 50.0, 7), // FIRELASER (Trooper / Captain / Turret / Recon)
        SPIT | LOOGIE | 1605 => (ProjectileType::Spit, 35.0, 8),     // SPIT (Enforcer venom / Octabrain spit)
        FREEZEBLAST | FREEZE => (ProjectileType::FreezeShard, 45.0, 20), // FREEZEBLAST
        SHRINKSPARK | SHRINKER => (ProjectileType::ShrinkRay, 40.0, 0), // SHRINKSPARK / SHRINKER
        MORTER => (ProjectileType::Mortar, 30.0, 50),                // MORTER (Battlelord / Tank artillery)
        SHOTSPARK1 | CHAINGUN => (ProjectileType::HitscanBullet, 150.0, 9), // SHOTSPARK1 / CHAINGUN (Enforcer / Battlelord minigun)
        RPG => (ProjectileType::Rocket, 45.0, 140),                 // RPG (Commander / Overlord / Cycloid)
        SHOTGUN => (ProjectileType::ShotgunPellet, 80.0, 10),       // SHOTGUN (Pigcop)
        COOLEXPLOSION1 => (ProjectileType::PsiBlast, 30.0, 38),     // COOLEXPLOSION1 (Octabrain)
        DEVISTATORBLAST => (ProjectileType::Rocket, 60.0, 40),      // DEVISTATORBLAST
        _ => (ProjectileType::HitscanBullet, 100.0, 10),
    }
}

/// Trigger squash/stomp kill on a shrunk enemy.
pub fn execute_shrunk_enemy_stomp(
    enemy: &mut EnemyActor,
    origin: Vec3,
    sound_events: &mut EventWriter<crate::audio::PlaySoundEvent>,
    gib_events: &mut EventWriter<GibEvent>,
) {
    enemy.is_shrunk = false;
    enemy.state = EnemyAiState::Gibbed;
    enemy.health = -100;
    sound_events.send(crate::audio::PlaySoundEvent { sound_id: 69 }); // SQUISHED
    gib_events.send(GibEvent {
        origin,
        gib_count: 8,
    });
}

/// Shatter a frozen enemy upon receiving damage.
pub fn execute_frozen_enemy_shatter(
    enemy: &mut EnemyActor,
    origin: Vec3,
    sound_events: &mut EventWriter<crate::audio::PlaySoundEvent>,
    gib_events: &mut EventWriter<GibEvent>,
) {
    enemy.is_frozen = false;
    enemy.state = EnemyAiState::Gibbed;
    enemy.health = -50;
    sound_events.send(crate::audio::PlaySoundEvent { sound_id: 19 }); // GLASS_BREAKING
    gib_events.send(GibEvent {
        origin,
        gib_count: 8,
    });
}
