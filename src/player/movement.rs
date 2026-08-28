#![allow(dead_code)]

use bevy::prelude::*;
use bevy_rapier3d::prelude::*;
use crate::player::types::*;

pub fn update_player_movement(
    keys: Res<ButtonInput<KeyCode>>,
    time: Res<Time>,
    mut query: Query<(
        &mut PlayerController,
        &mut KinematicCharacterController,
        Option<&KinematicCharacterControllerOutput>,
    )>,
    camera_query: Query<&Transform, With<Camera>>,
) {
    let Ok(camera_transform) = camera_query.get_single() else { return; };
    let dt = time.delta_seconds();

    for (mut player, mut controller, output) in query.iter_mut() {
        if player.freeze_timer > 0.0 {
            // Cannot move while frozen
            controller.translation = Some(Vec3::new(0.0, -9.81 * dt, 0.0));
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

        let current_speed = player.speed * speed_multiplier;
        let horizontal_movement = direction.normalize_or_zero() * current_speed * dt;

        // Jetpack / Gravity / Jumping
        let gravity = -20.0;
        let jump_speed = 7.5;

        if player.inventory.jetpack_active {
            // Jetpack vertical control
            if keys.pressed(KeyCode::Space) {
                player.velocity_y = 6.0;
            } else if keys.pressed(KeyCode::KeyC) {
                player.velocity_y = -6.0;
            } else {
                player.velocity_y = 0.0; // Hover in place
            }
        } else if is_grounded {
            if player.velocity_y < 0.0 {
                player.velocity_y = -0.5; // Slope adherence glue
            }
            if keys.just_pressed(KeyCode::Space) && player.movement_mode != PlayerMovementMode::Crouching {
                player.velocity_y = jump_speed;
            }
        } else {
            player.velocity_y += gravity * dt;
            player.velocity_y = player.velocity_y.clamp(-25.0, jump_speed);
        }

        let vertical_movement = Vec3::Y * (player.velocity_y * dt);
        controller.translation = Some(horizontal_movement + vertical_movement);
    }
}
