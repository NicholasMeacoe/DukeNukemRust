# Duke Nukem 3D — Rust / Bevy Source Port (Gold Master)

An authentic, modern Duke Nukem 3D game engine and renderer built in [Rust](https://www.rust-lang.org/) with the [Bevy engine](https://bevyengine.org/) (and [Rapier 3D](https://rapier.rs/) physics).

---

## ⚠️ Important: Proprietary Game Assets

This repository **does not include** the proprietary game data file `duke3d.grp`.

To run the project, you must provide your own copy of `duke3d.grp` from a licensed version of Duke Nukem 3D (such as the Atomic Edition, shareware version, or modern digital releases).

- **Location**: Place `duke3d.grp` in the `dukenukem3d/` directory (i.e. `dukenukem3d/duke3d.grp`).
- **Git Policy**: `duke3d.grp` (and all `*.grp` files) are listed in `.gitignore` and must **not** be committed or uploaded to any public or upstream repository.

---

## 📥 Cloning & Submodules

The original 1996 C/C++ source code is tracked as a Git submodule in `dukenukem3d/`.

### Clone with Submodules:
```bash
git clone --recurse-submodules https://github.com/NicholasMeacoe/DukeNukemRust.git
cd DukeNukemRust
```

### If Already Cloned:
If you cloned without `--recurse-submodules`, initialize and pull the submodule:
```bash
git submodule update --init --recursive
```

---

## 🎮 Features

- **Full Asset Pipeline**: Directly ingests `duke3d.grp`, `PALETTE.DAT`, `LOOKUP.DAT`, `TILES*.ART`, `WAVES.KWV`, and `*.MID` audio files with real-time software MIDI synthesis.
- **Complete 12-Weapon Arsenal**: Mighty Foot, Pistol (with clip reload & casing ejection), Shotgun (7-pellet raycast), Chaingun (spread & brass shower), RPG, Pipebomb (with bouncing arc & remote detonator queue), Shrinker (with boot squish/stomp), Devastator (dual salvos), Laser Tripbomb (wall-mounted laser trigger), Freezethrower (ice shatter), and Expander (organic burst).
- **Comprehensive Bestiary & Boss AI**: All 14 enemy types (Troopers, Pigcops, Octabrains, Enforcers, Drones, Commanders, etc.) + Episode Bosses (Battlelord, Overlord, Cycloid Emperor, Alien Queen) with 3D flight, swimming, situational wake-up states, and death drops.
- **Dynamic CON Scripting Engine**: Bytecode compiler and CON VM supporting `USER.CON` / `GAME.CON` state subroutines, AI actions, and dynamic definitions (`definevolumename`, `defineskillname`, `definelevelname`, `definequote`, `definesound`).
- **Full 4-Episode Campaign Matrix**: 44 maps, par times, secret level routing (e.g. E1L3 -> E1L8 -> E1L4), and intermission statistics rollout.
- **Save / Load Game Snapshot System**: Full binary serialization across 10 save slots with QuickSave (`F6`) and QuickLoad (`F9`).
- **Deterministic Demo Recording & Attract Mode**: Frame-accurate input stream recording and playback with idle attract mode loop.
- **2D/3D Vector Overhead Automap**: Toggleable radar overlay (`Tab`) with unvisited line culling, color-coded walls, and zoom controls.
- **In-Game Developer Console & Cheats**: Dropdown console (`~`) with CVars (`god`, `noclip`, `give`, `map <name>`, `r_crt <0|1>`, `r_scanlines <intensity>`, `r_quantize <0|1>`, `r_stats <0|1>`) and authentic cheat codes (`dnkroz`, `dnstuff`, `dnitems`, `dnclip`, `dnhyper`, `dnrate`, `dnkeys`, `dnweapons`, `dninventory`, `dnshowmap`).
- **Retro CRT Post-Processing**: 32-level distance lighting attenuation, 6-bit VGA DAC conversion, scanlines, shadow mask, and screen damage palette flash tints.
- **High-Precision Slopes & Interactive Platforms**: Sloped sector height interpolation, room-over-room portal classification, and sticky passenger momentum transfer for moving elevators, drop floors, and subway trains.

---

## 🛠️ Building and Running

1. **Prerequisites**: [Rust toolchain](https://www.rust-lang.org/tools/install) (Rust 1.75+ or newer).
2. **Place `duke3d.grp`** inside the `dukenukem3d/` directory.
3. **Build & Run**:
   ```bash
   cargo run --release
   ```
4. **Run Test Suite** (140 automated tests):
   ```bash
   cargo test --bin dukenukemrust
   ```

---

## ⌨️ Controls

| Key / Action | Function |
|---|---|
| **W, A, S, D** | Walk & Strafe |
| **Mouse** | Look around (Mouselook) |
| **Left Click / Ctrl** | Fire Active Weapon |
| **Right Click** | Alt Action / Detonate Pipebomb |
| **Space** | Jump |
| **C** | Crouch |
| **Tab** | Toggle 2D/3D Overhead Automap |
| **~ (Backquote)** | Toggle In-Game Developer Console |
| **F3** | Level Select & Episode Warping Menu |
| **F6 / F9** | QuickSave / QuickLoad |
| **F7** | Dukematch Frag Scoreboard |
| **1 .. 0** | Weapon Selection (0 = Foot, 1 = Pistol, .. 0 = Expander) |
| **J / N / B / M** | Activate Jetpack / Nightvision / Boots / Medkit |
| **E / Space on Wall** | Interact (Switches, Doors, Toilets, Fountains, Mirrors) |
