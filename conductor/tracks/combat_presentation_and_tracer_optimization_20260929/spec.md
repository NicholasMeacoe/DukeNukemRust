# Track Specification: Combat Presentation and Tracer Asset Optimization

## 1. Overview
This track addresses combat presentation inaccuracies and high-frequency asset allocation bottlenecks identified during Principal Engineer review. Specifically, it corrects the RPG firing viewmodel frame, caches bullet tracer meshes and materials in a shared resource to prevent CPU/GPU allocation churn, aligns hitscan hit registration with authentic Duke 3D instantaneous response, and prevents point-blank muzzle clipping through walls.

## 2. Functional Requirements
- **FR-1: RPG Firing Animation Authenticity**: Correct `WeaponType::Rpg` in `src/player/weapons.rs` to display tile `2606` (`RPG+1`, rocket blast recoil) instead of tile `2544` (Chaingun) during the firing cycle.
- **FR-2: Bullet Tracer Asset Caching**: Create a `BulletTracerAssets` resource containing pre-instantiated `Handle<Mesh>` and `Handle<StandardMaterial>` for bullet tracers. In `spawn_projectiles`, clone these handles instead of calling `Assets::add()` on every bullet and shotgun pellet.
- **FR-3: Instantaneous Hitscan Hit Registration**: Ensure hitscan weapons (Pistol, Shotgun, Chaingun) register raycast hits immediately upon firing so damage applies at trigger pull, while visible tracers quickly traverse the line of sight for visual readability.
- **FR-4: Muzzle Wall-Standoff Safety**: Ensure muzzle flash dynamic lights and bullet spawn origins check player collision bounds or use a safe standoff ($\le 0.25\,\text{m}$) so point-blank firing against walls does not spawn lights or projectiles inside or behind geometry.

## 3. Non-Functional Requirements
- Maintain 0 memory/descriptor allocations per fired projectile in the hot combat loop.
- Zero compiler warnings.
- 100% test pass rate.

## 4. Acceptance Criteria
1. Firing the RPG displays the rocket blast tube frame (tile `2606`) rather than the chaingun barrel.
2. Firing 100 shotgun blasts or continuous chaingun fire results in 0 additional `Mesh` or `StandardMaterial` allocations in `Assets`.
3. Hitscan weapons deal instantaneous damage to crosshair targets at long distance without travel lag.
4. Firing while pressed point-blank against a wall does not cause muzzle flashes to appear on the other side of the wall.
