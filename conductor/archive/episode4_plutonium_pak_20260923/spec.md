# Specification: Episode 4 Plutonium Pak Campaign Maps & Alien Queen Boss Fight

## Overview
This track implements the full Episode 4 ("The Birth" / Plutonium Pak / Atomic Edition) campaign experience in the Duke Nukem 3D Rust engine. It brings the climactic Alien Queen (`Boss4Queen`) multi-phase boss encounter to life, wires up complete campaign map progression across Episode 4 (E4L1 through E4L10), integrates the canonical secret level routing (E4L4 -> E4L11 "Area 51" -> E4L5 "Pigsty"), assigns the authentic Atomic Red Sky (`REDSKY1` #98) skybox, and connects Episode 4 selection in the main menu to the campaign pipeline.

---

## Functional Requirements

### 1. Alien Queen Combat AI & Multi-Phase Attack Behavior
- **Actor Kind:** `EnemyKind::Boss4Queen` with 4,500 HP (scaling with difficulty) and full status effect immunities (immune to Shrink Ray, Freeze Shards, and Expander Ray).
- **Multi-Phase Attack Cycle:**
  - **Phase 1 (Eye Lightning Discharge):** Fires concentrated electrical/plasma bolts (`ProjectileType::AlienBlaster` / `PsiBlast`) with slight spread towards the player.
  - **Phase 2 (Protector/Egg Spurt Barrage):** Launches an organic burst attack (venom/spit barrage or minion projectiles) accompanied by distinct boss sounds.
  - **Phase 3 (Close-Range Tail Strike):** When the player is within close range (<= 12.0m), delivers a heavy tail sting/swipe dealing significant damage and triggering a camera rumble.
- **Boss Atmosphere & Audio:**
  - Boss walk footstep tremors shaking the camera when moving near the player.
  - Sound triggers for Alien Queen attacks (`BOS4_ATTACK`), pain, and death screams (`BOS4_DYING`).
- **Boss Defeat Drops:** Drops Atomic Health (`PickupKind::AtomicHealth`) upon defeat.

### 2. Episode 4 Boss Level Victory Sequence
- Defeating the Alien Queen on canonical boss level E4L10 ("The Queen") triggers `LevelCompletedEvent { is_secret: false, is_boss_victory: true }`.
- Non-boss maps or mini-bosses do not trigger level completion upon death.
- Intermission screen renders the gold "EPISODE VICTORY!" header, "EPISODE COMPLETED" subheader, and "PRESS SPACE OR ENTER TO ADVANCE TO NEXT EPISODE" footer.
- Duke's bonus victory quote (`BONUS_SPEECH1` #195) plays upon tally completion.

### 3. Campaign Map Progression & Secret Level Routing
- Traversal across all 11 Episode 4 maps defined in `ALL_CAMPAIGN_MAPS`:
  - E4L1: It's Impossible (`GOINGDN.MID`)
  - E4L2: Duke-Burger (`BRIEFING.MID`)
  - E4L3: Shop-N-Bag (`WAREHSE.MID`)
  - E4L4: Babe Land (`BABES.MID`) - contains secret exit button
  - E4L5: Pigsty (`PIGSTY.MID`)
  - E4L6: Going Postal (`POSTAL.MID`)
  - E4L7: XXX-Stacy (`STACY.MID`)
  - E4L8: Critical Mass (`CRITICAL.MID`)
  - E4L9: Derelict (`DERELICT.MID`)
  - E4L10: The Queen (`QUEEN.MID`) - Boss Finale
  - E4L11: Area 51 (`GOINGDN.MID`) - Secret Level
- **Secret Exit Routing:** Nuke button with `is_secret == true` on E4L4 routes player directly to E4L11 ("Area 51").
- **Canonical Return:** Completing E4L11 returns the player to E4L5 ("Pigsty").
- **Episode Completion:** Beating E4L10 completes Episode 4 and returns the player to the Main Menu.

### 4. Episode 4 Atmospheric Skybox
- Update `sky_tile_for_episode(4)` to return `crate::names::REDSKY1` (tile #98 - Red Alien/Atomic sky).
- Ensure parallax ceiling sectors in Episode 4 maps correctly display the red atmospheric dome.

### 5. Menu Integration
- Selecting "4: THE PLUTONIUM PAK" from the Episode Select menu loads E4L1 and initializes Episode 4 progression.

---

## Non-Functional Requirements
- **Performance:** Boss AI updates and projectile spawning must run at a smooth 60+ FPS without garbage allocation.
- **Code Standards:** 0 compiler warnings (`cargo check --tests`), 0 dead code annotations (`#![allow(dead_code)]`), 100% test pass rate across the full repository test suite.
- **Modularity:** Adhere to existing Bevy ECS patterns in `src/combat/`, `src/campaign/`, and `src/game_flow/`.

---

## Acceptance Criteria
1. `EnemyKind::Boss4Queen` executes the 3-phase attack cycle, plays authentic audio cues, and drops Atomic Health on death.
2. Boss status effect immunities prevent Queen from being shrunk, frozen, or expanded.
3. Defeating the Queen on E4L10 completes the level with `is_boss_victory = true` and shows the gold Episode Victory banner.
4. Level progression sequentially navigates E4L1..E4L10.
5. Secret exit on E4L4 routes to E4L11; completing E4L11 returns to E4L5.
6. Episode 4 skybox selects `REDSKY1` (#98).
7. Episode 4 selection in menu loads E4L1.
8. Comprehensive unit tests cover Queen AI, E4 progression, secret routing, and skybox selection.
9. Full test suite passes 100% with 0 warnings.
