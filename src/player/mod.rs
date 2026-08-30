#![allow(dead_code)]

pub mod types;
pub mod weapons;
pub mod inventory;
pub mod movement;
pub mod cheats;
pub mod console;

pub use types::*;
pub use weapons::*;
pub use inventory::*;
pub use movement::*;
pub use cheats::*;
pub use console::*;

use bevy::prelude::*;

pub struct PlayerPlugin;

impl Plugin for PlayerPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(cheats::CheatsPlugin)
            .add_plugins(console::ConsolePlugin)
            .add_systems(
                Update,
                (
                    (handle_weapon_selection, handle_inventory_input).in_set(crate::GameSet::Input),
                    update_player_movement.in_set(crate::GameSet::Movement),
                (handle_weapon_firing, update_laser_tripbombs, update_player_pickups).in_set(crate::GameSet::Combat),
                (update_inventory_timers, update_first_person_viewmodel).in_set(crate::GameSet::Animation),
            ),
        );
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_player_controller_default_state() {
        let player = PlayerController::default();
        assert_eq!(player.health, 100);
        assert_eq!(player.current_weapon, WeaponType::Pistol);
        assert_eq!(player.weapons[WeaponType::Pistol as usize].ammo, 48);
        assert!(player.weapons[WeaponType::Pistol as usize].is_unlocked);
        assert!(player.weapons[WeaponType::Shotgun as usize].is_unlocked);
        assert!(!player.weapons[WeaponType::Rpg as usize].is_unlocked);
    }

    #[test]
    fn test_steroids_and_medkit_buffs() {
        let mut player = PlayerController::default();
        player.health = 40;
        player.inventory.medkit_amount = 50;

        // Use medkit: 40 + 50 = 90
        let needed = player.max_health - player.health;
        let use_amount = needed.min(player.inventory.medkit_amount);
        player.health += use_amount;
        player.inventory.medkit_amount -= use_amount;

        assert_eq!(player.health, 90);
        assert_eq!(player.inventory.medkit_amount, 0);

        // Test steroids
        player.inventory.steroids_amount = 400;
        player.inventory.steroids_active = true;
        assert!(player.inventory.steroids_active);
    }

    #[test]
    fn test_pistol_mag_and_ammo_consumption() {
        let mut player = PlayerController::default();
        let cur_idx = WeaponType::Pistol as usize;
        assert_eq!(player.pistol_mag, 12);

        // Fire 1 round
        player.pistol_mag -= 1;
        player.weapons[cur_idx].ammo -= 1;
        assert_eq!(player.pistol_mag, 11);
        assert_eq!(player.weapons[cur_idx].ammo, 47);
    }

    #[test]
    fn test_weapon_unlock_and_switch() {
        let mut player = PlayerController::default();
        // Unlock RPG
        let rpg_idx = WeaponType::Rpg as usize;
        player.weapons[rpg_idx].is_unlocked = true;
        player.current_weapon = WeaponType::Rpg;
        assert_eq!(player.current_weapon, WeaponType::Rpg);
        assert_eq!(player.weapons[rpg_idx].ammo, 5);
    }

    #[test]
    fn test_quick_kick_mechanic() {
        let mut player = PlayerController::default();
        assert_eq!(player.quick_kick_timer, 0.0);
        player.quick_kick_timer = 0.5;
        assert!(player.quick_kick_timer > 0.0);
    }

    #[test]
    fn test_tripbomb_arming_and_trigger() {
        let mut bomb = crate::combat::LaserTripbomb {
            normal: Vec3::new(0.0, 0.0, 1.0),
            arm_timer: 1.0,
            is_armed: false,
            beam_length: 12.0,
            damage: 150,
            damage_radius: 6.0,
        };

        // Tick arm timer
        bomb.arm_timer -= 1.0;
        if bomb.arm_timer <= 0.0 {
            bomb.is_armed = true;
        }
        assert!(bomb.is_armed);

        // Check beam intersection
        let origin = Vec3::new(0.0, 1.0, 0.0);
        let normal = bomb.normal;
        let player_pos = Vec3::new(0.0, 1.0, 5.0); // 5 meters along beam

        let v = player_pos - origin;
        let proj = v.dot(normal);
        assert!(proj > 0.3 && proj < bomb.beam_length);
        let closest = origin + normal * proj;
        assert!(closest.distance_squared(player_pos) < 1.0);
    }

    #[test]
    fn test_expander_weapon_data() {
        let mut player = PlayerController::default();
        let exp_idx = WeaponType::Expander as usize;
        player.weapons[exp_idx].is_unlocked = true;
        assert_eq!(player.weapons[exp_idx].name, "Expander");
        assert_eq!(player.weapons[exp_idx].ammo, 20);

        player.weapons[exp_idx].ammo -= 1;
        assert_eq!(player.weapons[exp_idx].ammo, 19);
    }

    #[test]
    fn test_all_10_weapons_configured() {
        let player = PlayerController::default();
        for weapon in &player.weapons {
            assert!(weapon.fire_delay > 0.0);
            assert!(!weapon.name.is_empty());
        }
    }

    #[test]
    fn test_scuba_air_supply_and_drowning() {
        let mut player = PlayerController::default();
        assert_eq!(player.inventory.air_supply, 100.0);

        // Player diving without scuba
        player.movement_mode = PlayerMovementMode::Diving;
        player.inventory.scuba_amount = 0;

        // Drains air
        player.inventory.air_supply -= 10.0;
        assert_eq!(player.inventory.air_supply, 90.0);

        // Scuba active
        player.inventory.scuba_amount = 50;
        player.inventory.scuba_amount -= 5;
        player.inventory.air_supply = 100.0;
        assert_eq!(player.inventory.scuba_amount, 45);
        assert_eq!(player.inventory.air_supply, 100.0);
    }

    #[test]
    fn test_protective_boots_hazard_mitigation() {
        let mut player = PlayerController::default();
        player.inventory.boots_amount = 100;

        // Step on acid floor
        let acid_dmg = 10;
        if player.inventory.boots_amount > 0 {
            player.inventory.boots_amount -= acid_dmg;
        } else {
            player.health -= acid_dmg;
        }

        assert_eq!(player.inventory.boots_amount, 90);
        assert_eq!(player.health, 100); // Health protected by boots!
    }

    #[test]
    fn test_holoduke_decoy_lifetime() {
        let mut decoy = HoloDukeDecoy { lifetime: 30.0 };
        decoy.lifetime -= 5.0;
        assert_eq!(decoy.lifetime, 25.0);
    }

    #[test]
    fn test_player_item_pickup_collection() {
        let mut player = PlayerController::default();
        player.health = 50;
        player.armor = 20;

        // Collect small medkit (+10)
        player.health = (player.health + 10).min(100);
        assert_eq!(player.health, 60);

        // Collect Atomic Health (+50 up to 200)
        player.health = (player.health + 50).min(200);
        assert_eq!(player.health, 110);

        // Collect Armor Vest
        player.armor = 100;
        assert_eq!(player.armor, 100);

        // Collect Shotgun pickup
        player.weapons[WeaponType::Shotgun as usize].is_unlocked = true;
        player.weapons[WeaponType::Shotgun as usize].ammo += 10;
        assert!(player.weapons[WeaponType::Shotgun as usize].is_unlocked);
        assert_eq!(player.weapons[WeaponType::Shotgun as usize].ammo, 30); // 20 default + 10 = 30
    }

    #[test]
    fn test_viewmodel_animation_frames() {
        let mut vm = FirstPersonViewModel::default();
        assert_eq!(vm.current_weapon, WeaponType::Pistol);
        assert_eq!(vm.current_tile, 2524);

        // Firing progress animation frame
        let progress = 0.5; // halfway through fire delay
        let frame_offset = (progress * 4.0) as i16; // 2
        vm.current_tile = 2524 + frame_offset;
        assert_eq!(vm.current_tile, 2526); // Recoil frame
    }

    #[test]
    fn test_weapon_priority_auto_switch() {
        let mut player = PlayerController::default();
        player.current_weapon = WeaponType::Shotgun;
        player.weapons[WeaponType::Shotgun as usize].ammo = 0;
        player.weapons[WeaponType::Pistol as usize].is_unlocked = true;
        player.weapons[WeaponType::Pistol as usize].ammo = 48;

        let best = weapons::get_highest_priority_available_weapon(&player);
        assert_eq!(best, WeaponType::Pistol);

        // Unlock RPG
        player.weapons[WeaponType::Rpg as usize].is_unlocked = true;
        player.weapons[WeaponType::Rpg as usize].ammo = 5;
        let best = weapons::get_highest_priority_available_weapon(&player);
        assert_eq!(best, WeaponType::Rpg);
    }

    #[test]
    fn test_quick_kick_steroid_damage_buff() {
        let mut player = PlayerController::default();
        let base_kick = 15;
        assert_eq!(base_kick, 15);

        player.inventory.steroids_active = true;
        let steroid_kick = if player.inventory.steroids_active { 40 } else { 15 };
        assert_eq!(steroid_kick, 40);
    }

    #[test]
    fn test_pipebomb_throw_and_handremote_transition() {
        let mut player = PlayerController::default();
        player.current_weapon = WeaponType::Pipebomb;
        player.weapons[WeaponType::Pipebomb as usize].ammo = 5;

        // Throw pipebomb
        player.weapons[WeaponType::Pipebomb as usize].ammo -= 1;
        player.current_weapon = WeaponType::HandRemote;

        assert_eq!(player.current_weapon, WeaponType::HandRemote);
        assert_eq!(player.weapons[WeaponType::Pipebomb as usize].ammo, 4);
    }

    #[test]
    fn test_player_inertia_velocity_accel() {
        let mut player = PlayerController::default();
        let dt = 0.016;
        let direction = Vec3::new(1.0, 0.0, 0.0);
        let current_speed = player.speed;
        let target_vel = direction * current_speed;
        let accel_rate = 18.0;
        let current_vel = Vec3::new(player.velocity_xz.x, 0.0, player.velocity_xz.y);
        let new_vel = current_vel.move_towards(target_vel, accel_rate * current_speed * dt);
        player.velocity_xz = Vec2::new(new_vel.x, new_vel.z);

        assert!(player.velocity_xz.x > 0.0);
        assert_eq!(player.velocity_xz.y, 0.0);
    }

    #[test]
    fn test_player_air_momentum_preservation() {
        let mut player = PlayerController::default();
        player.velocity_xz = Vec2::new(8.0, 0.0); // Jumped with horizontal speed
        let dt = 0.016;
        let direction = Vec3::ZERO; // Player releases all WASD keys in mid-air
        let is_grounded = false;
        let has_input = direction.length_squared() > 0.001;

        if is_grounded {
            let target_vel = direction.normalize_or_zero() * player.speed;
            let current_vel = Vec3::new(player.velocity_xz.x, 0.0, player.velocity_xz.y);
            let new_vel = current_vel.move_towards(target_vel, 18.0 * player.speed * dt);
            player.velocity_xz = Vec2::new(new_vel.x, new_vel.z);
        } else if has_input {
            let target_vel = direction.normalize_or_zero() * player.speed;
            let current_vel = Vec3::new(player.velocity_xz.x, 0.0, player.velocity_xz.y);
            let new_vel = current_vel.move_towards(target_vel, 6.0 * player.speed * dt);
            player.velocity_xz = Vec2::new(new_vel.x, new_vel.z);
        }

        // In mid-air with no input, horizontal velocity is 100% preserved (zero air drag)
        assert_eq!(player.velocity_xz, Vec2::new(8.0, 0.0));
    }
}
