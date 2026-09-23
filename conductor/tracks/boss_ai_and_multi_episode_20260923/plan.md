# Implementation Plan: Boss AI & Multi-Episode Level Progression

## Phase 1: Boss Combat AI & Multi-Phase Attack Behaviors
- [x] Task: Write Tests for Boss Attack Cycles & Projectile Spawning
  - [x] Unit test verifying Overlord attack cycle: dual shoulder rockets, rapid blaster fire, and close-range stomp
  - [x] Unit test verifying Cycloid Emperor attack cycle: forehead eye psychic blasts, quad arm rocket salvos, and ground shockwave
  - [x] Unit test verifying Battlelord death sound (115) and boss damage resistances / HP scaling
- [x] Task: Implement Overlord & Cycloid Emperor Attack Logic in `src/combat/ai.rs`
  - [x] Implement multi-phase attack timer for `Boss2Overlord` (dual shoulder rockets + chest blasters + tail whip)
  - [x] Implement multi-phase attack timer for `Boss3Cycloid` (eye beam + rocket salvos + ground stomp shockwave)
  - [x] Add sound triggers for boss attack, pain, and death screams (BOS1, BOS2, BOS3)
- [x] Task: Phase 1 Verification & Checkpoint (Refer to workflow.md)
  - [x] Verify boss entities initiate correct attack sequences and spawn projectiles toward target

## Phase 2: Boss Level Victory Triggers & Death Sequence
- [ ] Task: Write Tests for Boss Level Defeat Detection
  - [ ] Unit test verifying boss defeat on E1L6 (Battlelord), E2L9 (Overlord), and E3L9 (Cycloid) triggers level completion
  - [ ] Unit test verifying mini-bosses (e.g. Boss1Mini in E2L7/E3L7) and non-boss levels do NOT trigger level completion
- [ ] Task: Implement Accurate Boss Level Victory Triggers in `src/combat/ai.rs` & `src/game_flow/`
  - [ ] Update `is_boss_level` detection to query `CampaignMapInfo` from `src/campaign/episodes.rs`
  - [ ] Update `LevelCompletedEvent` to support `is_boss_victory: bool` and `is_secret: bool`
  - [ ] Dispatch boss death taunts, atomic health drops, and victory event transition
- [ ] Task: Phase 2 Verification & Checkpoint (Refer to workflow.md)
  - [ ] Verify defeating each episode's boss initiates the victory sequence

## Phase 3: Multi-Episode Campaign Progression & Secret Level Routing
- [ ] Task: Write Tests for Campaign Map Traversal & Secret Destinations
  - [ ] Unit test verifying `advance_to_next_level` supports variable level counts across Episode 1, 2, and 3
  - [ ] Unit test verifying secret exit button routes player to secret level destination (E1L3 -> E1L8, E2L5 -> E2L10, E3L5 -> E3L10)
  - [ ] Unit test verifying completing secret level returns to canonical post-secret stage (E1L8 -> E1L4, E2L10 -> E2L6, E3L10 -> E3L6)
- [ ] Task: Implement Dynamic Campaign Traversal in `src/game_flow/state.rs` & `src/game_flow/intermission.rs`
  - [ ] Update `LevelProgress::advance_to_next_level` to accept `is_secret: bool` and use `ALL_CAMPAIGN_MAPS`
  - [ ] Connect `NukeExitSwitch { is_secret }` to `LevelCompletedEvent { is_secret }` in `src/interactivity/props.rs`
  - [ ] Implement episode transition on boss victory (advancing to next episode or MainMenu if campaign complete)
- [ ] Task: Phase 3 Verification & Checkpoint (Refer to workflow.md)
  - [ ] Verify level transitions, secret level warping, and episode advancements flow seamlessly

## Phase 4: Episode-Specific Atmospheric Skyboxes & Victory Intermission UI
- [ ] Task: Write Tests for Episode Skybox Selection & Victory Banner
  - [ ] Unit test verifying correct skybox tile selection per episode (Ep 1 LA_SKY #89, Ep 2 MOONSKY1 #80, Ep 3 CITY_SKY #84)
  - [ ] Unit test verifying intermission stats screen renders "EPISODE VICTORY" banner on boss defeat
- [ ] Task: Implement Episode Skybox Selection & Victory UI in `src/game_flow/` & `src/sky/`
  - [ ] Update `handle_load_level_events` in `src/game_flow/level_loader.rs` to select episode-authentic skybox tiles
  - [ ] Update intermission UI in `src/game_flow/ui.rs` to render "EPISODE COMPLETED" / "VICTORY" banner and play victory quote
- [ ] Task: Phase 4 Verification & Checkpoint (Refer to workflow.md)
  - [ ] Verify authentic visual skybox in Episodes 1, 2, and 3, and victory banner appearance

## Phase 5: Full Verification, Warnings Audit & Review
- [ ] Task: Comprehensive Test Suite & Warning Audit
  - [ ] Run `cargo check --tests` (ensure 0 compiler warnings)
  - [ ] Run full test suite (ensure 100% pass rate across all tests)
  - [ ] Review completed track with `conductor-review`
- [ ] Task: Phase 5 Verification & Checkpoint (Refer to workflow.md)
  - [ ] Final end-to-end verification and commit
