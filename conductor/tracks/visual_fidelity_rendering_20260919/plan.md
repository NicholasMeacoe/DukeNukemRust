# Implementation Plan: Visual Fidelity & Authentic Build Rendering

## Phase 1: 32-Level Palette Shade Pipeline & Distance Lighting
- [ ] Task: Write Unit Tests for Shade Calculation & Distance Falloff (TDD)
  - [ ] Test 32-level shade table clamping, shade multiplier math, and distance falloff formula
- [ ] Task: Implement Build Shade Material / Pipeline
  - [ ] Create custom Bevy material/shader utilizing `PALETTE.DAT` 32 shade tables
  - [ ] Apply sector and wall base shades (`floorshade`, `ceilingshade`, `wall.shade`, `sprite.shade`)
- [ ] Task: Runtime Sector Shade Modulation
  - [ ] Connect Light Strobe, Flicker, and Glow sector effectors to material uniform mutations
- [ ] Task: Phase 1 Verification & Checkpoint (Refer to workflow.md)

## Phase 2: Translucent Water & Liquid Surface Rendering
- [ ] Task: Write Unit Tests for Water Sector Surface Bounds & Geometry (TDD)
  - [ ] Test surface plane generation for sectors with lotag 1 (above water) and lotag 2 (underwater)
- [ ] Task: Generate Water Surface Mesh Entities
  - [ ] Spawn horizontal boundary quads using `AlphaMode::Blend` with ~66% opacity
  - [ ] Link `AnimatedTileMaterial` to cycle water/slime UV frames via `picanm`
- [ ] Task: Phase 2 Verification & Checkpoint (Refer to workflow.md)

## Phase 3: Alpha-Blended Masked Walls, Glass & Translucent Props
- [ ] Task: Write Unit Tests for `cstat` Translucency Parsing & Material Assignment (TDD)
  - [ ] Test 33% (`cstat & 128`) and 66% (`cstat & 512`) translucency flag evaluation
- [ ] Task: Upgrade Masked Wall & Prop Rendering
  - [ ] Enable `AlphaMode::Blend` on transparent two-sided walls (windows, grates, curtains)
  - [ ] Apply proper alpha blending to projector beam and breakable glass
- [ ] Task: Phase 3 Verification & Checkpoint (Refer to workflow.md)

## Phase 4: Screen-Space Parallax Sky Projection
- [ ] Task: Write Unit Tests for Camera Orientation to Parallax UV Mapping (TDD)
  - [ ] Test wrapping and aspect-ratio scaling of Build sky textures across 360-degree yaw
- [ ] Task: Implement Screen-Space Parallax Sky System
  - [ ] Track camera yaw/pitch and update parallax ceiling UV offsets
  - [ ] Replace static inverted sky cylinder with screen-space aligned parallax projection
- [ ] Task: Phase 4 Verification & Checkpoint (Refer to workflow.md)
