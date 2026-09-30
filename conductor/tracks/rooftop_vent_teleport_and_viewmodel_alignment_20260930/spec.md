# Track Specification: Rooftop Ventilation Shaft Teleport (SE 7) and Viewmodel Alignment

## 1. Overview
This track addresses two critical gameplay and visual issues reported during playtesting of Episode 1 Level 1 (Hollywood Countdown):
1. **Viewmodel Sizing & Reticle Alignment**: The first-person weapon viewmodel was excessively large (~58% screen height) and misaligned horizontally relative to the targeting crosshair reticle. The viewmodel must be scaled to authentic Duke 3D proportions (~38% screen height) and horizontally offset so the weapon's iron sights and barrel sit dead-center directly beneath the crosshair.
2. **Rooftop Ventilation Shaft Teleport (Sector Effector 7)**: Blowing up the rooftop gas canisters and dropping into the ventilation shaft (Sector 269) caused Duke to plummet 200,000 units to the floor of an unlinked dummy shaft, taking 56 fall damage and landing in an empty pit with skybox textures (`dn3d_screen_20260930.JPG`). In authentic Duke 3D, Sector 269 contains Sector Effector 7 (Sprite #554, hitag 252) which seamlessly teleports the player to Sector 256 (Sprite #86, hitag 252) overlooking the cinema alleyway with safe vertical exit velocity and correct yaw.

## 2. Functional Requirements
- **FR-1: SE 7 Teleporter Pairing & Registration**:
  - Scan all Sector Effector sprites (`picnum == 1 && lotag == 7`) during map loading.
  - Pair sprites by matching `hitag` (e.g. Sprite #554 in Sector 269 and Sprite #86 in Sector 256 both have `hitag = 252`).
  - Configure mutual teleporter properties: `target_sector`, `target_pos`, `target_yaw`, `trigger_height`, and `teleport_cooldown`.
- **FR-2: Seamless Teleportation Execution**:
  - Implement system `update_teleporter_sector_effectors` in `src/interactivity/effectors.rs`.
  - When the player enters the teleporter sector and drops/moves past the trigger threshold:
    - Update player translation to `target_pos`.
    - Set player yaw to `target_yaw`.
    - Update `CurrentSector` to `target_sector`.
    - Reset downward fall velocity to safe exit speed (`velocity_y = -1.5`) and apply gentle exit push.
    - Set cooldown to prevent instant re-triggering.
    - Play teleporter sound (`sound_id: 11`).
- **FR-3: Viewmodel Proportion Scaling**:
  - In `src/main.rs:sync_first_person_viewmodel`, reduce base viewmodel heights:
    - Pistol: 275.0px (down from 420.0px)
    - Shotgun: 310.0px (down from 450.0px)
    - Chaingun: 320.0px (down from 460.0px)
    - RPG: 330.0px (down from 480.0px)
    - Knee: 360.0px (down from 500.0px)
- **FR-4: Iron Sight & Reticle Alignment**:
  - Calculate weapon-specific horizontal aim offsets based on sprite geometry:
    - Pistol: `+89.0px` offset shifts the front sight ($x = 10$ in 72px sprite) directly beneath the center reticle.
    - Shotgun: `+85.0px`
    - Chaingun: `+40.0px`
    - RPG: `+30.0px`
  - Apply offset to `style.margin.left = Val::Px(bob_x + aim_offset_x);`.

## 3. Non-Functional Requirements
- Zero compiler warnings, zero `#![allow(dead_code)]`.
- 100% test pass rate across the full test suite.
- Maintain smooth 60+ FPS without allocation in the per-frame viewmodel or interactivity systems.

## 4. Acceptance Criteria
1. Dropping into the rooftop ventilation shaft in E1L1 immediately teleports Duke to the cinema alleyway (Sector 256) facing forward without lethal fall damage.
2. The pistol viewmodel is compact (~38% screen height) with the iron sights positioned dead-center beneath the targeting reticle.
3. Unit tests pass verifying SE 7 pairing, teleportation trigger, and viewmodel layout metrics.
