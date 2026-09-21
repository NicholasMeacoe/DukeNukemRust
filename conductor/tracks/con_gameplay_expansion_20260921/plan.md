# Implementation Plan: Authentic CON Scripting & Full Enemy Roster Expansion

## Phase 1: Real GAME.CON Parsing & Pipeline Integration
- [x] Task: Audit & Enhance Real `GAME.CON` Compilation
  - [x] Test compiling `GAME.CON` directly from `duke3d.grp` with `#include` resolution for `DEFS.CON` and `USER.CON`
  - [x] Resolve any tokenization or grammar edge cases in `Lexer` and `Compiler` (case insensitivity, comment variations, multi-line string quotes)
  - [x] Expand `DEFAULT_CORE_CON_SCRIPT` with comprehensive declarations for all standard actors and actions
- [x] Task: Phase 1 Verification & Checkpoint
  - [x] Verify `ConScriptEngine::from_grp` compiles the full script with >20 actors and registers all symbols

## Phase 2: Full 20+ Enemy Roster Definition & Spawning
- [x] Task: Map All Enemy Picnums & State Machines
  - [x] Register all 20+ enemy picnums in `src/names.rs` and `src/combat/ai.rs` (Troopers, Captains, Pig Cops, Octabrains, Enforcers, Recon Vehicles, Drones, Commanders, Slimers, Turrets, Sharks)
  - [x] Update `map_tile_to_projectile` for all enemy weapon types (Trooper lasers, Octabrain spit, Pigcop shotgun, Enforcer vulcan, RPV missiles, Commander rockets, Battlelord mortar)
  - [x] Wire authentic enemy death drops (Pigcop -> Shotgun/Armor, Boss -> Atomic Health, Enforcer -> Chaingun)
- [x] Task: Phase 2 Verification & Checkpoint
  - [x] Unit tests verifying all 20+ enemy picnums spawn valid `ConActor` states and dispatch authentic projectiles

## Phase 3: Boss Encounters & Climax Mechanics
- [ ] Task: Battlelord & Boss AI Combat Logic
  - [ ] Implement Battlelord (`BOSS1`) minigun barrage and lobbed mortar artillery in `src/combat/ai.rs`
  - [ ] Add footstep screen shake for massive boss steps using `EarthquakeCameraShake`
  - [ ] Wire boss defeat to emit `LevelCompletedEvent` / `endofgame` in boss levels (`E1L7` / `E1L8`)
- [ ] Task: Phase 3 Verification & Checkpoint
  - [ ] Unit tests verifying Battlelord combat behavior, mortar fire, and boss death triggering level victory

## Phase 4: Environmental Actors & Polish
- [ ] Task: Ambient Actors & Explosions
  - [ ] Add non-AI ambient actor loops (rats, debris, fire hazards, radioactive barrels)
  - [ ] Connect `palfrom` screen tinting to major explosions
- [ ] Task: Phase 4 Verification & Checkpoint
  - [ ] Verify ambient actors and explosion effects tick without errors

## Phase 5: Verification, Integration & Review
- [ ] Task: Comprehensive Test Suite & Warning Audit
  - [ ] Run `cargo check --tests` (ensure 0 warnings across all 71 source files)
  - [ ] Run full test suite (ensure 100% pass rate across 231+ tests)
  - [ ] Review completed track with `conductor-review`
