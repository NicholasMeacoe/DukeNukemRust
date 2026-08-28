#![allow(dead_code)]

use bevy::prelude::*;
use crate::player::types::*;
use crate::combat::types::{SpawnProjectileEvent, ProjectileType, Projectile};
use crate::interactivity::types::ExplosionDamageEvent;

pub fn handle_weapon_selection(
    keys: Res<ButtonInput<KeyCode>>,
    mut query: Query<&mut PlayerController>,
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
        Some(WeaponType::Shrinker)
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
        }
    }
}

pub fn handle_weapon_firing(
    time: Res<Time>,
    btn: Res<ButtonInput<MouseButton>>,
    keys: Res<ButtonInput<KeyCode>>,
    mut query: Query<(&Transform, &mut PlayerController)>,
    camera_query: Query<&Transform, With<Camera>>,
    pipebomb_query: Query<(Entity, &Transform, &Projectile)>,
    mut projectile_events: EventWriter<SpawnProjectileEvent>,
    mut explosion_events: EventWriter<ExplosionDamageEvent>,
    mut commands: Commands,
) {
    let dt = time.delta_seconds();
    let Ok((player_trans, mut player)) = query.get_single_mut() else { return; };
    let Ok(cam_trans) = camera_query.get_single() else { return; };

    // Update weapon timers
    for weapon in player.weapons.iter_mut() {
        if weapon.fire_timer > 0.0 {
            weapon.fire_timer -= dt;
        }
        if weapon.reload_timer > 0.0 {
            weapon.reload_timer -= dt;
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
        projectile_events.send(SpawnProjectileEvent {
            projectile_type: ProjectileType::MightyBoot,
            origin: player_trans.translation + Vec3::Y * 0.2,
            direction: fwd_vec,
            velocity: 15.0,
            damage: 30,
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
                player.weapons[cur_idx].fire_timer = player.weapons[cur_idx].fire_delay;
                projectile_events.send(SpawnProjectileEvent {
                    projectile_type: ProjectileType::MightyBoot,
                    origin: fire_pos,
                    direction: fwd_vec,
                    velocity: 15.0,
                    damage: 30,
                    is_player_source: true,
                });
            }
            WeaponType::Pistol => {
                if player.pistol_mag > 0 {
                    player.pistol_mag -= 1;
                    player.weapons[cur_idx].ammo = player.weapons[cur_idx].ammo.saturating_sub(1);
                    player.weapons[cur_idx].fire_timer = player.weapons[cur_idx].fire_delay;

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
                        player.weapons[cur_idx].reload_timer = 1.0;
                        player.pistol_mag = player.weapons[cur_idx].ammo.min(12);
                    }
                }
            }
            WeaponType::Shotgun => {
                if player.weapons[cur_idx].ammo > 0 {
                    player.weapons[cur_idx].ammo -= 1;
                    player.weapons[cur_idx].fire_timer = player.weapons[cur_idx].fire_delay;

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

                    let spread_x = (rand::random::<f32>() - 0.5) * 0.04;
                    let spread_y = (rand::random::<f32>() - 0.5) * 0.04;
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

                    projectile_events.send(SpawnProjectileEvent {
                        projectile_type: ProjectileType::Pipebomb,
                        origin: fire_pos,
                        direction: (fwd_vec + Vec3::Y * 0.2).normalize_or_zero(),
                        velocity: 18.0,
                        damage: 150,
                        is_player_source: true,
                    });
                }
            }
            WeaponType::Shrinker => {
                if player.weapons[cur_idx].ammo > 0 {
                    player.weapons[cur_idx].ammo -= 1;
                    player.weapons[cur_idx].fire_timer = player.weapons[cur_idx].fire_delay;

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
            WeaponType::Freezethrower => {
                if player.weapons[cur_idx].ammo > 0 {
                    player.weapons[cur_idx].ammo -= 1;
                    player.weapons[cur_idx].fire_timer = player.weapons[cur_idx].fire_delay;

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
            _ => {}
        }
    }
}
