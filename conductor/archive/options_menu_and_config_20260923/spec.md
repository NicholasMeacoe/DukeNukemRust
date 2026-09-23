# Specification: In-Game Options Menu & Configuration Persistence

## Overview
This track delivers a comprehensive in-game Options and Settings system for Duke Nukem 3D Rust, accessible from both the Main Menu (`GamePhase::MainMenu`) and the Pause Menu (`GamePhase::Paused`). Players can customize Sound settings (Master, FX, Music, Voice volumes), Video & Display settings (CRT post-processing shader, 3D KVX voxel models, dynamic point lighting, window mode), and Gameplay & Controls settings (mouse sensitivity, invert mouse Y, auto-switch weapons on empty). All user configuration is saved to disk via `config.json` and automatically loaded on application startup.

---

## Functional Requirements

### 1. Menu Architecture & State Flow
- **Options Hub (`GamePhase::OptionsMenu`)**:
  - Accessible via `OPTIONS` on the Main Menu and Pause Menu.
  - Submenu choices:
    1. `SOUND SETUP`
    2. `VIDEO & DISPLAY`
    3. `GAMEPLAY & CONTROLS`
    4. `RESTORE DEFAULTS`
- **Submenu States**:
  - `GamePhase::SoundSetup`
  - `GamePhase::VideoSetup`
  - `GamePhase::ControlsSetup`
- **Navigation & Controls**:
  - Up/Down (ArrowUp, ArrowDown, W, S) to navigate rows with animated Duke cursor and audio blips.
  - Enter / Space to activate toggles or submenus.
  - Left/Right (ArrowLeft, ArrowRight, A, D) to adjust slider values and toggle states.
  - Escape key returns to parent menu (returning to `OptionsMenu` from submenus, or returning to `SaveLoadOrigin` from `OptionsMenu`).

### 2. Sound Setup (`GamePhase::SoundSetup`)
- **Master Volume**: 0% to 100% in 10% steps. Multiplies all audio output.
- **Sound FX Volume**: 0% to 100% in 10% steps. Scales game sound effects and ambient sounds.
- **Music Volume**: 0% to 100% in 10% steps. Scales MIDI SoundFont background music playback.
- **Voice / Taunt Volume**: 0% to 100% in 10% steps. Scales Duke RTS voice quotes and one-liners.
- **Real-Time Feedback**: Volume slider changes take effect immediately on active audio channels.

### 3. Video & Display Setup (`GamePhase::VideoSetup`)
- **CRT Shader**: `Enabled` / `Disabled`. Controls fullscreen retro CRT curvature and scanline post-processing (`CrtSettings.enabled`).
- **3D Voxel Models**: `Enabled` / `Disabled`. Controls whether pickups and environmental props render as 3D KVX voxel models or classic 2D billboards (`VoxelConfig.enabled` / `r_voxels`).
- **Dynamic Point Lights**: `Enabled` / `Disabled`. Controls muzzle flash, projectile glow, and explosion light sources (`DynamicLightingConfig.enabled` / `r_dynamic_lights`).
- **Display Mode**: `Windowed` / `Borderless Fullscreen`. Controls Bevy window mode via `bevy::window::Window`.

### 4. Gameplay & Controls Setup (`GamePhase::ControlsSetup`)
- **Mouse Sensitivity**: 0.5x to 3.0x in 0.25x steps. Scales mouse delta on player camera look.
- **Invert Mouse Y**: `Off` / `On`. Inverts vertical mouse pitch delta.
- **Auto-Switch Weapons**: `On` / `Off`. Automatically selects best available weapon when active ammo is exhausted.
- **Screen Bobbing**: `On` / `Off`. Controls viewmodel and camera walking bob.

### 5. Persistent Configuration (`config.json`)
- Serializable `GameConfig` containing `SoundConfig`, `VideoConfig`, and `ControlsConfig`.
- Auto-loaded at engine startup; falls back cleanly to defaults if file is missing or corrupted.
- Auto-saved whenever returning from the Options Menu or modifying settings.

---

## Non-Functional Requirements
- Zero compiler warnings on `cargo check --tests`.
- 100% test pass rate across the full test suite.
- Zero `#![allow(dead_code)]`.
- Responsive, authentic 1996 3D Realms visual style using Duke font styling and sound cues.

---

## Out of Scope
- Custom gamepad controller remapping (future gamepad polish track).
- Network multiplayer lobby browser.
