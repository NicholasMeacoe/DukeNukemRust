# Implementation Plan: Dynamic Lighting & Ken Silverman KVX Voxel Model Support

## Phase 1: Dynamic Point Lighting Engine & Transient Light Lifecycle
- [x] Task: Write Tests for Dynamic Light Lifecycles & Pooling
  - [x] Unit tests for transient light decay (`DynamicLightPool`, `TransientLight`, max count limits)
  - [x] Unit tests for muzzle flash and explosion light parameter mapping (radius, intensity, color)
- [x] Task: Implement Dynamic Light Entities & Systems
  - [x] Create `src/lighting.rs` with `DynamicLight`, `TransientLight`, and `DynamicLightingConfig`
  - [x] Implement decay system `update_transient_lights` attenuating intensity and despawning expired lights
  - [x] Implement dynamic light emitter hooks in `player/weapons.rs` (muzzle flashes), `combat/projectiles.rs` (rocket/laser trails), and `interactivity/props.rs` (explosions)
  - [x] Add console/cvar toggle `r_dynamiclights 0/1` in `src/player/console.rs`
- [x] Task: Phase 1 Verification & Checkpoint (Refer to workflow.md)
  - [x] Verify weapon discharges, flying projectiles, and explosions cast dynamic light with smooth decay and no leaks

## Phase 2: Ken Silverman KVX Voxel Binary Parser
- [x] Task: Write Tests for KVX Parsing & Slab Extraction
  - [x] Unit test parsing KVX headers (`xsiz`, `ysiz`, `zsiz`, `xpivot`, `ypivot`, `zpivot`)
  - [x] Unit test column offset decoding and slab structure (`ztop`, `zend`, color indices)
  - [x] Unit test boundary validation on malformed/truncated KVX byte arrays
- [x] Task: Implement Pure-Rust KVX Parser
  - [x] Create `src/voxel/kvx.rs` and `src/voxel/mod.rs`
  - [x] Implement `KvxModel::parse(&[u8]) -> Result<KvxModel, KvxError>`
  - [x] Expose 3D voxel color grid queries `get_voxel(x, y, z)` and dimensions
- [x] Task: Phase 2 Verification & Checkpoint (Refer to workflow.md)
  - [x] Verify parsing of sample/synthesized KVX assets and check data integrity against Build specs

## Phase 3: Voxel 3D Mesh Generation & Palette Shading
- [x] Task: Write Tests for Voxel Surface Meshing
  - [x] Unit tests verifying visible face extraction (unexposed internal face culling)
  - [x] Unit tests for quad vertex generation, normals, and vertex colors mapped from `PALETTE.DAT`
  - [x] Unit tests for pivot offset centering
- [x] Task: Implement Voxel Mesh Builder
  - [x] Implement `generate_voxel_mesh(model: &KvxModel, palette: &Palette) -> Mesh` in `src/voxel/mesh.rs`
  - [x] Generate Bevy `Mesh` with `Mesh::ATTRIBUTE_POSITION`, `Mesh::ATTRIBUTE_NORMAL`, and `Mesh::ATTRIBUTE_COLOR`
- [x] Task: Phase 3 Verification & Checkpoint (Refer to workflow.md)
  - [x] Verify generated Bevy `Mesh` compiles with valid vertex positions, normals, and colors

## Phase 4: Sprite Voxel Replacement Registry & Pickup Integration
- [x] Task: Write Tests for Voxel Sprite Registry & Rotation
  - [x] Unit tests verifying picnum registration (`VoxelRegistry`)
  - [x] Unit tests for spinning item rotation and bobbing animation
- [x] Task: Implement Voxel Model Spawning & World Integration
  - [x] Create `VoxelRegistry` resource in `src/voxel/registry.rs`
  - [x] Map core items (Medkits, Armor, Ammo, Keycards, Atomic Health) to voxel models
  - [x] Hook into `src/builder.rs` / `src/map.rs` sprite spawning: instantiate 3D voxel mesh instead of flat billboard sprite when model exists
  - [x] Add console/cvar toggle `r_voxels 0/1` in `src/player/console.rs`
- [x] Task: Phase 4 Verification & Checkpoint (Refer to workflow.md)
  - [x] Verify in-game pickups render as rotating 3D voxel models standing properly on sector floors

## Phase 5: Full Verification, Warnings Audit & Review
- [x] Task: Comprehensive Test Suite & Warning Audit
  - [x] Run `cargo check --tests` (ensure 0 compiler warnings)
  - [x] Run full test suite (ensure 100% pass rate across all tests)
  - [x] Review completed track with `conductor-review`
- [x] Task: Phase 5 Verification & Checkpoint (Refer to workflow.md)
  - [x] Final end-to-end verification and commit
