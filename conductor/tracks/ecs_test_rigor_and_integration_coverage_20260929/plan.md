# Implementation Plan: ECS Test Rigor and Integration Coverage

## Phase 1: Viewmodel ECS System Tests
- [ ] Task 1.1: Refactor viewmodel animation tests to execute `update_first_person_viewmodel`
  - [ ] Construct a Bevy `App` with `Time` resource and `PlayerController` + `FirstPersonViewModel` entities
  - [ ] Advance time across 0.05s, 0.20s, and 0.50s ticks and assert `vm.current_tile` progression through 2525, 2526, and 2524
- [ ] Task 1.2: Add weapon switching mid-animation test
  - [ ] Test switching from Pistol to Shotgun while `fire_timer` is non-zero, asserting viewmodel resets cleanly to idle
- [ ] Task 1.3: Phase Verification & Checkpoint
  - [ ] Run `cargo test weapons`

## Phase 2: Explosive Despawn & Chain Reaction Integration Tests
- [ ] Task 2.1: Write integration test for `CrackWall` destruction via `handle_explosions`
  - [ ] Spawn `CrackWall` entity with `Transform`, `Collider`, and `RigidBody`
  - [ ] Send `ExplosionDamageEvent` within damage radius
  - [ ] Run system chain `(handle_wall_damage, handle_explosions).chain()`
  - [ ] Assert entity is completely despawned from the ECS world (`app.world().get_entity(id).is_none()`)
- [ ] Task 2.2: Write integration test for barrel chain reaction
  - [ ] Spawn two adjacent `ExplodingBarrel` entities
  - [ ] Send bullet `WallDamageEvent` to Barrel 1
  - [ ] Run systems and assert Barrel 2 also detonates and both entities despawn
- [ ] Task 2.3: Phase Verification & Checkpoint
  - [ ] Run `cargo test` across all targets
  - [ ] Confirm 100% test pass rate with 0 compiler warnings
