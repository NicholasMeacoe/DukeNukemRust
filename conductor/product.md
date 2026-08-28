# Product Definition

## Vision
To develop a faithful, high-performance modern port and renderer of **Duke Nukem 3D** written in **Rust**, leveraging the **Bevy Engine** (ECS, modern graphics, 3D rendering pipeline) alongside accurate Build engine data format parsing, sector physics, CON scripting, and authentic retro gameplay mechanics.

## Core Features
1. **Asset & File Extraction**: Native loading of original game data (`.GRP`, `TILES*.ART`, `PALETTE.DAT`, `LOOKUP.DAT`, `WAVES.KWV`, `.MAP`, `.RTS`) directly without external conversion.
2. **Build Engine Geometry Compiler**: Precise sector-to-3D mesh generation supporting two-sided portal walls (upper, lower, masked), sloped floors/ceilings (`heinum`), authentic texture mapping (repeats/panning), and parallax skies.
3. **Palette & Visual Aesthetics**: Classic 32-level distance shade attenuation, palette swaps, and animated tile cycles (`picanm`).
4. **Sector Effector & Interaction Engine**: Complete interactive map features (doors, switches, elevators, lighting FX, exploding walls, subway trains).
5. **CON Scripting & Actor VM**: Bytecode execution of `GAME.CON`, `USER.CON`, and `DEFS.CON` for authentic enemy AI, weapons, pickups, and damage mechanics.
6. **Combat, Audio & UI**: Responsive player controller, full weapon arsenal (Pistol, Shotgun, Chaingun, RPG, Pipebombs, Shrinker, Devastator, Freeze), KWV sound effects, MIDI music, and classic status HUD.
