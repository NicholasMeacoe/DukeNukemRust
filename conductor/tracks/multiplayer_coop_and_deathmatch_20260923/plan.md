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
- [ ] Task: Write Tests for PvP Damage Attribution, Frag Matrix & Scoreboard
  - [ ] Unit tests for `EntityDamageEvent` and `ExplosionDamageEvent` with `attacker_id` propagation
  - [ ] Unit test for frag scoring: killer kill count increment, victim death count increment, suicide penalty
  - [ ] Unit test for match termination condition (kill limit or time limit reached)
  - [ ] Unit test for F7 scoreboard UI rendering and player stats display
- [ ] Task: Implement PvP Damage, Frag Tracking & Scoreboard HUD
  - [ ] Update projectile impact and explosion systems to record attacking `PlayerId`
  - [ ] Connect player lethal damage to `DukematchState::record_frag`
  - [ ] Implement multi-spawn point selection and 3.0s spawn invulnerability
  - [ ] Wire F7 interactive scoreboard overlay and in-game frag notifications
- [ ] Task: Phase 3 Verification & Checkpoint (Refer to workflow.md)
  - [ ] Verify player vs player damage, frag attribution, and F7 scoreboard display

## Phase 4: Non-Blocking UDP Network Socket Transport
- [ ] Task: Write Tests for Non-Blocking UDP Transport & State Synchronization
  - [ ] Unit test for UDP socket initialization with non-blocking mode on client/server ports
  - [ ] Unit test for peer connection handshake, packet serialization/deserialization over UDP loopback
  - [ ] Unit test for `InputSync` and `PlayerStateSync` interpolation and position smoothing
- [ ] Task: Implement Non-Blocking UDP Transport in `src/net/`
  - [ ] Implement `NetTransport` resource wrapping `std::net::UdpSocket` with non-blocking recv/send
  - [ ] Add client/server network loop systems: `net_send_sync_system` and `net_receive_packets_system`
  - [ ] Synchronize remote player positions, rotations, animations, and weapon firing over UDP
  - [ ] Handle peer disconnection and timeout detection gracefully
- [ ] Task: Phase 4 Verification & Checkpoint (Refer to workflow.md)
  - [ ] Verify non-blocking UDP packets transmit and sync player state across peers

## Phase 5: Cooperative Campaign Mechanics (Shared Keys & Co-op Respawning)
- [ ] Task: Write Tests for Shared Keycards & Cooperative Respawning
  - [ ] Unit test for team-shared keycards (Player 1 pickup unlocks for Player 2)
  - [ ] Unit test for cooperative respawn at sector checkpoint without map reset
  - [ ] Unit test for friendly fire setting evaluation (damage ignored when friendly fire is disabled)
  - [ ] Unit test for cooperative level transition synchronization
- [ ] Task: Implement Cooperative Campaign Features
  - [ ] Implement `SharedKeycards` resource or synchronize `player.has_keycard` across all co-op players
  - [ ] Implement co-op respawning logic in `update_player_movement`: revive player at team start without reloading sector meshes/enemies
  - [ ] Integrate friendly fire rule into `EntityDamageEvent` filter
  - [ ] Ensure nuke button/level exit advances all players in cooperative sessions
- [ ] Task: Phase 5 Verification & Checkpoint (Refer to workflow.md)
  - [ ] Verify co-op gameplay mechanics: shared keys, co-op respawn, friendly fire toggle, level advance

## Phase 6: Full Verification, Warnings Audit & Review
- [ ] Task: Comprehensive Test Suite & Warning Audit
  - [ ] Run `cargo check --tests` (ensure 0 compiler warnings)
  - [ ] Run full test suite (ensure 100% pass rate across all tests)
  - [ ] Review completed track with `conductor-review`
- [ ] Task: Phase 6 Verification & Checkpoint (Refer to workflow.md)
  - [ ] Final end-to-end verification and commit
