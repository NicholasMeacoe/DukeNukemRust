# Implementation Plan: Environmental & Interactive 3D Voxel Props Expansion

## Phase 1: Environmental Prop Voxel Models & Procedural Generation
- [x] Task: Write Tests for Prop Voxel Models
  - [x] Unit tests verifying registration of prop picnums in `VoxelRegistry` (Barrels, Fire Extinguisher, Camera, Fan, Fountain)
  - [x] Unit tests verifying prop voxel dimensions, solid voxel counts, and pivot offsets
- [x] Task: Implement Procedural KVX Voxel Models
  - [x] Implement `create_barrel_model`, `create_fireext_model`, `create_camera_model`, `create_fan_model`, and `create_fountain_model` in `src/voxel/registry.rs`
  - [x] Register all environmental prop models in `VoxelRegistry::register_default_models`
- [x] Task: Phase 1 Verification & Checkpoint (Refer to workflow.md)
  - [x] Verify generated voxel models produce valid surface-culled meshes with correct palette colors

## Phase 2: Kinetic & Motorized Voxel Prop Systems
- [x] Task: Write Tests for Kinetic Prop Animations
  - [x] Unit test verifying ceiling fan continuous rotation over time
  - [x] Unit test verifying security camera yaw oscillation sweep over time
- [x] Task: Implement Kinetic Prop Systems
  - [x] Create `CeilingFanVoxel` and `SecurityCameraVoxel` components in `src/voxel/registry.rs`
  - [x] Implement `update_kinetic_voxel_props` system updating rotation and sweep angles
  - [x] Register system in `VoxelPlugin`
- [x] Task: Phase 2 Verification & Checkpoint (Refer to workflow.md)
  - [x] Verify ceiling fans spin smoothly and security cameras oscillate without jitter

## Phase 3: Wall, Floor & Ceiling Alignment & Spawning
- [x] Task: Write Tests for Prop World Spawning & Alignment
  - [x] Unit test verifying floor alignment for standing barrels
  - [x] Unit test verifying wall-normal alignment and surface offset for wall-mounted props (`FIREEXT`, `CAMERA1`, `WATERFOUNTAIN`)
  - [x] Unit test verifying ceiling anchoring for ceiling fans
- [x] Task: Integrate Voxel Prop Spawning in Map Builder
  - [x] Update `src/builder.rs` sprite spawning to instantiate 3D voxel models for environmental props
  - [x] Attach interactive components (`ExplodingBarrel`, `FireExtinguisher`, `SecurityCamera`, `WaterFountain`) directly to voxel entities
- [x] Task: Phase 3 Verification & Checkpoint (Refer to workflow.md)
  - [x] Verify props spawn in their authentic orientations flush against walls, floors, and ceilings

## Phase 4: Destructible Voxel Prop State Transitions
- [x] Task: Write Tests for Voxel Prop Destruction & Debris
  - [x] Unit test verifying barrel explosion despawns voxel model, damages radius, and triggers dynamic point light
  - [x] Unit test verifying fire extinguisher explosion and water fountain state transition
- [x] Task: Integrate Destruction State Transitions
  - [x] Connect voxel entity destruction hooks in `src/interactivity/props.rs`
  - [x] Spawn debris particles and dynamic explosion point lights upon barrel / fire extinguisher destruction
- [x] Task: Phase 4 Verification & Checkpoint (Refer to workflow.md)
  - [x] Verify shooting barrels or fire extinguishers causes high-impact explosion with debris and light flash

## Phase 5: Full Verification, Warnings Audit & Review
- [ ] Task: Comprehensive Test Suite & Warning Audit
  - [ ] Run `cargo check --tests` (ensure 0 compiler warnings)
  - [ ] Run full test suite (ensure 100% pass rate across all tests)
  - [ ] Review completed track with `conductor-review`
- [ ] Task: Phase 5 Verification & Checkpoint (Refer to workflow.md)
  - [ ] Final end-to-end verification and commit
