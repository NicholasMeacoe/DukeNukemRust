#![allow(dead_code)]

pub mod types;
pub mod weapons;
pub mod inventory;
pub mod movement;

pub use types::*;
pub use weapons::*;
pub use inventory::*;
pub use movement::*;

use bevy::prelude::*;

pub struct PlayerPlugin;

impl Plugin for PlayerPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            Update,
            (
                update_player_movement,
                handle_weapon_selection,
                handle_weapon_firing,
                handle_inventory_input,
                update_inventory_timers,
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
}
