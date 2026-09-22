# Specification: Environmental & Interactive 3D Voxel Props Expansion

## Overview
This track expands the 3D Ken Silverman KVX voxel model catalog from item pickups to Duke Nukem 3D's interactive and environmental props. Explosive barrels, radioactive drums, fire extinguishers, security cameras, ceiling fans, and drinking water fountains will be replaced with rich, surface-culled 3D voxel models featuring authentic kinetic animations (spinning ceiling fans, sweeping surveillance cameras) and responsive destruction state transitions.

---

## Functional Requirements

### 1. Environmental Prop Voxel Models
- **Explosive & Radioactive Barrels**:
  - `EXPLODINGBARREL` (Tile 1238) & `EXPLODINGBARREL2` (Tile 1239): Cylindrical 16x16x24 red/black explosive barrels with hazard stripes and warning markings.
  - `FIREBARREL` (Tile 1240): Steel burn barrel.
  - `NUKEBARREL` (Tile 1227), `NUKEBARRELDENTED` (Tile 1228), `NUKEBARRELLEAKED` (Tile 1229): Green radioactive waste drums with yellow trefoil radiation emblems and toxic sludge.
- **Fire Extinguisher (`FIREEXT` / Tile 916)**:
  - Cylindrical red fire extinguisher canister with brass valve, black pressure gauge, wall-mount bracket, and directional hose nozzle.
- **Security Camera (`CAMERA1` / Tile 500)**:
  - Ceiling/wall surveillance camera housing with mounting bracket, motorized swivel pivot, camera lens, and glowing red status LED.
- **Ceiling Fan (Tile 617)**:
  - Industrial 4-blade ceiling fan with central motor casing and downrod mount.
- **Drinking Water Fountain (`WATERFOUNTAIN` / Tile 564, 565)**:
  - Stainless steel wall-mounted drinking fountain with contoured basin, bubbler spigot, and side push button.

### 2. Kinetic & Motorized Prop Systems
- **Ceiling Fan Motor System**:
  - Automatically spins fan blades continuously around the vertical Y-axis at configurable rpm (`FanConfig` / `CeilingFanVoxel`).
- **Security Camera Surveillance Sweep**:
  - Horizontally oscillates camera yaw back and forth across its surveillance arc matching `SecurityCamera` tag and sweep angle.

### 3. Wall, Floor & Ceiling Alignment
- **Wall Mounting**: Wall-aligned props (`FIREEXT`, `CAMERA1`, `WATERFOUNTAIN`) orient outward along the wall normal angle (`sprite.ang`) and offset from the wall surface to prevent clipping.
- **Floor Grounding**: Barrels stand upright on sector floors (`pos.y` aligned with bottom of voxel mesh).
- **Ceiling Anchoring**: Ceiling fans anchor to the sector ceiling elevation.

### 4. Destruction & State Transitions
- **Explosive & Destructible Props**:
  - When barrels or fire extinguishers take lethal damage:
    - Trigger explosion event, radius damage, and dynamic point light flash (`SpawnDynamicLightEvent`).
    - Despawn intact 3D voxel entity and spawn destroyed debris / shattered remnants.
- **Water Fountain Depletion & Destruction**:
  - When water fountain reaches 0 uses or takes damage, transition to broken prop.

### 5. Console & CVar Integration
- All environmental voxel models respect `r_voxels <0|1>` console cvar, hiding or reverting to 2D billboards if disabled.

---

## Non-Functional Requirements
- Zero compiler warnings on `cargo check --tests`.
- 100% test pass rate across the full test suite.
- Zero `#![allow(dead_code)]`.
- Maintain smooth 60+ FPS rendering performance.

---

## Out of Scope
- Full 3D animated character/enemy skeletal meshes (Pig Cop, Liztroop, Duke).
- Fully destructible level geometry voxels (e.g. voxel terrain excavation).
