# Implementation Plan: Rooftop Ventilation Shaft Teleport (SE 7) and Viewmodel Alignment

## Phase 1: Viewmodel Proportions and Iron Sight Reticle Alignment
- [x] Task 1.1: TDD - Unit test for viewmodel sizing and horizontal reticle alignment
  - [x] Write unit test in `src/main.rs` verifying pistol viewmodel base height is 275.0px and horizontal aim offset is 89.0px
- [x] Task 1.2: Implement compact viewmodel scaling and aim offset in `src/main.rs`
  - [x] Update `base_height` for Pistol, Shotgun, Chaingun, RPG, Knee in `sync_first_person_viewmodel`
  - [x] Apply `aim_offset_x` to `style.margin.left`
- [x] Task 1.3: Phase Verification & Checkpoint
  - [x] Run `cargo test` on main / viewmodel tests

## Phase 2: Sector Effector 7 Teleporter Pairing and Execution
- [x] Task 2.1: TDD - Unit test for SE 7 teleporter hitag pairing and teleport execution
  - [x] Write unit test in `src/interactivity/effectors.rs` verifying player falling past trigger height in sector 269 is teleported to sector 256 with safe velocity and matching yaw
- [x] Task 2.2: Update `EffectorKind::UnderwaterTeleport` data structure in `src/interactivity/types.rs`
  - [x] Add `target_yaw`, `trigger_height`, `trigger_radius`, and `teleport_cooldown` fields
- [x] Task 2.3: Implement SE 7 sprite hitag pairing in `src/interactivity/mod.rs`
  - [x] Pre-scan all SE 7 sprites (`picnum == 1 && lotag == 7`), pair matching hitags, and spawn components with paired destination parameters
- [x] Task 2.4: Implement `update_teleporter_sector_effectors` system in `src/interactivity/effectors.rs`
  - [x] Handle trigger crossing, player translation update, yaw update, sector update, fall velocity clamping, and cooldown management
  - [x] Remove old non-functional stub in `src/player/movement.rs`
  - [x] Register `update_teleporter_sector_effectors` in `InteractivityPlugin`
- [x] Task 2.5: Phase Verification & Checkpoint
  - [x] Run full test suite via `cargo test`
  - [x] Verify 100% test pass rate with 0 compiler warnings
