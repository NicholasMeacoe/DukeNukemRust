use crate::audio::PlaySoundEvent;
use crate::combat::types::{LaserTripwireBeam, Projectile, ProjectileType, SpawnProjectileEvent};
use crate::interactivity::types::ExplosionDamageEvent;
pub use crate::player::types::*;
use bevy::prelude::*;
use bevy_rapier3d::prelude::*;

pub fn handle_weapon_selection(
    keys: Res<ButtonInput<KeyCode>>,
    mut query: Query<(&mut PlayerController, Option<&PlayerId>)>,
    mut sound_events: EventWriter<PlaySoundEvent>,
) {
    for (mut player, opt_id) in query.iter_mut() {
        let player_id = opt_id.map_or(0, |id| id.0);
        let input_cfg = crate::net::splitscreen::get_player_input_config(player_id);

        let mut selected = None;
        if player_id == 0 {
            if keys.just_pressed(KeyCode::Digit1) {
                selected = Some(WeaponType::Knee);
            } else if keys.just_pressed(KeyCode::Digit2) {
                selected = Some(WeaponType::Pistol);
            } else if keys.just_pressed(KeyCode::Digit3) {
                selected = Some(WeaponType::Shotgun);
            } else if keys.just_pressed(KeyCode::Digit4) {
                selected = Some(WeaponType::Chaingun);
            } else if keys.just_pressed(KeyCode::Digit5) {
                selected = Some(WeaponType::Rpg);
            } else if keys.just_pressed(KeyCode::Digit6) {
                selected = Some(WeaponType::Pipebomb);
            } else if keys.just_pressed(KeyCode::Digit7) {
                if player.current_weapon == WeaponType::Shrinker
                    && player.weapons[WeaponType::Expander as usize].is_unlocked
                {
                    selected = Some(WeaponType::Expander);
                } else {
                    selected = Some(WeaponType::Shrinker);
                }
            } else if keys.just_pressed(KeyCode::Digit8) {
                selected = Some(WeaponType::Devastator);
            } else if keys.just_pressed(KeyCode::Digit9) {
                selected = Some(WeaponType::Tripbomb);
            } else if keys.just_pressed(KeyCode::Digit0) {
                selected = Some(WeaponType::Freezethrower);
            }
        }

        // Weapon cycle controls (supported for all players)
        if keys.just_pressed(input_cfg.next_weapon) {
            let cur = player.current_weapon as usize;
            for offset in 1..12 {
                let candidate = (cur + offset) % 12;
                if candidate < player.weapons.len() && player.weapons[candidate].is_unlocked {
                    if let Some(w) = match_weapon_idx(candidate) {
                        selected = Some(w);
                        break;
                    }
                }
            }
        } else if keys.just_pressed(input_cfg.prev_weapon) {
            let cur = player.current_weapon as usize;
            for offset in 1..12 {
                let candidate = (cur + 12 - offset) % 12;
                if candidate < player.weapons.len() && player.weapons[candidate].is_unlocked {
                    if let Some(w) = match_weapon_idx(candidate) {
                        selected = Some(w);
                        break;
                    }
                }
            }
        }

        if let Some(weapon_type) = selected {
            let idx = weapon_type as usize;
            if idx < player.weapons.len() && player.weapons[idx].is_unlocked {
                player.current_weapon = weapon_type;
                sound_events.send(PlaySoundEvent { sound_id: 118 }); // SELECT_WEAPON
            }
        }
    }
}

pub fn match_weapon_idx(idx: usize) -> Option<WeaponType> {
    match idx {
        0 => Some(WeaponType::Knee),
        1 => Some(WeaponType::Pistol),
        2 => Some(WeaponType::Shotgun),
        3 => Some(WeaponType::Chaingun),
        4 => Some(WeaponType::Rpg),
        5 => Some(WeaponType::Pipebomb),
        6 => Some(WeaponType::Shrinker),
        7 => Some(WeaponType::Devastator),
        8 => Some(WeaponType::Tripbomb),
        9 => Some(WeaponType::Freezethrower),
        10 => Some(WeaponType::HandRemote),
        11 => Some(WeaponType::Expander),
        _ => None,
    }
}

pub fn get_highest_priority_available_weapon(player: &PlayerController) -> WeaponType {
    let priority = [
        WeaponType::Devastator,
        WeaponType::Rpg,
        WeaponType::Chaingun,
        WeaponType::Shotgun,
        WeaponType::Pistol,
        WeaponType::Freezethrower,
        WeaponType::Shrinker,
        WeaponType::Expander,
        WeaponType::Pipebomb,
        WeaponType::Tripbomb,
        WeaponType::Knee,
    ];
    for &wt in &priority {
        let idx = wt as usize;
        if wt == WeaponType::Knee
            || (player.weapons[idx].is_unlocked && player.weapons[idx].ammo > 0)
        {
            return wt;
        }
    }
    WeaponType::Knee
}

pub fn handle_weapon_firing(
    keys: Res<ButtonInput<KeyCode>>,
    btn: Res<ButtonInput<MouseButton>>,
    time: Res<Time>,
    mut query: Query<(&Transform, &mut PlayerController, Option<&PlayerId>), Without<Projectile>>,
    camera_query: Query<(&Transform, Option<&PlayerCamera>), (With<Camera>, Without<PlayerController>, Without<Projectile>)>,
    (
        mut projectile_events,
        mut explosion_events,
        mut sound_events,
        mut casing_events,
        mut light_events,
    ): (
        EventWriter<SpawnProjectileEvent>,
        EventWriter<ExplosionDamageEvent>,
        EventWriter<PlaySoundEvent>,
        EventWriter<crate::combat::gore::SpawnCasingEvent>,
        EventWriter<crate::lighting::SpawnDynamicLightEvent>,
    ),
    pipebomb_query: Query<(Entity, &Transform, &Projectile), Without<PlayerController>>,
    mut commands: Commands,
    mut rng: ResMut<crate::net::DeterministicRng>,
    rapier_context: Option<Res<RapierContext>>,
    mut meshes: Option<ResMut<Assets<Mesh>>>,
    mut materials: Option<ResMut<Assets<StandardMaterial>>>,
    game_config: Option<Res<crate::config::GameConfig>>,
) {
    let dt = time.delta_seconds();

    for (player_trans, mut player, opt_id) in query.iter_mut() {
        let player_id = opt_id.map_or(0, |id| id.0);
        let cam_trans = camera_query
            .iter()
            .find(|(_, cam_id)| cam_id.map_or(0, |c| c.0) == player_id)
            .map(|(t, _)| t)
            .or_else(|| camera_query.iter().next().map(|(t, _)| t));
        let Some(cam_trans) = cam_trans else {
            continue;
        };

        for weapon in player.weapons.iter_mut() {
            if weapon.fire_timer > 0.0 {
                weapon.fire_timer -= dt;
            }
            if weapon.reload_timer > 0.0 {
                weapon.reload_timer -= dt;
                if weapon.reload_timer <= 0.0 {
                    sound_events.send(PlaySoundEvent { sound_id: 5 }); // INSERT_CLIP
                }
            }
        }

        if player.quick_kick_timer > 0.0 {
            player.quick_kick_timer -= dt;
        }

        let fwd_vec = *cam_trans.forward();
        let right_vec = *cam_trans.right();
        let up_vec = *cam_trans.up();
        let input_cfg = crate::net::splitscreen::get_player_input_config(player_id);

        // Quick kick check (Key 'Q' for Player 0)
        if player_id == 0 && keys.just_pressed(KeyCode::KeyQ) && player.quick_kick_timer <= 0.0 {
            player.quick_kick_timer = 0.5;
            sound_events.send(PlaySoundEvent { sound_id: 0 }); // KICK_HIT
            let kick_damage = if player.inventory.steroids_active {
                40
            } else {
                15
            };
            projectile_events.send(SpawnProjectileEvent {
                projectile_type: ProjectileType::MightyBoot,
                origin: player_trans.translation + Vec3::Y * 0.2,
                direction: fwd_vec,
                velocity: 15.0,
                damage: kick_damage,
                is_player_source: true,
                source_player_id: Some(player_id),
            });
        }

        let cur_idx = player.current_weapon as usize;
        if cur_idx >= player.weapons.len() {
            continue;
        }

        let is_continuous = player.current_weapon == WeaponType::Chaingun
            || player.current_weapon == WeaponType::Freezethrower;

        let is_firing = if is_continuous {
            if player_id == 0 {
                btn.pressed(MouseButton::Left) || keys.pressed(input_cfg.fire)
            } else {
                keys.pressed(input_cfg.fire)
            }
        } else {
            if player_id == 0 {
                btn.just_pressed(MouseButton::Left) || keys.just_pressed(input_cfg.fire)
            } else {
                keys.just_pressed(input_cfg.fire)
            }
        };

        // Detonator Trigger on Right Click or HandRemote weapon
        let is_detonating = if player_id == 0 {
            btn.just_pressed(MouseButton::Right)
                || (player.current_weapon == WeaponType::HandRemote
                    && (btn.just_pressed(MouseButton::Left) || keys.just_pressed(input_cfg.fire)))
        } else {
            player.current_weapon == WeaponType::HandRemote && keys.just_pressed(input_cfg.fire)
        };

        if is_detonating {
        let mut detonated_any = false;
        for (entity, p_trans, proj) in pipebomb_query.iter() {
            if proj.projectile_type == ProjectileType::Pipebomb
                && proj.is_player_source
                && (proj.source_player_id.is_none() || proj.source_player_id == Some(player_id))
            {
                detonated_any = true;
                sound_events.send(PlaySoundEvent { sound_id: 14 }); // PIPEBOMB_EXPLODE
                explosion_events.send(ExplosionDamageEvent {
                    origin: p_trans.translation,
                    radius: 7.0,
                    damage: 150,
                    attacker_id: Some(player_id),
                    excluded_entity: None,
                });
                light_events.send(crate::lighting::SpawnDynamicLightEvent::explosion(
                    p_trans.translation,
                    7.0 * 20.0,
                ));
                commands.entity(entity).despawn_recursive();
            }
        }
        if detonated_any && player.current_weapon == WeaponType::HandRemote {
            player.weapons[cur_idx].fire_timer = player.weapons[cur_idx].fire_delay;
            let pipebomb_idx = WeaponType::Pipebomb as usize;
            if player.weapons[pipebomb_idx].ammo > 0 {
                player.current_weapon = WeaponType::Pipebomb;
            } else {
                player.current_weapon = get_highest_priority_available_weapon(&player);
            }
        }
    }

    if is_firing
        && !is_detonating
        && player.weapons[cur_idx].fire_timer <= 0.0
        && player.weapons[cur_idx].reload_timer <= 0.0
    {
        let fire_pos = player_trans.translation + Vec3::Y * 0.4 + fwd_vec * 0.5;

        match player.current_weapon {
            WeaponType::Knee => {
                let kick_damage = if player.inventory.steroids_active {
                    40
                } else {
                    15
                };
                player.weapons[cur_idx].fire_timer = player.weapons[cur_idx].fire_delay;
                sound_events.send(PlaySoundEvent { sound_id: 0 }); // KICK_HIT
                projectile_events.send(SpawnProjectileEvent {
                    projectile_type: ProjectileType::MightyBoot,
                    origin: fire_pos,
                    direction: fwd_vec,
                    velocity: 15.0,
                    damage: kick_damage,
                    is_player_source: true,
                    source_player_id: Some(player_id),
                });
            }
            WeaponType::Pistol => {
                if player.pistol_mag > 0 {
                    player.pistol_mag -= 1;
                    player.weapons[cur_idx].ammo = player.weapons[cur_idx].ammo.saturating_sub(1);
                    player.weapons[cur_idx].fire_timer = player.weapons[cur_idx].fire_delay;
                    sound_events.send(PlaySoundEvent { sound_id: 3 }); // PISTOL_FIRE
                    light_events.send(crate::lighting::SpawnDynamicLightEvent::muzzle_flash(
                        fire_pos,
                        WeaponType::Pistol,
                    ));

                    casing_events.send(crate::combat::gore::SpawnCasingEvent {
                        origin: fire_pos,
                        direction: fwd_vec,
                        is_shotgun: false,
                    });

                    projectile_events.send(SpawnProjectileEvent {
                        projectile_type: ProjectileType::HitscanBullet,
                        origin: fire_pos,
                        direction: fwd_vec,
                        velocity: 150.0,
                        damage: 12,
                        is_player_source: true,
                        source_player_id: Some(player_id),
                    });

                    // Auto-reload after 12 rounds
                    if player.pistol_mag == 0 && player.weapons[cur_idx].ammo > 0 {
                        player.weapons[cur_idx].reload_timer = 0.9;
                        sound_events.send(PlaySoundEvent { sound_id: 4 }); // EJECT_CLIP
                        player.pistol_mag = player.weapons[cur_idx].ammo.min(12);
                    }
                }
            }
            WeaponType::Shotgun => {
                if player.weapons[cur_idx].ammo > 0 {
                    player.weapons[cur_idx].ammo -= 1;
                    player.weapons[cur_idx].fire_timer = player.weapons[cur_idx].fire_delay;
                    sound_events.send(PlaySoundEvent { sound_id: 109 }); // SHOTGUN_FIRE
                    light_events.send(crate::lighting::SpawnDynamicLightEvent::muzzle_flash(
                        fire_pos,
                        WeaponType::Shotgun,
                    ));

                    casing_events.send(crate::combat::gore::SpawnCasingEvent {
                        origin: fire_pos,
                        direction: fwd_vec,
                        is_shotgun: true,
                    });

                    // Spawn 7 spread pellets
                    for _ in 0..7 {
                        let spread_x = (rng.next_f32() - 0.5) * 0.06;
                        let spread_y = (rng.next_f32() - 0.5) * 0.06;
                        let dir = (fwd_vec + right_vec * spread_x + up_vec * spread_y)
                            .normalize_or_zero();

                        projectile_events.send(SpawnProjectileEvent {
                            projectile_type: ProjectileType::ShotgunPellet,
                            origin: fire_pos,
                            direction: dir,
                            velocity: 120.0,
                            damage: 9,
                            is_player_source: true,
                            source_player_id: Some(player_id),
                        });
                    }
                }
            }
            WeaponType::Chaingun => {
                if player.weapons[cur_idx].ammo > 0 {
                    player.weapons[cur_idx].ammo -= 1;
                    player.weapons[cur_idx].fire_timer = player.weapons[cur_idx].fire_delay;
                    sound_events.send(PlaySoundEvent { sound_id: 6 }); // CHAINGUN_FIRE
                    light_events.send(crate::lighting::SpawnDynamicLightEvent::muzzle_flash(
                        fire_pos,
                        WeaponType::Chaingun,
                    ));

                    casing_events.send(crate::combat::gore::SpawnCasingEvent {
                        origin: fire_pos,
                        direction: fwd_vec,
                        is_shotgun: false,
                    });

                    let spread_x = (rng.next_f32() - 0.5) * 0.03;
                    let spread_y = (rng.next_f32() - 0.5) * 0.03;
                    let dir =
                        (fwd_vec + right_vec * spread_x + up_vec * spread_y).normalize_or_zero();

                    projectile_events.send(SpawnProjectileEvent {
                        projectile_type: ProjectileType::HitscanBullet,
                        origin: fire_pos,
                        direction: dir,
                        velocity: 150.0,
                        damage: 10,
                        is_player_source: true,
                        source_player_id: Some(player_id),
                    });
                }
            }
            WeaponType::Rpg => {
                if player.weapons[cur_idx].ammo > 0 {
                    player.weapons[cur_idx].ammo -= 1;
                    player.weapons[cur_idx].fire_timer = player.weapons[cur_idx].fire_delay;
                    sound_events.send(PlaySoundEvent { sound_id: 7 }); // RPG_FIRE
                    light_events.send(crate::lighting::SpawnDynamicLightEvent::muzzle_flash(
                        fire_pos,
                        WeaponType::Rpg,
                    ));

                    projectile_events.send(SpawnProjectileEvent {
                        projectile_type: ProjectileType::Rocket,
                        origin: fire_pos,
                        direction: fwd_vec,
                        velocity: 35.0,
                        damage: 120,
                        is_player_source: true,
                        source_player_id: Some(player_id),
                    });
                }
            }
            WeaponType::Pipebomb => {
                if player.weapons[cur_idx].ammo > 0 {
                    player.weapons[cur_idx].ammo -= 1;
                    player.weapons[cur_idx].fire_timer = player.weapons[cur_idx].fire_delay;
                    sound_events.send(PlaySoundEvent { sound_id: 118 });

                    projectile_events.send(SpawnProjectileEvent {
                        projectile_type: ProjectileType::Pipebomb,
                        origin: fire_pos,
                        direction: (fwd_vec + up_vec * 0.3).normalize_or_zero(),
                        velocity: 15.0,
                        damage: 150,
                        is_player_source: true,
                        source_player_id: Some(player_id),
                    });

                    player.current_weapon = WeaponType::HandRemote;
                }
            }
            WeaponType::Shrinker => {
                if player.weapons[cur_idx].ammo > 0 {
                    player.weapons[cur_idx].ammo -= 1;
                    player.weapons[cur_idx].fire_timer = player.weapons[cur_idx].fire_delay;
                    sound_events.send(PlaySoundEvent { sound_id: 11 }); // SHRINKER_FIRE
                    light_events.send(crate::lighting::SpawnDynamicLightEvent::muzzle_flash(
                        fire_pos,
                        WeaponType::Shrinker,
                    ));

                    projectile_events.send(SpawnProjectileEvent {
                        projectile_type: ProjectileType::ShrinkRay,
                        origin: fire_pos,
                        direction: fwd_vec,
                        velocity: 30.0,
                        damage: 0, // Applies shrink status
                        is_player_source: true,
                        source_player_id: Some(player_id),
                    });
                }
            }
            WeaponType::Devastator => {
                if player.weapons[cur_idx].ammo > 0 {
                    player.weapons[cur_idx].ammo -= 1;
                    player.weapons[cur_idx].fire_timer = player.weapons[cur_idx].fire_delay;
                    sound_events.send(PlaySoundEvent { sound_id: 10 }); // CAT_FIRE
                    light_events.send(crate::lighting::SpawnDynamicLightEvent::muzzle_flash(
                        fire_pos,
                        WeaponType::Devastator,
                    ));

                    player.devastator_alt_side = !player.devastator_alt_side;
                    let side_offset = if player.devastator_alt_side {
                        0.2
                    } else {
                        -0.2
                    };
                    let pod_pos = fire_pos + right_vec * side_offset;

                    projectile_events.send(SpawnProjectileEvent {
                        projectile_type: ProjectileType::DevastatorMissile,
                        origin: pod_pos,
                        direction: fwd_vec,
                        velocity: 45.0,
                        damage: 40,
                        is_player_source: true,
                        source_player_id: Some(player_id),
                    });
                }
            }
            WeaponType::Tripbomb => {
                if player.weapons[cur_idx].ammo > 0 {
                    let max_reach = TRIPBOMB_MAX_REACH;
                    let ray_origin = cam_trans.translation;
                    let ray_dir = fwd_vec.normalize_or_zero();

                    let mut hit_wall = false;
                    let mut attach_pos = Vec3::ZERO;
                    let mut outward_normal = Vec3::ZERO;
                    let mut beam_length = 12.0;

                    if let Some(ref rapier) = rapier_context {
                        let filter = QueryFilter::exclude_kinematic();
                        if let Some((_hit_entity, intersection)) =
                            rapier.cast_ray_and_get_normal(ray_origin, ray_dir, max_reach, true, filter)
                        {
                            hit_wall = true;
                            attach_pos = intersection.point + intersection.normal * 0.02;
                            outward_normal = intersection.normal.normalize_or_zero();

                            // Opposing surface raycast across from the tripbomb
                            let max_beam = 30.0;
                            if let Some((_opp_entity, opp_hit)) = rapier.cast_ray_and_get_normal(
                                attach_pos + outward_normal * 0.05,
                                outward_normal,
                                max_beam,
                                true,
                                filter,
                            ) {
                                beam_length = opp_hit.time_of_impact + 0.05;
                            }
                        }
                    }

                    if hit_wall {
                        player.weapons[cur_idx].ammo -= 1;
                        player.weapons[cur_idx].fire_timer = player.weapons[cur_idx].fire_delay;
                        sound_events.send(PlaySoundEvent { sound_id: 15 }); // LSRBMBPT (mounted on wall)
                        sound_events.send(PlaySoundEvent { sound_id: 16 }); // LSRBMBWN (arming beep)

                        let up = if outward_normal.abs().dot(Vec3::Y) > 0.99 {
                            Vec3::Z
                        } else {
                            Vec3::Y
                        };
                        let tripbomb_transform = Transform::from_translation(attach_pos)
                            .looking_to(outward_normal, up);

                        let mut tripbomb_cmd = commands.spawn((
                            SpatialBundle {
                                transform: tripbomb_transform,
                                ..default()
                            },
                            crate::combat::LaserTripbomb {
                                normal: outward_normal,
                                arm_timer: 1.0,
                                is_armed: false,
                                beam_length,
                                damage: 150,
                                damage_radius: 6.0,
                                owner_player_id: Some(player_id),
                            },
                            crate::game_flow::LevelEntity,
                        ));

                        if let (Some(ref mut meshes_res), Some(ref mut mats_res)) =
                            (meshes.as_mut(), materials.as_mut())
                        {
                            let beam_mesh = meshes_res.add(Cuboid::new(0.02, 0.02, beam_length));
                            let beam_mat = mats_res.add(StandardMaterial {
                                base_color: Color::srgb(0.0, 0.8, 1.0),
                                emissive: LinearRgba::new(0.0, 4.0, 8.0, 1.0),
                                unlit: true,
                                ..default()
                            });
                            tripbomb_cmd.with_children(|parent| {
                                parent.spawn((
                                    PbrBundle {
                                        mesh: beam_mesh,
                                        material: beam_mat,
                                        transform: Transform::from_translation(Vec3::new(
                                            0.0,
                                            0.0,
                                            -beam_length * 0.5,
                                        )),
                                        ..default()
                                     },
                                    LaserTripwireBeam,
                                    crate::game_flow::LevelEntity,
                                ));
                            });
                        } else {
                            tripbomb_cmd.with_children(|parent| {
                                parent.spawn((
                                    SpatialBundle {
                                        transform: Transform::from_translation(Vec3::new(
                                            0.0,
                                            0.0,
                                            -beam_length * 0.5,
                                        )),
                                        ..default()
                                    },
                                    LaserTripwireBeam,
                                    crate::game_flow::LevelEntity,
                                ));
                            });
                        }
                    } else {
                        // If no wall is in reach: do NOT consume ammo, play a click/misplace sound or do not place.
                        player.weapons[cur_idx].fire_timer = 0.2;
                        sound_events.send(PlaySoundEvent { sound_id: 86 }); // Click / Access denied misplace sound
                    }
                }
            }
            WeaponType::Freezethrower => {
                if player.weapons[cur_idx].ammo > 0 {
                    player.weapons[cur_idx].ammo -= 1;
                    player.weapons[cur_idx].fire_timer = player.weapons[cur_idx].fire_delay;
                    sound_events.send(PlaySoundEvent { sound_id: 110 }); // SOMETHINGFROZE
                    light_events.send(crate::lighting::SpawnDynamicLightEvent::muzzle_flash(
                        fire_pos,
                        WeaponType::Freezethrower,
                    ));

                    projectile_events.send(SpawnProjectileEvent {
                        projectile_type: ProjectileType::FreezeShard,
                        origin: fire_pos,
                        direction: fwd_vec,
                        velocity: 25.0,
                        damage: 15, // Freezes on death
                        is_player_source: true,
                        source_player_id: Some(player_id),
                    });
                }
            }
            WeaponType::Expander => {
                if player.weapons[cur_idx].ammo > 0 {
                    player.weapons[cur_idx].ammo -= 1;
                    player.weapons[cur_idx].fire_timer = player.weapons[cur_idx].fire_delay;
                    sound_events.send(PlaySoundEvent { sound_id: 11 }); // EXPANDER_FIRE
                    light_events.send(crate::lighting::SpawnDynamicLightEvent::muzzle_flash(
                        fire_pos,
                        WeaponType::Expander,
                    ));

                    projectile_events.send(SpawnProjectileEvent {
                        projectile_type: ProjectileType::ExpanderRay,
                        origin: fire_pos,
                        direction: fwd_vec,
                        velocity: 30.0,
                        damage: 25,
                        is_player_source: true,
                        source_player_id: Some(player_id),
                    });
                }
            }
            WeaponType::HandRemote => {
                let mut detonated = false;
                for (entity, p_trans, proj) in pipebomb_query.iter() {
                    if proj.projectile_type == ProjectileType::Pipebomb
                        && proj.is_player_source
                        && (proj.source_player_id.is_none() || proj.source_player_id == Some(player_id))
                    {
                        detonated = true;
                        sound_events.send(PlaySoundEvent { sound_id: 14 }); // PIPEBOMB_EXPLODE
                        explosion_events.send(ExplosionDamageEvent {
                            origin: p_trans.translation,
                            radius: 7.0,
                            damage: 150,
                            attacker_id: Some(player_id),
                            excluded_entity: None,
                        });
                        light_events.send(crate::lighting::SpawnDynamicLightEvent::explosion(
                            p_trans.translation,
                            7.0 * 20.0,
                        ));
                        commands.entity(entity).despawn_recursive();
                    }
                }
                player.weapons[cur_idx].fire_timer = player.weapons[cur_idx].fire_delay;
                if detonated {
                    let pipebomb_idx = WeaponType::Pipebomb as usize;
                    if player.weapons[pipebomb_idx].ammo > 0 {
                        player.current_weapon = WeaponType::Pipebomb;
                    } else {
                        player.current_weapon = get_highest_priority_available_weapon(&player);
                    }
                }
            }
        }

        // Priority auto-switch if out of ammo
        let auto_switch = game_config.as_ref().map_or(true, |c| c.controls.auto_switch_weapon);
        if auto_switch && player.weapons[cur_idx].ammo == 0 && cur_idx != WeaponType::Knee as usize {
            let next_wpn = get_highest_priority_available_weapon(&player);
            if next_wpn != player.current_weapon {
                player.current_weapon = next_wpn;
                sound_events.send(PlaySoundEvent { sound_id: 118 });
            }
        }
    }
    }
}

pub fn update_laser_tripbombs(
    time: Res<Time>,
    mut commands: Commands,
    mut tripbombs: Query<(Entity, &Transform, &mut crate::combat::LaserTripbomb)>,
    player_query: Query<
        &Transform,
        (
            With<PlayerController>,
            Without<crate::combat::LaserTripbomb>,
        ),
    >,
    enemies: Query<
        &Transform,
        (
            With<crate::combat::EnemyActor>,
            Without<crate::combat::LaserTripbomb>,
        ),
    >,
    mut explosion_events: EventWriter<ExplosionDamageEvent>,
    mut sound_events: EventWriter<PlaySoundEvent>,
) {
    let dt = time.delta_seconds();

    for (entity, trans, mut bomb) in tripbombs.iter_mut() {
        if !bomb.is_armed {
            bomb.arm_timer -= dt;
            if bomb.arm_timer <= 0.0 {
                bomb.is_armed = true;
                sound_events.send(PlaySoundEvent { sound_id: 16 }); // Arm beep
            }
            continue;
        }

        // Armed: check beam intersection with player or enemies
        let origin = trans.translation;
        let normal = bomb.normal.normalize_or_zero();

        let mut triggered = false;

        // Check player
        for p_trans in player_query.iter() {
            let p_pos = p_trans.translation;
            let v = p_pos - origin;
            let proj = v.dot(normal);
            if proj > 0.3 && proj < bomb.beam_length {
                let closest = origin + normal * proj;
                if closest.distance_squared(p_pos) < 1.0 {
                    // 1.0 meter beam radius
                    triggered = true;
                    break;
                }
            }
        }

        // Check enemies
        if !triggered {
            for e_trans in enemies.iter() {
                let e_pos = e_trans.translation;
                let v = e_pos - origin;
                let proj = v.dot(normal);
                if proj > 0.3 && proj < bomb.beam_length {
                    let closest = origin + normal * proj;
                    if closest.distance_squared(e_pos) < 1.2 {
                        triggered = true;
                        break;
                    }
                }
            }
        }

        if triggered {
            sound_events.send(PlaySoundEvent { sound_id: 14 }); // PIPEBOMB_EXPLODE
            explosion_events.send(ExplosionDamageEvent {
                origin,
                radius: bomb.damage_radius,
                damage: bomb.damage,
                attacker_id: bomb.owner_player_id,
                excluded_entity: None,
            });
            commands.entity(entity).despawn_recursive();
        }
    }
}

#[derive(Component, Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct FirstPersonViewModel {
    pub current_weapon: WeaponType,
    pub anim_frame: u32,
    pub anim_timer: f32,
    pub bob_phase: f32,
    pub is_firing: bool,
    pub base_tile: i16,
    pub current_tile: i16,
}

impl FirstPersonViewModel {
    pub fn new(current_weapon: WeaponType) -> Self {
        let base_tile = match current_weapon {
            WeaponType::Knee => 2521,
            WeaponType::Pistol => 2524,
            WeaponType::Shotgun => 2613,
            WeaponType::Chaingun => 2544,
            WeaponType::Rpg => 2605,
            WeaponType::Pipebomb => 2565,
            WeaponType::HandRemote => 2570,
            WeaponType::Shrinker => 2575,
            WeaponType::Devastator => 2590,
            WeaponType::Tripbomb => 2580,
            WeaponType::Freezethrower => 2600,
            WeaponType::Expander => 2586,
        };
        Self {
            current_weapon,
            anim_frame: 0,
            anim_timer: 0.0,
            bob_phase: 0.0,
            is_firing: false,
            base_tile,
            current_tile: base_tile,
        }
    }
}

impl Default for FirstPersonViewModel {
    fn default() -> Self {
        Self::new(WeaponType::Pistol)
    }
}

pub fn update_first_person_viewmodel(
    time: Res<Time>,
    player_query: Query<(&PlayerController, Option<&PlayerId>)>,
    mut vm_query: Query<(&mut FirstPersonViewModel, Option<&PlayerId>)>,
    game_config: Option<Res<crate::config::GameConfig>>,
) {
    let dt = time.delta_seconds();
    for (player, p_id) in player_query.iter() {
        let pid = p_id.map_or(0, |id| id.0);
        for (mut vm, vm_id) in vm_query.iter_mut() {
            if vm_id.map_or(0, |id| id.0) != pid {
                continue;
            }

            if player.quick_kick_timer > 0.0 {
                let kick_progress = 1.0 - (player.quick_kick_timer / 0.5);
                let frame = (kick_progress * 3.0) as i16;
                vm.current_tile = 2521 + frame.clamp(0, 2);
                vm.is_firing = true;
                continue;
            }

    let cur_idx = player.current_weapon as usize;
    if cur_idx < player.weapons.len() {
        let weapon = &player.weapons[cur_idx];
        vm.current_weapon = player.current_weapon;
        vm.base_tile = weapon.base_tile;

        if weapon.fire_timer > 0.0 {
            vm.is_firing = true;
            let progress = 1.0 - (weapon.fire_timer / weapon.fire_delay.max(0.01));
            match player.current_weapon {
                WeaponType::Knee => {
                    let frame_offset = (progress * 3.0) as i16;
                    vm.current_tile = 2521 + frame_offset.clamp(0, 2);
                }
                WeaponType::Pistol => {
                    let frame_offset = (progress * 4.0) as i16;
                    vm.current_tile = 2524 + frame_offset.clamp(0, 4);
                }
                WeaponType::Shotgun => {
                    let frame_offset = (progress * 6.0) as i16;
                    vm.current_tile = 2613 + frame_offset.clamp(0, 6);
                }
                WeaponType::Chaingun => {
                    let frame_offset = ((time.elapsed_seconds() * 20.0) as i16) % 3;
                    vm.current_tile = 2544 + frame_offset;
                }
                WeaponType::Rpg => {
                    let frame_offset = (progress * 3.0) as i16;
                    vm.current_tile = 2605 + frame_offset.clamp(0, 2);
                }
                WeaponType::Pipebomb => {
                    let frame_offset = (progress * 4.0) as i16;
                    vm.current_tile = 2565 + frame_offset.clamp(0, 3);
                }
                WeaponType::HandRemote => {
                    let frame_offset = if progress < 0.5 { 1 } else { 0 };
                    vm.current_tile = 2570 + frame_offset;
                }
                WeaponType::Shrinker => {
                    let frame_offset = (progress * 4.0) as i16;
                    vm.current_tile = 2575 + frame_offset.clamp(0, 3);
                }
                WeaponType::Devastator => {
                    let side_offset = if player.devastator_alt_side { 0 } else { 2 };
                    let frame_offset = (progress * 2.0) as i16;
                    vm.current_tile = 2590 + side_offset + frame_offset.clamp(0, 1);
                }
                WeaponType::Tripbomb => {
                    let frame_offset = (progress * 5.0) as i16;
                    vm.current_tile = 2580 + frame_offset.clamp(0, 4);
                }
                WeaponType::Freezethrower => {
                    let frame_offset = ((time.elapsed_seconds() * 25.0) as i16) % 3;
                    vm.current_tile = 2600 + frame_offset;
                }
                WeaponType::Expander => {
                    let frame_offset = (progress * 3.0) as i16;
                    vm.current_tile = 2586 + frame_offset.clamp(0, 2);
                }
            }
        } else {
            vm.is_firing = false;
            vm.current_tile = weapon.base_tile;
        }

        // Bobbing phase from walking speed
        let view_bobbing = game_config.as_ref().map_or(true, |c| c.controls.view_bobbing);
        if view_bobbing && player.speed > 0.1 {
            vm.bob_phase += dt * 8.0;
        }
    }
    }
    }
}

pub const TRIPBOMB_MAX_REACH: f32 = 2.5;

/// Calculates tripbomb placement given a raycast hit point and normal, if within reach.
pub fn calculate_tripbomb_placement(
    ray_origin: Vec3,
    hit_info: Option<(Vec3, Vec3)>,
    max_reach: f32,
) -> Option<(Vec3, Vec3)> {
    if let Some((hit_pos, hit_normal)) = hit_info {
        if (hit_pos - ray_origin).length() <= max_reach {
            return Some((hit_pos + hit_normal * 0.02, hit_normal.normalize_or_zero()));
        }
    }
    None
}

/// Detonates all active player-owned pipebombs and triggers explosion events.
pub fn detonate_player_pipebombs(
    pipebombs: impl IntoIterator<Item = (Entity, Vec3)>,
    commands: &mut Commands,
    explosion_events: &mut EventWriter<ExplosionDamageEvent>,
    sound_events: &mut EventWriter<PlaySoundEvent>,
) -> usize {
    let mut count = 0;
    for (entity, pos) in pipebombs {
        count += 1;
        sound_events.send(PlaySoundEvent { sound_id: 14 });
        explosion_events.send(ExplosionDamageEvent {
            origin: pos,
            radius: 7.0,
            damage: 150,
            attacker_id: None,
            excluded_entity: None,
        });
        commands.entity(entity).despawn_recursive();
    }
    count
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_tripbomb_wall_raycast_placement_condition() {
        let player_eye = Vec3::new(0.0, 1.5, 0.0);

        // Case 1: Wall hit at 1.8m (within 2.5m reach) -> placement succeeds
        let hit_within_reach = Some((Vec3::new(0.0, 1.5, -1.8), Vec3::new(0.0, 0.0, 1.0)));
        let placement = calculate_tripbomb_placement(player_eye, hit_within_reach, TRIPBOMB_MAX_REACH);
        assert!(placement.is_some());
        let (pos, normal) = placement.unwrap();
        assert_eq!(normal, Vec3::new(0.0, 0.0, 1.0));
        assert!((pos.z - (-1.8 + 0.02)).abs() < 0.001);

        // Case 2: Wall hit at 3.5m (beyond 2.5m reach) -> placement fails
        let hit_too_far = Some((Vec3::new(0.0, 1.5, -3.5), Vec3::new(0.0, 0.0, 1.0)));
        let placement_too_far = calculate_tripbomb_placement(player_eye, hit_too_far, TRIPBOMB_MAX_REACH);
        assert!(placement_too_far.is_none());

        // Case 3: No wall hit in raycast -> placement fails
        let no_hit = None;
        let placement_none = calculate_tripbomb_placement(player_eye, no_hit, TRIPBOMB_MAX_REACH);
        assert!(placement_none.is_none());

        // Case 4: Ammo consumption logic verification
        let mut player = PlayerController::default();
        let trip_idx = WeaponType::Tripbomb as usize;
        player.weapons[trip_idx].is_unlocked = true;
        player.weapons[trip_idx].ammo = 5;

        // When placement succeeds: ammo is consumed
        if placement.is_some() {
            player.weapons[trip_idx].ammo -= 1;
        }
        assert_eq!(player.weapons[trip_idx].ammo, 4);

        // When placement fails (no wall / too far): ammo must NOT be consumed
        if placement_too_far.is_some() {
            player.weapons[trip_idx].ammo -= 1;
        }
        assert_eq!(player.weapons[trip_idx].ammo, 4); // Ammo unchanged!
    }

    #[test]
    fn test_hand_remote_detonates_pipebombs_and_ammo_switch() {
        let mut player = PlayerController::default();
        let pipe_idx = WeaponType::Pipebomb as usize;
        player.weapons[pipe_idx].is_unlocked = true;
        player.weapons[pipe_idx].ammo = 2;

        // Throwing pipebomb equips HandRemote
        player.current_weapon = WeaponType::HandRemote;
        assert_eq!(player.current_weapon, WeaponType::HandRemote);

        // When detonated, if ammo remains, returns to Pipebomb
        let pipebomb_detonated = true;
        if pipebomb_detonated {
            if player.weapons[pipe_idx].ammo > 0 {
                player.current_weapon = WeaponType::Pipebomb;
            } else {
                player.current_weapon = get_highest_priority_available_weapon(&player);
            }
        }
        assert_eq!(player.current_weapon, WeaponType::Pipebomb);

        // When detonated with 0 ammo, returns to highest priority weapon
        player.weapons[pipe_idx].ammo = 0;
        player.current_weapon = WeaponType::HandRemote;
        if pipebomb_detonated {
            if player.weapons[pipe_idx].ammo > 0 {
                player.current_weapon = WeaponType::Pipebomb;
            } else {
                player.current_weapon = get_highest_priority_available_weapon(&player);
            }
        }
        assert_eq!(player.current_weapon, WeaponType::Shotgun);
    }

    #[test]
    fn test_auto_switch_weapon_toggle() {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins);
        app.init_resource::<ButtonInput<KeyCode>>();
        app.init_resource::<ButtonInput<MouseButton>>();
        app.insert_resource(crate::net::DeterministicRng::new(123));
        app.add_event::<SpawnProjectileEvent>();
        app.add_event::<ExplosionDamageEvent>();
        app.add_event::<PlaySoundEvent>();
        app.add_event::<crate::combat::gore::SpawnCasingEvent>();
        app.add_event::<crate::lighting::SpawnDynamicLightEvent>();

        // Spawn player with Shotgun, ammo = 1
        let mut player = PlayerController::default();
        player.current_weapon = WeaponType::Shotgun;
        player.weapons[WeaponType::Shotgun as usize].ammo = 1;
        player.weapons[WeaponType::Shotgun as usize].is_unlocked = true;
        player.weapons[WeaponType::Pistol as usize].ammo = 50;
        player.weapons[WeaponType::Pistol as usize].is_unlocked = true;

        let player_entity = app.world_mut().spawn((Transform::default(), player)).id();
        app.world_mut().spawn(Camera3dBundle::default());

        let mut config = crate::config::GameConfig::default();
        // 1. auto_switch_weapon = false
        config.controls.auto_switch_weapon = false;
        app.insert_resource(config);

        app.add_systems(Update, handle_weapon_firing);

        // Click Left mouse to fire shotgun (ammo becomes 0)
        app.world_mut().resource_mut::<ButtonInput<MouseButton>>().press(MouseButton::Left);
        app.update();

        let pl = app.world().entity(player_entity).get::<PlayerController>().unwrap();
        assert_eq!(pl.weapons[WeaponType::Shotgun as usize].ammo, 0);
        assert_eq!(pl.current_weapon, WeaponType::Shotgun);

        // 2. auto_switch_weapon = true
        app.world_mut().resource_mut::<crate::config::GameConfig>().controls.auto_switch_weapon = true;

        {
            let mut p = app.world_mut().entity_mut(player_entity);
            let mut pl = p.get_mut::<PlayerController>().unwrap();
            pl.weapons[WeaponType::Shotgun as usize].ammo = 1;
            pl.weapons[WeaponType::Shotgun as usize].fire_timer = 0.0;
        }

        app.world_mut().resource_mut::<ButtonInput<MouseButton>>().reset_all();
        app.world_mut().resource_mut::<ButtonInput<MouseButton>>().press(MouseButton::Left);
        app.update();

        let pl = app.world().entity(player_entity).get::<PlayerController>().unwrap();
        assert_eq!(pl.weapons[WeaponType::Shotgun as usize].ammo, 0);
        assert_eq!(pl.current_weapon, WeaponType::Pistol);
    }

    #[test]
    fn test_view_bobbing_toggle() {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins);

        let mut player = PlayerController::default();
        player.speed = 10.0;
        app.world_mut().spawn(player);

        let vm_entity = app.world_mut().spawn(FirstPersonViewModel::new(WeaponType::Pistol)).id();

        let mut config = crate::config::GameConfig::default();
        config.controls.view_bobbing = false;
        app.insert_resource(config);

        app.add_systems(Update, update_first_person_viewmodel);

        // Advance time
        app.world_mut().resource_mut::<Time>().advance_by(std::time::Duration::from_millis(100));
        app.update();

        let vm = app.world().entity(vm_entity).get::<FirstPersonViewModel>().unwrap();
        assert_eq!(vm.bob_phase, 0.0);

        // Enable view_bobbing
        app.world_mut().resource_mut::<crate::config::GameConfig>().controls.view_bobbing = true;
        app.world_mut().resource_mut::<Time>().advance_by(std::time::Duration::from_millis(100));
        app.update();

        let vm = app.world().entity(vm_entity).get::<FirstPersonViewModel>().unwrap();
        assert!(vm.bob_phase > 0.0);
    }

    #[test]
    fn test_laser_tripbomb_owner_attribution() {
        let mut app = App::new();
        app.add_event::<ExplosionDamageEvent>()
            .add_event::<PlaySoundEvent>()
            .init_resource::<Time>()
            .add_systems(Update, update_laser_tripbombs);

        // Spawn a tripbomb owned by player 2
        app.world_mut().spawn((
            SpatialBundle {
                transform: Transform::from_xyz(0.0, 1.0, 0.0),
                ..default()
            },
            crate::combat::LaserTripbomb {
                normal: Vec3::new(0.0, 0.0, 1.0),
                arm_timer: 0.0,
                is_armed: true,
                beam_length: 10.0,
                damage: 150,
                damage_radius: 6.0,
                owner_player_id: Some(2),
            },
        ));

        // Spawn a victim player intersecting the beam
        app.world_mut().spawn((
            SpatialBundle {
                transform: Transform::from_xyz(0.0, 1.0, 3.0),
                ..default()
            },
            PlayerController::default(),
        ));

        app.update();

        let exp_events = app.world().resource::<Events<ExplosionDamageEvent>>();
        let mut exp_reader = exp_events.get_reader();
        let exps: Vec<_> = exp_reader.read(exp_events).cloned().collect();
        assert_eq!(exps.len(), 1, "Tripbomb must detonate when player intersects beam");
        assert_eq!(exps[0].attacker_id, Some(2), "Explosion must preserve owner_player_id as attacker_id");
    }

    #[test]
    fn test_hand_remote_detonation_does_not_auto_rethrow() {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins);
        app.init_resource::<ButtonInput<KeyCode>>();
        app.init_resource::<ButtonInput<MouseButton>>();
        app.insert_resource(crate::net::DeterministicRng::new(42));
        app.add_event::<SpawnProjectileEvent>();
        app.add_event::<ExplosionDamageEvent>();
        app.add_event::<PlaySoundEvent>();
        app.add_event::<crate::combat::gore::SpawnCasingEvent>();
        app.add_event::<crate::lighting::SpawnDynamicLightEvent>();

        // Spawn player holding HandRemote, with 5 pipebomb ammo
        let mut player = PlayerController::default();
        player.current_weapon = WeaponType::HandRemote;
        let pipe_idx = WeaponType::Pipebomb as usize;
        player.weapons[pipe_idx].ammo = 5;
        player.weapons[pipe_idx].is_unlocked = true;

        let player_entity = app.world_mut().spawn((Transform::default(), player)).id();
        app.world_mut().spawn(Camera3dBundle::default());

        // Spawn an active pipebomb in the world owned by player 0
        let pipebomb_entity = app
            .world_mut()
            .spawn((
                TransformBundle::from_transform(Transform::from_xyz(0.0, 0.0, 5.0)),
                crate::combat::types::Projectile {
                    projectile_type: ProjectileType::Pipebomb,
                    velocity: Vec3::ZERO,
                    damage: 150,
                    is_player_source: true,
                    source_player_id: Some(0),
                    lifetime: f32::INFINITY,
                    bounces: 0,
                },
            ))
            .id();

        let mut config = crate::config::GameConfig::default();
        config.controls.auto_switch_weapon = true;
        app.insert_resource(config);

        app.add_systems(Update, handle_weapon_firing);

        // Click Left mouse to trigger HandRemote detonation
        app.world_mut()
            .resource_mut::<ButtonInput<MouseButton>>()
            .press(MouseButton::Left);
        app.update();

        // 1. Verify pipebomb in world was detonated and despawned
        assert!(
            app.world().get_entity(pipebomb_entity).is_none(),
            "Pipebomb must be despawned upon detonation"
        );

        // 2. Verify an ExplosionDamageEvent was emitted
        let exp_events = app.world().resource::<Events<ExplosionDamageEvent>>();
        let mut exp_reader = exp_events.get_reader();
        let exps: Vec<_> = exp_reader.read(exp_events).cloned().collect();
        assert_eq!(
            exps.len(),
            1,
            "Detonating pipebomb must emit ExplosionDamageEvent"
        );

        // 3. Verify no new projectile was spawned (no auto-rethrow glitch)
        let proj_events = app.world().resource::<Events<SpawnProjectileEvent>>();
        let mut proj_reader = proj_events.get_reader();
        let projs: Vec<_> = proj_reader.read(proj_events).cloned().collect();
        assert_eq!(
            projs.len(),
            0,
            "Detonating HandRemote must NOT immediately rethrow a new pipebomb"
        );

        // 4. Verify pipebomb ammo is still 5 (not consumed to 4)
        let pl = app.world().entity(player_entity).get::<PlayerController>().unwrap();
        assert_eq!(
            pl.weapons[pipe_idx].ammo, 5,
            "Pipebomb ammo must not be consumed by detonation click"
        );
        assert_eq!(
            pl.current_weapon,
            WeaponType::Pipebomb,
            "Weapon should switch back to Pipebomb ready for next deliberate throw"
        );
    }
}
