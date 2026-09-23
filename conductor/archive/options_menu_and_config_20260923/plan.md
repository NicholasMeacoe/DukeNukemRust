# Implementation Plan: In-Game Options Menu & Configuration Persistence

## Phase 1: Persistent Configuration Architecture & Disk I/O (`config.json`)
- [x] Task: Write Tests for Configuration Serialization & Loading
  - [x] Unit test verifying default `GameConfig` values
  - [x] Unit test verifying JSON round-trip serialization and deserialization
  - [x] Unit test verifying graceful fallback to defaults on corrupt/missing file
- [x] Task: Implement `GameConfig` and Persistence Services
  - [x] Define `GameConfig`, `SoundConfig`, `VideoConfig`, `ControlsConfig` in `src/config.rs`
  - [x] Implement `load_config`, `save_config`, and Bevy resource `ResMut<GameConfig>`
  - [x] Integrate initial config loading in `App` initialization in `src/main.rs`
- [x] Task: Phase 1 Verification & Checkpoint (Refer to workflow.md)
  - [x] Verify `config.json` reads and writes reliably with correct types and paths

## Phase 2: Game Flow State Machine & Navigation Infrastructure
- [x] Task: Write Tests for Options States & Submenu Navigation
  - [x] Unit test verifying navigation between MainMenu/Paused and OptionsMenu
  - [x] Unit test verifying navigation between OptionsMenu and Sound/Video/Controls submenus
  - [x] Unit test verifying cursor wrapping and Escape key returning to proper parent origin
- [x] Task: Implement Options Menu Phases & Cursor Handlers
  - [x] Add `OptionsMenu`, `SoundSetup`, `VideoSetup`, `ControlsSetup` to `GamePhase` in `src/game_flow/state.rs`
  - [x] Update `MenuCursor` navigation and Escape back-stack in `src/game_flow/menu.rs`
  - [x] Wire `OPTIONS` entry from Main Menu and Pause Menu
- [x] Task: Phase 2 Verification & Checkpoint (Refer to workflow.md)
  - [x] Verify keyboard navigation, cursor wrapping, and sub-menu transitions without state locks

## Phase 3: Sound Setup Submenu & Live Audio Volume Modulation
- [x] Task: Write Tests for Audio Settings & Slider Adjustments
  - [x] Unit test verifying volume slider value clamping (0.0 to 1.0 in 0.1 steps)
  - [x] Unit test verifying audio system volume scaling applied to music and sound channels
- [x] Task: Implement Sound Setup UI & Slider Controls
  - [x] Add UI rendering for Master, Sound FX, Music, and Voice volume sliders in `src/game_flow/ui.rs`
  - [x] Implement left/right arrow/key input handler for slider adjustments in `src/game_flow/menu.rs`
  - [x] Connect volume settings directly to Bevy audio/music players and SoundFont synth in `src/audio/`
- [x] Task: Phase 3 Verification & Checkpoint (Refer to workflow.md)
  - [x] Verify volume changes take effect immediately in real-time

## Phase 4: Video & Display Setup Submenu (CRT, Voxels, Lights, Window)
- [x] Task: Write Tests for Video Settings Toggles
  - [x] Unit test verifying CRT shader toggle mutates `CrtSettings.enabled`
  - [x] Unit test verifying 3D Voxel Models toggle mutates `VoxelConfig.enabled`
  - [x] Unit test verifying Dynamic Point Lights toggle mutates `DynamicLightingConfig.enabled`
- [x] Task: Implement Video Setup UI & Toggle Logic
  - [x] Add UI rendering for CRT shader, 3D voxels, dynamic lights, and window mode in `src/game_flow/ui.rs`
  - [x] Implement toggle handlers on Enter / Space / Left / Right
  - [x] Apply window mode changes via `bevy::window::Window` (Windowed vs BorderlessFullscreen)
- [x] Task: Phase 4 Verification & Checkpoint (Refer to workflow.md)
  - [x] Verify toggles immediately update active rendering features

## Phase 5: Gameplay & Controls Setup Submenu (Sensitivity, Invert Y, Auto-switch)
- [x] Task: Write Tests for Controls Settings
  - [x] Unit test verifying mouse sensitivity scaling on camera pitch and yaw
  - [x] Unit test verifying Invert Y flips mouse pitch delta
  - [x] Unit test verifying auto-switch on empty weapon behavior
- [x] Task: Implement Gameplay & Controls Setup UI
  - [x] Add UI rendering for Mouse Sensitivity, Invert Mouse Y, and Auto-Switch in `src/game_flow/ui.rs`
  - [x] Connect mouse sensitivity and invert Y to camera rotation in `src/player/camera.rs` / `src/player/mod.rs`
  - [x] Connect auto-switch setting to weapon management in `src/player/weapons.rs`
- [x] Task: Phase 5 Verification & Checkpoint (Refer to workflow.md)
  - [x] Verify camera mouse response adheres strictly to sensitivity and Invert Y settings

## Phase 6: Full Verification, Warnings Audit & Review
- [x] Task: Comprehensive Test Suite & Warning Audit
  - [x] Run `cargo check --tests` (ensure 0 compiler warnings)
  - [x] Run full test suite (ensure 100% pass rate across all tests)
  - [x] Review completed track with `conductor-review`
- [x] Task: Phase 6 Verification & Checkpoint (Refer to workflow.md)
  - [x] Final end-to-end verification and commit
