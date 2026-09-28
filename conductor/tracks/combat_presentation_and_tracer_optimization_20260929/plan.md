# Implementation Plan: Combat Presentation and Tracer Asset Optimization

## Phase 1: Viewmodel Frames and Asset Allocation Optimization
- [ ] Task 1.1: TDD - Unit test for RPG blast frame and tracer asset reuse
  - [ ] Write unit test asserting `WeaponType::Rpg` viewmodel displays tile `2606` when `progress < 0.35`
  - [ ] Write unit test in `src/combat/projectiles.rs` asserting multiple projectile spawns reuse identical `Handle<Mesh>` and `Handle<StandardMaterial>`
- [ ] Task 1.2: Correct RPG firing tile in `src/player/weapons.rs`
  - [ ] Update `WeaponType::Rpg` firing frame from `2544` to `2606`
- [ ] Task 1.3: Implement `BulletTracerAssets` resource and handle reuse in `src/combat/projectiles.rs`
  - [ ] Define `pub struct BulletTracerAssets { pub mesh: Handle<Mesh>, pub material: Handle<StandardMaterial> }`
  - [ ] Initialize resource during app startup or lazy initialization
  - [ ] Update `spawn_projectiles` to clone existing handles from `BulletTracerAssets`
- [ ] Task 1.4: Phase Verification & Checkpoint
  - [ ] Verify test suite passes with `cargo test combat` and `cargo test weapons`

## Phase 2: Hitscan Feel and Muzzle Safety
- [ ] Task 2.1: TDD - Unit test for instantaneous hitscan collision and standoff clamping
  - [ ] Write unit test verifying hitscan bullets resolve collision sweeps immediately
  - [ ] Write unit test verifying muzzle spawn offset does not exceed player capsule radius
- [ ] Task 2.2: Instantaneous hitscan damage and sweep velocity
  - [ ] In `src/combat/projectiles.rs`, ensure `HitscanBullet` and `ShotgunPellet` sweep velocity covers the effective engagement range immediately on spawn frame while rendering high-speed tracers
- [ ] Task 2.3: Safe muzzle standoff positioning
  - [ ] Adjust forward spawn offset in `src/player/weapons.rs` from $0.4\text{--}0.5\,\text{m}$ to a safe $0.22\,\text{m}$ within the player's 0.3m capsule radius
- [ ] Task 2.4: Phase Verification & Checkpoint
  - [ ] Run full test suite via `cargo test`
  - [ ] Confirm clean compilation with 0 warnings
