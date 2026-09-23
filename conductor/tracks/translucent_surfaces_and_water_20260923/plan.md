# Implementation Plan: Translucent Surfaces, Alpha Blending & Animated Water Planes

## Phase 1: Translucency Model & Build `cstat` Material Pipeline
- [x] Task: Write Tests for Translucency Detection & Material Properties
  - [x] Unit test verifying cstat bit 2 (0x0004) maps to standard translucency alpha (0.33)
  - [x] Unit test verifying cstat bit 9 (0x0200) maps to high translucency alpha (0.66)
  - [x] Unit test verifying non-translucent cstat defaults to opaque / alpha mask
- [x] Task: Implement Translucent Material Generation in `src/builder.rs` & `src/main.rs`
  - [x] Update wall mesh generation to inspect `wall.cstat` for translucency bits
  - [x] Create `StandardMaterial` with `AlphaMode::Blend` and base_color alpha channel
  - [x] Update middle masked wall and sprite billboard pipelines to support `AlphaMode::Blend`
- [x] Task: Phase 1 Verification & Checkpoint (Refer to workflow.md)
  - [x] Verify glass windows and translucent walls render with alpha blending in E1L1

## Phase 2: Procedural Water Surface Mesh Generation (Lotag 1 & 2)
- [x] Task: Write Tests for Water Surface Plane Tessellation
  - [x] Unit test identifying sectors with lotag 1 and 2
  - [x] Unit test calculating water plane elevation and bounding polygon triangulation
  - [x] Unit test verifying two-sided mesh normal and UV coordinate generation
- [x] Task: Implement Water Surface Mesh Spawning in `src/builder.rs` / `src/sector_map/`
  - [x] Generate horizontal polygon mesh for water sectors at water surface boundary
  - [x] Tag water surface entity with `WaterSurface { sector_index, elevation }`
  - [x] Apply `AlphaMode::Blend` material with water tile texture and 60% alpha
- [x] Task: Phase 2 Verification & Checkpoint (Refer to workflow.md)
  - [x] Verify water surface plane appears in E1L1 cinema pool and E1L2 water basins

## Phase 3: Water Animation & Caustics Cycling (`picanm`)
- [x] Task: Write Tests for Water Surface Tile Animation Tick
  - [x] Unit test verifying water tile frame progression based on picanm header
  - [x] Unit test verifying material texture handle swapping on water surfaces
- [x] Task: Connect Water Surfaces to Tile Animation System
  - [x] Tag water surface meshes with `AnimatedTileMaterial` (or dedicated water animator)
  - [x] Support animated slime (green) and lava/blood (red) surfaces based on sector picnum
- [x] Task: Phase 3 Verification & Checkpoint (Refer to workflow.md)
  - [x] Verify water ripples and frame animation cycle continuously in real-time

## Phase 4: Water Surface Boundary Transitions, Screen Tint & FX
- [ ] Task: Write Tests for Water Boundary Crossing & Sound Dispatch
  - [ ] Unit test verifying player crossing water boundary triggers splash sound event
  - [ ] Unit test verifying underwater screen tint overlay activation
  - [ ] Unit test verifying projectile splash triggering upon water entry
- [ ] Task: Implement Water Boundary Detection & Visual/Audio Dispatch
  - [ ] System detecting player crossing water surface elevation
  - [ ] Trigger `WATER_SPLASH` (sound 112) and spawn water splash particles
  - [ ] Activate `ScreenTintOverlay` with blue tint while submerged, clear upon exiting
  - [ ] Connect to underwater audio muffling system
- [ ] Task: Phase 4 Verification & Checkpoint (Refer to workflow.md)
  - [ ] Verify smooth diving in and climbing out of water with splash sound, tint, and audio muffling

## Phase 5: Full Verification, Warnings Audit & Review
- [ ] Task: Comprehensive Test Suite & Warning Audit
  - [ ] Run `cargo check --tests` (ensure 0 compiler warnings)
  - [ ] Run full test suite (ensure 100% pass rate across all tests)
  - [ ] Review completed track with `conductor-review`
- [ ] Task: Phase 5 Verification & Checkpoint (Refer to workflow.md)
  - [ ] Final end-to-end verification and commit
