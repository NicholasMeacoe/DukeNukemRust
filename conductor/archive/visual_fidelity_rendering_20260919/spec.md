# Specification: Visual Fidelity & Authentic Build Rendering

## Overview
Elevate the visual fidelity of `DukeNukemRust` from flat modern PBR rendering to authentic 1996 Build engine aesthetics, implementing 32-level distance palette shading, translucent water surfaces, alpha-blended masked walls/glass, and screen-space parallax skies.

## Functional Requirements
1. **32-Level Palette Shade Pipeline**: Custom Bevy material/shader utilizing `PALETTE.DAT` shade tables. Calculates distance attenuation using Build's authentic formula (`shade = curshade + (dist * factor)`), preserving the signature dark atmosphere of Build engine maps.
2. **Translucent Water Surfaces**: Generate horizontal boundary meshes for water sectors (`lotag == 1` or `2`) using `AlphaMode::Blend` with ~66% opacity and animated tile cycling (`picanm`), enabling players to look down into submerged areas.
3. **Alpha-Blended Masked Walls & Props**: Upgrade masked walls and breakable glass (`cstat & 128` / `cstat & 512`) from 1-bit `AlphaMode::Mask` to true smooth alpha blending (`AlphaMode::Blend`) for stained glass, projector beams, and force fields.
4. **Authentic Screen-Space Parallax Sky**: Replace the static inverted cylinder with a camera-yaw/pitch synchronized UV projection for parallax ceilings (`ceilingstat & 1`), emulating the infinite horizon of the original Build engine.
5. **Runtime Sector Shade Modulation**: Connect Sector Effectors (Light Strobe, Flicker, Glow, Switches) to dynamic uniform updates on sector mesh materials.

## Non-Functional Requirements
- 100% Safe Rust, 0 compiler warnings via `cargo check --tests`.
- Maintain 60+ FPS at 1280x720 rendering.
- Zero regressions across the 216 existing automated tests.

## Acceptance Criteria
- All 216 existing tests continue to pass without regression.
- `cargo check --tests` compiles with 0 warnings.
- E1L1 cinema, alleys, and vents exhibit genuine atmospheric depth and lighting falloff.
- Pools in E1L1 (or E1L2) render transparent, animated water surfaces.
- Parallax sky scrolls smoothly and proportionally to player look angles.

## Out of Scope
- Full planar stencil mirror reflections (deferred to subsequent polish).
- Overlapping room-over-room (TROR) clipping.
