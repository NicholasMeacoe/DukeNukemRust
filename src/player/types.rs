#![allow(dead_code)]

use bevy::prelude::*;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum WeaponType {
    Knee = 0,
    Pistol = 1,
    Shotgun = 2,
    Chaingun = 3,
    Rpg = 4,
    Pipebomb = 5,
    Shrinker = 6,
    Devastator = 7,
    Tripbomb = 8,
    Freezethrower = 9,
    HandRemote = 10,
    Expander = 11,
}

#[derive(Debug, Clone)]
pub struct WeaponData {
    pub weapon_type: WeaponType,
    pub name: &'static str,
    pub ammo: i32,
    pub max_ammo: i32,
    pub base_tile: i16,
    pub fire_delay: f32,
    pub fire_timer: f32,
    pub reload_timer: f32,
    pub is_unlocked: bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum InventoryItemType {
    Steroids,
    Medkit,
    Nightvision,
    ScubaGear,
    Boots,
    Holoduke,
    Jetpack,
}

#[derive(Debug, Clone)]
pub struct InventoryState {
    pub steroids_amount: i32,      // 0..400
    pub steroids_active: bool,
    pub medkit_amount: i32,        // 0..100
    pub nightvision_amount: i32,   // 0..100
    pub nightvision_active: bool,
    pub scuba_amount: i32,         // 0..100
    pub boots_amount: i32,         // 0..100
    pub holoduke_amount: i32,      // 0..100
    pub holoduke_active: bool,
    pub jetpack_amount: i32,       // 0..100
    pub jetpack_active: bool,
    pub air_supply: f32,           // 0..100.0 (suffocation when 0)
}

impl Default for InventoryState {
    fn default() -> Self {
        Self {
            steroids_amount: 0,
            steroids_active: false,
            medkit_amount: 0,
            nightvision_amount: 0,
            nightvision_active: false,
            scuba_amount: 0,
            boots_amount: 0,
            holoduke_amount: 0,
            holoduke_active: false,
            jetpack_amount: 0,
            jetpack_active: false,
            air_supply: 100.0,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PlayerMovementMode {
    Standing,
    Crouching,
    Swimming,
    Diving,
    JetpackFlying,
}

#[derive(Component, Debug, Clone)]
pub struct PlayerController {
    pub health: i32,
    pub max_health: i32,
    pub armor: i32,
    pub max_armor: i32,
    pub movement_mode: PlayerMovementMode,
    pub speed: f32,
    pub pitch: f32,
    pub yaw: f32,
    pub velocity_y: f32,
    pub current_weapon: WeaponType,
    pub weapons: [WeaponData; 12],
    pub inventory: InventoryState,
    pub shrink_timer: f32,
    pub freeze_timer: f32,
    pub quick_kick_timer: f32,
    pub pistol_mag: i32, // Current clip (0..12)
    pub devastator_alt_side: bool,
}

impl Default for PlayerController {
    fn default() -> Self {
        let weapons = [
            WeaponData { weapon_type: WeaponType::Knee, name: "Mighty Boot", ammo: 0, max_ammo: 0, base_tile: 2524, fire_delay: 0.4, fire_timer: 0.0, reload_timer: 0.0, is_unlocked: true },
            WeaponData { weapon_type: WeaponType::Pistol, name: "Pistol", ammo: 48, max_ammo: 200, base_tile: 2524, fire_delay: 0.3, fire_timer: 0.0, reload_timer: 0.0, is_unlocked: true },
            WeaponData { weapon_type: WeaponType::Shotgun, name: "Shotgun", ammo: 20, max_ammo: 50, base_tile: 2613, fire_delay: 0.8, fire_timer: 0.0, reload_timer: 0.0, is_unlocked: true },
            WeaponData { weapon_type: WeaponType::Chaingun, name: "Chaingun Cannon", ammo: 50, max_ammo: 200, base_tile: 2548, fire_delay: 0.1, fire_timer: 0.0, reload_timer: 0.0, is_unlocked: false },
            WeaponData { weapon_type: WeaponType::Rpg, name: "RPG", ammo: 5, max_ammo: 50, base_tile: 2562, fire_delay: 1.0, fire_timer: 0.0, reload_timer: 0.0, is_unlocked: false },
            WeaponData { weapon_type: WeaponType::Pipebomb, name: "Pipebomb", ammo: 5, max_ammo: 50, base_tile: 2570, fire_delay: 0.6, fire_timer: 0.0, reload_timer: 0.0, is_unlocked: false },
            WeaponData { weapon_type: WeaponType::Shrinker, name: "Shrinker", ammo: 10, max_ammo: 50, base_tile: 2580, fire_delay: 0.8, fire_timer: 0.0, reload_timer: 0.0, is_unlocked: false },
            WeaponData { weapon_type: WeaponType::Devastator, name: "Devastator", ammo: 20, max_ammo: 99, base_tile: 2590, fire_delay: 0.15, fire_timer: 0.0, reload_timer: 0.0, is_unlocked: false },
            WeaponData { weapon_type: WeaponType::Tripbomb, name: "Laser Tripbomb", ammo: 3, max_ammo: 10, base_tile: 2600, fire_delay: 0.8, fire_timer: 0.0, reload_timer: 0.0, is_unlocked: false },
            WeaponData { weapon_type: WeaponType::Freezethrower, name: "Freezethrower", ammo: 25, max_ammo: 99, base_tile: 2610, fire_delay: 0.2, fire_timer: 0.0, reload_timer: 0.0, is_unlocked: false },
            WeaponData { weapon_type: WeaponType::HandRemote, name: "Pipebomb Detonator", ammo: 0, max_ammo: 0, base_tile: 2575, fire_delay: 0.3, fire_timer: 0.0, reload_timer: 0.0, is_unlocked: true },
            WeaponData { weapon_type: WeaponType::Expander, name: "Expander", ammo: 20, max_ammo: 99, base_tile: 2585, fire_delay: 0.4, fire_timer: 0.0, reload_timer: 0.0, is_unlocked: false },
        ];

        Self {
            health: 100,
            max_health: 100,
            armor: 0,
            max_armor: 100,
            movement_mode: PlayerMovementMode::Standing,
            speed: 10.0,
            pitch: 0.0,
            yaw: 0.0,
            velocity_y: 0.0,
            current_weapon: WeaponType::Pistol,
            weapons,
            inventory: InventoryState::default(),
            shrink_timer: 0.0,
            freeze_timer: 0.0,
            quick_kick_timer: 0.0,
            pistol_mag: 12,
            devastator_alt_side: false,
        }
    }
}
