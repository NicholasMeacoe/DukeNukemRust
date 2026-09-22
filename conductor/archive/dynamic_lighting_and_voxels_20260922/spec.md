# Specification: Dynamic Lighting & Ken Silverman KVX Voxel Model Support

## Overview
This track introduces real-time dynamic point lighting and authentic Ken Silverman Build engine `.KVX` voxel model rendering to Duke Nukem 3D Rust. Gunfire, rocket trails, explosions, and strobe sectors will dynamically illuminate dark retro corridors, while pickups, weapons, and key items will gain 3D depth through surface-culled KVX voxel meshing and rotation animations.

---

## Functional Requirements

### 1. Dynamic Point Lighting Engine
- **Weapon Muzzle Flashes**: Firing weapons (Pistol, Shotgun, Chaingun, RPG, Devastator) spawns a localized point light at the muzzle position with high initial intensity and fast decay (0.05s - 0.15s).
- **Projectile Trails**: Active projectile entities (RPG rockets, shrinker energy spheres, Liztroop fire lasers) carry dynamic point lights matching their projectile tint (red/orange, green, purple).
- **Explosions**: Explosions spawn large-radius, high-intensity flashlights (10-15m radius) that illuminate surrounding walls, floors, and ceilings before rapidly fading out.
- **Sector Effectors**: Strobe and flicker sector effectors pulse dynamic point lights within their bounds.
- **Light Limiter & Pool**: Cap maximum simultaneous active dynamic lights (e.g., 32 lights) to maintain stable frame rates on all hardware, culling the oldest/dimmest lights when capacity is reached.
- **Console / CVar Integration**: `r_dynamiclights <0|1>` in `ConsoleState` to toggle dynamic lights on/off at runtime.

### 2. Ken Silverman KVX Voxel Binary Parser
- **Binary Parsing**: Pure-Rust parser in `src/voxel/kvx.rs` conforming to Ken Silverman's Build KVX specification:
  - Header: `xsiz`, `ysiz`, `zsiz`, `xpivot`, `ypivot`, `zpivot`.
  - Column and slab offsets table.
  - Slab decompression: `ztop`, `zend`, and palette color index byte arrays.
- **Robustness**: Graceful error handling (`KvxError`) with zero panics on truncated or malformed voxel data.
- **Voxel Volume Data**: Fast 3D grid voxel color lookup `model.get_voxel(x, y, z) -> Option<u8>`.

### 3. Voxel Mesh Generation & Palette Shading
- **Surface Face Culling**: Extract only visible exterior faces (adjacent to empty space or model bounds), omitting all interior/hidden voxel faces to produce lightweight Bevy meshes.
- **Quad Generation**: Generate 3D quad vertices, normals, and UVs/vertex colors for visible faces.
- **Palette Mapping**: Map 8-bit voxel palette indices through `PALETTE.DAT` into RGB vertex colors.
- **Pivot Centering**: Shift mesh vertices by the KVX model pivot so voxel items stand upright and rotate naturally on sector floors.

### 4. Sprite Voxel Replacement Registry
- **Registry Resource**: `VoxelRegistry` resource mapping tile picnums (e.g. `FIRSTAID`, `ATOMICHEALTH`, `ARMOR`, `SHOTGUNBOX`, `KEYCARD`, weapon pickups) to loaded `KvxModel` assets.
- **Pickup Spawning**: When spawning items in `src/builder.rs` or `src/map.rs`, if a voxel model is registered for the tile picnum, spawn the 3D voxel mesh.
- **Idle Animation**: Pickups render with continuous Y-axis rotation and subtle vertical floating bob.
- **Console / CVar Integration**: `r_voxels <0|1>` in `ConsoleState` to toggle voxel model replacement on/off.

---

## Non-Functional Requirements
- Zero compiler warnings on `cargo check --tests`.
- 100% test pass rate across the full test suite.
- Zero `#![allow(dead_code)]`.
- Maintain smooth 60+ FPS rendering performance.

---

## Out of Scope
- Skeletal / MD2 / MD3 character model replacements for Duke and enemies (reserved for future track).
- Hardware raytraced global illumination or dynamic shadow mapping on all dynamic lights.
