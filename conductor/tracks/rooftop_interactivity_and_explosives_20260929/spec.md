# Track Specification: Rooftop Interactivity and Explosives Integrity

## 1. Overview
During testing on the E1L1 cinema rooftop, bullet hits against explosive gas canisters (`SEENINE`, tile 1247) were unreliable, and blowing up canisters adjacent to the rooftop ventilation grate (`FANSPRITE`, tile 407 / `PANNEL1`, tile 342) left an invisible solid collider blocking Duke from entering the ventilation duct. Furthermore, shooting explosive barrels triggered dual explosion damage events. This track resolves these collision geometry and event handling deficiencies.

## 2. Functional Requirements
- **FR-1: SEENINE Canister Hitbox Alignment**: 2D sprite barrels and gas canisters must have their rigid body colliders centered in the upper-half space above the floor ($Y \in [0.0, 0.8]\,\text{m}$) rather than half-submerged into the floor ($Y \in [-0.4, 0.4]\,\text{m}$), enabling bullet raycasts at standing player eye-line ($Y \approx 1.3\,\text{m}$) to reliably hit the canister.
- **FR-2: Floor Vent Collider Orientation**: Floor-aligned destructible grates (`is_floor_aligned`) must define their thin axis along local Z (`Collider::cuboid(scale_x / 2.0, scale_y / 2.0, 0.05)`) so Rapier properly rotates the collider flat onto the roof surface rather than creating an upright invisible wall.
- **FR-3: Vent Grate Despawn on AoE Explosions**: When explosive radius damage in `handle_explosions` reduces `CrackWall` health to 0, the system must trigger `GibEvent` and call `commands.entity(entity).despawn_recursive()`, completely removing the physical collider and opening the ductway into the cinema.
- **FR-4: Single-Event Barrel Explosion Pipeline**: Remove duplicate `ExplosionDamageEvent` in `src/interactivity/wall_damage.rs` when a barrel explodes, delegating damage dispatch uniformly through `BarrelExplodeEvent` and `handle_barrel_chain_explosions` to prevent double damage (200 HP instead of 100 HP).

## 3. Non-Functional Requirements
- Maintain 100% test pass rate across the entire test suite.
- Zero compiler warnings (`cargo check --tests`).
- Attacker attribution (`attacker_id`) must be preserved across direct hits and chain reactions for multiplayer PvP scoring.

## 4. Acceptance Criteria
1. Firing a pistol bullet at the top, middle, or base of a `SEENINE` canister detonates it immediately.
2. Detonating a `SEENINE` canister next to the rooftop air vent shatters the vent with sound (`VENT_BUST`), spawns debris gibs, and completely removes the collider so Duke can walk over and drop down the shaft.
3. Barrel explosions produce exactly one explosion damage sphere (100 HP max damage).
