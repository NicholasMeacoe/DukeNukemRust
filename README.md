# Duke Nukem 3D - Rust / Bevy Renderer

A Duke Nukem 3D Build Engine map and asset renderer built with [Rust](https://www.rust-lang.org/) and the [Bevy engine](https://bevyengine.org/) (with [Rapier 3D](https://rapier.rs/) physics).

## ⚠️ Important: Proprietary Game Assets

This repository **does not include** the proprietary game data file `duke3d.grp`.

To run the project, you must provide your own copy of `duke3d.grp` from a licensed version of Duke Nukem 3D (such as the Atomic Edition, shareware version, or modern digital releases).

- **Location**: Place `duke3d.grp` in the `dukenukem3d/` directory (i.e. `dukenukem3d/duke3d.grp`).
- **Git Policy**: `duke3d.grp` (and all `*.grp` files) are listed in `.gitignore` and must **not** be committed or uploaded to any public or upstream repository.

---

## Features

- **GRP Archive Parsing**: Directly reads game assets, textures, sounds, and maps from `duke3d.grp`.
- **Palette & ART Texture Decoding**: Decodes `PALETTE.DAT` and `TILES*.ART` texture sheets with nearest-neighbor pixel sampling.
- **Build Map Geometry**: Parses `.MAP` files (`E1L1.MAP`), tessellates sector floor/ceiling polygons using `lyon_tessellation`, and constructs 3D wall meshes.
- **Player Movement & Physics**: First-person controller with mouselook, movement, and collisions powered by `bevy_rapier3d`.
- **Billboard Sprites & Weapons**: Renders camera-facing 2D sprites and animated HUD weapons.
- **Audio Support**: Reads VOC / WAV audio files from `WAVES.KWV`.

---

## Prerequisites

- [Rust toolchain](https://www.rust-lang.org/tools/install) (edition 2021, Rust 1.75+)
- Operating system dependencies required by Bevy (e.g. Vulkan / DirectX 12 graphics drivers).

---

## Building and Running

1. Clone this repository:
   ```bash
   git clone <repository-url>
   cd DukeNukemRust
   ```

2. Copy your `duke3d.grp` into the `dukenukem3d/` folder:
   ```text
   DukeNukemRust/
   ├── dukenukem3d/
   │   └── duke3d.grp
   ├── src/
   ├── Cargo.toml
   └── README.md
   ```

3. Build and launch:
   ```bash
   cargo run --release
   ```

### Controls

- **WASD / Move**: Walk around the map
- **Mouse**: Look around
- **Left Click**: Grab / release cursor
- **E**: Play a random Duke sound
- **Space**: Jump
