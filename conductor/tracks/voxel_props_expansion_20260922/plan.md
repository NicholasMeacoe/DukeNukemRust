# Implementation Plan: Environmental & Interactive 3D Voxel Props Expansion

## Phase 1: Environmental Prop Voxel Models & Procedural Generation
- [ ] Task: Write Tests for Prop Voxel Models
  - [ ] Unit tests verifying registration of prop picnums in `VoxelRegistry` (Barrels, Fire Extinguisher, Camera, Fan, Fountain)
  - [ ] Unit tests verifying prop voxel dimensions, solid voxel counts, and pivot offsets
- [ ] Task: Implement Procedural KVX Voxel Models
  - [ ] Implement `create_barrel_model`, `create_fireext_model`, `create_camera_model`, `create_fan_model`, and `create_fountain_model` in `src/voxel/registry.rs`
  - [ ] Register all environmental prop models in `VoxelRegistry::register_default_models`
- [ ] Task: Phase 1 Verification & Checkpoint (Refer to workflow.md)
  - [ ] Verify generated voxel models produce valid surface-culled meshes with correct palette colors

## Phase 2: Kinetic & Motorized Voxel Prop Systems
- [ ] Task: Write Tests for Kinetic Prop Animations
  - [ ] Unit test verifying ceiling fan continuous rotation over time
  - [ ] Unit test verifying security camera yaw oscillation sweep over time
- [ ] Task: Implement Kinetic Prop Systems
  - [ ] Create `CeilingFanVoxel` and `SecurityCameraVoxel` components in `src/voxel/registry.rs`
  - [ ] Implement `update_kinetic_voxel_props` system updating rotation and sweep angles
  - [ ] Register system in `VoxelPlugin`
- [ ] Task: Phase 2 Verification & Checkpoint (Refer to workflow.md)
  - [ ] Verify ceiling fans spin smoothly and security cameras oscillate without jitter

## Phase 3: Wall, Floor & Ceiling Alignment & Spawning
- [ ] Task: Write Tests for Prop World Spawning & Alignment
  - [ ] Unit test verifying floor alignment for standing barrels
  - [ ] Unit test verifying wall-normal alignment and surface offset for wall-mounted props (`FIREEXT`, `CAMERA1`, `WATERFOUNTAIN`)
  - [ ] Unit test verifying ceiling anchoring for ceiling fans
- [ ] Task: Integrate Voxel Prop Spawning in Map Builder
  - [ ] Update `src/builder.rs` sprite spawning to instantiate 3D voxel models for environmental props
  - [ ] Attach interactive components (`ExplodingBarrel`, `FireExtinguisher`, `SecurityCamera`, `WaterFountain`) directly to voxel entities
- [ ] Task: Phase 3 Verification & Checkpoint (Refer to workflow.md)
  - [ ] Verify props spawn in their authentic orientations flush against walls, floors, and ceilings

## Phase 4: Destructible Voxel Prop State Transitions
- [ ] Task: Write Tests for Voxel Prop Destruction & Debris
  - [ ] Unit test verifying barrel explosion despawns voxel model, damages radius, and triggers dynamic point light
  - [ ] Unit test verifying fire extinguisher explosion and water fountain state transition
- [ ] Task: Integrate Destruction State Transitions
  - [ ] Connect voxel entity destruction hooks in `src/interactivity/props.rs`
  - [ ] Spawn debris particles and dynamic explosion point lights upon barrel / fire extinguisher destruction
- [ ] Task: Phase 4 Verification & Checkpoint (Refer to workflow.md)
  - [ ] Verify shooting barrels or fire extinguishers causes high-impact explosion with debris and light flash

## Phase 5: Full Verification, Warnings Audit & Review
- [ ] Task: Comprehensive Test Suite & Warning Audit
  - [ ] Run `cargo check --tests` (ensure 0 compiler warnings)
  - [ ] Run full test suite (ensure 100% pass rate across all tests)
  - [ ] Review completed track with `conductor-review`
- [ ] Task: Phase 5 Verification & Checkpoint (Refer to workflow.md)
  - [ ] Final end-to-end verification and commit
