# Track Specification: Boss AI & Multi-Episode Level Progression

## Overview
This track delivers authentic boss encounters (Battlelord, Overlord, Cycloid Emperor), boss death victory sequences, and complete multi-episode campaign progression across Episode 1 ("L.A. Meltdown"), Episode 2 ("Lunar Apocalypse"), and Episode 3 ("Shrapnel City"), including secret level routing, episode-specific skyboxes, and intermission victory banners.

## Functional Requirements

### 1. Boss AI Combat Mechanics & Multi-Phase Attack Cycles
- **Boss 1: Battlelord & Mini-Battlelord (`Boss1Battlelord`, `Boss1Mini`)**:
  - Minigun bullet barrage and lobbed mortar artillery.
  - Lethal close-range stomp and ground pound when player is within melee reach.
  - Boss death scream (sound 115 `BOS1_DYING`) and heavy blood/gib burst upon defeat.
  - Mini-Battlelords (pal 21) have 1,000 HP, mini scale, and do not trigger boss level completion.
- **Boss 2: Overlord (`Boss2Overlord`, E2L9)**:
  - Multi-phase attack cycle:
    - Phase 1: Dual shoulder rocket salvos (spawns 2 `ProjectileType::Rocket` with horizontal offset and slight spread).
    - Phase 2: Rapid chest energy blaster / laser barrage (`ProjectileType::AlienBlaster` / high velocity).
    - Phase 3: Tail whip or lethal ground stomp if player is within 2.5m.
  - Boss audio cues: attack roar (sound 122 `BOS2_ATTACK`), pain roar, and dying roar (sound 123 `BOS2_DYING`).
  - Boss drop: Atomic Health on death.
- **Boss 3: Cycloid Emperor (`Boss3Cycloid`, E3L9)**:
  - Multi-phase attack cycle:
    - Phase 1: Forehead eye psychic beam / rapid plasma bolts (`ProjectileType::PsiBlast`).
    - Phase 2: Arm rocket volleys (continuous burst of 4 rockets).
    - Phase 3: Quake shockwave foot-slam: when stomping, triggers `EarthquakeCameraShake`, inflicts 50 damage if in radius, and repels player backward.
  - Boss audio cues: attack roar, pain roar, and dying roar (sound 128 `BOS3_DYING`).
  - Boss drop: Atomic Health on death.

### 2. Boss Level Victory & Defeat Triggers
- Detect boss defeat on canonical boss levels using `ALL_CAMPAIGN_MAPS`:
  - Episode 1, Level 6 (`E1L6.MAP` Launch Facility) -> Battlelord defeat triggers `LevelCompletedEvent { is_secret: false, is_boss_victory: true }`.
  - Episode 2, Level 9 (`E2L9.MAP` Overlord) -> Overlord defeat triggers `LevelCompletedEvent { is_secret: false, is_boss_victory: true }`.
  - Episode 3, Level 9 (`E3L9.MAP` Stadium) -> Cycloid Emperor defeat triggers `LevelCompletedEvent { is_secret: false, is_boss_victory: true }`.
- Non-boss level encounters or mini-bosses (e.g. `Boss1Mini` in E2L7 or E3L7) do NOT trigger level completion.

### 3. Multi-Episode Campaign Progression & Secret Level Routing
- Dynamic map count per episode using `ALL_CAMPAIGN_MAPS` rather than hardcoded 7:
  - Episode 1: 7 standard + 1 secret (E1L1..E1L8).
  - Episode 2: 9 standard + 2 secret (E2L1..E2L11).
  - Episode 3: 9 standard + 2 secret (E3L1..E3L11).
- Secret Level Branching:
  - Secret exit switches (`NukeExitSwitch { is_secret: true }`) trigger secret level transition to `secret_destination` (e.g. E1L3 -> E1L8, E2L5 -> E2L10, E3L5 -> E3L10).
  - Completing a secret level routes player back to the canonical destination map (e.g. E1L8 -> E1L4, E2L10 -> E2L6, E3L10 -> E3L6).
- Episode-Specific Skyboxes:
  - Episode 1: tile 89 `LA_SKY`
  - Episode 2: tile 80 `MOONSKY1`
  - Episode 3: tile 84 `CITY_SKY`
- Episode Victory Sequence:
  - Defeating an episode boss displays intermission stats with "EPISODE VICTORY" banner and victory fanfare.
  - Pressing Enter on the episode victory intermission screen advances to the next episode (e.g. Ep 1 -> Ep 2, Ep 2 -> Ep 3) or returns to Main Menu after Episode 3/4.

## Acceptance Criteria
- All 3 main bosses (Battlelord, Overlord, Cycloid Emperor) execute their complete multi-phase attack routines and sound effects.
- Defeating the boss on each episode's boss map triggers boss death sounds, intermission stats with episode victory, and progression to the next episode.
- Secret levels correctly branch from secret switches and return to the proper campaign map upon completion.
- `advance_to_next_level` accurately queries `ALL_CAMPAIGN_MAPS` for map limits and secret destinations.
- Episode 1, 2, and 3 skyboxes render their authentic background textures.
- Full test suite passes with 100% success rate, 0 warnings on `cargo check --tests`, and zero `#![allow(dead_code)]`.
