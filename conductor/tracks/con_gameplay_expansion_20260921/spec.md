# Specification: Authentic CON Scripting & Full Enemy Roster Expansion

## 1. Overview & Objective
This track transitions Duke Nukem 3D Rust from using a hardcoded 4-enemy fallback script to loading and executing authentic `GAME.CON` (with its included `DEFS.CON` and `USER.CON`) directly from `duke3d.grp`. It expands actor definitions, sprite animation mappings, and projectile dispatching to support the complete 20+ enemy roster, ambient environmental actors, and authentic episode boss encounters.

---

## 2. Functional Requirements

### 2.1 Authentic CON Script Pipeline
- **Default Loading**: `ConScriptEngine::from_grp` must automatically load and compile `GAME.CON` from `duke3d.grp` on startup.
- **Include Resolution**: Recursively resolve and parse `#include DEFS.CON` and `#include USER.CON` from GRP bytes.
- **Directive Completeness**: Ensure the lexer and AST compiler support all standard CON syntax structures:
  - `define`, `definesound`, `definelevelname`, `definevolumename`, `defineskillname`, `definequote`
  - `action`, `move`, `ai`, `state`, `ends`, `useractor`, `eventcode`
  - All standard flow control (`ifcansee`, `ifpdistl`, `ifhitweapon`, `ifdead`, `ifsquished`, `ifwasweapon`, `else`, `break`, `enda`)
- **Fallback Safety**: If `duke3d.grp` is not present, gracefully fall back to an expanded `DEFAULT_CORE_CON_SCRIPT` containing definitions for all standard monsters and bosses.

### 2.2 Complete 20+ Enemy Roster Coverage
- Expand `ConActor` spawning, AI update loops, and sound definitions for:
  - **Troopers**: `LIZTROOP` (1680), `LIZTROOPJETPACK` (1705), `LIZTROOPONTOILET` (1725), `LIZTROOPJUSTSIT` (1730)
  - **Captains / Lizman**: `LIZMAN` (1800), `LIZMANSPITTING` (1805), `LIZMANFEEDING` (1810), `LIZMANJUMP` (1815)
  - **Pig Cops**: `PIGCOP` (2000), `PIGCOPSTAYPUT` (2050), `PIGCOPDRIVE` (2060)
  - **Octabrains**: `OCTABRAIN` (1820), `OCTABRAINSTAYPUT` (1850)
  - **Enforcers**: `ENFORCER` (2120), `ENFORCERSTAYPUT` (2150), `ENFORCERJUMP` (2160)
  - **Recon Patrol Vehicle (RPV)**: `RECON` (1960)
  - **Sentry Drones**: `DRONE` (1880)
  - **Assault Commanders**: `COMMANDER` (1920), `COMMANDERSTAYPUT` (1950)
  - **Protozoid Slimers & Eggs**: `SLIMER` (2370), `EGG` (2360)
  - **Sharks**: `SHARK` (2400)
  - **Ceiling / Floor Turrets**: `ROTATEGUN` (1760)
  - **Bosses**: `BOSS1` Battlelord (2630), `BOSS2` Overlord (2710), `BOSS3` Cycloid Emperor (2760), Mini-Battlelord (2660)

### 2.3 Projectile, Damage & Sound Mapping
- Update `map_tile_to_projectile` to produce authentic velocities, projectile components, and damage for:
  - `FIRELASER` (Trooper blaster)
  - `SPIT` (Octabrain energy blast)
  - `COOLLABS_SHOTGUN` / `SHOTGUN` (Pig cop shotgun pellets)
  - `CHAINGUN` (Enforcer laser vulcan)
  - `RPVMISSILE` (Recon vehicle homing rockets)
  - `COMMANDERBLASTER` (Commander rocket barrage)
  - `BOULDER` / `MORTAR` (Battlelord heavy lobbed artillery)
  - `SHRINKSPARK` & `FREEZEBLAST`
- Ensure death drops spawn authentic pickups (Pigcop -> Shotgun/Armor, Boss -> Atomic Health).

### 2.4 Boss Encounter & Climax Mechanics
- **Battlelord (`BOSS1`)**:
  - Twin minigun bursts + mortar artillery lobbing.
  - Footstep screen shake when close to player.
  - On death in boss level (e.g. `E1L7`), trigger `endofgame` / level completion event and victory quote.
- **Overlord (`BOSS2`)**:
  - Rocket launcher pods and leaping bounds.
- **Cycloid Emperor (`BOSS3`)**:
  - Triple-rocket arm and psychic energy blast.

---

## 3. Acceptance Criteria
1. `ConScriptEngine::from_grp` successfully compiles real `GAME.CON` from `duke3d.grp` without errors, producing >20 actor script pointers and >1,000 instructions.
2. All 20+ enemy types have active AI state machines and spawn in their designated maps (verified in tests and map loading).
3. Boss encounters execute combat patterns and properly trigger level completion upon defeat.
4. `cargo check --tests` produces zero warnings.
5. All automated unit and integration tests pass (231+ tests).
