# Implementation Plan: Combat and Physics Integrity

## Phase 1: Hitscan Weapon Range & Muzzle Raycast Refinement
- [ ] Task: Write Tests for Hitscan Bullet Travel & Max Effective Range
  - [ ] Unit test: Hitscan bullet travels $\ge 50$ meters over multiple frames without premature despawn
  - [ ] Unit test: Muzzle raycast aiming downward does not get blocked by player capsule collider
- [ ] Task: Implement Hitscan Lifetime & Muzzle Offset Adjustments in `src/combat/projectiles.rs`
  - [ ] Update `HitscanBullet` and `ShotgunPellet` lifetimes to 1.5s
  - [ ] Refine muzzle raycast origin and solid-test filter
- [ ] Task: Phase 1 Verification & Checkpoint (Refer to workflow.md)
  - [ ] Verify hitscan projectiles travel full distances and test pass rate

## Phase 2: Attacker Attribution for Tripbombs, Barrels & Props
- [ ] Task: Write Tests for Trap & Prop Explosion Attribution
  - [ ] Unit test: `LaserTripbomb` preserves `owner_player_id` and forwards to `ExplosionDamageEvent`
  - [ ] Unit test: Shooting an explosive barrel propagates shooter's `attacker_id` and awards frag to shooter
  - [ ] Unit test: Victim of enemy trap/barrel death is credited with a death, not a suicide (-1 frag)
- [ ] Task: Implement Attacker Propagation in `src/combat/`, `src/interactivity/`, and `src/player/`
  - [ ] Add `owner_player_id: Option<usize>` to `LaserTripbomb` in `src/combat/types.rs` and wire during placement in `src/player/weapons.rs`
  - [ ] Add `attacker_id: Option<usize>` to `WallDamageEvent` in `src/interactivity/types.rs`
  - [ ] Forward attacker from projectile impacts to barrels/extinguishers in `src/interactivity/wall_damage.rs` and `src/interactivity/props.rs`
- [ ] Task: Phase 2 Verification & Checkpoint (Refer to workflow.md)
  - [ ] Verify trap and barrel explosion kills attribute correctly to the attacker

## Phase 3: Explosive Double-Damage Fix & HandRemote Glitch Resolution
- [ ] Task: Write Tests for Rocket Splash Anti-Double-Dipping & HandRemote Detonation
  - [ ] Unit test: Direct RPG rocket hit applies 120 direct damage without applying duplicate epicenter splash to the primary target
  - [ ] Unit test: Firing `HandRemote` detonates active pipebombs without immediately throwing another pipebomb
- [ ] Task: Implement Anti-Double-Dipping and Detonation Fire Guard in `src/combat/projectiles.rs` and `src/player/weapons.rs`
  - [ ] Guard `handle_weapon_firing` auto-fire check with `!is_detonating`
  - [ ] Exclude direct-hit victim from taking stacked epicenter explosion splash damage
- [ ] Task: Phase 3 Verification & Checkpoint (Refer to workflow.md)
  - [ ] Verify rocket damage values and clean single-trigger pipebomb detonation

## Phase 4: Exotic PvP Weapon Effects (Shrinker & Freezethrower)
- [ ] Task: Write Tests for Shrinker & Freezethrower Effects on Players
  - [ ] Unit test: Player hit by Shrinker enters `shrink_timer = 9.0`
  - [ ] Unit test: Lethal damage from Freezethrower triggers `freeze_timer = 4.6` and `health = 1`
- [ ] Task: Implement Exotic PvP Status Effects in `src/net/pvp.rs`
  - [ ] Wire weapon types 6 (Shrinker) and 8 (Freezethrower) in `apply_pvp_damage`
- [ ] Task: Phase 4 Verification & Checkpoint (Refer to workflow.md)
  - [ ] Verify player shrinkage and freezing behavior under PvP conditions

## Phase 5: Full Verification, Warnings Audit & Track Completion
- [ ] Task: Comprehensive Test Suite & Warning Audit
  - [ ] Run `cargo check --tests` (ensure 0 compiler warnings)
  - [ ] Run full test suite across all tests (ensure 100% pass rate)
  - [ ] Review completed track with `conductor-review`
- [ ] Task: Phase 5 Verification & Checkpoint (Refer to workflow.md)
  - [ ] Final end-to-end verification and commit
