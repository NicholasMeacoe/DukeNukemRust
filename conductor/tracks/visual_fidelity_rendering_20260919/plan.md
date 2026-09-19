# Implementation Plan: Visual Fidelity & Authentic Build Rendering

## Phase 1: 32-Level Palette Shade Pipeline & Distance Lighting
- [x] Task: Write Unit Tests for Shade Calculation & Distance Falloff (TDD)
  - [x] Test 32-level shade table clamping, shade multiplier math, and distance falloff formula
- [x] Task: Implement Build Shade Material / Pipeline
  - [x] Create custom Bevy material/shader utilizing `PALETTE.DAT` 32 shade tables
  - [x] Apply sector and wall base shades (`floorshade`, `ceilingshade`, `wall.shade`, `sprite.shade`)
- [x] Task: Runtime Sector Shade Modulation
  - [x] Connect Light Strobe, Flicker, and Glow sector effectors to material uniform mutations
- [x] Task: Phase 1 Verification & Checkpoint (Refer to workflow.md)

## Phase 2: Translucent Water & Liquid Surface Rendering
- [x] Task: Write Unit Tests for Water Sector Surface Bounds & Geometry (TDD)
  - [x] Test surface plane generation for sectors with lotag 1 (above water) and lotag 2 (underwater)
- [x] Task: Generate Water Surface Mesh Entities
  - [x] Spawn horizontal boundary quads using `AlphaMode::Blend` with ~66% opacity
  - [x] Link `AnimatedTileMaterial` to cycle water/slime UV frames via `picanm`
- [x] Task: Phase 2 Verification & Checkpoint (Refer to workflow.md)

## Phase 3: Alpha-Blended Masked Walls, Glass & Translucent Props
- [x] Task: Write Unit Tests for `cstat` Translucency Parsing & Material Assignment (TDD)
  - [x] Test 33% (`cstat & 128`) and 66% (`cstat & 512`) translucency flag evaluation
- [x] Task: Upgrade Masked Wall & Prop Rendering
  - [x] Enable `AlphaMode::Blend` on transparent two-sided walls (windows, grates, curtains)
  - [x] Apply proper alpha blending to projector beam and breakable glass
- [x] Task: Phase 3 Verification & Checkpoint (Refer to workflow.md)

## Phase 4: Screen-Space Parallax Sky Projection
- [x] Task: Write Unit Tests for Camera Orientation to Parallax UV Mapping (TDD)
  - [x] Test wrapping and aspect-ratio scaling of Build sky textures across 360-degree yaw
- [x] Task: Implement Screen-Space Parallax Sky System
  - [x] Track camera yaw/pitch and update parallax ceiling UV offsets
  - [x] Replace static inverted sky cylinder with screen-space aligned parallax projection
- [x] Task: Phase 4 Verification & Checkpoint (Refer to workflow.md)
