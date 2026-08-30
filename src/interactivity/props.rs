#![allow(dead_code)]

use bevy::prelude::*;
use crate::interactivity::types::*;

pub fn handle_player_interactions(
    mut interact_events: EventReader<InteractEvent>,
    mut switches: Query<(&Transform, &mut InteractiveSwitch)>,
    mut effectors: Query<(&Transform, &mut SectorEffectorComponent)>,
    mut nuke_switches: Query<(&Transform, &mut NukeExitSwitch)>,
    mut fountains: Query<(&Transform, &mut WaterFountain)>,
    mut toilets: Query<(&Transform, &mut ToiletProp)>,
    mut mirrors: Query<(&Transform, &mut MirrorProp)>,
    keycards: Query<(Entity, &Transform, &KeycardPickup)>,
    mut player_query: Query<&mut crate::player::PlayerController>,
    (
        mut tag_events,
        mut heal_events,
        mut sound_events,
        mut duke_voice_events,
        mut level_completed_events,
    ): (
        EventWriter<ActivateTagEvent>,
        EventWriter<PlayerHealEvent>,
        EventWriter<PlaySoundEvent>,
        EventWriter<crate::audio::PlayDukeVoiceEvent>,
        EventWriter<crate::game_flow::LevelCompletedEvent>,
    ),
    mut materials: ResMut<Assets<StandardMaterial>>,
    assets: Option<Res<crate::GameAssets>>,
    mut commands: Commands,
) {
    for event in interact_events.read() {
        let player_pos = event.player_pos;
        let player_dir = event.player_dir.normalize_or_zero();

        // 0. Check nearby Keycard pickups
        if let Ok(mut player) = player_query.get_single_mut() {
            for (k_entity, k_trans, k_pickup) in keycards.iter() {
                if k_trans.translation.distance_squared(player_pos) < 6.25 {
                    match k_pickup.key_type {
                        1 => player.has_blue_key = true,
                        2 => player.has_red_key = true,
                        3 => player.has_yellow_key = true,
                        _ => {}
                    }
                    sound_events.send(PlaySoundEvent { sound_id: 65 }); // KEYCARD_GET
                    commands.entity(k_entity).despawn_recursive();
                }
            }
        }

        // 1. Check nearby switches with facing angle check
        for (trans, mut switch) in switches.iter_mut() {
            let to_obj = trans.translation - player_pos;
            let dist_sq = to_obj.length_squared();
            if dist_sq < 9.0 { // 3.0 meters
                let facing = if dist_sq > 0.01 {
                    player_dir.dot(to_obj.normalize_or_zero())
                } else {
                    1.0
                };

                if facing > 0.1 {
                    // Check Keycard requirement for Access Switches
                    if let SwitchType::AccessSwitch { key_required } = switch.switch_type {
                        if let Ok(player) = player_query.get_single() {
                            let has_key = match key_required {
                                1 => player.has_blue_key,
                                2 => player.has_red_key,
                                3 => player.has_yellow_key,
                                _ => true,
                            };
                            if !has_key {
                                sound_events.send(PlaySoundEvent { sound_id: 86 }); // ACCESS_DENIED
                                continue;
                            }
                        }
                    }

                    switch.is_on = !switch.is_on;
                    if switch.sound_id != 0 {
                        sound_events.send(PlaySoundEvent { sound_id: switch.sound_id });
                    }
                    let target_tile = if switch.is_on { switch.on_tile } else { switch.off_tile };

                    // Swap visual texture on the switch material
                    if let Some(ref mat_handle) = switch.material_handle {
                        if let Some(ref game_assets) = assets {
                            if let Some(tex_handle) = game_assets.tile_textures.get(&target_tile) {
                                if let Some(mat) = materials.get_mut(mat_handle) {
                                    mat.base_color_texture = Some(tex_handle.clone());
                                }
                            }
                        }
                    }

                    if switch.lotag != 0 {
                        tag_events.send(ActivateTagEvent {
                            lotag: switch.lotag,
                        });
                    }
                }
            }
        }

        // 2. Direct Door & Sector Effector Interaction (e.g. pressing E on door or elevator)
        for (trans, mut effector) in effectors.iter_mut() {
            let to_obj = trans.translation - player_pos;
            let dist_sq = to_obj.length_squared();
            if dist_sq < 16.0 { // 4.0 meters
                let facing = if dist_sq > 0.01 {
                    player_dir.dot(to_obj.normalize_or_zero())
                } else {
                    1.0
                };

                if facing > -0.2 { // Facing generally towards the effector or inside the sector
                    if effector.lotag != 0 {
                        tag_events.send(ActivateTagEvent {
                            lotag: effector.lotag,
                        });
                    } else {
                        effector.active = true;
                        match &mut effector.kind {
                            EffectorKind::RotatingDoor { is_open, .. } => {
                                *is_open = !*is_open;
                                sound_events.send(PlaySoundEvent { sound_id: 110 }); // DOOR_OPERATE1
                            }
                            EffectorKind::SlidingDoor { is_open, auto_close_timer, auto_close_delay, .. } => {
                                *is_open = !*is_open;
                                if *is_open {
                                    *auto_close_timer = Some(*auto_close_delay);
                                }
                                sound_events.send(PlaySoundEvent { sound_id: 110 }); // DOOR_OPERATE1
                            }
                            EffectorKind::Elevator { is_at_top, auto_return_timer, .. } => {
                                *is_at_top = !*is_at_top;
                                *auto_return_timer = if *is_at_top { Some(5.0) } else { None };
                                sound_events.send(PlaySoundEvent { sound_id: 110 }); // DOOR_OPERATE1
                            }
                            _ => {}
                        }
                    }
                }
            }
        }

        // 2. Check drinking fountains
        for (trans, mut fountain) in fountains.iter_mut() {
            let to_obj = trans.translation - player_pos;
            let dist_sq = to_obj.length_squared();
            if dist_sq < 4.84 && !fountain.is_broken && fountain.uses_left > 0 { // 2.2 * 2.2
                fountain.uses_left -= 1;
                heal_events.send(PlayerHealEvent { amount: 1 });
                sound_events.send(PlaySoundEvent { sound_id: 36 }); // DUKE_DRINKING
            }
        }

        // 3. Check toilets & stalls
        for (trans, mut toilet) in toilets.iter_mut() {
            let to_obj = trans.translation - player_pos;
            let dist_sq = to_obj.length_squared();
            if dist_sq < 4.84 && !toilet.is_broken { // 2.2 * 2.2
                if toilet.last_used_time <= 0.0 {
                    toilet.last_used_time = 10.0;
                    heal_events.send(PlayerHealEvent { amount: 10 });
                    sound_events.send(PlaySoundEvent { sound_id: 79 }); // FLUSH_TOILET
                }
            }
        }

        // 4. Check Nuke Buttons (Level Exit) with facing check
        for (trans, mut nuke) in nuke_switches.iter_mut() {
            let to_obj = trans.translation - player_pos;
            let dist_sq = to_obj.length_squared();
            if dist_sq < 7.84 && !nuke.is_activated { // 2.8 * 2.8
                let facing = if dist_sq > 0.01 {
                    player_dir.dot(to_obj.normalize_or_zero())
                } else {
                    1.0
                };

                if facing > 0.1 {
                    nuke.is_activated = true;
                    sound_events.send(PlaySoundEvent { sound_id: 83 }); // END_OF_LEVEL_WARN
                    level_completed_events.send(crate::game_flow::LevelCompletedEvent);
                }
            }
        }

        // 5. Check Mirrors (Duke Taunt Quote)
        for (trans, mut mirror) in mirrors.iter_mut() {
            let to_obj = trans.translation - player_pos;
            let dist_sq = to_obj.length_squared();
            if dist_sq < 6.25 && mirror.cooldown_timer <= 0.0 { // 2.5 meters
                let facing = if dist_sq > 0.01 {
                    player_dir.dot(to_obj.normalize_or_zero())
                } else {
                    1.0
                };
                if facing > 0.3 {
                    mirror.cooldown_timer = 15.0;
                    duke_voice_events.send(crate::audio::PlayDukeVoiceEvent {
                        name: Some("LOOKING_GOOD".into()),
                    });
                }
            }
        }
    }
}

pub fn handle_touchplates(
    player_query: Query<&Transform, With<crate::Player>>,
    mut touchplates: Query<(&Transform, &mut Touchplate)>,
    mut tag_events: EventWriter<ActivateTagEvent>,
) {
    let Ok(player_trans) = player_query.get_single() else { return; };
    let p_pos = player_trans.translation;

    for (trans, mut touchplate) in touchplates.iter_mut() {
        let dist_sq = trans.translation.distance_squared(p_pos);
        let rad_sq = touchplate.radius * touchplate.radius;
        if dist_sq <= rad_sq {
            if !touchplate.triggered {
                touchplate.triggered = true;
                if touchplate.lotag != 0 {
                    tag_events.send(ActivateTagEvent {
                        lotag: touchplate.lotag,
                    });
                }
            }
        } else {
            touchplate.triggered = false;
        }
    }
}

pub fn handle_explosions(
    mut explosion_events: EventReader<ExplosionDamageEvent>,
    mut barrel_explode_events: EventWriter<BarrelExplodeEvent>,
    mut barrels: Query<(Entity, &Transform, &mut ExplodingBarrel)>,
    mut crack_walls: Query<(&Transform, &mut CrackWall)>,
    mut glass_windows: Query<(Entity, &Transform, &mut BreakableGlass)>,
    mut tag_events: EventWriter<ActivateTagEvent>,
    mut sound_events: EventWriter<PlaySoundEvent>,
    mut commands: Commands,
) {
    for exp in explosion_events.read() {
        let origin = exp.origin;
        let radius_sq = exp.radius * exp.radius;

        // 1. Check exploding barrels
        for (entity, trans, mut barrel) in barrels.iter_mut() {
            if !barrel.is_exploded {
                let dist_sq = trans.translation.distance_squared(origin);
                if dist_sq <= radius_sq {
                    barrel.health -= exp.damage;
                    if barrel.health <= 0 {
                        barrel.is_exploded = true;
                        sound_events.send(PlaySoundEvent { sound_id: 14 }); // PIPEBOMB_EXPLODE
                        barrel_explode_events.send(BarrelExplodeEvent {
                            origin: trans.translation,
                            radius: barrel.damage_radius,
                            damage: barrel.damage,
                        });
                        commands.entity(entity).despawn_recursive();
                    }
                }
            }
        }

        // 2. Check crack walls
        for (trans, mut crack) in crack_walls.iter_mut() {
            if !crack.is_blown {
                let dist_sq = trans.translation.distance_squared(origin);
                if dist_sq <= radius_sq {
                    crack.health -= exp.damage;
                    if crack.health <= 0 {
                        crack.is_blown = true;
                        crack.stage = 4;
                        sound_events.send(PlaySoundEvent { sound_id: 18 }); // VENT_BUST
                        if crack.lotag != 0 {
                            tag_events.send(ActivateTagEvent {
                                lotag: crack.lotag,
                            });
                        }
                    }
                }
            }
        }

        // 3. Check breakable glass
        for (entity, trans, mut glass) in glass_windows.iter_mut() {
            if !glass.is_broken {
                let dist_sq = trans.translation.distance_squared(origin);
                if dist_sq <= radius_sq {
                    glass.is_broken = true;
                    sound_events.send(PlaySoundEvent { sound_id: 19 }); // GLASS_BREAKING
                    commands.entity(entity).despawn_recursive();
                }
            }
        }
    }
}

pub fn handle_barrel_chain_explosions(
    mut barrel_events: EventReader<BarrelExplodeEvent>,
    mut explosion_events: EventWriter<ExplosionDamageEvent>,
) {
    for exp in barrel_events.read() {
        explosion_events.send(ExplosionDamageEvent {
            origin: exp.origin,
            radius: exp.radius,
            damage: exp.damage,
        });
    }
}

pub fn update_surveillance_monitors(
    time: Res<Time>,
    mut viewscreens: Query<(&Transform, &mut ViewscreenProp)>,
    mut cameras: Query<(&mut Transform, &mut SecurityCamera), Without<ViewscreenProp>>,
    player_query: Query<&Transform, (With<crate::Player>, Without<ViewscreenProp>, Without<SecurityCamera>)>,
) {
    let dt = time.delta_seconds();
    let Ok(player_trans) = player_query.get_single() else { return; };
    let p_pos = player_trans.translation;

    // 1. Tick camera sweeping motion
    for (mut cam_trans, mut cam) in cameras.iter_mut() {
        cam.sweep_angle += cam.sweep_speed * dt;
        let yaw_offset = cam.sweep_angle.sin() * 0.5;
        let base_rad = (cam.base_yaw / 2048.0) * std::f32::consts::TAU;
        cam_trans.rotation = Quat::from_rotation_y(base_rad + yaw_offset);
    }

    // 2. Update viewscreen active status and scanline timer
    for (v_trans, mut viewscreen) in viewscreens.iter_mut() {
        if viewscreen.is_broken {
            continue;
        }
        let dist_sq = v_trans.translation.distance_squared(p_pos);
        viewscreen.is_active = dist_sq < 100.0; // Within 10 meters
        if viewscreen.is_active {
            viewscreen.scanline_timer = (viewscreen.scanline_timer + dt) % 1.0;
        }
    }
}

pub fn update_mirror_props(
    time: Res<Time>,
    mut mirrors: Query<&mut MirrorProp>,
) {
    let dt = time.delta_seconds();
    for mut mirror in mirrors.iter_mut() {
        if mirror.cooldown_timer > 0.0 {
            mirror.cooldown_timer -= dt;
        }
    }
}
