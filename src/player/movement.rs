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
    )>,
    camera_query: Query<&Transform, (With<Camera>, Without<PlayerController>)>,
    mut sound_events: EventWriter<PlaySoundEvent>,
    mut tint: Option<ResMut<crate::hud::ScreenTintState>>,
) {
    let Ok(camera_transform) = camera_query.get_single() else {
        return;
    };
    let dt = time.delta_seconds();

    for (mut trans, mut player, mut controller, output) in query.iter_mut() {
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

        // Crouch toggle / hold (Key 'C')
        if keys.pressed(KeyCode::KeyC) {
            player.movement_mode = PlayerMovementMode::Crouching;
        } else if player.inventory.jetpack_active {
            player.movement_mode = PlayerMovementMode::JetpackFlying;
        } else {
            player.movement_mode = PlayerMovementMode::Standing;
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
            speed_multiplier *= 0.5;
        }

        let is_swimming = matches!(
            player.movement_mode,
            PlayerMovementMode::Swimming | PlayerMovementMode::Diving
        );
        if is_swimming {
            speed_multiplier *= 0.7;
            if let Some(ref mut t) = tint {
                t.target_color = Color::srgba(0.0, 0.2, 0.7, 0.4);
            }
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

        let horizontal_movement = Vec3::new(player.velocity_xz.x, 0.0, player.velocity_xz.y) * dt;

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
            if player.velocity_y < -4.0 {
                sound_events.send(PlaySoundEvent { sound_id: 42 }); // DUKE_LAND
            }
            if player.velocity_y < 0.0 {
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
