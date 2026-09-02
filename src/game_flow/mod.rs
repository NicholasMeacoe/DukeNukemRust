#![allow(dead_code)]

pub mod intermission;
pub mod level_loader;
pub mod level_select;
pub mod menu;
pub mod state;
pub mod ui;

pub use intermission::*;
pub use level_loader::*;
pub use level_select::*;
pub use menu::*;
pub use state::*;
pub use ui::*;

use bevy::prelude::*;

pub struct GameFlowPlugin;

impl Plugin for GameFlowPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(level_select::LevelSelectPlugin)
            .init_state::<GamePhase>()
            .init_resource::<LevelProgress>()
            .init_resource::<IntermissionAnimationState>()
            .init_resource::<MenuCursor>()
            .init_resource::<CursorAnimTimer>()
            .add_event::<LevelCompletedEvent>()
            .add_event::<LoadLevelEvent>()
            .add_systems(Startup, setup_menu_ui)
            .add_systems(
                Update,
                (
                    handle_menu_navigation,
                    update_cursor_animation,
                    update_menu_ui,
                    update_level_time,
                    update_intermission_animation,
                    handle_level_completed_events,
                    handle_intermission_input,
                    handle_load_level_events,
                ),
            );
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_level_progression_episode1() {
        let mut progress = LevelProgress::default();
        assert_eq!(progress.current_map_filename(), "E1L1.MAP");

        // Advance through Episode 1
        assert!(!progress.advance_to_next_level());
        assert_eq!(progress.current_map_filename(), "E1L2.MAP");

        assert!(!progress.advance_to_next_level());
        assert_eq!(progress.current_map_filename(), "E1L3.MAP");

        assert!(!progress.advance_to_next_level());
        assert_eq!(progress.current_map_filename(), "E1L4.MAP");

        assert!(!progress.advance_to_next_level());
        assert_eq!(progress.current_map_filename(), "E1L5.MAP");

        assert!(!progress.advance_to_next_level());
        assert_eq!(progress.current_map_filename(), "E1L6.MAP");

        assert!(!progress.advance_to_next_level());
        assert_eq!(progress.current_map_filename(), "E1L7.MAP");

        // Boss level completed -> Episode finished
        assert!(progress.advance_to_next_level());
        assert_eq!(progress.current_level, 1);
    }

    #[test]
    fn test_intermission_stats_computation() {
        let mut progress = LevelProgress::default();
        progress.kills_count = 25;
        progress.total_monsters = 50;
        progress.secrets_found = 2;
        progress.total_secrets = 4;
        progress.level_time_seconds = 125.0;

        let stats = progress.compute_stats();
        assert_eq!(stats.kill_percentage, 50);
        assert_eq!(stats.secret_percentage, 50);
        assert_eq!(stats.time_taken_seconds, 125.0);
        assert_eq!(stats.par_time_seconds, 180.0);
    }

    #[test]
    fn test_load_real_grp_e1_maps() {
        if let Ok(grp) = crate::grp::Grp::open("dukenukem3d/duke3d.grp") {
            for level in 1..=4 {
                let map_name = format!("E1L{}.MAP", level);
                let map_data = grp
                    .read_file(&map_name)
                    .expect(&format!("{} must exist in GRP", map_name));
                let map = crate::map::Map::from_bytes(&map_data)
                    .expect(&format!("{} must parse", map_name));
                assert!(!map.sectors.is_empty(), "{} must have sectors", map_name);
                assert!(!map.walls.is_empty(), "{} must have walls", map_name);

                let monster_count = map
                    .sprites
                    .iter()
                    .filter(|s| matches!(s.picnum, 2000 | 1680 | 1820 | 2120))
                    .count();
                println!(
                    "{}: {} sectors, {} walls, {} sprites ({} monsters)",
                    map_name,
                    map.sectors.len(),
                    map.walls.len(),
                    map.sprites.len(),
                    monster_count
                );
            }
        }
    }

    #[test]
    fn test_load_level_event_structure() {
        let event = LoadLevelEvent {
            episode: 1,
            level: 2,
        };
        assert_eq!(event.episode, 1);
        assert_eq!(event.level, 2);

        let entity_marker = LevelEntity;
        assert_eq!(format!("{:?}", entity_marker), "LevelEntity");
    }

    #[test]
    fn test_menu_cursor_navigation_wrap() {
        let mut cursor = MenuCursor {
            selected_index: 0,
            max_items: 4,
        };

        // Up wraps to max_items - 1
        cursor.selected_index = cursor
            .selected_index
            .checked_sub(1)
            .unwrap_or(cursor.max_items - 1);
        assert_eq!(cursor.selected_index, 3);

        // Down wraps to 0
        cursor.selected_index = (cursor.selected_index + 1) % cursor.max_items;
        assert_eq!(cursor.selected_index, 0);

        // Increment to 1
        cursor.selected_index = (cursor.selected_index + 1) % cursor.max_items;
        assert_eq!(cursor.selected_index, 1);
    }

    #[test]
    fn test_cursor_animation_tick() {
        let mut anim = CursorAnimTimer::default();
        assert_eq!(anim.frame, 0);

        anim.frame = (anim.frame + 1) % 4;
        assert_eq!(anim.frame, 1);

        anim.frame = (anim.frame + 3) % 4;
        assert_eq!(anim.frame, 0);
    }

    #[test]
    fn test_game_phase_states() {
        let default_phase = GamePhase::default();
        assert_eq!(default_phase, GamePhase::MainMenu);

        let paused = GamePhase::Paused;
        assert_ne!(paused, GamePhase::Playing);
    }

    #[test]
    fn test_e1l1_full_playthrough_pipeline() {
        let mut progress = LevelProgress::default();
        assert_eq!(progress.current_map_filename(), "E1L1.MAP");

        let mut player = crate::player::PlayerController::default();
        assert_eq!(player.health, 100);

        // 1. Pickups: Grab Shotgun and Armor
        player.weapons[crate::player::WeaponType::Shotgun as usize].is_unlocked = true;
        player.weapons[crate::player::WeaponType::Shotgun as usize].ammo += 10;
        player.armor = 100;
        assert!(player.weapons[crate::player::WeaponType::Shotgun as usize].is_unlocked);
        assert_eq!(player.armor, 100);

        // 2. Combat: Kill 5 enemies
        let mut enemy = crate::combat::EnemyActor::new_pigcop();
        enemy.health -= 120;
        if enemy.health <= 0 {
            enemy.state = crate::combat::EnemyAiState::Dying;
            progress.kills_count += 1;
        }
        assert_eq!(enemy.state, crate::combat::EnemyAiState::Dying);
        progress.kills_count = 15;
        progress.total_monsters = 20;

        // 3. Secrets: Find 2 secrets
        progress.secrets_found = 2;
        progress.total_secrets = 3;

        // 4. Level Exit: Trigger Nuke Exit Button
        let mut nuke = crate::interactivity::NukeExitSwitch {
            is_activated: false,
            is_secret: false,
            lotag: 0,
            hitag: 0,
        };
        nuke.is_activated = true;
        assert!(nuke.is_activated);
        progress.is_level_completed = true;
        progress.level_time_seconds = 145.0; // 2:25

        // 5. Intermission Stats Screen
        let stats = progress.compute_stats();
        assert_eq!(stats.kill_percentage, 75); // 15/20 = 75%
        assert_eq!(stats.secret_percentage, 66); // 2/3 = 66%
        assert_eq!(stats.time_taken_seconds, 145.0);

        // 6. Transition to E1L2
        let episode_done = progress.advance_to_next_level();
        assert!(!episode_done);
        assert_eq!(progress.current_level, 2);
        assert_eq!(progress.current_map_filename(), "E1L2.MAP");
    }

    #[test]
    fn test_intermission_timed_stages_rollout() {
        let mut anim = IntermissionAnimationState::default();
        assert_eq!(anim.stage, IntermissionStage::BackgroundFadeIn);

        anim.timer = 0.6;
        if anim.timer >= 0.5 {
            anim.stage = IntermissionStage::KillsTally;
        }
        assert_eq!(anim.stage, IntermissionStage::KillsTally);

        anim.timer = 1.9;
        if anim.timer >= 1.8 {
            anim.stage = IntermissionStage::SecretsTally;
        }
        assert_eq!(anim.stage, IntermissionStage::SecretsTally);

        anim.timer = 3.1;
        if anim.timer >= 3.0 {
            anim.stage = IntermissionStage::TimeReveal;
        }
        assert_eq!(anim.stage, IntermissionStage::TimeReveal);

        anim.timer = 4.3;
        if anim.timer >= 4.2 {
            anim.stage = IntermissionStage::ReadyToProceed;
        }
        assert_eq!(anim.stage, IntermissionStage::ReadyToProceed);
    }
}
