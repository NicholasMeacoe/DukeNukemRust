# Specification: Combat and Physics Integrity

## Overview
This track resolves critical combat simulation, projectile physics, and damage attribution flaws identified during the subsystem review. It ensures bullet hitscans travel across realistic room distances, guarantees lethal trap and environmental explosion kills are correctly attributed to the responsible combatant rather than penalizing victims with suicides, eliminates explosive double-damage stacking on direct rocket impacts, resolves the HandRemote pipebomb double-throw glitch, and implements authentic exotic PvP status effects for the Shrinker and Freezethrower.

---

## Functional Requirements

### 1. Hitscan Weapon Range Restoration
- In `src/combat/projectiles.rs`, increase the projectile lifetime of `ProjectileType::HitscanBullet` and `ProjectileType::ShotgunPellet` from `0.05s` to `1.5s`.
- At velocities of 150.0 m/s (Pistol, Chaingun) and 120.0 m/s (Shotgun), this extends effective reach to 225m and 180m, ensuring consistent bullet collision across large halls, outdoors, and deathmatch arenas.
- Ensure muzzle raycast ray-solid filter avoids self-capsule blocking when aiming downward.

### 2. Attacker Attribution for Traps & Environmental Explosives
- **Laser Tripbombs:**
  - Add `pub owner_player_id: Option<usize>` to `LaserTripbomb` in `src/combat/types.rs`.
  - In `src/player/weapons.rs`, record the placing player's ID upon tripbomb deployment.
  - In `src/player/weapons.rs::update_laser_tripbombs`, propagate `attacker_id: bomb.owner_player_id` into `ExplosionDamageEvent`.
- **Explosive Barrels & Extinguishers:**
  - Add `pub attacker_id: Option<usize>` to `WallDamageEvent` in `src/interactivity/types.rs`.
  - In `src/combat/projectiles.rs`, when a projectile damages a destructible wall/prop, forward `attacker_id: proj.source_player_id`.
  - In `src/interactivity/wall_damage.rs` and `src/interactivity/props.rs`, forward `attacker_id` to `ExplosionDamageEvent` upon detonation.
- **Suicide Correction:**
  - When an enemy player dies from an explosion initiated by an attacker, the attacker is awarded +1 frag, and the victim is credited with a death rather than an erroneous suicide (-1 frag).

### 3. Explosive Damage Anti-Double-Dipping
- Prevent direct rocket (`ProjectileType::Rocket`) and Devastator (`ProjectileType::DevastatorMissile`) hits from dealing both direct impact damage and full point-blank epicenter splash damage to the same victim entity.
- The direct hit deals direct projectile damage (120 for RPG, 40 for Devastator). Surrounding entities within the blast radius receive distance-attenuated splash damage, while the primary direct-hit victim does not take duplicate epicenter damage.

### 4. HandRemote Detonation Glitch Fix
- In `src/player/weapons.rs`, guard the auto-fire check after pipebomb detonation with `if is_firing && !is_detonating`.
- Prevents the player from inadvertently re-throwing a new pipebomb on the exact frame existing pipebombs are detonated.

### 5. Exotic PvP Weapon Effects (Shrinker & Freezethrower)
- **Shrinker:** In `src/net/pvp.rs` (`apply_pvp_damage`), hitting a player with a Shrinker blast (`weapon_type == 6`) applies `player.shrink_timer = 9.0`, reducing movement speed and enabling squash vulnerability.
- **Freezethrower:** In `src/net/pvp.rs` (`apply_pvp_damage`), lethal damage from a Freezethrower blast (`weapon_type == 8`) sets `player.freeze_timer = 4.6` and `player.health = 1`, immobilizing the combatant until thawed or shattered.

---

## Non-Functional Requirements
- **Performance:** Hitscan projectile updates and raycast sweeps must maintain $\ge 60$ FPS.
- **Code Standards:**
  - 0 compiler warnings on `cargo check --tests`.
  - 0 dead code annotations (`#![allow(dead_code)]`).
  - 100% test pass rate across all existing and new unit tests.
  - Windows rule: Always use `cmd /c` for terminal commands.

---

## Acceptance Criteria
1. Pistol and Shotgun shots register hits against targets at distances exceeding 15 meters without mid-air despawning.
2. Players killed by enemy laser tripbombs or shot barrels award +1 frag to the trap owner/shooter, with zero suicide penalty to the victim.
3. Direct RPG hits deal 120 damage, not 240 damage.
4. Firing `HandRemote` detonates pipebombs without immediately tossing a new pipebomb.
5. Shrinker blasts shrink opposing players in PvP, and lethal Freezethrower blasts freeze combatants.
6. Full test suite passes 100% with 0 compiler warnings.
