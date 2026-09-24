# Implementation Plan: Combat and Physics Integrity

## Phase 1: Hitscan Weapon Range & Muzzle Raycast Refinement
- [x] Task: Write Tests for Hitscan Bullet Travel & Max Effective Range
  - [x] Unit test: Hitscan bullet travels $\ge 50$ meters over multiple frames without premature despawn
  - [x] Unit test: Muzzle raycast aiming downward does not get blocked by player capsule collider
- [x] Task: Implement Hitscan Lifetime & Muzzle Offset Adjustments in `src/combat/projectiles.rs`
  - [x] Update `HitscanBullet` and `ShotgunPellet` lifetimes to 1.5s
  - [x] Refine muzzle raycast origin and solid-test filter
- [x] Task: Phase 1 Verification & Checkpoint (Refer to workflow.md)
  - [x] Verify hitscan projectiles travel full distances and test pass rate

## Phase 2: Attacker Attribution for Tripbombs, Barrels & Props
- [x] Task: Write Tests for Trap & Prop Explosion Attribution
  - [x] Unit test: `LaserTripbomb` preserves `owner_player_id` and forwards to `ExplosionDamageEvent`
  - [x] Unit test: Shooting an explosive barrel propagates shooter's `attacker_id` and awards frag to shooter
  - [x] Unit test: Victim of enemy trap/barrel death is credited with a death, not a suicide (-1 frag)
- [x] Task: Implement Attacker Propagation in `src/combat/`, `src/interactivity/`, and `src/player/`
  - [x] Add `owner_player_id: Option<usize>` to `LaserTripbomb` in `src/combat/types.rs` and wire during placement in `src/player/weapons.rs`
  - [x] Add `attacker_id: Option<usize>` to `WallDamageEvent` in `src/interactivity/types.rs`
  - [x] Forward attacker from projectile impacts to barrels/extinguishers in `src/interactivity/wall_damage.rs` and `src/interactivity/props.rs`
- [x] Task: Phase 2 Verification & Checkpoint (Refer to workflow.md)
  - [x] Verify trap and barrel explosion kills attribute correctly to the attacker

## Phase 3: Explosive Double-Damage Fix & HandRemote Glitch Resolution
- [x] Task: Write Tests for Rocket Splash Anti-Double-Dipping & HandRemote Detonation
  - [x] Unit test: Direct RPG rocket hit applies 120 direct damage without applying duplicate epicenter splash to the primary target
  - [x] Unit test: Firing `HandRemote` detonates active pipebombs without immediately throwing another pipebomb
- [x] Task: Implement Anti-Double-Dipping and Detonation Fire Guard in `src/combat/projectiles.rs` and `src/player/weapons.rs`
  - [x] Guard `handle_weapon_firing` auto-fire check with `!is_detonating`
  - [x] Exclude direct-hit victim from taking stacked epicenter explosion splash damage
- [x] Task: Phase 3 Verification & Checkpoint (Refer to workflow.md)
  - [x] Verify rocket damage values and clean single-trigger pipebomb detonation

## Phase 4: Exotic PvP Weapon Effects (Shrinker & Freezethrower)
- [x] Task: Write Tests for Shrinker & Freezethrower Effects on Players
  - [x] Unit test: Player hit by Shrinker enters `shrink_timer = 9.0`
  - [x] Unit test: Lethal damage from Freezethrower triggers `freeze_timer = 4.6` and `health = 1`
- [x] Task: Implement Exotic PvP Status Effects in `src/net/pvp.rs`
  - [x] Wire weapon types 6 (Shrinker) and 8 (Freezethrower) in `apply_pvp_damage`
- [x] Task: Phase 4 Verification & Checkpoint (Refer to workflow.md)
  - [x] Verify player shrinkage and freezing behavior under PvP conditions

## Phase 5: Full Verification, Warnings Audit & Track Completion
- [ ] Task: Comprehensive Test Suite & Warning Audit
  - [ ] Run `cargo check --tests` (ensure 0 compiler warnings)
  - [ ] Run full test suite across all tests (ensure 100% pass rate)
  - [ ] Review completed track with `conductor-review`
- [ ] Task: Phase 5 Verification & Checkpoint (Refer to workflow.md)
  - [ ] Final end-to-end verification and commit
