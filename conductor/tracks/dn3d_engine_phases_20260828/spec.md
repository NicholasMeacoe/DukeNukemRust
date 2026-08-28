# Specification: Duke Nukem 3D Rust Implementation Phases

## Overview
This specification outlines the comprehensive engineering roadmap to develop a complete, authentic, and performant Rust port of Duke Nukem 3D using Bevy 0.14, utilizing original game assets (`duke3d.grp`) and referencing the original C source code (`dukenukem3d/`).

---

## Functional Requirements

### 1. Build Geometry & Portal Compiler (Phase 1)
- **Two-Sided Portal Walls**: Split walls into Upper (ceiling height differences), Lower (floor height differences), and Masked Middle walls (transparent textures / fences / windows).
- **Sector Slopes (`heinum`)**: Calculate vertex Z elevation variations across sectors using `floorheinum` and `ceilingheinum` anchored to the sector's first wall.
- **Authentic Texture Coordinates**: Implement Build's UV projection using `xrepeat`, `yrepeat`, `xpanning`, and `ypanning`.
- **Skybox & Parallax Ceilings**: Handle ceiling parallax flags (`cstat` bit 0) with a cylindrical/panoramic sky mesh.

### 2. Palette Shading & Texture Animation (Phase 2)
- **32-Level Distance Shading**: Parse the 32 shade lookup tables from `PALETTE.DAT` and apply authentic distance-based lighting/fog attenuation.
- **Palette Swaps (`LOOKUP.DAT`)**: Support special color tables (slime, nightvision, freeze tint, multiplayer colors).
- **Texture Animation (`picanm`)**: Read the 4-byte animation metadata per tile in `TILES*.ART` (frames, oscillation, speed) and cycle textures at standard engine tick rates.

### 3. CON Scripting & Actor VM (Phase 3)
- **CON Lexer & Parser**: Parse `DEFS.CON`, `USER.CON`, and `GAME.CON`.
- **Bytecode Compiler & VM**: Implement a runtime VM matching `GAMEDEF.C` to execute actor states, actions, moves, and event hooks (`EVENT_GAME`, `EVENT_JUMP`, etc.).

### 4. Sector Effector & Map Mechanics (Phase 4)
- **Sector Effectors (`SE`)**: Implement core SE types (SE 0 rotating doors, SE 7 underwater warps, SE 15 sliding doors/elevators, SE 25 subways/trains).
- **Sector Triggers & Activators**: Implement touchplates, master switches, and light effectors (strobes, flickers).
- **Exploding & Destructible Geometry**: Support crack walls, glass breaking, and explosive triggers.

### 5. Player Controller, Weapons & Enemy AI (Phase 5)
- **Complete Player States**: Crouching, swimming, diving, air depletion, jetpack, steroids, nightvision, holoduke, shrinking, freezing.
- **Full Weapon Arsenal**: Mighty Boot, Pistol, Shotgun, Chaingun, RPG, Pipebomb (throw & detonate), Shrinker, Devastator, Tripbomb, Freezethrower.
- **Enemy AI State Machine**: Pig Cops, Octabrains, Liztroops, Enforcers, Bosses driven by CON scripts.

### 6. Audio Engine, HUD & Menus (Phase 6)
- **MIDI / Sound System**: Implement MIDI background music playback and RTS (Real-Time Speech) Duke taunts.
- **Status Bar & UI**: Faithful 2D HUD (Health, Armor, Ammo, Inventory, Keys, Weapon icons) using Bevy UI.
- **Episode & Level Progression**: Level transitions, intermission screens, menus, and save/load system.

---

## Acceptance Criteria
- [ ] Maps render with correct portal openings without solid walls blocking doorways.
- [ ] Sloped sectors render with smooth inclined planes.
- [ ] Textures animate according to `picanm` definitions.
- [ ] Doors, elevators, and switches operate authentically in `E1L1.MAP`.
- [ ] CON scripts load and drive enemy AI behaviors and weapons.
- [ ] Game runs smoothly at 60+ FPS with accurate Duke Nukem 3D audio, physics, and gameplay feel.
