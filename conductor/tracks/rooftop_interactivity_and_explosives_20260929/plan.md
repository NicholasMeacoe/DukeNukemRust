# Implementation Plan: Rooftop Interactivity and Explosives Integrity

## Phase 1: Collider Geometry and Positioning Alignment
- [x] Task 1.1: TDD - Unit test for 2D barrel collider vertical centering and floor vent orientation
  - [x] Write unit test in `src/builder.rs` asserting `SEENINE` collider extends from floor $Y=0.0$ upward to full height ($Y \approx 0.8$)
  - [x] Write unit test asserting `FANSPRITE` floor-aligned collider lies flat horizontally along roof
- [x] Task 1.2: Correct 2D barrel vertical offset in `src/builder.rs`
  - [x] Check if barrel has voxel model; if purely 2D sprite (`SEENINE`, `FIREBARREL`), offset `pos.y += scale_y / 2.0` so cylinder collider rests on floor
  - [x] Ensure voxel barrels with bottom pivots preserve grounded translation
- [x] Task 1.3: Correct floor-aligned cuboid dimensions in `src/builder.rs`
  - [x] Update `is_floor_aligned` branch to `Collider::cuboid(scale_x / 2.0, scale_y / 2.0, 0.05)`
- [x] Task 1.4: Phase Verification & Checkpoint
  - [x] Run `cargo test builder` to verify collider geometry

## Phase 2: Explosive Radius Handling and Despawn Pipeline
- [x] Task 2.1: TDD - Unit test for AoE damage despawning `CrackWall` and chain reaction deduplication
  - [x] Write unit test in `src/interactivity/props.rs` verifying that an `ExplosionDamageEvent` hitting `CrackWall` despawns the entity and writes `GibEvent`
  - [x] Write unit test verifying that shooting a barrel generates only a single `ExplosionDamageEvent` via `BarrelExplodeEvent`
- [x] Task 2.2: Add entity despawn and gib dispatch to `handle_explosions` in `src/interactivity/props.rs`
  - [x] In `props.rs` `CrackWall` branch, query `Entity` and call `commands.entity(entity).despawn_recursive()`
  - [x] Send `GibEvent` with origin at vent translation and `gib_count: 6`
- [x] Task 2.3: Remove redundant `ExplosionDamageEvent` in `src/interactivity/wall_damage.rs`
  - [x] Remove `explosion_events.send(...)` in the `handle_wall_damage` barrel destruction block
- [x] Task 2.4: Phase Verification & Checkpoint
  - [x] Run `cargo test interactivity` and `cargo test props`
  - [x] Confirm full test suite passes with 0 warnings
