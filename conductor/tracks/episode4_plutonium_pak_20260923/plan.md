# Implementation Plan: Episode 4 Plutonium Pak Campaign Maps & Alien Queen Boss Fight

## Phase 1: Alien Queen Combat AI & Multi-Phase Attack Behaviors
- [x] Task: Write Tests for Alien Queen Attack Cycle & Status Immunities
  - [x] Unit test verifying Queen attack cycle: triple eye lightning discharge, spit/venom burst, and close-range tail swipe
  - [x] Unit test verifying Queen sound definitions (`BOS4_ATTACK`, `BOS4_DYING`) and dropped Atomic Health
  - [x] Unit test verifying Queen status effect immunities (immune to shrink, freeze, expander)
- [x] Task: Implement Alien Queen Attack Routines & Sounds in `src/combat/` and `src/audio/`
  - [x] Add `BOS4_ATTACK` and `BOS4_DYING` sound constants in `src/audio/sound_defs.rs`
  - [x] Implement multi-phase attack timer for `Boss4Queen` in `src/combat/ai.rs`
  - [x] Wire boss footstep tremors and death scream in `src/combat/ai.rs`
- [x] Task: Phase 1 Verification & Checkpoint (Refer to workflow.md)
  - [x] Verify Alien Queen initiates combat attacks, spawns projectiles towards player, and resists status effects

## Phase 2: Episode 4 Boss Level Victory Sequence
- [ ] Task: Write Tests for Queen Boss Defeat Level Completion on E4L10
  - [ ] Unit test verifying Queen defeat on E4L10 ("The Queen") triggers `LevelCompletedEvent { is_secret: false, is_boss_victory: true }`
  - [ ] Unit test verifying non-boss levels or mini-bosses do NOT trigger level completion
- [ ] Task: Wire Boss Defeat Level Completion on E4L10 & Victory Intermission Banner
  - [ ] Verify `crate::campaign::episodes::is_boss_level(4, 10)` triggers victory event on Queen death
  - [ ] Verify Intermission UI displays gold "E4L10: EPISODE VICTORY!" and plays bonus victory quote
- [ ] Task: Phase 2 Verification & Checkpoint (Refer to workflow.md)
  - [ ] Verify beating the Queen on E4L10 cleanly advances to episode completion

## Phase 3: Episode 4 Campaign Map Traversal & Secret Level Routing
- [ ] Task: Write Tests for Episode 4 Map Traversal & Secret Level Routing
  - [ ] Unit test verifying `advance_to_next_level` navigates E4L1 through E4L10 sequentially
  - [ ] Unit test verifying secret exit on E4L4 ("Babe Land") routes to E4L11 ("Area 51")
  - [ ] Unit test verifying completing secret level E4L11 returns to canonical post-secret stage E4L5 ("Pigsty")
  - [ ] Unit test verifying completing E4L10 resets level to 1 and returns `true` (campaign/episode complete)
- [ ] Task: Implement Episode 4 Progression Logic & Secret Level Routing
  - [ ] Verify `advance_to_next_level` handles `(4, 11) -> 5` canonical secret return
  - [ ] Verify secret nuke switch triggers `is_secret_exit` on E4L4
- [ ] Task: Phase 3 Verification & Checkpoint (Refer to workflow.md)
  - [ ] Verify seamless map navigation from E4L1 through E4L10 and secret level warping

## Phase 4: Episode 4 Atmospheric Red Skybox & Menu Pipeline Integration
- [ ] Task: Write Tests for Episode 4 Skybox & Menu Episode 4 Selection
  - [ ] Unit test verifying `sky_tile_for_episode(4)` returns `REDSKY1` (#98)
  - [ ] Unit test verifying Episode Select menu index 3 selects Episode 4 and initializes level 1
- [ ] Task: Implement `sky_tile_for_episode(4)` & Menu Navigation
  - [ ] Update `src/sky.rs` `sky_tile_for_episode(4)` to return `crate::names::REDSKY1`
  - [ ] Verify Episode 4 selection in `src/game_flow/menu.rs` dispatches `LoadLevelEvent { episode: 4, level: 1 }`
- [ ] Task: Phase 4 Verification & Checkpoint (Refer to workflow.md)
  - [ ] Verify Episode 4 loads with authentic red atmospheric sky

## Phase 5: Full Verification, Warnings Audit & Review
- [ ] Task: Comprehensive Test Suite & Warning Audit
  - [ ] Run `cargo check --tests` (ensure 0 compiler warnings)
  - [ ] Run full test suite (ensure 100% pass rate across all tests)
  - [ ] Review completed track with `conductor-review`
- [ ] Task: Phase 5 Verification & Checkpoint (Refer to workflow.md)
  - [ ] Final end-to-end verification and commit
