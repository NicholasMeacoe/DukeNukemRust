# Specification: Wall Translucency and Cstat Alpha Fix

## 1. Overview
In Build Engine maps (e.g. Duke Nukem 3D E1L1), wall and sprite attributes are packed into a 16-bit integer bitfield called `cstat`.
An erroneous check in `src/map.rs` conflated bit 2 (`cstat & 4`) with translucency.
- On walls, bit 2 (`cstat & 4`) is `align_bottom` (bottom texture alignment, used on almost all building facades, street curbs, and alley walls).
- On sprites, bit 2 (`cstat & 4`) is `x_flipped` (horizontal mirroring).
Because `Wall::is_translucent()` returned `true` for any wall with `cstat & 4`, solid walls were assigned `MaterialAlphaMode::Blend(33)` (33% opacity). In Bevy, this enabled alpha blending without depth writing, making foreground street and building walls translucent and allowing the background panoramic sky (`LA_SKY`) to show straight through solid geometry.

## 2. Functional Requirements
1. **Wall Cstat Correction**:
   - `Wall::is_translucent()`: Must return `(self.cstat & 128) != 0` (Build engine wall translucency bit 7). Remove `(self.cstat & 4) != 0`.
   - `Wall::is_translucent_reversed()`: Must return `(self.cstat & 512) != 0` (Build engine wall reverse translucency bit 9).
   - `Wall::alpha_mode(&self, is_masked: bool)`: Solid walls (`!is_masked`) must ALWAYS return `MaterialAlphaMode::Opaque`. Only masked middle walls (`is_masked == true`) may return `Blend(66)`, `Blend(33)`, or `Mask`.
2. **Sprite Cstat Correction**:
   - `Sprite::is_translucent()`: Must return `(self.cstat & 2) != 0` (Build engine sprite translucency bit 1). Remove `(self.cstat & 4) != 0`.
   - `Sprite::is_translucent_reversed()`: Must return `(self.cstat & 512) != 0` (Build engine sprite reverse translucency bit 9).
   - Add `Sprite::is_x_flipped()` (`(self.cstat & 4) != 0`) and `Sprite::is_y_flipped()` (`(self.cstat & 8) != 0`).
3. **Actor View Model Correction**:
   - In `src/main.rs`, update actor material evaluation: remove `|| (con_actor.cstat & 4) != 0` so horizontally flipped actors do not render with 33% translucency.
4. **Sprite Horizontal/Vertical UV Mirroring**:
   - In `src/builder.rs`, use `sprite.is_x_flipped()` and `sprite.is_y_flipped()` to invert UV coordinates for mirrored sprites.

## 3. Acceptance Criteria
1. All solid single-sided walls and upper/lower sector walls render fully opaque (`AlphaMode::Opaque`).
2. Bottom-aligned walls (`cstat & 4`) are opaque and never translucent.
3. The background skyline (`LA_SKY`) is only visible in open sky sectors, completely obscured by solid walls.
4. Masked walls (fences, grates, breakable glass) and translucent sprites retain correct transparency.
5. All tests in the test suite pass cleanly (100% pass rate) with 0 compiler warnings.
