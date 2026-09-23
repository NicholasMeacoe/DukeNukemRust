# Specification: Cooperative and Deathmatch Multiplayer (Networking / Splitscreen)

## Overview
This track implements complete multiplayer support for Duke Nukem 3D in Rust. It introduces a dual multiplayer architecture supporting both local splitscreen (multi-viewport 2 to 4 players on a single machine) and non-blocking UDP socket networking for LAN and remote peers. It incorporates authentic Duke player actor rendering (`APLAYER` #1405) with 8-directional viewing and palette color swaps, Dukematch (deathmatch) PvP combat with frag matrix scoring and interactive F7 scoreboard, and a Cooperative campaign mode with shared keys, cooperative respawning, and level transitions.

---

## Functional Requirements

### 1. Dual Multiplayer Architecture: Splitscreen & UDP Networking
- **Local Splitscreen Multi-Viewport:**
  - Support 2-player horizontal splitscreen (Top: Player 1, Bottom: Player 2) and 4-player quadrant splitscreen (Top-Left P1, Top-Right P2, Bottom-Left P3, Bottom-Right P4).
  - Use Bevy's `Camera.viewport` with `bevy::render::camera::Viewport` to segment the window render target dynamically based on active player count.
  - Dynamically resize viewports when the window dimensions change.
  - Tag cameras with `PlayerCamera(pub usize)` and associate with corresponding `PlayerId(pub usize)`.
  - Independent player controllers and input mappings:
    - Player 1: WASD movement + Mouse aim / Mouse button fire / Number keys 1-0 weapon select.
    - Player 2: Arrow keys movement + Right Ctrl / Space / Enter fire / Keypad or bracket weapon select.
    - Gamepad support: Map gamepad axes and buttons to respective `PlayerId`.
- **Non-Blocking UDP Network Transport:**
  - Non-blocking UDP socket binding (`std::net::UdpSocket` with `set_nonblocking(true)`).
  - Client / Server and Peer connection management handling `NetPacket::Connect`, `Disconnect`, and keep-alive heartbeats.
  - Lockstep input sync and client prediction: `NetPacket::InputSync` transmission every tick (or 30Hz) and `NetPacket::PlayerStateSync` for transform reconciliation and lag mitigation.
  - Network error handling: graceful handling of dropped packets, out-of-order delivery, and player disconnection.

### 2. Duke Player Actor Rendering & Visuals
- **Actor Kind & Tile Representation:**
  - Render other players in the 3D world using the canonical Duke player actor (`APLAYER` #1405).
  - Billboard sprite facing camera, with 8-directional viewing based on relative angle between the observer's viewing vector and the target player's yaw.
- **Palette Swapping (0..15):**
  - Support Duke palette lookup color tints (0: Classic Red Shirt/Blue Pants, 9: Blue, 10: Red, 11: Green, 12: Grey, 13: Dark Green, 14: Brown, 15: Dark Blue).
  - Apply player palette index to the sprite material or vertex tint so each combatant is immediately identifiable.
- **Animations & Gib Sequence:**
  - Player animation states: Idle, Running/Walking, Firing/Shooting, Crouching, and Swimming.
  - Standard death animation (collapsing) on bullet/melee lethal damage.
  - Visceral Gib Death sequence (`DUKEGIB` / meat chunks and blood splatters) when killed by explosions (RPG, Pipebomb, Tripbomb, Devastator) or crushing sectors.

### 3. Dukematch (Deathmatch) Mode
- **PvP Combat & Damage Attribution:**
  - Projectiles (RPG rockets, Shotgun pellets, Pistol hitscans, Chaingun bullets, Freezethrower blasts, Shrinker blasts, Pipebombs, Tripbombs) collide with and deal damage to other player colliders.
  - Propagate `attacker: Option<usize>` (killer player ID) through `EntityDamageEvent` and `ExplosionDamageEvent`.
  - Self-damage (e.g. rocket point-blank or pipebomb self-blast) attributed as suicide.
- **Frag Matrix & Scoring (`DukematchState`):**
  - Kill awarded to killer: `frags[killer_id][victim_id] += 1`.
  - Suicide penalty: `frags[victim_id][victim_id] -= 1`.
  - Frag limit (e.g. 25 kills) and Time limit (e.g. 15 minutes) match termination conditions.
- **Scoreboard & Match HUD:**
  - Interactive F7 scoreboard overlay rendering the 8x8 frag matrix, total frags, deaths, and ping for each connected player.
  - In-game frag announcements (e.g. "Duke fragged Player 2 with RPG", "Player 2 committed suicide").
- **Respawning & Multi Spawns:**
  - Random or round-robin selection among valid multiplayer spawn spots (`MULTI_SPAWN` / sector starts).
  - Temporary spawn invulnerability (3.0 seconds) to prevent instant spawn-camping.

### 4. Cooperative Campaign Mode
- **Shared Keycards:**
  - Blue, Red, and Yellow keycard pickups are shared globally across the cooperative team or mirrored to all active players.
  - When Player 1 collects a Blue Keycard, all players receive access to unlock Blue Key doors.
- **Cooperative Respawn & Checkpoints:**
  - When a player dies in Co-op, they respawn at the start sector or the nearest unlocked checkpoint with default starter equipment without resetting map state or reviving killed monsters.
  - If all players die simultaneously, the level reloads.
- **Configurable Friendly Fire:**
  - Server/match toggle for friendly fire: when disabled, teammate attacks pass through or deal zero damage.
- **Cooperative Level Progression:**
  - Hitting the exit nuke button or level transition switch advances all connected players to the next campaign level in unison, preserving cumulative inventory and stats.

---

## Non-Functional Requirements
- **Performance:** Multi-viewport splitscreen rendering must maintain >= 60 FPS in Bevy 0.14. UDP packet polling must not block the main game thread.
- **Code Standards:**
  - 0 compiler warnings (`cargo check --tests`).
  - 0 dead code annotations (`#![allow(dead_code)]`).
  - 100% test pass rate across the full repository test suite.
  - Adherence to the Windows rule: always use `cmd /c` for shell executions.
- **Modularity:** Integrate cleanly with existing `src/net/`, `src/player/`, `src/combat/`, and `src/hud/` modules.

---

## Acceptance Criteria
1. Splitscreen system dynamically splits viewport horizontally for 2 players and into quadrants for 3-4 players with independent cameras and controls.
2. UDP networking handles packet transmission and reception across client and server without blocking.
3. Player entities render authentic `APLAYER` #1405 billboard sprites with 8-directional view angles and distinct palette color swaps.
4. Dukematch PvP registers hits between players, attributes frags correctly, penalizes suicides, and renders the F7 scoreboard.
5. Co-op mode shares keycards among players, supports non-resetting respawns, and coordinates level transitions.
6. Comprehensive unit tests cover splitscreen viewport math, UDP packet sync, frag scoring, APLAYER angle math, and co-op keycard sharing.
7. Full test suite passes 100% with 0 compiler warnings.
