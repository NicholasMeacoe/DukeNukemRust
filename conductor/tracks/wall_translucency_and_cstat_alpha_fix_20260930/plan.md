# Implementation Plan: Wall Translucency and Cstat Alpha Fix

## Phase 1: Core Cstat & Map Type Corrections (TDD)
- [x] Task 1.1: TDD - Update unit tests in `src/map.rs`
  - [x] Assert `make_wall(4)` has `align_bottom() == true`, `is_translucent() == false`, and `alpha_mode(false) == MaterialAlphaMode::Opaque`
  - [x] Assert `make_sprite(4, 100)` has `is_x_flipped() == true`, `is_translucent() == false`, and `alpha_mode() == MaterialAlphaMode::Mask`
- [x] Task 1.2: Correct `Wall` and `Sprite` cstat methods in `src/map.rs`
  - [x] Fix `Wall::is_translucent()` to check only `(self.cstat & 128) != 0`
  - [x] Fix `Wall::alpha_mode(&self, is_masked: bool)`: return `Opaque` when `!is_masked`
  - [x] Fix `Sprite::is_translucent()` to check only `(self.cstat & 2) != 0`
  - [x] Add `Sprite::is_x_flipped()` and `Sprite::is_y_flipped()`
- [x] Task 1.3: Phase Verification & Checkpoint
  - [x] Run `cargo test map`

## Phase 2: Sprite Mirroring & Rendering Hardening
- [x] Task 2.1: Update actor material generation in `src/main.rs`
  - [x] Remove `(con_actor.cstat & 4) != 0` from actor translucency checks
- [x] Task 2.2: Apply sprite UV mirroring in `src/builder.rs`
  - [x] Invert UVs horizontally when `sprite.is_x_flipped()` is true
  - [x] Invert UVs vertically when `sprite.is_y_flipped()` is true
- [x] Task 2.3: Phase Verification & Checkpoint
  - [x] Run full test suite via `cargo test`
  - [x] Run `cargo check --all-targets` to confirm 0 warnings
