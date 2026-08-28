#![allow(dead_code)]

pub mod state;
pub mod menu;
pub mod intermission;

pub use state::*;
pub use menu::*;
pub use intermission::*;

use bevy::prelude::*;

pub struct GameFlowPlugin;

impl Plugin for GameFlowPlugin {
    fn build(&self, app: &mut App) {
        app.init_state::<GamePhase>()
            .init_resource::<LevelProgress>()
            .init_resource::<MenuCursor>()
            .add_event::<LevelCompletedEvent>()
            .add_systems(
                Update,
                (
                    handle_menu_navigation,
                    update_level_time,
                    handle_level_completed_events,
                    handle_intermission_input,
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
}
