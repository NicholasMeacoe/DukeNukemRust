# Implementation Plan: Codebase Dead Code & System Wiring Audit

## Phase 1: Parsers, Formats & Assets Audit
- [ ] Task: Audit & Clean Asset Modules
  - [ ] Remove `#![allow(dead_code)]` from `src/art.rs`, `src/palette.rs`, `src/animation.rs`, `src/kwv.rs`, `src/config.rs`, `src/names.rs`
  - [ ] Remove `#![allow(dead_code)]` from `src/audio/voc.rs`, `src/audio/rts.rs`, `src/audio/midi.rs`
  - [ ] Wire or test any uncalled parsing utilities; add targeted `#[allow(dead_code)]` only to authentic Build binary struct fields
- [ ] Task: Audit & Clean Map & Demo Formats
  - [ ] Remove `#![allow(dead_code)]` from `src/map.rs`, `src/demo/format.rs`, `src/demo/mod.rs`, `src/demo/player.rs`, `src/demo/recorder.rs`, `src/demo/attract.rs`
  - [ ] Verify demo player/recorder integration paths
- [ ] Task: Phase 1 Verification & Checkpoint (Refer to workflow.md)

## Phase 2: Player Controller, Movement & Weapons Audit
- [ ] Task: Audit & Clean Player Core & Movement
  - [ ] Remove `#![allow(dead_code)]` from `src/player/mod.rs`, `src/player/types.rs`, `src/player/movement.rs`
  - [ ] Verify player crouch, swim, jump, and slope adherence systems are registered in `PlayerPlugin`
- [ ] Task: Audit & Clean Weapons, Inventory & Console
  - [ ] Remove `#![allow(dead_code)]` from `src/player/inventory.rs`, `src/player/weapons.rs`, `src/player/console.rs`
  - [ ] Wire all 10 weapon fire/tick handlers and inventory consumption routines; eliminate orphaned helpers
- [ ] Task: Phase 2 Verification & Checkpoint (Refer to workflow.md)

## Phase 3: Interactivity, Effectors, Props & Wall Damage Audit
- [ ] Task: Audit & Clean Interactivity Core & Types
  - [ ] Remove `#![allow(dead_code)]` from `src/interactivity/mod.rs`, `src/interactivity/types.rs`, `src/sector_map.rs`
  - [ ] Ensure all switch, activator, and touchplate handlers are registered in `InteractivityPlugin`
- [ ] Task: Audit & Clean Props, Effectors & Wall Damage
  - [ ] Remove `#![allow(dead_code)]` from `src/interactivity/props.rs`, `src/interactivity/effectors.rs`, `src/interactivity/wall_damage.rs`
  - [ ] Verify all 30+ sector effector kinds (doors, lifts, subways, rotators, water, earthquakes) are dispatched without unhandled dead variants
- [ ] Task: Phase 3 Verification & Checkpoint (Refer to workflow.md)

## Phase 4: Combat, Projectiles & AI Audit
- [ ] Task: Audit & Clean Combat Types & Projectiles
  - [ ] Remove `#![allow(dead_code)]` from `src/combat/types.rs`, `src/combat/projectiles.rs`, `src/combat/mod.rs`
  - [ ] Verify all projectile types (rocket, pipebomb, shrinker ray, freezer blast, bullet tracer, spit) have active spawn/update/collision loops
- [ ] Task: Audit & Clean AI, Gore & Decals
  - [ ] Remove `#![allow(dead_code)]` from `src/combat/ai.rs`, `src/combat/gore.rs`, `src/combat/decals.rs`
  - [ ] Wire all enemy AI state machines (patrol, seek, attack, dodge, pain, die, gib) and decal spawning into Bevy `CombatPlugin`
- [ ] Task: Phase 4 Verification & Checkpoint (Refer to workflow.md)

## Phase 5: Audio, Save, Game Flow, HUD & CON Scripting Audit
- [ ] Task: Audit & Clean Audio Subsystem
  - [ ] Remove `#![allow(dead_code)]` from `src/audio/mod.rs`, `src/audio/sound_defs.rs`
  - [ ] Verify voice limiter, spatial audio, ambient emitters, and RTS voice triggers are active in `AudioPlugin`
- [ ] Task: Audit & Clean Save/Load Subsystem
  - [ ] Remove `#![allow(dead_code)]` from `src/save/mod.rs`, `src/save/format.rs`, `src/save/snapshot.rs`
  - [ ] Verify 10-slot disk serialization, hotkeys (F2/F3/F6/F9), and sector height restoration
- [ ] Task: Audit & Clean Game Flow, Campaign & HUD Subsystem
  - [ ] Remove `#![allow(dead_code)]` from `src/game_flow/*`, `src/campaign/*`, `src/hud/*`
  - [ ] Verify all UI states (MainMenu, EpisodeSelect, SkillSelect, Playing, Paused, SaveMenu, LoadMenu, Intermission) and HUD elements are connected
- [ ] Task: Audit & Clean CON Scripting Engine
  - [ ] Remove `#![allow(dead_code)]` from `src/scripting/types.rs`, `src/scripting/physics.rs`, `src/scripting/mod.rs`, `src/scripting/vm.rs`, `src/scripting/compiler.rs`, `src/scripting/lexer.rs`
  - [ ] Verify all VM opcode handlers, AST structures, and script physics systems are wired and warning-free
- [ ] Task: Phase 5 Verification & Checkpoint (Refer to workflow.md)
