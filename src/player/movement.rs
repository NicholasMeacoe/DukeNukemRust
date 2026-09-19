#![allow(dead_code)]

use crate::audio::PlaySoundEvent;
use crate::player::types::*;
use bevy::prelude::*;
use bevy_rapier3d::prelude::*;

pub fn update_player_movement(
    keys: Res<ButtonInput<KeyCode>>,
    time: Res<Time>,
    mut query: Query<(
        &mut Transform,
        &mut PlayerController,
        &mut KinematicCharacterController,
        Option<&KinematicCharacterControllerOutput>,
        Option<&crate::sector_map::CurrentSector>,
        Option<&mut Collider>,
    )>,
    camera_query: Query<&Transform, (With<Camera>, Without<PlayerController>)>,
    effectors: Query<&crate::interactivity::SectorEffectorComponent>,
    sector_map: Option<Res<crate::sector_map::SectorMap>>,
    mut sound_events: EventWriter<PlaySoundEvent>,
    mut tint: Option<ResMut<crate::hud::ScreenTintState>>,
) {
    let Ok(camera_transform) = camera_query.get_single() else {
        return;
    };
    let dt = time.delta_seconds();

    for (mut trans, mut player, mut controller, output, current_sector, mut collider) in query.iter_mut() {
        if player.health <= 0 {
            player.death_timer -= dt;
            controller.translation = Some(Vec3::new(0.0, -9.81 * dt, 0.0));
            if player.death_timer <= 0.0 {
                // Respawn at level spawn position
                player.health = player.max_health;
                player.armor = 0;
                trans.translation = player.spawn_position;
                player.velocity_y = 0.0;
                player.velocity_xz = Vec2::ZERO;
                player.death_timer = 0.0;
                player.current_weapon = WeaponType::Pistol;
                player.pistol_mag = 12;
            }
            continue;
        }

        if player.freeze_timer > 0.0 {
            // Cannot move while frozen
            controller.translation = Some(Vec3::new(0.0, -9.81 * dt, 0.0));
            continue;
        }

        // Void fall protection: if player ever falls below map limits, teleport back to start
        if trans.translation.y < -15.0 {
            println!(
                "Player fell into void (y = {}). Teleporting back to start position!",
                trans.translation.y
            );
            trans.translation = player.spawn_position + Vec3::Y * 0.5;
            player.velocity_y = 0.0;
            player.velocity_xz = Vec2::ZERO;
            controller.translation = None;
            continue;
        }

        let is_grounded = output.map_or(false, |out| out.grounded);
        let mut direction = Vec3::ZERO;

        let forward = camera_transform.forward().with_y(0.0).normalize_or_zero();
        let right = camera_transform.right().with_y(0.0).normalize_or_zero();

        if keys.pressed(KeyCode::KeyW) {
            direction += forward;
        }
        if keys.pressed(KeyCode::KeyS) {
            direction -= forward;
        }
        if keys.pressed(KeyCode::KeyA) {
            direction -= right;
        }
        if keys.pressed(KeyCode::KeyD) {
            direction += right;
        }

        // Sector lotag water detection (lotag 1: water surface, lotag 2: underwater)
        let in_water = match current_sector {
            Some(sec) if sec.0 >= 0 => {
                if let Some(ref sm) = sector_map {
                    let lotag = sm.get_sector_lotag(sec.0 as usize);
                    lotag == 1 || lotag == 2
                } else {
                    false
                }
            }
            _ => false,
        };

        let is_underwater = match current_sector {
            Some(sec) if sec.0 >= 0 => {
                if let Some(ref sm) = sector_map {
                    sm.get_sector_lotag(sec.0 as usize) == 2
                } else {
                    false
                }
            }
            _ => false,
        };

        // Check ceiling headroom for crouch un-toggling
        let ceiling_y = match current_sector {
            Some(sec) if sec.0 >= 0 => {
                if let Some(ref sm) = sector_map {
                    sm.get_ceil_y_at(sec.0 as usize, trans.translation.x, trans.translation.z)
                } else {
                    f32::INFINITY
                }
            }
            _ => f32::INFINITY,
        };
        let feet_y = trans.translation.y - 0.5;
        let headroom = ceiling_y - feet_y;
        let wants_crouch = keys.pressed(KeyCode::KeyC);
        // Prevent standing up if headroom is too low (< 1.6m)
        let force_crouch = player.movement_mode == PlayerMovementMode::Crouching && headroom < 1.6;

        let prev_movement_mode = player.movement_mode;

        // Determine player movement mode
        if is_underwater || (in_water && wants_crouch) {
            player.movement_mode = PlayerMovementMode::Diving;
        } else if in_water {
            player.movement_mode = PlayerMovementMode::Swimming;
        } else if player.inventory.jetpack_active {
            player.movement_mode = PlayerMovementMode::JetpackFlying;
        } else if wants_crouch || force_crouch {
            player.movement_mode = PlayerMovementMode::Crouching;
        } else {
            player.movement_mode = PlayerMovementMode::Standing;
        }

        // Resize Rapier collider based on crouching vs standing and adjust translation to keep feet anchored
        if let Some(ref mut col) = collider {
            if player.movement_mode == PlayerMovementMode::Crouching && prev_movement_mode != PlayerMovementMode::Crouching {
                **col = Collider::capsule_y(0.2, 0.3);
                // Lower center by 0.3 so capsule bottom (half_height 0.2 + radius 0.3 = 0.5)
                // matches previous bottom (half_height 0.5 + radius 0.3 = 0.8)
                trans.translation.y -= 0.3;
            } else if player.movement_mode != PlayerMovementMode::Crouching && prev_movement_mode == PlayerMovementMode::Crouching {
                **col = Collider::capsule_y(0.5, 0.3);
                // Raise center by 0.3 when standing up
                trans.translation.y += 0.3;
            }
        }

        // Diving blue tint
        if player.movement_mode == PlayerMovementMode::Diving {
            if let Some(ref mut t) = tint {
                t.target_color = Color::srgba(0.0, 0.25, 0.75, 0.45);
            }
        }

        // Calculate speed multiplier based on active buffs/debuffs
        let mut speed_multiplier = 1.0;
        if player.inventory.steroids_active {
            speed_multiplier *= 2.0;
        }
        if player.shrink_timer > 0.0 {
            speed_multiplier *= 0.5;
        }
        if player.movement_mode == PlayerMovementMode::Crouching {
            speed_multiplier *= 0.6;
        }

        let is_swimming = matches!(
            player.movement_mode,
            PlayerMovementMode::Swimming | PlayerMovementMode::Diving
        );
        if is_swimming {
            speed_multiplier *= 0.7;
        }

        let current_speed = player.speed * speed_multiplier;
        let has_input = direction.length_squared() > 0.001;

        if is_grounded {
            let target_vel = direction.normalize_or_zero() * current_speed;
            let accel_rate = 18.0;
            let current_vel = Vec3::new(player.velocity_xz.x, 0.0, player.velocity_xz.y);
            let new_vel = current_vel.move_towards(target_vel, accel_rate * current_speed * dt);
            player.velocity_xz = Vec2::new(new_vel.x, new_vel.z);
        } else if has_input {
            let target_vel = direction.normalize_or_zero() * current_speed;
            let accel_rate = 6.0;
            let current_vel = Vec3::new(player.velocity_xz.x, 0.0, player.velocity_xz.y);
            let new_vel = current_vel.move_towards(target_vel, accel_rate * current_speed * dt);
            player.velocity_xz = Vec2::new(new_vel.x, new_vel.z);
        } else {
            // Authentic mid-air aerodynamic damping: light air drag when no directional keys held
            player.velocity_xz *= (1.0 - 0.75 * dt).max(0.0);
        }

        let mut conveyor_drift = Vec3::ZERO;
        if let Some(sec) = current_sector {
            if sec.0 >= 0 {
                let sec_idx = sec.0 as usize;
                for effector in effectors.iter() {
                    if effector.sector_idx == sec_idx {
                        match &effector.kind {
                            crate::interactivity::EffectorKind::ConveyorBelt { direction, speed } => {
                                conveyor_drift += Vec3::new(direction.x, 0.0, direction.y) * (*speed * dt);
                            }
                            crate::interactivity::EffectorKind::UnderwaterTeleport { target_pos, .. } => {
                                if is_swimming && trans.translation.distance_squared(*target_pos) > 9.0 {
                                    trans.translation = *target_pos;
                                    sound_events.send(PlaySoundEvent { sound_id: 11 }); // TELEPORTER
                                }
                            }
                            _ => {}
                        }
                    }
                }
            }
        }

        let horizontal_movement =
            Vec3::new(player.velocity_xz.x, 0.0, player.velocity_xz.y) * dt + conveyor_drift;

        // Jetpack / Swimming / Gravity / Jumping
        let gravity = -18.0;
        let jump_speed = 7.0;

        if player.inventory.jetpack_active {
            // Jetpack vertical control
            if keys.pressed(KeyCode::Space) {
                player.velocity_y = 5.0;
            } else if keys.pressed(KeyCode::KeyC) {
                player.velocity_y = -5.0;
            } else {
                player.velocity_y = 0.0; // Hover in place
            }
        } else if is_swimming {
            // Swimming / Diving buoyancy controls
            if keys.pressed(KeyCode::Space) {
                player.velocity_y = 3.5;
            } else if keys.pressed(KeyCode::KeyC) {
                player.velocity_y = -3.5;
            } else {
                player.velocity_y = -0.3; // Gentle sinking buoyancy
            }
        } else if is_grounded {
            if player.velocity_y < -10.0 {
                let fall_speed = -player.velocity_y;
                let fall_damage = ((fall_speed - 9.0) * 8.0) as i32;
                if fall_damage > 0 {
                    let mut damage = fall_damage;
                    if player.inventory.boots_amount > 0 {
                        let absorbed = player.inventory.boots_amount.min(damage);
                        player.inventory.boots_amount -= absorbed;
                        damage -= absorbed;
                        sound_events.send(PlaySoundEvent { sound_id: 42 }); // DUKE_LAND
                    }
                    if damage > 0 && !player.god_mode {
                        player.health = (player.health - damage).max(0);
                        if player.health == 0 {
                            player.death_timer = 3.0;
                            sound_events.send(PlaySoundEvent { sound_id: 41 }); // DUKE_DEAD
                        } else {
                            sound_events.send(PlaySoundEvent { sound_id: 37 }); // DUKE_PAIN
                        }
                    }
                }
            } else if player.velocity_y < -4.0 {
                sound_events.send(PlaySoundEvent { sound_id: 42 }); // DUKE_LAND
            }
            if let Some(sec) = current_sector {
                if sec.0 >= 0 {
                    if let Some(ref sm) = sector_map {
                        let floor_y = sm.get_floor_y_at(sec.0 as usize, trans.translation.x, trans.translation.z);
                        let feet_y = trans.translation.y - if player.is_crouching() { 0.5 } else { 0.8 };
                        let height_diff = feet_y - floor_y;
                        if height_diff < 0.25 && height_diff > -0.25 && player.velocity_y < 0.0 {
                            player.velocity_y = -0.5; // Smooth sloped floor adherence
                        } else if player.velocity_y < 0.0 {
                            player.velocity_y = -0.2;
                        }
                    } else if player.velocity_y < 0.0 {
                        player.velocity_y = -0.2;
                    }
                } else if player.velocity_y < 0.0 {
                    player.velocity_y = -0.2;
                }
            } else if player.velocity_y < 0.0 {
                player.velocity_y = -0.2; // Gentle slope adherence
            }
            if keys.just_pressed(KeyCode::Space)
                && player.movement_mode != PlayerMovementMode::Crouching
            {
                player.velocity_y = jump_speed;
                sound_events.send(PlaySoundEvent { sound_id: 38 }); // DUKE_GRUNT
            }
        } else {
            player.velocity_y += gravity * dt;
            player.velocity_y = player.velocity_y.clamp(-14.0, jump_speed);
        }

        let vertical_movement = Vec3::Y * (player.velocity_y * dt);
        controller.translation = Some(horizontal_movement + vertical_movement);
    }
}
