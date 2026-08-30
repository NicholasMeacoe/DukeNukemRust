#![allow(dead_code)]

use bevy::prelude::*;
use crate::player::types::*;
use crate::combat::types::{SpawnProjectileEvent, ProjectileType, Projectile};
use crate::interactivity::types::ExplosionDamageEvent;
use crate::audio::PlaySoundEvent;

pub fn handle_weapon_selection(
    keys: Res<ButtonInput<KeyCode>>,
    mut query: Query<&mut PlayerController>,
    mut sound_events: EventWriter<PlaySoundEvent>,
) {
    let Ok(mut player) = query.get_single_mut() else { return; };

    let selected = if keys.just_pressed(KeyCode::Digit1) {
        Some(WeaponType::Knee)
    } else if keys.just_pressed(KeyCode::Digit2) {
        Some(WeaponType::Pistol)
    } else if keys.just_pressed(KeyCode::Digit3) {
        Some(WeaponType::Shotgun)
    } else if keys.just_pressed(KeyCode::Digit4) {
        Some(WeaponType::Chaingun)
    } else if keys.just_pressed(KeyCode::Digit5) {
        Some(WeaponType::Rpg)
    } else if keys.just_pressed(KeyCode::Digit6) {
        Some(WeaponType::Pipebomb)
    } else if keys.just_pressed(KeyCode::Digit7) {
        if player.current_weapon == WeaponType::Shrinker && player.weapons[WeaponType::Expander as usize].is_unlocked {
            Some(WeaponType::Expander)
        } else {
            Some(WeaponType::Shrinker)
        }
    } else if keys.just_pressed(KeyCode::Digit8) {
        Some(WeaponType::Devastator)
    } else if keys.just_pressed(KeyCode::Digit9) {
        Some(WeaponType::Tripbomb)
    } else if keys.just_pressed(KeyCode::Digit0) {
        Some(WeaponType::Freezethrower)
    } else {
        None
    };

    if let Some(weapon_type) = selected {
        let idx = weapon_type as usize;
        if idx < player.weapons.len() && player.weapons[idx].is_unlocked {
            player.current_weapon = weapon_type;
            sound_events.send(PlaySoundEvent { sound_id: 118 }); // SELECT_WEAPON
        }
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
        if wt == WeaponType::Knee || (player.weapons[idx].is_unlocked && player.weapons[idx].ammo > 0) {
            return wt;
        }
    }
    WeaponType::Knee
}

pub fn handle_weapon_firing(
    keys: Res<ButtonInput<KeyCode>>,
    btn: Res<ButtonInput<MouseButton>>,
    time: Res<Time>,
    mut query: Query<(&Transform, &mut PlayerController), Without<Projectile>>,
    camera_query: Query<&Transform, (With<Camera>, Without<PlayerController>, Without<Projectile>)>,
    mut projectile_events: EventWriter<SpawnProjectileEvent>,
    mut explosion_events: EventWriter<ExplosionDamageEvent>,
    mut sound_events: EventWriter<PlaySoundEvent>,
    mut casing_events: EventWriter<crate::combat::gore::SpawnCasingEvent>,
    pipebomb_query: Query<(Entity, &Transform, &Projectile), Without<PlayerController>>,
    mut commands: Commands,
) {
    let Ok(cam_trans) = camera_query.get_single() else { return; };
    let Ok((player_trans, mut player)) = query.get_single_mut() else { return; };
    let dt = time.delta_seconds();

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

    // Quick kick check (Key 'Q')
    if keys.just_pressed(KeyCode::KeyQ) && player.quick_kick_timer <= 0.0 {
        player.quick_kick_timer = 0.5;
        sound_events.send(PlaySoundEvent { sound_id: 0 }); // KICK_HIT
        let kick_damage = if player.inventory.steroids_active { 40 } else { 15 };
        projectile_events.send(SpawnProjectileEvent {
            projectile_type: ProjectileType::MightyBoot,
            origin: player_trans.translation + Vec3::Y * 0.2,
            direction: fwd_vec,
            velocity: 15.0,
            damage: kick_damage,
            is_player_source: true,
        });
    }

    let cur_idx = player.current_weapon as usize;
    if cur_idx >= player.weapons.len() {
        return;
    }

    let is_firing = if player.current_weapon == WeaponType::Chaingun || player.current_weapon == WeaponType::Freezethrower {
        btn.pressed(MouseButton::Left)
    } else {
        btn.just_pressed(MouseButton::Left)
    };

    // Detonator Trigger on Right Click or HandRemote weapon
    if btn.just_pressed(MouseButton::Right) || (player.current_weapon == WeaponType::HandRemote && btn.just_pressed(MouseButton::Left)) {
        for (entity, p_trans, proj) in pipebomb_query.iter() {
            if proj.projectile_type == ProjectileType::Pipebomb && proj.is_player_source {
                sound_events.send(PlaySoundEvent { sound_id: 14 }); // PIPEBOMB_EXPLODE
                explosion_events.send(ExplosionDamageEvent {
                    origin: p_trans.translation,
                    radius: 7.0,
                    damage: 150,
                });
                commands.entity(entity).despawn_recursive();
            }
        }
    }

    if is_firing && player.weapons[cur_idx].fire_timer <= 0.0 && player.weapons[cur_idx].reload_timer <= 0.0 {
        let fire_pos = player_trans.translation + Vec3::Y * 0.4 + fwd_vec * 0.5;

        match player.current_weapon {
            WeaponType::Knee => {
                let kick_damage = if player.inventory.steroids_active { 40 } else { 15 };
                player.weapons[cur_idx].fire_timer = player.weapons[cur_idx].fire_delay;
                sound_events.send(PlaySoundEvent { sound_id: 0 }); // KICK_HIT
                projectile_events.send(SpawnProjectileEvent {
                    projectile_type: ProjectileType::MightyBoot,
                    origin: fire_pos,
                    direction: fwd_vec,
                    velocity: 15.0,
                    damage: kick_damage,
                    is_player_source: true,
                });
            }
            WeaponType::Pistol => {
                if player.pistol_mag > 0 {
                    player.pistol_mag -= 1;
                    player.weapons[cur_idx].ammo = player.weapons[cur_idx].ammo.saturating_sub(1);
                    player.weapons[cur_idx].fire_timer = player.weapons[cur_idx].fire_delay;
                    sound_events.send(PlaySoundEvent { sound_id: 3 }); // PISTOL_FIRE

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

                    casing_events.send(crate::combat::gore::SpawnCasingEvent {
                        origin: fire_pos,
                        direction: fwd_vec,
                        is_shotgun: true,
                    });

                    // Spawn 7 spread pellets
                    for _ in 0..7 {
                        let spread_x = (rand::random::<f32>() - 0.5) * 0.06;
                        let spread_y = (rand::random::<f32>() - 0.5) * 0.06;
                        let dir = (fwd_vec + right_vec * spread_x + up_vec * spread_y).normalize_or_zero();

                        projectile_events.send(SpawnProjectileEvent {
                            projectile_type: ProjectileType::ShotgunPellet,
                            origin: fire_pos,
                            direction: dir,
                            velocity: 120.0,
                            damage: 9,
                            is_player_source: true,
                        });
                    }
                }
            }
            WeaponType::Chaingun => {
                if player.weapons[cur_idx].ammo > 0 {
                    player.weapons[cur_idx].ammo -= 1;
                    player.weapons[cur_idx].fire_timer = player.weapons[cur_idx].fire_delay;
                    sound_events.send(PlaySoundEvent { sound_id: 6 }); // CHAINGUN_FIRE

                    casing_events.send(crate::combat::gore::SpawnCasingEvent {
                        origin: fire_pos,
                        direction: fwd_vec,
                        is_shotgun: false,
                    });

                    let spread_x = (rand::random::<f32>() - 0.5) * 0.03;
                    let spread_y = (rand::random::<f32>() - 0.5) * 0.03;
                    let dir = (fwd_vec + right_vec * spread_x + up_vec * spread_y).normalize_or_zero();

                    projectile_events.send(SpawnProjectileEvent {
                        projectile_type: ProjectileType::HitscanBullet,
                        origin: fire_pos,
                        direction: dir,
                        velocity: 150.0,
                        damage: 10,
                        is_player_source: true,
                    });
                }
            }
            WeaponType::Rpg => {
                if player.weapons[cur_idx].ammo > 0 {
                    player.weapons[cur_idx].ammo -= 1;
                    player.weapons[cur_idx].fire_timer = player.weapons[cur_idx].fire_delay;
                    sound_events.send(PlaySoundEvent { sound_id: 7 }); // RPG_FIRE

                    projectile_events.send(SpawnProjectileEvent {
                        projectile_type: ProjectileType::Rocket,
                        origin: fire_pos,
                        direction: fwd_vec,
                        velocity: 35.0,
                        damage: 120,
                        is_player_source: true,
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
                    });

                    player.current_weapon = WeaponType::HandRemote;
                }
            }
            WeaponType::Shrinker => {
                if player.weapons[cur_idx].ammo > 0 {
                    player.weapons[cur_idx].ammo -= 1;
                    player.weapons[cur_idx].fire_timer = player.weapons[cur_idx].fire_delay;
                    sound_events.send(PlaySoundEvent { sound_id: 11 }); // SHRINKER_FIRE

                    projectile_events.send(SpawnProjectileEvent {
                        projectile_type: ProjectileType::ShrinkRay,
                        origin: fire_pos,
                        direction: fwd_vec,
                        velocity: 30.0,
                        damage: 0, // Applies shrink status
                        is_player_source: true,
                    });
                }
            }
            WeaponType::Devastator => {
                if player.weapons[cur_idx].ammo > 0 {
                    player.weapons[cur_idx].ammo -= 1;
                    player.weapons[cur_idx].fire_timer = player.weapons[cur_idx].fire_delay;
                    sound_events.send(PlaySoundEvent { sound_id: 10 }); // CAT_FIRE

                    player.devastator_alt_side = !player.devastator_alt_side;
                    let side_offset = if player.devastator_alt_side { 0.2 } else { -0.2 };
                    let pod_pos = fire_pos + right_vec * side_offset;

                    projectile_events.send(SpawnProjectileEvent {
                        projectile_type: ProjectileType::DevastatorMissile,
                        origin: pod_pos,
                        direction: fwd_vec,
                        velocity: 45.0,
                        damage: 40,
                        is_player_source: true,
                    });
                }
            }
            WeaponType::Tripbomb => {
                if player.weapons[cur_idx].ammo > 0 {
                    player.weapons[cur_idx].ammo -= 1;
                    player.weapons[cur_idx].fire_timer = player.weapons[cur_idx].fire_delay;
                    sound_events.send(PlaySoundEvent { sound_id: 16 }); // TRIPBOMB_ARM

                    let attach_pos = fire_pos + fwd_vec * 1.5;
                    let outward_normal = -fwd_vec;

                    commands.spawn((
                        SpatialBundle {
                            transform: Transform::from_translation(attach_pos),
                            ..default()
                        },
                        crate::combat::LaserTripbomb {
                            normal: outward_normal,
                            arm_timer: 1.0,
                            is_armed: false,
                            beam_length: 12.0,
                            damage: 150,
                            damage_radius: 6.0,
                        },
                        crate::game_flow::LevelEntity,
                    ));
                }
            }
            WeaponType::Freezethrower => {
                if player.weapons[cur_idx].ammo > 0 {
                    player.weapons[cur_idx].ammo -= 1;
                    player.weapons[cur_idx].fire_timer = player.weapons[cur_idx].fire_delay;
                    sound_events.send(PlaySoundEvent { sound_id: 110 }); // SOMETHINGFROZE

                    projectile_events.send(SpawnProjectileEvent {
                        projectile_type: ProjectileType::FreezeShard,
                        origin: fire_pos,
                        direction: fwd_vec,
                        velocity: 25.0,
                        damage: 15, // Freezes on death
                        is_player_source: true,
                    });
                }
            }
            WeaponType::Expander => {
                if player.weapons[cur_idx].ammo > 0 {
                    player.weapons[cur_idx].ammo -= 1;
                    player.weapons[cur_idx].fire_timer = player.weapons[cur_idx].fire_delay;
                    sound_events.send(PlaySoundEvent { sound_id: 11 }); // EXPANDER_FIRE

                    projectile_events.send(SpawnProjectileEvent {
                        projectile_type: ProjectileType::ExpanderRay,
                        origin: fire_pos,
                        direction: fwd_vec,
                        velocity: 30.0,
                        damage: 25,
                        is_player_source: true,
                    });
                }
            }
            _ => {}
        }

        // Priority auto-switch if out of ammo
        if player.weapons[cur_idx].ammo == 0 && cur_idx != WeaponType::Knee as usize {
            let next_wpn = get_highest_priority_available_weapon(&player);
            if next_wpn != player.current_weapon {
                player.current_weapon = next_wpn;
                sound_events.send(PlaySoundEvent { sound_id: 118 });
            }
        }
    }
}

pub fn update_laser_tripbombs(
    time: Res<Time>,
    mut commands: Commands,
    mut tripbombs: Query<(Entity, &Transform, &mut crate::combat::LaserTripbomb)>,
    player_query: Query<&Transform, (With<PlayerController>, Without<crate::combat::LaserTripbomb>)>,
    enemies: Query<&Transform, (With<crate::combat::EnemyActor>, Without<crate::combat::LaserTripbomb>)>,
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
                if closest.distance_squared(p_pos) < 1.0 { // 1.0 meter beam radius
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
            });
            commands.entity(entity).despawn_recursive();
        }
    }
}

#[derive(Component, Debug, Clone)]
pub struct FirstPersonViewModel {
    pub current_weapon: WeaponType,
    pub anim_frame: u32,
    pub anim_timer: f32,
    pub bob_phase: f32,
    pub is_firing: bool,
    pub base_tile: i16,
    pub current_tile: i16,
}

impl Default for FirstPersonViewModel {
    fn default() -> Self {
        Self {
            current_weapon: WeaponType::Pistol,
            anim_frame: 0,
            anim_timer: 0.0,
            bob_phase: 0.0,
            is_firing: false,
            base_tile: 2524,
            current_tile: 2524,
        }
    }
}

pub fn update_first_person_viewmodel(
    time: Res<Time>,
    player_query: Query<&PlayerController>,
    mut vm_query: Query<&mut FirstPersonViewModel>,
) {
    let dt = time.delta_seconds();
    let Ok(player) = player_query.get_single() else { return; };
    let Ok(mut vm) = vm_query.get_single_mut() else { return; };

    let cur_idx = player.current_weapon as usize;
    if cur_idx < player.weapons.len() {
        let weapon = &player.weapons[cur_idx];
        vm.current_weapon = player.current_weapon;
        vm.base_tile = weapon.base_tile;

        if weapon.fire_timer > 0.0 {
            vm.is_firing = true;
            let progress = 1.0 - (weapon.fire_timer / weapon.fire_delay.max(0.01));
            match player.current_weapon {
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
                _ => {
                    vm.current_tile = weapon.base_tile;
                }
            }
        } else {
            vm.is_firing = false;
            vm.current_tile = weapon.base_tile;
        }

        // Bobbing phase from walking speed
        if player.speed > 0.1 {
            vm.bob_phase += dt * 8.0;
        }
    }
}
