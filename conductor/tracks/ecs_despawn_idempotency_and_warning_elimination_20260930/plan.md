# Implementation Plan: ECS Despawn Idempotency and Warning Elimination

## Phase 1: SafeDespawnExt Infrastructure & Unit Tests
- [x] Task 1.1: TDD - Unit test for `safe_despawn_recursive` idempotency
  - [x] Write unit test asserting multiple `safe_despawn_recursive` calls on the same entity execute cleanly with 0 warnings or panics
- [x] Task 1.2: Implement `SafeDespawnExt` in `src/interactivity/types.rs`
  - [x] Define `pub trait SafeDespawnExt { fn safe_despawn_recursive(&mut self, entity: Entity); }`
  - [x] Implement for `Commands<'w, 's>` using `world.get_entity_mut`
  - [x] Re-export in `src/interactivity/mod.rs`
- [x] Task 1.3: Phase Verification & Checkpoint
  - [x] Run `cargo test interactivity`

## Phase 2: System Hardening & Batch Deduplication
- [x] Task 2.1: Implement batch deduplication and safe despawn in `src/interactivity/wall_damage.rs`
  - [x] Add `despawned_entities: HashSet<Entity>` in `handle_wall_damage`
  - [x] Guard `destructible_query` with health check and deduplication set
  - [x] Switch to `commands.safe_despawn_recursive`
- [x] Task 2.2: Implement batch deduplication and safe despawn in `src/interactivity/props.rs`
  - [x] Add `despawned_entities: HashSet<Entity>` in `handle_explosions` and `interact_with_props`
  - [x] Switch to `commands.safe_despawn_recursive`
- [x] Task 2.3: Implement safe light eviction and despawn in `src/lighting.rs`
  - [x] Track `evicted_entities: HashSet<Entity>` in `spawn_dynamic_lights`
  - [x] Filter out `l.progress() >= 1.0` and already-evicted lights
  - [x] Switch to `commands.safe_despawn_recursive`
- [x] Task 2.4: Update remaining systems (`level_loader.rs`, `inventory.rs`, `projectiles.rs`, `weapons.rs`)
  - [x] Use `commands.safe_despawn_recursive` in `level_loader.rs`, `inventory.rs`, `projectiles.rs`, `weapons.rs`
- [x] Task 2.5: Phase Verification & Checkpoint
  - [x] Run full test suite via `cargo test`
  - [x] Confirm clean compilation with 0 warnings
