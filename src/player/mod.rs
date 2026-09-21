pub mod cheats;
pub mod console;
pub mod inventory;
pub mod movement;
pub mod types;
pub mod weapons;

pub use cheats::*;
pub use console::*;
pub use inventory::*;
pub use movement::*;
pub use types::*;
pub use weapons::*;

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
                    (
                        handle_weapon_firing,
                        update_laser_tripbombs,
                        update_player_pickups,
                        update_hazard_sectors,
                    )
                        .in_set(crate::GameSet::Combat),
                    (update_inventory_timers, update_first_person_viewmodel)
                        .in_set(crate::GameSet::Animation),
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
    fn test_viewmodel_new_constructor() {
        let vm = FirstPersonViewModel::new(WeaponType::Pistol);
        assert_eq!(vm.current_weapon, WeaponType::Pistol);
        assert_eq!(vm.base_tile, 2524);
        assert_eq!(vm.current_tile, 2524);
        assert!(!vm.is_firing);

        let vm_shotgun = FirstPersonViewModel::new(WeaponType::Shotgun);
        assert_eq!(vm_shotgun.current_weapon, WeaponType::Shotgun);
        assert_eq!(vm_shotgun.base_tile, 2613);
        assert_eq!(vm_shotgun.current_tile, 2613);
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
        let steroid_kick = if player.inventory.steroids_active {
            40
        } else {
            15
        };
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
        } else {
            // Authentic mid-air aerodynamic damping
            player.velocity_xz *= (1.0 - 0.75 * dt).max(0.0);
        }

        // In mid-air with no input, horizontal velocity decays gently with aerodynamic drag
        assert!(player.velocity_xz.x < 8.0 && player.velocity_xz.x > 7.8);
        assert_eq!(player.velocity_xz.y, 0.0);
    }

    #[test]
    fn test_swimming_and_diving_submerged_movement() {
        let mut player = PlayerController::default();
        player.movement_mode = PlayerMovementMode::Swimming;

        let _dt = 0.016;
        let mut speed_multiplier = 1.0;
        let is_swimming = matches!(
            player.movement_mode,
            PlayerMovementMode::Swimming | PlayerMovementMode::Diving
        );
        if is_swimming {
            speed_multiplier *= 0.7; // Water fluid drag
        }

        let current_speed = player.speed * speed_multiplier;
        assert_eq!(current_speed, 10.0 * 0.7);

        // Buoyancy downward sink
        player.velocity_y = -0.3;
        assert_eq!(player.velocity_y, -0.3);

        // Active upward swimming
        player.velocity_y = 3.5;
        assert_eq!(player.velocity_y, 3.5);
    }

    #[test]
    fn test_player_armor_absorption_calculation() {
        let mut player = PlayerController::default();
        player.health = 100;
        player.armor = 50;

        let mut damage = 40;
        let absorbed = (damage * 3 / 4).min(player.armor); // 30 absorbed
        player.armor -= absorbed;
        damage -= absorbed; // 10 remaining
        player.health -= damage;

        assert_eq!(player.armor, 20);
        assert_eq!(player.health, 90);
    }

    #[test]
    fn test_player_death_timer_and_respawn() {
        let mut player = PlayerController::default();
        player.spawn_position = Vec3::new(10.0, 1.0, 20.0);
        player.health = 0;
        player.death_timer = 3.0;

        let dt = 1.0;
        player.death_timer -= dt;
        assert_eq!(player.death_timer, 2.0);

        player.death_timer -= 2.0;
        if player.death_timer <= 0.0 {
            player.health = player.max_health;
            player.death_timer = 0.0;
        }

        assert_eq!(player.health, 100);
        assert_eq!(player.death_timer, 0.0);
    }

    #[test]
    fn test_keycard_types_and_status() {
        let mut player = PlayerController::default();
        assert!(!player.has_blue_key);
        assert!(!player.has_red_key);
        assert!(!player.has_yellow_key);

        player.has_blue_key = true;
        player.has_red_key = true;
        player.has_yellow_key = true;

        assert!(player.has_blue_key && player.has_red_key && player.has_yellow_key);
    }

    #[test]
    fn test_crouch_collider_and_translation_offset() {
        // Standing: center is at y = 1.0, half_height = 0.5, radius = 0.3 -> feet at 1.0 - 0.8 = 0.2
        let initial_y = 1.0f32;
        let standing_feet_y = initial_y - (0.5 + 0.3);

        // Transition to crouching:
        // Center drops by 0.3 -> crouch_y = 0.7
        // Half-height becomes 0.2, radius = 0.3
        // Feet at 0.7 - (0.2 + 0.3) = 0.2
        let crouch_y = initial_y - 0.3;
        let crouching_feet_y = crouch_y - (0.2 + 0.3);
        assert!((standing_feet_y - crouching_feet_y).abs() < 1e-6);

        // Transition back to standing:
        // Center rises by 0.3 -> stand_y = 1.0
        let stand_y = crouch_y + 0.3;
        let standing_again_feet_y = stand_y - (0.5 + 0.3);
        assert!((standing_feet_y - standing_again_feet_y).abs() < 1e-6);
    }

    #[test]
    fn test_crouch_headroom_math_behavior() {
        // Floor at 0.0. A standing player is 1.6m tall (feet at 0.0, top of head at 1.6m).
        // If the ceiling is at 2.0m, a standing player has 0.4m of clearance above their head.
        let floor_y = 0.0f32;
        let ceiling_y = 2.0f32;

        // When crouching, center is at y = 0.5 (feet at 0.0, half-height 0.2 + radius 0.3).
        let crouch_center_y = floor_y + 0.5;

        // The headroom calculated from center:
        let headroom_from_center = ceiling_y - crouch_center_y; // 1.5m
        // Notice: 1.5m < 1.6m, so `headroom < 1.6` is TRUE even though ceiling (2.0m) is taller than Duke (1.6m)!
        assert!(headroom_from_center < 1.6);

        // The correct headroom check should calculate clearance from feet to ceiling:
        let feet_y = crouch_center_y - 0.5;
        let total_clearance_from_feet = ceiling_y - feet_y; // 2.0m
        assert!(total_clearance_from_feet >= 1.6); // Can safely stand!
    }

    #[test]
    fn test_sloped_floor_adherence_feet_vs_center() {
        // Floor at 0.0.
        let floor_y = 0.0f32;

        // Standing player center is at y = 0.8:
        let standing_center_y = 0.8f32;
        let height_diff_center = standing_center_y - floor_y; // 0.8

        // If checking height_diff_center < 0.25 && height_diff_center > -0.25:
        // 0.8 is NOT in (-0.25..0.25), so center comparison fails to detect that feet are on floor!
        assert!(height_diff_center > 0.25);

        // Correct comparison checks feet:
        let feet_y = standing_center_y - 0.8;
        let height_diff_feet = feet_y - floor_y;
        assert!(height_diff_feet.abs() < 0.25); // Feet are within tolerance!
    }

    #[test]
    fn test_scuba_truncation_with_subsecond_dt() {
        // At 60 FPS, dt = 0.01666...
        let dt = 0.016f32;
        // In movement.rs line 163: `(2.0 * dt) as i32`
        let direct_cast_drain = (2.0 * dt) as i32;
        // Direct integer cast truncates to 0:
        assert_eq!(direct_cast_drain, 0);

        // Whereas using an accumulator (as in inventory.rs) preserves drain:
        let mut accum = 0.0f32;
        let mut drained = 0;
        for _ in 0..60 {
            accum += 2.0 * dt;
            if accum >= 1.0 {
                drained += 1;
                accum -= 1.0;
            }
        }
        // At 60 frames (~1 second), approximately 2 units are drained:
        assert!(drained >= 1);
    }

    #[test]
    fn test_protective_boots_fall_damage_overflow() {
        let mut player = PlayerController::default();
        player.health = 100;
        player.inventory.boots_amount = 20;

        let fall_damage = 50;
        let mut damage = fall_damage;
        if player.inventory.boots_amount > 0 {
            let absorbed = player.inventory.boots_amount.min(damage);
            player.inventory.boots_amount -= absorbed;
            damage -= absorbed;
        }
        if damage > 0 && !player.god_mode {
            player.health = player.health.saturating_sub(damage);
        }

        // 20 boots durability absorbed, remaining 30 applied to health
        assert_eq!(player.inventory.boots_amount, 0);
        assert_eq!(player.health, 70);

        // Case where boots completely absorb fall damage
        let mut player2 = PlayerController::default();
        player2.health = 100;
        player2.inventory.boots_amount = 100;
        let fall_damage2 = 30;
        let mut damage2 = fall_damage2;
        if player2.inventory.boots_amount > 0 {
            let absorbed = player2.inventory.boots_amount.min(damage2);
            player2.inventory.boots_amount -= absorbed;
            damage2 -= absorbed;
        }
        if damage2 > 0 && !player2.god_mode {
            player2.health = player2.health.saturating_sub(damage2);
        }
        assert_eq!(player2.inventory.boots_amount, 70);
        assert_eq!(player2.health, 100);
    }

    #[test]
    fn test_crouch_headroom_math_standing_allowed_and_prevented() {
        let mut player = PlayerController::default();
        player.movement_mode = PlayerMovementMode::Crouching;
        assert!(player.is_crouching());

        // Capsule center at y = 0.5 (feet at 0.0)
        let crouch_center_y = 0.5f32;
        let feet_y = crouch_center_y - 0.5;

        // Low ceiling at 1.4m: standing Duke requires 1.6m clearance
        let ceiling_low = 1.4f32;
        let headroom_low = ceiling_low - feet_y;
        let force_crouch_low = player.is_crouching() && headroom_low < 1.6;
        assert!(force_crouch_low); // Must remain crouching

        // Tall ceiling at 2.0m: standing Duke has plenty of room
        let ceiling_tall = 2.0f32;
        let headroom_tall = ceiling_tall - feet_y;
        let force_crouch_tall = player.is_crouching() && headroom_tall < 1.6;
        assert!(!force_crouch_tall); // Can safely stand up
    }

    #[test]
    fn test_slope_adherence_crouching_and_standing() {
        let mut player = PlayerController::default();
        let floor_y = 0.0f32;

        // Standing player: center at y = 0.8
        player.movement_mode = PlayerMovementMode::Standing;
        player.velocity_y = -1.0;
        let standing_center_y = 0.8f32;
        let standing_feet_y = standing_center_y - if player.is_crouching() { 0.5 } else { 0.8 };
        let height_diff = standing_feet_y - floor_y;
        assert!(height_diff < 0.25 && height_diff > -0.25);
        if height_diff < 0.25 && height_diff > -0.25 && player.velocity_y < 0.0 {
            player.velocity_y = -0.5;
        }
        assert_eq!(player.velocity_y, -0.5);

        // Crouching player: center at y = 0.5
        player.movement_mode = PlayerMovementMode::Crouching;
        player.velocity_y = -1.0;
        let crouching_center_y = 0.5f32;
        let crouching_feet_y = crouching_center_y - if player.is_crouching() { 0.5 } else { 0.8 };
        let height_diff_crouch = crouching_feet_y - floor_y;
        assert!(height_diff_crouch < 0.25 && height_diff_crouch > -0.25);
        if height_diff_crouch < 0.25 && height_diff_crouch > -0.25 && player.velocity_y < 0.0 {
            player.velocity_y = -0.5;
        }
        assert_eq!(player.velocity_y, -0.5);
    }

    #[test]
    fn test_inventory_drowning_damage_with_god_mode_and_fatality() {
        let mut player = PlayerController::default();
        player.health = 5;
        player.inventory.air_supply = 0.0;
        player.inventory.drowning_damage_timer = 1.0;

        // Fatal drowning tick
        if player.inventory.drowning_damage_timer >= 1.0 {
            player.inventory.drowning_damage_timer = 0.0;
            if !player.god_mode {
                player.health = (player.health - 10).max(0);
                if player.health == 0 {
                    player.death_timer = 3.0;
                }
            }
        }
        assert_eq!(player.health, 0);
        assert_eq!(player.death_timer, 3.0);

        // God mode protection against drowning
        let mut god_player = PlayerController::default();
        god_player.health = 100;
        god_player.god_mode = true;
        god_player.inventory.air_supply = 0.0;
        god_player.inventory.drowning_damage_timer = 1.0;

        if god_player.inventory.drowning_damage_timer >= 1.0 {
            god_player.inventory.drowning_damage_timer = 0.0;
            if !god_player.god_mode {
                god_player.health = (god_player.health - 10).max(0);
            }
        }
        assert_eq!(god_player.health, 100);
    }

    #[test]
    fn test_hazard_sectors_damage_and_boot_protection() {
        let mut app = App::new();
        app.add_event::<crate::audio::PlaySoundEvent>()
            .init_resource::<Time>()
            .init_resource::<crate::hud::ScreenTintState>()
            .add_systems(Update, movement::update_hazard_sectors);

        // 1. Player without boots in hazard sector takes damage
        let mut player = PlayerController::default();
        player.health = 100;
        player.inventory.boots_amount = 0;

        let player_entity = app.world_mut().spawn((
            player,
            TransformBundle::from_transform(Transform::from_xyz(0.0, 0.0, 0.0)),
        )).id();

        // Spawn hazard sector (slime or fire)
        app.world_mut().spawn((
            HazardSector { damage_per_sec: 20.0 },
            TransformBundle::from_transform(Transform::from_xyz(0.0, 0.0, 0.5)),
        ));

        // Advance time by 0.5s
        {
            let mut time = app.world_mut().resource_mut::<Time>();
            time.advance_by(std::time::Duration::from_millis(500));
        }
        app.update();

        let updated_player = app.world().get::<PlayerController>(player_entity).unwrap();
        assert!(
            updated_player.health < 100,
            "Player without boots standing on hazard sector must take damage"
        );

        // 2. Player WITH boots has hazard damage absorbed by boots
        {
            let mut p = app.world_mut().get_mut::<PlayerController>(player_entity).unwrap();
            p.health = 100;
            p.inventory.boots_amount = 100;
            let mut time = app.world_mut().resource_mut::<Time>();
            time.advance_by(std::time::Duration::from_millis(500));
        }
        app.update();

        let protected_player = app.world().get::<PlayerController>(player_entity).unwrap();
        assert_eq!(
            protected_player.health, 100,
            "Boots must protect player from hazard sector damage"
        );
        assert_eq!(protected_player.inventory.boots_amount, 99);
    }
}

