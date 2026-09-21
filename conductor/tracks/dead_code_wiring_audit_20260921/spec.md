# Specification: Codebase Dead Code & System Wiring Audit

## Overview
Conduct a comprehensive codebase audit to eliminate file-level `#![allow(dead_code)]` annotations across 59+ Rust source files in `DukeNukemRust`. Wire any orphaned Bevy systems, components, resources, and event handlers into the runtime ECS schedule, clean up obsolete dead code, and apply targeted field-level `#[allow(dead_code)]` exclusively for authentic Build binary format specifications and reference constants.

## Functional Requirements
1. **Subsystem 1: Parsers, Formats & Assets**
   - Audit `art.rs`, `kwv.rs`, `palette.rs`, `animation.rs`, `config.rs`, `names.rs`, `map.rs`, `audio/voc.rs`, `audio/rts.rs`, `audio/midi.rs`, `demo/format.rs`.
   - Remove file-level `#![allow(dead_code)]`.
   - Wire any unused asset decoding/lookup utilities; apply targeted `#[allow(dead_code)]` only on authentic binary specification fields.
2. **Subsystem 2: Player Controller, Movement & Weapons**
   - Audit `player/mod.rs`, `player/types.rs`, `player/movement.rs`, `player/inventory.rs`, `player/weapons.rs`, `player/console.rs`.
   - Remove file-level `#![allow(dead_code)]`.
   - Ensure all weapon firing routines, inventory items, cheat handlers, and console functions are wired into the Bevy schedule or tested.
3. **Subsystem 3: Interactivity, Effectors & Props**
   - Audit `interactivity/mod.rs`, `interactivity/types.rs`, `interactivity/props.rs`, `interactivity/effectors.rs`, `interactivity/wall_damage.rs`.
   - Remove file-level `#![allow(dead_code)]`.
   - Ensure all sector effector kinds (SE 0..30), triggers, touchplates, breakable props, and wall damage handlers are actively registered and updated.
4. **Subsystem 4: Combat, Projectiles & AI**
   - Audit `combat/mod.rs`, `combat/types.rs`, `combat/projectiles.rs`, `combat/ai.rs`, `combat/gore.rs`, `combat/decals.rs`.
   - Remove file-level `#![allow(dead_code)]`.
   - Connect any orphaned projectile physics, AI attack behaviors, gibbing particles, or bullet decal systems to game events.
5. **Subsystem 5: Audio, Save, Game Flow, HUD & CON Scripting**
   - Audit `audio/mod.rs`, `audio/sound_defs.rs`, `save/mod.rs`, `save/format.rs`, `save/snapshot.rs`, `game_flow/*`, `hud/*`, `campaign/*`, `demo/*`, `scripting/*`.
   - Remove file-level `#![allow(dead_code)]`.
   - Verify every UI screen, sound trigger, intermission handler, save serializer, and CON VM instruction handler is wired.

## Non-Functional Requirements
- 100% Safe Rust, 0 compiler warnings with `cargo check --tests`.
- Zero compiler warnings when running `cargo clippy --tests` (or minimized to only external Bevy ECS query complexity).
- Preserve all existing 224 automated tests with zero regressions.
- No functional regressions in map loading, player controls, combat, or audio playback.

## Acceptance Criteria
- All 59 source files have `#![allow(dead_code)]` removed at the crate/module level.
- Any orphaned ECS systems identified during the audit are registered in their respective Bevy plugins or test fixtures.
- `cargo check --tests` compiles with 0 warnings.
- Test suite passes 100% (224+ passing tests).

## Out of Scope
- Rewriting the CON VM architecture (covered by separate scripting track).
- Adding new game assets not in original `duke3d.grp`.
