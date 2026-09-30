# Track Specification: ECS Despawn Idempotency and Warning Elimination

## 1. Overview
During gameplay playtesting, multiple Bevy ECS runtime warnings were logged:
```
WARN bevy_ecs::world: error[B0003]: Could not despawn entity Entity { index: 2670, generation: 4 } because it doesn't exist in this World.
WARN bevy_ecs::world: error[B0003]: Could not despawn entity Entity { index: 2008, generation: 1 } because it doesn't exist in this World. (x5)
WARN bevy_ecs::world: error[B0003]: Could not despawn entity Entity { index: 2225, generation: 1 } because it doesn't exist in this World. (x5)
```
These warnings occur when:
1. Multi-pellet hitscan attacks (e.g. shotgun firing 7 pellets simultaneously) hit destructible walls, vents, barrels, or props in the same frame. Because command buffers are deferred, each pellet hitting the entity queues a separate `commands.entity(entity).despawn_recursive()` command, resulting in 1 successful despawn followed by 5 duplicate despawn errors.
2. Dynamic point lights reach capacity or expire simultaneously across `spawn_dynamic_lights` and `update_transient_lights`.
3. Simultaneous explosion events or co-op pickup interactions attempt to despawn the same entity multiple times in a single frame.

## 2. Functional Requirements
- **FR-1: Idempotent Safe Despawn Command Extension (`SafeDespawnExt`)**:
  - Implement `SafeDespawnExt` extension trait for `Commands` with `safe_despawn_recursive(&mut self, entity: Entity)`.
  - When the deferred command executes on `&mut World`, check `world.get_entity_mut(entity)`. If the entity still exists, despawn it recursively; if it has already been despawned by an earlier command, silently no-op without emitting warning `[B0003]`.
- **FR-2: Despawn Deduplication in `handle_wall_damage`**:
  - In `src/interactivity/wall_damage.rs`, maintain a frame-local `HashSet<Entity>` tracking entities queued for despawn during the event batch.
  - Check `despawned_entities` before processing glass, crack walls, fire extinguishers, barrels, and generic destructibles.
  - Guard Section 8 generic destructibles with `if dest.health > 0` so zero/negative health entities are never re-damaged and re-despawned by subsequent pellets in the batch.
- **FR-3: Despawn Deduplication in `handle_explosions`**:
  - In `src/interactivity/props.rs`, track entities already queued for despawn across multiple explosion events in the frame.
- **FR-4: Safe Light Eviction in `spawn_dynamic_lights`**:
  - In `src/lighting.rs`, track lights already evicted in the current frame batch and filter out expired lights (`progress < 1.0`).
- **FR-5: Safe Despawn in Level Teardown & Item Pickups**:
  - Update `src/game_flow/level_loader.rs` and `src/player/inventory.rs` to use safe despawns.

## 3. Non-Functional Requirements
- Zero `[B0003]` warnings in console logs during intense combat, shotgun firing, or explosive chain reactions.
- Zero compiler warnings.
- 100% test pass rate.

## 4. Acceptance Criteria
1. Firing a shotgun blast into destructible objects and exploding barrels produces 0 duplicate despawn warnings.
2. Multiple concurrent explosion events or light spawning do not trigger `error[B0003]`.
3. Unit test verifying idempotent despawning passes.
