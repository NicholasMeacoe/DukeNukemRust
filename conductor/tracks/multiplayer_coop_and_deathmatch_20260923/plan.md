# Implementation Plan: Cooperative and Deathmatch Multiplayer (Networking / Splitscreen)

## Phase 1: Local Splitscreen Multi-Viewport & Player Controllers
- [x] Task: Write Tests for Splitscreen Viewport Layout & Multi-Player Input Mapping
  - [x] Unit test for 1, 2, 3, and 4-player viewport physical rect calculations (horizontal split, quadrant split)
  - [x] Unit test for `PlayerId` component and multi-player controller input assignment (Player 1 vs Player 2 keyboard & gamepad mapping)
  - [x] Unit test verifying camera and controller association without `get_single()` panics
- [x] Task: Implement Multi-Player Controllers & Viewport Slicing in `src/player/` & `src/render/`
  - [x] Add `PlayerId(pub usize)` and `PlayerCamera(pub usize)` components
  - [x] Refactor `update_player_movement`, `handle_weapon_firing`, and `handle_weapon_selection` to support multiple players
  - [x] Implement `setup_splitscreen_viewports` and `update_splitscreen_viewports` using Bevy `Camera.viewport`
  - [x] Implement secondary keyboard controls (Arrow keys, RCtrl, RShift, Keypad) and gamepad support for Player 2..4
- [x] Task: Phase 1 Verification & Checkpoint (Refer to workflow.md)
  - [x] Verify independent player movement and split viewports in running engine

## Phase 2: Duke Player Actor Rendering, Palette Swaps & Animations
- [x] Task: Write Tests for APLAYER 8-Directional Viewing & Palette Swapping
  - [x] Unit test for 8-directional viewing angle calculation relative to camera forward vector
  - [x] Unit test for palette lookup index translation (0..15 color swap mappings)
  - [x] Unit test for animation state machine transitions (Idle, Run, Shoot, Crouch, Gib)
- [x] Task: Implement Player Actor Sprite Rendering in `src/net/` and `src/render/`
  - [x] Create Duke player actor billboard rendering with tile `APLAYER` (#1405)
  - [x] Implement 8-directional sprite frame selection based on delta yaw between player orientation and camera look-at angle
  - [x] Implement palette swap tinting for multiplayer player colors
  - [x] Implement death collapse vs explosive gib spawn sequence for players
- [x] Task: Phase 2 Verification & Checkpoint (Refer to workflow.md)
  - [x] Verify player sprite rendered accurately with 8-directional views and custom shirt palettes

## Phase 3: Dukematch PvP Combat, Frag Attribution & Interactive Scoreboard
- [x] Task: Write Tests for PvP Damage Attribution, Frag Matrix & Scoreboard
  - [x] Unit tests for `EntityDamageEvent` and `ExplosionDamageEvent` with `attacker_id` propagation
  - [x] Unit test for frag scoring: killer kill count increment, victim death count increment, suicide penalty
  - [x] Unit test for match termination condition (kill limit or time limit reached)
  - [x] Unit test for F7 scoreboard UI rendering and player stats display
- [x] Task: Implement PvP Damage, Frag Tracking & Scoreboard HUD
  - [x] Update projectile impact and explosion systems to record attacking `PlayerId`
  - [x] Connect player lethal damage to `DukematchState::record_frag`
  - [x] Implement multi-spawn point selection and 3.0s spawn invulnerability
  - [x] Wire F7 interactive scoreboard overlay and in-game frag notifications
- [x] Task: Phase 3 Verification & Checkpoint (Refer to workflow.md)
  - [x] Verify player vs player damage, frag attribution, and F7 scoreboard display

## Phase 4: Non-Blocking UDP Network Socket Transport
- [x] Task: Write Tests for Non-Blocking UDP Transport & State Synchronization
  - [x] Unit test for UDP socket initialization with non-blocking mode on client/server ports
  - [x] Unit test for peer connection handshake, packet serialization/deserialization over UDP loopback
  - [x] Unit test for `InputSync` and `PlayerStateSync` interpolation and position smoothing
- [x] Task: Implement Non-Blocking UDP Transport in `src/net/`
  - [x] Implement `NetTransport` resource wrapping `std::net::UdpSocket` with non-blocking recv/send
  - [x] Add client/server network loop systems: `net_send_sync_system` and `net_receive_packets_system`
  - [x] Synchronize remote player positions, rotations, animations, and weapon firing over UDP
  - [x] Handle peer disconnection and timeout detection gracefully
- [x] Task: Phase 4 Verification & Checkpoint (Refer to workflow.md)
  - [x] Verify non-blocking UDP packets transmit and sync player state across peers

## Phase 5: Cooperative Campaign Mechanics (Shared Keys & Co-op Respawning)
- [x] Task: Write Tests for Shared Keycards & Cooperative Respawning
  - [x] Unit test for team-shared keycards (Player 1 pickup unlocks for Player 2)
  - [x] Unit test for cooperative respawn at sector checkpoint without map reset
  - [x] Unit test for friendly fire setting evaluation (damage ignored when friendly fire is disabled)
  - [x] Unit test for cooperative level transition synchronization
- [x] Task: Implement Cooperative Campaign Features
  - [x] Implement `SharedKeycards` resource or synchronize `player.has_keycard` across all co-op players
  - [x] Implement co-op respawning logic in `update_player_movement`: revive player at team start without reloading sector meshes/enemies
  - [x] Integrate friendly fire rule into `EntityDamageEvent` filter
  - [x] Ensure nuke button/level exit advances all players in cooperative sessions
- [x] Task: Phase 5 Verification & Checkpoint (Refer to workflow.md)
  - [x] Verify co-op gameplay mechanics: shared keys, co-op respawn, friendly fire toggle, level advance

## Phase 6: Full Verification, Warnings Audit & Review
- [x] Task: Comprehensive Test Suite & Warning Audit
  - [x] Run `cargo check --tests` (ensure 0 compiler warnings)
  - [x] Run full test suite (ensure 100% pass rate across all tests)
  - [x] Review completed track with `conductor-review`
- [x] Task: Phase 6 Verification & Checkpoint (Refer to workflow.md)
  - [x] Final end-to-end verification and commit
