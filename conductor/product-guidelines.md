# Product Guidelines

## Design Principles
1. **Faithfulness to Original Gameplay**: Game feel, weapon timings, damage values, and actor behaviors must match original 1996 MS-DOS Duke Nukem 3D behavior (guided by the reference C codebase in `dukenukem3d/`).
2. **Modern Architecture**: Clean Rust idioms, Bevy ECS systems, type safety, deterministic state management, and zero memory corruption bugs.
3. **Data-Driven Architecture**: Respect the original game data files and CON scripts rather than hardcoding game values in Rust.
4. **Performance**: Efficient mesh tessellation, zero runtime allocations during gameplay loops, and smooth 60+ FPS rendering on modern graphics hardware.
