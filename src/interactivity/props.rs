use crate::interactivity::types::*;
use bevy::prelude::*;

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
    mut shared_keycards: Option<ResMut<crate::net::coop::SharedKeycards>>,
    mut commands: Commands,
) {
    for event in interact_events.read() {
        let player_pos = event.player_pos;
        let player_dir = event.player_dir.normalize_or_zero();

        // 0. Check nearby Keycard pickups
        for mut player in player_query.iter_mut() {
            for (k_entity, k_trans, k_pickup) in keycards.iter() {
                if k_trans.translation.distance_squared(player_pos) < 6.25 {
                    match k_pickup.key_type {
                        1 => player.has_blue_key = true,
                        2 => player.has_red_key = true,
                        3 => player.has_yellow_key = true,
                        _ => {}
                    }
                    if let Some(ref mut sk) = shared_keycards {
                        sk.give_key(k_pickup.key_type);
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
            if dist_sq < 9.0 {
                // 3.0 meters
                let facing = if dist_sq > 0.01 {
                    player_dir.dot(to_obj.normalize_or_zero())
                } else {
                    1.0
                };

                if facing > 0.1 {
                    // Check Keycard requirement for Access Switches
                    if let SwitchType::AccessSwitch { key_required } = switch.switch_type {
                        let has_key = player_query.iter().any(|player| match key_required {
                            1 => player.has_blue_key,
                            2 => player.has_red_key,
                            3 => player.has_yellow_key,
                            _ => true,
                        }) || shared_keycards.as_ref().map_or(false, |s| s.has_key(key_required));

                        if !has_key {
                            sound_events.send(PlaySoundEvent { sound_id: 86 }); // ACCESS_DENIED
                            continue;
                        }
                    }

                    switch.is_on = !switch.is_on;
                    if switch.sound_id != 0 {
                        sound_events.send(PlaySoundEvent {
                            sound_id: switch.sound_id,
                        });
                    }
                    let target_tile = if switch.is_on {
                        switch.on_tile
                    } else {
                        switch.off_tile
                    };

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
            if dist_sq < 16.0 {
                // 4.0 meters
                let facing = if dist_sq > 0.01 {
                    player_dir.dot(to_obj.normalize_or_zero())
                } else {
                    1.0
                };

                if facing > -0.2 {
                    // Facing generally towards the effector or inside the sector
                    if effector.lotag != 0 {
                        tag_events.send(ActivateTagEvent {
                            lotag: effector.lotag,
                        });
                    } else {
                        effector.active = true;
                        match &mut effector.kind {
                            EffectorKind::RotatingDoor { is_open, .. } => {
                                *is_open = !*is_open;
                                sound_events.send(PlaySoundEvent { sound_id: 110 });
                                // DOOR_OPERATE1
                            }
                            EffectorKind::SlidingDoor {
                                is_open,
                                auto_close_timer,
                                auto_close_delay,
                                ..
                            } => {
                                *is_open = !*is_open;
                                if *is_open {
                                    *auto_close_timer = Some(*auto_close_delay);
                                }
                                sound_events.send(PlaySoundEvent { sound_id: 110 });
                                // DOOR_OPERATE1
                            }
                            EffectorKind::Elevator {
                                is_at_top,
                                auto_return_timer,
                                ..
                            } => {
                                *is_at_top = !*is_at_top;
                                *auto_return_timer = if *is_at_top { Some(5.0) } else { None };
                                sound_events.send(PlaySoundEvent { sound_id: 110 });
                                // DOOR_OPERATE1
                            }
                            EffectorKind::Earthquake {
                                is_triggered,
                                elapsed,
                                ..
                            } => {
                                *is_triggered = true;
                                *elapsed = 0.0;
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
            if dist_sq < 4.84 && !fountain.is_broken && fountain.uses_left > 0 {
                // 2.2 * 2.2
                fountain.uses_left -= 1;
                heal_events.send(PlayerHealEvent { amount: 1 });
                sound_events.send(PlaySoundEvent { sound_id: 36 }); // DUKE_DRINKING
                if fountain.uses_left == 0 {
                    fountain.is_broken = true;
                    sound_events.send(PlaySoundEvent { sound_id: 19 }); // GLASS_BREAKING
                }
            }
        }

        // 3. Check toilets & stalls
        for (trans, mut toilet) in toilets.iter_mut() {
            let to_obj = trans.translation - player_pos;
            let dist_sq = to_obj.length_squared();
            if dist_sq < 4.84 && !toilet.is_broken {
                // 2.2 * 2.2
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
            if dist_sq < 7.84 && !nuke.is_activated {
                // 2.8 * 2.8
                let facing = if dist_sq > 0.01 {
                    player_dir.dot(to_obj.normalize_or_zero())
                } else {
                    1.0
                };

                if facing > 0.1 {
                    nuke.is_activated = true;
                    sound_events.send(PlaySoundEvent { sound_id: 83 }); // END_OF_LEVEL_WARN
                    level_completed_events.send(crate::game_flow::LevelCompletedEvent {
                        is_secret: nuke.is_secret,
                        is_boss_victory: false,
                    });
                }
            }
        }

        // 5. Check Mirrors (Duke Taunt Quote)
        for (trans, mut mirror) in mirrors.iter_mut() {
            let to_obj = trans.translation - player_pos;
            let dist_sq = to_obj.length_squared();
            if dist_sq < 6.25 && mirror.cooldown_timer <= 0.0 {
                // 2.5 meters
                let facing = if dist_sq > 0.01 {
                    player_dir.dot(to_obj.normalize_or_zero())
                } else {
                    1.0
                };
                if facing > 0.3 {
                    mirror.cooldown_timer = 15.0;
                    duke_voice_events.send(crate::audio::PlayDukeVoiceEvent {
                        name: Some("LOOK01".into()),
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
    let Ok(player_trans) = player_query.get_single() else {
        return;
    };
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

#[inline]
pub fn calculate_hitradius_damage(dist: f32, radius: f32, max_damage: i32) -> i32 {
    if dist > radius || radius <= 0.0 {
        return 0;
    }
    let fraction = dist / radius;
    if fraction <= 0.25 {
        max_damage
    } else if fraction <= 0.50 {
        max_damage * 3 / 4
    } else if fraction <= 0.75 {
        max_damage / 2
    } else {
        max_damage / 4
    }
}

pub fn handle_explosions(
    mut explosion_events: EventReader<ExplosionDamageEvent>,
    mut barrel_explode_events: EventWriter<BarrelExplodeEvent>,
    mut barrels: Query<(Entity, &Transform, &mut ExplodingBarrel)>,
    mut fire_extinguishers: Query<(Entity, &Transform, &mut FireExtinguisher)>,
    mut fountains: Query<(&Transform, &mut WaterFountain)>,
    mut crack_walls: Query<(Entity, &Transform, &mut CrackWall)>,
    mut glass_windows: Query<(Entity, &Transform, &mut BreakableGlass)>,
    mut players: Query<(Entity, &Transform, &mut crate::player::PlayerController, Option<&crate::player::types::PlayerId>)>,
    mut enemies: Query<(
        Entity,
        &Transform,
        &mut crate::combat::EnemyActor,
        Option<&mut crate::scripting::ConActor>,
    )>,
    mut tag_events: EventWriter<ActivateTagEvent>,
    mut sound_events: EventWriter<PlaySoundEvent>,
    mut gib_events: EventWriter<crate::combat::GibEvent>,
    mut light_events: EventWriter<crate::lighting::SpawnDynamicLightEvent>,
    mut frag_events: EventWriter<crate::net::PlayerFragEvent>,
    mut tint: Option<ResMut<crate::hud::ScreenTintState>>,
    mut commands: Commands,
) {
    for exp in explosion_events.read() {
        let origin = exp.origin;

        // Dynamic point light flash for explosion
        light_events.send(crate::lighting::SpawnDynamicLightEvent::explosion(
            origin,
            exp.radius * 20.0,
        ));

        // Screen tint flash on major explosion
        if let Some(ref mut t) = tint {
            t.target_color = Color::srgba(1.0, 0.65, 0.15, 0.65);
        }

        // 1. Damage Player with 4-tier hitradius falloff and armor mitigation
        for (p_entity, p_trans, mut player, opt_id) in players.iter_mut() {
            if Some(p_entity) == exp.excluded_entity {
                continue;
            }
            let dist = p_trans.translation.distance(origin);
            if dist <= exp.radius && !player.god_mode && player.health > 0 {
                let mut damage = calculate_hitradius_damage(dist, exp.radius, exp.damage);
                if damage > 0 {
                    if player.armor > 0 {
                        let absorbed = (damage * 3 / 4).min(player.armor);
                        player.armor -= absorbed;
                        damage -= absorbed;
                    }
                    let was_alive = player.health > 0;
                    player.health -= damage;
                    if player.health <= 0 && was_alive {
                        player.health = 0;
                        player.death_timer = 3.0;
                        sound_events.send(PlaySoundEvent { sound_id: 41 }); // DUKE_DEAD
                        let victim_id = opt_id.map_or(0, |id| id.0);
                        let killer_id = exp.attacker_id.unwrap_or(victim_id);
                        frag_events.send(crate::net::PlayerFragEvent {
                            killer_id,
                            victim_id,
                            weapon_type: 4, // RPG/Explosive
                        });
                    } else if player.health > 0 {
                        sound_events.send(PlaySoundEvent { sound_id: 37 }); // DUKE_PAIN
                    }
                    if let Some(ref mut t) = tint {
                        t.target_color = Color::srgba(0.9, 0.2, 0.0, 0.7);
                    }
                }
            }
        }

        // 2. Damage Enemies with 4-tier hitradius falloff & gibbing
        for (e_entity, e_trans, mut enemy, con_actor) in enemies.iter_mut() {
            if Some(e_entity) == exp.excluded_entity {
                continue;
            }
            let dist = e_trans.translation.distance(origin);
            if dist <= exp.radius && enemy.health > 0 {
                let damage = calculate_hitradius_damage(dist, exp.radius, exp.damage);
                if damage > 0 {
                    enemy.health -= damage;
                    if let Some(mut con) = con_actor {
                        con.extra = enemy.health as i16;
                    }
                    if enemy.health <= 0 {
                        enemy.state = crate::combat::EnemyAiState::Gibbed;
                        gib_events.send(crate::combat::GibEvent {
                            origin: e_trans.translation,
                            gib_count: 8,
                        });
                    } else {
                        enemy.state = crate::combat::EnemyAiState::Flinching;
                    }
                }
            }
        }

        // 3. Damage Exploding Barrels
        for (entity, trans, mut barrel) in barrels.iter_mut() {
            if !barrel.is_exploded {
                let dist = trans.translation.distance(origin);
                if dist <= exp.radius {
                    let dmg = calculate_hitradius_damage(dist, exp.radius, exp.damage);
                    barrel.health -= dmg;
                    if barrel.health <= 0 {
                        barrel.is_exploded = true;
                        sound_events.send(PlaySoundEvent { sound_id: 14 }); // PIPEBOMB_EXPLODE
                        barrel_explode_events.send(BarrelExplodeEvent {
                            origin: trans.translation,
                            radius: barrel.damage_radius,
                            damage: barrel.damage,
                            attacker_id: exp.attacker_id,
                        });
                        gib_events.send(crate::combat::GibEvent {
                            origin: trans.translation,
                            gib_count: 6,
                        });
                        commands.entity(entity).despawn_recursive();
                    }
                }
            }
        }

        // 4. Damage Crack Walls & Air Vent Covers
        for (entity, trans, mut crack) in crack_walls.iter_mut() {
            if !crack.is_blown {
                let dist = trans.translation.distance(origin);
                if dist <= exp.radius {
                    let dmg = calculate_hitradius_damage(dist, exp.radius, exp.damage);
                    crack.health -= dmg;
                    if crack.health <= 0 {
                        crack.is_blown = true;
                        crack.stage = 4;
                        sound_events.send(PlaySoundEvent { sound_id: 18 }); // VENT_BUST
                        if crack.lotag != 0 {
                            tag_events.send(ActivateTagEvent { lotag: crack.lotag });
                        }
                        gib_events.send(crate::combat::GibEvent {
                            origin: trans.translation,
                            gib_count: 6,
                        });
                        commands.entity(entity).despawn_recursive();
                    }
                }
            }
        }

        // 5. Break Glass Windows
        for (entity, trans, mut glass) in glass_windows.iter_mut() {
            if !glass.is_broken {
                let dist = trans.translation.distance(origin);
                if dist <= exp.radius {
                    glass.is_broken = true;
                    sound_events.send(PlaySoundEvent { sound_id: 19 }); // GLASS_BREAKING
                    commands.entity(entity).despawn_recursive();
                }
            }
        }

        // 6. Damage Fire Extinguishers
        for (entity, trans, mut ext) in fire_extinguishers.iter_mut() {
            if !ext.is_exploded {
                let dist = trans.translation.distance(origin);
                if dist <= exp.radius {
                    let dmg = calculate_hitradius_damage(dist, exp.radius, exp.damage);
                    ext.health -= dmg;
                    if ext.health <= 0 {
                        ext.is_exploded = true;
                        sound_events.send(PlaySoundEvent { sound_id: 14 }); // PIPEBOMB_EXPLODE
                        barrel_explode_events.send(BarrelExplodeEvent {
                            origin: trans.translation,
                            radius: 4.0,
                            damage: 80,
                            attacker_id: exp.attacker_id,
                        });
                        gib_events.send(crate::combat::GibEvent {
                            origin: trans.translation,
                            gib_count: 4,
                        });
                        commands.entity(entity).despawn_recursive();
                    }
                }
            }
        }

        // 7. Damage Water Fountains
        for (trans, mut fountain) in fountains.iter_mut() {
            if !fountain.is_broken {
                let dist = trans.translation.distance(origin);
                if dist <= exp.radius {
                    fountain.is_broken = true;
                    fountain.uses_left = 0;
                    sound_events.send(PlaySoundEvent { sound_id: 19 }); // GLASS_BREAKING
                    gib_events.send(crate::combat::GibEvent {
                        origin: trans.translation,
                        gib_count: 3,
                    });
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
            attacker_id: exp.attacker_id,
            excluded_entity: None,
        });
    }
}

pub fn update_surveillance_monitors(
    time: Res<Time>,
    mut viewscreens: Query<(&Transform, &mut ViewscreenProp)>,
    mut cameras: Query<(&mut Transform, &mut SecurityCamera), Without<ViewscreenProp>>,
    player_query: Query<
        &Transform,
        (
            With<crate::Player>,
            Without<ViewscreenProp>,
            Without<SecurityCamera>,
        ),
    >,
) {
    let dt = time.delta_seconds();
    let Ok(player_trans) = player_query.get_single() else {
        return;
    };
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

pub fn update_mirror_props(time: Res<Time>, mut mirrors: Query<&mut MirrorProp>) {
    let dt = time.delta_seconds();
    for mut mirror in mirrors.iter_mut() {
        if mirror.cooldown_timer > 0.0 {
            mirror.cooldown_timer -= dt;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_calculate_hitradius_damage_tiers() {
        let max_dmg = 100;
        let radius = 10.0;

        // Tier 1: <= 25% distance -> 100% damage
        assert_eq!(calculate_hitradius_damage(0.0, radius, max_dmg), 100);
        assert_eq!(calculate_hitradius_damage(2.5, radius, max_dmg), 100);

        // Tier 2: <= 50% distance -> 75% damage
        assert_eq!(calculate_hitradius_damage(3.0, radius, max_dmg), 75);
        assert_eq!(calculate_hitradius_damage(5.0, radius, max_dmg), 75);

        // Tier 3: <= 75% distance -> 50% damage
        assert_eq!(calculate_hitradius_damage(6.0, radius, max_dmg), 50);
        assert_eq!(calculate_hitradius_damage(7.5, radius, max_dmg), 50);

        // Tier 4: <= 100% distance -> 25% damage
        assert_eq!(calculate_hitradius_damage(8.0, radius, max_dmg), 25);
        assert_eq!(calculate_hitradius_damage(10.0, radius, max_dmg), 25);

        // Outside radius -> 0 damage
        assert_eq!(calculate_hitradius_damage(10.1, radius, max_dmg), 0);
        assert_eq!(calculate_hitradius_damage(15.0, radius, max_dmg), 0);
    }

    #[test]
    fn test_explosion_screen_tint_and_radioactive_barrels() {
        let mut app = App::new();
        app.add_event::<ExplosionDamageEvent>()
            .add_event::<BarrelExplodeEvent>()
            .add_event::<ActivateTagEvent>()
            .add_event::<PlaySoundEvent>()
            .add_event::<crate::combat::GibEvent>()
            .add_event::<crate::lighting::SpawnDynamicLightEvent>()
            .add_event::<crate::net::PlayerFragEvent>()
            .init_resource::<crate::hud::ScreenTintState>()
            .add_systems(Update, handle_explosions);

        // Spawn a radioactive barrel entity
        let barrel_entity = app.world_mut().spawn((
            ExplodingBarrel {
                health: 20,
                damage_radius: 6.0,
                damage: 100,
                is_exploded: false,
            },
            TransformBundle::from_transform(Transform::from_xyz(0.0, 0.0, 2.0)),
        )).id();

        // Trigger an explosion near the barrel
        app.world_mut().send_event(ExplosionDamageEvent {
            origin: Vec3::new(0.0, 0.0, 0.0),
            radius: 5.0,
            damage: 50,
            attacker_id: None,
            excluded_entity: None,
        });

        app.update();

        // 1. Verify screen tint triggered
        let tint = app.world().resource::<crate::hud::ScreenTintState>();
        assert!(
            tint.target_color.alpha() > 0.0,
            "Explosion must trigger screen tint flash"
        );

        // 2. Verify radioactive barrel detonated
        let barrel_events = app.world().resource::<Events<BarrelExplodeEvent>>();
        let mut barrel_reader = barrel_events.get_reader();
        let explodes: Vec<_> = barrel_reader.read(barrel_events).cloned().collect();
        assert_eq!(
            explodes.len(),
            1,
            "Explosion must detonate nearby radioactive barrel"
        );

        // 3. Verify barrel entity despawned
        assert!(app.world().get_entity(barrel_entity).is_none());
    }

    #[test]
    fn test_voxel_prop_destruction_debris_and_lights() {
        let mut app = App::new();
        app.add_event::<ExplosionDamageEvent>()
            .add_event::<BarrelExplodeEvent>()
            .add_event::<ActivateTagEvent>()
            .add_event::<PlaySoundEvent>()
            .add_event::<crate::combat::GibEvent>()
            .add_event::<crate::lighting::SpawnDynamicLightEvent>()
            .add_event::<crate::net::PlayerFragEvent>()
            .init_resource::<crate::hud::ScreenTintState>()
            .add_systems(Update, handle_explosions);

        // Spawn a barrel and a fire extinguisher
        let barrel_entity = app.world_mut().spawn((
            ExplodingBarrel {
                health: 20,
                damage_radius: 6.0,
                damage: 100,
                is_exploded: false,
            },
            Transform::from_xyz(0.0, 0.0, 1.5),
        )).id();

        let fireext_entity = app.world_mut().spawn((
            FireExtinguisher {
                health: 10,
                is_exploded: false,
            },
            Transform::from_xyz(0.0, 0.0, 2.5),
        )).id();

        let fountain_entity = app.world_mut().spawn((
            WaterFountain {
                uses_left: 10,
                is_broken: false,
                broken_tile: 566,
            },
            Transform::from_xyz(0.0, 0.0, 3.0),
        )).id();

        // Trigger an explosion
        app.world_mut().send_event(ExplosionDamageEvent {
            origin: Vec3::new(0.0, 0.0, 0.0),
            radius: 5.0,
            damage: 80,
            attacker_id: None,
            excluded_entity: None,
        });

        app.update();

        // Verify dynamic point lights sent for explosion
        let light_events = app.world().resource::<Events<crate::lighting::SpawnDynamicLightEvent>>();
        assert!(!light_events.is_empty(), "Explosion must spawn dynamic light flash");

        // Verify debris gib events sent
        let gib_events = app.world().resource::<Events<crate::combat::GibEvent>>();
        assert!(!gib_events.is_empty(), "Prop explosions must emit debris particles");

        // Verify barrel and fire extinguisher despawned
        assert!(app.world().get_entity(barrel_entity).is_none(), "Barrel must despawn upon destruction");
        assert!(app.world().get_entity(fireext_entity).is_none(), "Fire extinguisher must despawn upon explosion");

        // Verify fountain is marked broken and uses depleted
        let fountain = app.world().get::<WaterFountain>(fountain_entity).unwrap();
        assert!(fountain.is_broken, "Water fountain must be broken by explosion");
        assert_eq!(fountain.uses_left, 0, "Water fountain uses must be 0 after break");
    }

    #[test]
    fn test_water_fountain_depletion_and_damage_break() {
        let mut app = App::new();
        app.add_event::<InteractEvent>()
            .add_event::<ActivateTagEvent>()
            .add_event::<PlayerHealEvent>()
            .add_event::<PlaySoundEvent>()
            .add_event::<crate::audio::PlayDukeVoiceEvent>()
            .add_event::<crate::game_flow::LevelCompletedEvent>()
            .init_resource::<Assets<StandardMaterial>>()
            .add_systems(Update, handle_player_interactions);

        // Spawn player at (0, 0, 0)
        app.world_mut().spawn(crate::player::PlayerController::default());

        // Spawn fountain with 1 use left within interaction range
        let fountain_entity = app.world_mut().spawn((
            Transform::from_xyz(0.0, 0.0, 1.0),
            WaterFountain {
                uses_left: 1,
                is_broken: false,
                broken_tile: 566,
            },
        )).id();

        // Send interact event
        app.world_mut().send_event(InteractEvent {
            player_pos: Vec3::ZERO,
            player_dir: Vec3::Z,
        });
        app.update();

        // Fountain should now be used up and broken
        let fountain = app.world().get::<WaterFountain>(fountain_entity).unwrap();
        assert_eq!(fountain.uses_left, 0);
        assert!(fountain.is_broken, "Fountain must become broken after last use");
    }

    #[test]
    fn test_shooting_barrel_propagates_attacker_and_awards_frag() {
        use crate::interactivity::WallDamageEvent;
        let mut app = App::new();
        app.add_event::<WallDamageEvent>()
            .add_event::<ExplosionDamageEvent>()
            .add_event::<BarrelExplodeEvent>()
            .add_event::<ActivateTagEvent>()
            .add_event::<PlaySoundEvent>()
            .add_event::<crate::combat::GibEvent>()
            .add_event::<crate::lighting::SpawnDynamicLightEvent>()
            .add_event::<crate::net::PlayerFragEvent>()
            .init_resource::<crate::hud::ScreenTintState>()
            .add_systems(
                Update,
                (
                    crate::interactivity::wall_damage::handle_wall_damage,
                    handle_barrel_chain_explosions,
                    handle_explosions,
                )
                    .chain(),
            );

        // Spawn explosive barrel at origin
        let barrel_entity = app
            .world_mut()
            .spawn((
                ExplodingBarrel {
                    health: 20,
                    damage_radius: 6.0,
                    damage: 100,
                    is_exploded: false,
                },
                TransformBundle::from_transform(Transform::from_xyz(0.0, 0.0, 0.0)),
            ))
            .id();

        // Spawn victim player (Player 1) at distance 1.0m
        let mut victim_ctrl = crate::player::PlayerController::default();
        victim_ctrl.health = 30;
        app.world_mut().spawn((
            victim_ctrl,
            crate::player::types::PlayerId(1),
            TransformBundle::from_transform(Transform::from_xyz(0.0, 0.0, 1.0)),
        ));

        // Shooter (Player 0) shoots the barrel
        app.world_mut().send_event(WallDamageEvent {
            hit_point: Vec3::ZERO,
            hit_normal: Vec3::Y,
            damage: 30,
            is_explosive: false,
            hit_entity: Some(barrel_entity),
            attacker_id: Some(0),
        });

        app.update();
        app.update();

        let exp_events = app.world().resource::<Events<ExplosionDamageEvent>>();
        let mut exp_reader = exp_events.get_reader();
        let exps: Vec<_> = exp_reader.read(exp_events).cloned().collect();
        assert_eq!(exps.len(), 1, "ExplosionDamageEvent must be emitted by handle_wall_damage");
        assert_eq!(exps[0].attacker_id, Some(0), "Attacker ID must be Some(0)");
        let victim_hp = app.world_mut().query::<&crate::player::PlayerController>().iter(app.world()).next().unwrap().health;
        assert_eq!(victim_hp, 0, "Victim health must be 0 after lethal explosion");

        // Verify PlayerFragEvent was sent with killer_id: 0, victim_id: 1
        let frag_events = app.world().resource::<Events<crate::net::PlayerFragEvent>>();
        let mut frag_reader = frag_events.get_reader();
        let frags: Vec<_> = frag_reader.read(frag_events).cloned().collect();
        assert_eq!(
            frags.len(),
            1,
            "Victim player dying to barrel explosion must emit PlayerFragEvent"
        );
        assert_eq!(frags[0].killer_id, 0, "Killer must be credited to shooter (Player 0)");
        assert_eq!(frags[0].victim_id, 1, "Victim must be Player 1");
    }

    #[test]
    fn test_victim_of_trap_or_barrel_credited_with_death_not_suicide() {
        let mut dmatch = crate::net::scoreboard::DukematchState::default();
        // Process a frag where player 0 killed player 1 via trap/barrel explosion
        dmatch.record_frag(0, 1);

        assert_eq!(
            dmatch.frags[0][1], 1,
            "Player 0 must receive frag credit against Player 1"
        );
        assert_eq!(dmatch.get_total_frags(0), 1, "Player 0 total frags must be 1");
        assert_eq!(
            dmatch.get_total_frags(1),
            0,
            "Player 1 total frags must be 0 (not -1 suicide penalty)"
        );
        assert_eq!(dmatch.get_deaths(1), 1, "Player 1 must have 1 death recorded");
    }

    #[test]
    fn test_aoe_explosion_destroys_and_despawns_crack_wall() {
        let mut app = App::new();
        app.add_event::<ExplosionDamageEvent>()
            .add_event::<BarrelExplodeEvent>()
            .add_event::<ActivateTagEvent>()
            .add_event::<PlaySoundEvent>()
            .add_event::<crate::combat::GibEvent>()
            .add_event::<crate::lighting::SpawnDynamicLightEvent>()
            .add_event::<crate::net::PlayerFragEvent>()
            .init_resource::<crate::hud::ScreenTintState>()
            .add_systems(Update, handle_explosions);

        let crack_entity = app
            .world_mut()
            .spawn((
                CrackWall {
                    health: 20,
                    stage: 1,
                    lotag: 55,
                    is_blown: false,
                },
                TransformBundle::from_transform(Transform::from_xyz(1.0, 0.0, 0.0)),
            ))
            .id();

        // Explosion originates at origin with 6.0m radius and 100 damage (hits entity at 1.0m)
        app.world_mut().send_event(ExplosionDamageEvent {
            origin: Vec3::ZERO,
            radius: 6.0,
            damage: 100,
            attacker_id: None,
            excluded_entity: None,
        });

        app.update();

        // 1. Entity must be despawned from world completely
        assert!(
            app.world().get_entity(crack_entity).is_none(),
            "CrackWall entity must be completely despawned from the world following lethal AoE explosion"
        );

        // 2. VENT_BUST sound must be emitted
        let sound_events = app.world().resource::<Events<PlaySoundEvent>>();
        let mut sound_reader = sound_events.get_reader();
        let sounds: Vec<_> = sound_reader.read(sound_events).cloned().collect();
        assert!(
            sounds.iter().any(|s| s.sound_id == 18),
            "VENT_BUST (18) sound effect must be emitted when crack wall explodes"
        );

        // 3. GibEvent must be emitted
        let gib_events = app.world().resource::<Events<crate::combat::GibEvent>>();
        let mut gib_reader = gib_events.get_reader();
        let gibs: Vec<_> = gib_reader.read(gib_events).cloned().collect();
        assert_eq!(gibs.len(), 1, "GibEvent must be emitted upon vent destruction");
        assert_eq!(gibs[0].origin, Vec3::new(1.0, 0.0, 0.0));

        // 4. Tag activation event must be emitted
        let tag_events = app.world().resource::<Events<ActivateTagEvent>>();
        let mut tag_reader = tag_events.get_reader();
        let tags: Vec<_> = tag_reader.read(tag_events).cloned().collect();
        assert_eq!(tags.len(), 1);
        assert_eq!(tags[0].lotag, 55);
    }
}
