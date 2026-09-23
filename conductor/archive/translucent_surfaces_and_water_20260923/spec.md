# Specification: Translucent Surfaces, Alpha Blending & Animated Water Planes

## 1. Overview
In original Duke Nukem 3D and the Build engine, water and transparent surfaces are iconic elements of environmental immersion. Sectors with lotag 1 (above water) and lotag 2 (underwater) feature a visible water surface dividing above and below. Walls and sprites with `cstat` bit 2 (`0x0004`, 33% translucent) or bit 9 (`0x0200`, 66% translucent) represent windows, glass panes, force fields, and water surfaces.

This track introduces authentic alpha-blended materials for translucent walls and sprites, procedural animated water surface planes on lotag 1/2 sectors with animated tile caustics, and boundary transition effects (blue screen tint, splash sound/particles, underwater muffling synchronization).

---

## 2. Functional Requirements

### 2.1 Build `cstat` Translucency Parsing & Material Generation
- Parse and detect `cstat & 0x0004` (translucent standard 33%) and `cstat & 0x0200` (translucent high 66%) on walls, middle masked walls, and sprites.
- Generate `StandardMaterial` with `AlphaMode::Blend` and base color alpha set to `0.33` (bit 2) or `0.66` (bit 9), or `0.60` for water surfaces.
- Support 1-sided and 2-sided masked walls that have translucency enabled (e.g. glass panes in E1L1 cinema, windows, force fields).

### 2.2 Procedural Water Surface Planes
- Identify water sectors where `sector.lotag == 1` (above water) or `sector.lotag == 2` (underwater).
- Generate a horizontal procedural polygon plane matching the sector boundary at the water surface height (`sector.floorz` for lotag 1 / `sector.ceilingz` for lotag 2).
- Apply animated water tile textures (e.g., tile 336 `W_WATER`, tile 337..340 water frames or slime/acid tiles based on sector picnum).
- Render the water surface with `AlphaMode::Blend` (60% alpha) visible from both above and below (two-sided).
- Tag the water surface entity with `WaterSurface { sector_index, elevation }`.

### 2.3 Water Animation & Caustics Cycling (`picanm`)
- Integrate water surface entities with the engine's `picanm` tile animation system so water surfaces cycle frames smoothly in real-time.
- Support slime (green) and lava/blood (red) surfaces based on sector picnum.

### 2.4 Water Surface Boundary Transitions & FX
- When the player transitions between `PlayerMovementMode::Walking` and `PlayerMovementMode::Diving` (or crosses the water plane):
  - Trigger blue screen tint overlay (`ScreenTintOverlay` with `Color::srgba(0.0, 0.2, 0.6, 0.3)`).
  - Emit splash sound event (Sound 112 `WATER_SPLASH` / Sound 113).
  - Spawn splash particle burst entity at entry coordinate.
  - Synchronize underwater audio muffling (attenuate music/sfx by 60%).
- Projectiles (bullets, RPG, pipebombs) entering water spawn splash particles and decelerate/detonate appropriately.

---

## 3. Non-Functional Requirements & Testing
- 0 compiler warnings on `cargo check --tests`.
- 100% test pass rate across all repository tests (280+ tests).
- Zero `#![allow(dead_code)]`.
- Automated unit tests covering:
  - `cstat` bit 2 and bit 9 extraction and alpha factor assignment.
  - Water surface plane generation for lotag 1/2 sectors with correct height and bounds.
  - Water boundary crossing triggers splash sound and screen tint.

---

## 4. Acceptance Criteria
- E1L1 cinema pool and E1L2 water basins display visible, translucent, animated water surfaces.
- Glass windows and masked translucent walls render with authentic alpha blending rather than 1-bit alpha mask or solid texture.
- Diving into water produces a splash sound, particle burst, blue tint overlay, and audio muffling.
