# Implementation Plan: Duke Nukem 3D Rust Engine Phases

## Phase 1: Build Map Geometry & Portal Compiler (Fix Geometry Pipeline)
- [x] Task: Two-Sided Portal Wall Segmentation
  - [x] Update `src/main.rs` and `src/map.rs` to distinguish one-sided (white) vs two-sided (red) walls (`nextsector != -1`)
  - [x] Implement Upper Wall generation (filling height gap between current ceiling and adjacent sector ceiling)
  - [x] Implement Lower Wall generation (filling height gap between current floor and adjacent sector floor)
  - [x] Implement Masked Middle Wall rendering for transparent/grate/window walls
- [x] Task: Sloped Floor & Ceiling Tessellation
  - [x] Parse and interpret `floorheinum` and `ceilingheinum` along the slope first-wall pivot
  - [x] Compute 3D vertex Z coordinates for triangulated sector polygons
  - [x] Generate matching physics colliders for sloped floors and ceilings
- [x] Task: Authentic Build UV Mapping & Texture Coordinates
  - [x] Implement Build engine UV formulas using `xrepeat`, `yrepeat`, `xpanning`, and `ypanning`
  - [x] Account for tile width and height scaling on wall quads and sector planes
- [x] Task: Skybox & Parallax Ceiling Rendering
  - [x] Detect parallax ceiling bit flag in `sector.ceilingstat`
  - [x] Spawn parallax sky dome / cylinder background with scrolling camera orientation
- [x] Task: Phase 1 Verification & Checkpoint
  - [x] Verify `E1L1.MAP` renders with open doorways, sloped cinema stairs/ramps, aligned textures, and open sky

---

## Phase 2: Palette Shading, Lookup Tables & Tile Animations
- [x] Task: Palette Shading Tables & Distance Attenuation
  - [x] Parse 32 shade tables from `PALETTE.DAT`
  - [x] Implement custom Bevy shader / material uniform to apply authentic 32-level distance lighting falloff
  - [x] Apply sector and wall shade modifiers (`ceilingshade`, `floorshade`, `wall.shade`, `sprite.shade`)
- [x] Task: Palette Swaps & Lookup Tables
  - [x] Parse `LOOKUP.DAT` for alternate color palettes
  - [x] Support sprite and sector palette remapping (e.g. green slime, red blood, player tints)
- [x] Task: Tile Animation System (`picanm`)
  - [x] Parse `picanm` 4-byte headers from `TILES*.ART` (frames, animation type, speed, oscillation)
  - [x] Implement Bevy system to tick and swap active texture handles for animated tiles
- [x] Task: Phase 2 Verification & Checkpoint
  - [x] Verify authentic retro lighting, glowing lamps, animated screens, and water/slime textures

---

## Phase 3: CON Scripting Parser, Compiler & Virtual Machine
- [x] Task: CON Script Lexer & AST Parser
  - [x] Create token lexer for `DEFS.CON`, `USER.CON`, and `GAME.CON`
  - [x] Parse defines, actions, moves, ai routines, state blocks, and actor declarations
- [x] Task: Bytecode Compiler & Runtime VM (`GAMEDEF.C` Equivalent)
  - [x] Compile CON AST into efficient runtime bytecode instructions
  - [x] Implement actor VM executor (executing per-actor CON state loops on engine ticks)
  - [x] Expose engine primitives to VM (distance checks, line of sight, projectile spawning, sound triggers)
- [x] Task: Event Hooks & Game Definitions
  - [x] Implement engine event handlers (`EVENT_GAME`, `EVENT_JUMP`, `EVENT_PREWORLDDRAW`, etc.)
  - [x] Bind weapon definitions, ammo limits, and damage lookup tables from `USER.CON`
- [x] Task: Phase 3 Verification & Checkpoint
  - [x] Verify CON scripts compile cleanly from GRP and execute basic actor state loops

---

## Phase 4: Sector Effector & Interactive Map Mechanics
- [x] Task: Sector Effector (`SE`) & Sector Trigger (`ST`) Interpreter
  - [x] Port core `SECTOR.C` effector dispatchers (SE 0 rotating doors, SE 15 sliding doors, SE 7 underwater)
  - [x] Support Sector Triggers, Touchplates, Activators, and Master Switches
- [x] Task: Moving Sectors, Elevators & Subways
  - [x] Implement smooth sector floor/ceiling interpolation for lifts and doors
  - [x] Implement moving sector physics colliders with Rapier 3D
  - [x] Implement subway / train movement (SE 25)
- [x] Task: Destructible & Interactive Environments
  - [x] Implement crack wall explosions and sector transformation triggers
  - [x] Implement breakable glass, light switches, and interactive toilets/water fountains
- [x] Task: Phase 4 Verification & Checkpoint
  - [x] Verify fully interactive `E1L1` map (doors open with use key/switches, cinema screen explodes, elevators work)

---

## Phase 5: Complete Player Controller, Weapons & Enemy AI
- [ ] Task: Advanced Player State Machine
  - [ ] Implement crouching, swimming, diving, air depletion/suffocation
  - [ ] Implement inventory items: Jetpack, Steroids, Nightvision, Holoduke, Scuba Gear, Medkit
  - [ ] Implement status effects: Shrink/Squish, Freeze/Shatter
- [ ] Task: Full 10-Weapon Arsenal
  - [ ] Implement Mighty Boot (melee kick)
  - [ ] Implement Shotgun, Chaingun, and RPG (with projectile rocket physics)
  - [ ] Implement Pipebombs (throw, physics bounce, remote detonator)
  - [ ] Implement Shrinker, Devastator, Tripbombs (wall placement & laser trigger), Freezethrower
- [ ] Task: Enemy AI Behaviors & Combat
  - [ ] Connect CON actor VM to enemy entities (Pig Cops, Octabrains, Liztroops, Enforcers)
  - [ ] Implement enemy pathfinding, sound detection, line-of-sight attacks, and death/gib animations
- [ ] Task: Phase 5 Verification & Checkpoint
  - [ ] Verify complete combat loop in `E1L1` with all weapons, inventory items, and responsive enemies

---

## Phase 6: Audio Engine, HUD & Menus
- [ ] Task: MIDI Music & Audio Synthesizer
  - [ ] Integrate MIDI player (e.g. `midir` / soundfont synthesizer) for authentic level background music (`*.MID`)
  - [ ] Implement Duke RTS (Real-Time Speech) trigger system for voice taunts
- [ ] Task: Classic 2D Status Bar & Full-Screen HUD
  - [ ] Render 2D status bar displaying Health, Armor, Ammo, Keys, and Selected Inventory item
  - [ ] Implement custom Duke font renderer from `TILES*.ART` for HUD text and messages
- [ ] Task: Game Flow, Menus & Level Progression
  - [ ] Implement main menu, difficulty selection, episode selector, and options screen
  - [ ] Implement end-of-level exit switch, statistics/intermission screen, and level loading transitions
  - [ ] Implement save game and load game serialization
- [ ] Task: Phase 6 Verification & Checkpoint
  - [ ] Play through the entire Episode 1 (L1 through L6) from menu start to boss completion
