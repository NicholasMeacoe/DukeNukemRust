# Track Specification: ECS Test Rigor and Integration Coverage

## 1. Overview
The architectural review revealed that while the project maintains high unit test pass counts, several tests evaluated local duplicate logic rather than executing runtime ECS systems. This track upgrades test suite rigor by introducing real Bevy `App` simulations for weapon viewmodels, multi-system integration tests for chain explosions and vent destruction, and hitscan raycast impact verification.

## 2. Functional Requirements
- **FR-1: Real ECS Viewmodel System Tests**: Replace static arithmetic tests in `src/player/weapons.rs` with an `App`-driven test that runs `update_first_person_viewmodel`, advances `Time`, and observes `FirstPersonViewModel.current_tile` transitions in the ECS world.
- **FR-2: Explosive Despawn Integration Test**: Create an ECS integration test verifying that an explosion in `handle_explosions` reduces `CrackWall` health to 0, despawns the entity from the world, and verifies no orphan colliders remain.
- **FR-3: Barrel Chain Reaction Integration Test**: Create an integration test verifying that a bullet hitting one barrel detonates it, triggers `BarrelExplodeEvent`, damages an adjacent barrel within radius, and causes a cascading chain explosion.
- **FR-4: Weapon Switching Animation Safety Test**: Verify that switching weapons while `fire_timer > 0` properly resets or clamps viewmodel animation progress to prevent visual artifacting.

## 3. Non-Functional Requirements
- Zero compiler warnings (`cargo check --tests`).
- 100% test pass rate across the workspace.
- Deterministic test execution without relying on live GPU or display server.

## 4. Acceptance Criteria
1. `test_pistol_viewmodel_animation_frames` runs as a genuine Bevy ECS system test using `app.update()`.
2. Tests explicitly assert `app.world().get_entity(vent_entity).is_none()` following an explosion.
3. All tests run fast (< 5 seconds total) and pass consistently in headless CI.
