# Technology Stack

## Core Language & Runtime
- **Language**: Rust (Edition 2021, 1.75+)
- **Game Engine**: Bevy 0.14 (Entity Component System, 3D rendering, audio, input)
- **Physics**: Bevy Rapier 3D 0.27 (Continuous collision detection, raycasting, character controller)
- **2D Polygon Tessellation**: Lyon Tessellation 1.0 (Sector floor and ceiling polygon triangulation)
- **Audio Processing**: Hound 3.5 (WAV encoding for in-memory KWV playback) + Bevy Audio
- **Random Number Generation**: Rand 0.8

## Reference Codebase
- **Duke Nukem 3D 1.3D/Atomic C Source**: Located at `dukenukem3d/` (submodule)
  - `ENGINE.C`, `BUILD.C`: Build engine rendering, math, and sector data structures
  - `SECTOR.C`: Sector effector, trigger, door, and switch mechanisms
  - `GAMEDEF.C`: CON script parser, compiler, and VM runtime
  - `ACTORS.C`: Enemy AI states and combat interactions
  - `PLAYER.C`: Player movement, inventory, and weapon handling

## Target Platforms
- **Primary**: Windows (x86_64), Linux (x86_64), macOS (Apple Silicon / Intel)
