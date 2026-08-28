#![allow(dead_code)]

use bevy::prelude::*;
use crate::player::types::*;

pub fn handle_inventory_input(
    keys: Res<ButtonInput<KeyCode>>,
    mut query: Query<&mut PlayerController>,
) {
    let Ok(mut player) = query.get_single_mut() else { return; };

    // 1. Steroids (Key 'U')
    if keys.just_pressed(KeyCode::KeyU) && player.inventory.steroids_amount > 0 {
        player.inventory.steroids_active = true;
        // Instant unshrink if shrunk!
        player.shrink_timer = 0.0;
    }

    // 2. Medkit (Key 'M')
    if keys.just_pressed(KeyCode::KeyM) && player.inventory.medkit_amount > 0 && player.health < player.max_health {
        let needed = player.max_health - player.health;
        let use_amount = needed.min(player.inventory.medkit_amount);
        player.health += use_amount;
        player.inventory.medkit_amount -= use_amount;
    }

    // 3. Nightvision (Key 'N')
    if keys.just_pressed(KeyCode::KeyN) && player.inventory.nightvision_amount > 0 {
        player.inventory.nightvision_active = !player.inventory.nightvision_active;
    }

    // 4. Jetpack (Key 'J')
    if keys.just_pressed(KeyCode::KeyJ) && player.inventory.jetpack_amount > 0 {
        player.inventory.jetpack_active = !player.inventory.jetpack_active;
    }

    // 5. Holoduke (Key 'H')
    if keys.just_pressed(KeyCode::KeyH) && player.inventory.holoduke_amount > 0 {
        player.inventory.holoduke_active = !player.inventory.holoduke_active;
    }
}

pub fn update_inventory_timers(
    time: Res<Time>,
    mut query: Query<&mut PlayerController>,
) {
    let dt = time.delta_seconds();
    let Ok(mut player) = query.get_single_mut() else { return; };

    // Steroids countdown
    if player.inventory.steroids_active {
        player.inventory.steroids_amount -= (dt * 20.0) as i32;
        if player.inventory.steroids_amount <= 0 {
            player.inventory.steroids_amount = 0;
            player.inventory.steroids_active = false;
        }
    }

    // Nightvision countdown
    if player.inventory.nightvision_active {
        player.inventory.nightvision_amount -= (dt * 10.0) as i32;
        if player.inventory.nightvision_amount <= 0 {
            player.inventory.nightvision_amount = 0;
            player.inventory.nightvision_active = false;
        }
    }

    // Jetpack countdown
    if player.inventory.jetpack_active {
        player.inventory.jetpack_amount -= (dt * 15.0) as i32;
        if player.inventory.jetpack_amount <= 0 {
            player.inventory.jetpack_amount = 0;
            player.inventory.jetpack_active = false;
        }
    }

    // Holoduke countdown
    if player.inventory.holoduke_active {
        player.inventory.holoduke_amount -= (dt * 10.0) as i32;
        if player.inventory.holoduke_amount <= 0 {
            player.inventory.holoduke_amount = 0;
            player.inventory.holoduke_active = false;
        }
    }

    // Status effect countdowns (Shrink & Freeze)
    if player.shrink_timer > 0.0 {
        player.shrink_timer -= dt;
    }
    if player.freeze_timer > 0.0 {
        player.freeze_timer -= dt;
    }
}
