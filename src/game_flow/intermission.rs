#![allow(dead_code)]

use bevy::prelude::*;
use crate::game_flow::state::*;

#[derive(Debug, Clone)]
pub struct IntermissionStats {
    pub kill_percentage: i32,
    pub secret_percentage: i32,
    pub time_taken_seconds: f32,
    pub par_time_seconds: f32,
}

pub fn update_level_time(
    time: Res<Time>,
    state: Res<State<GamePhase>>,
    mut progress: ResMut<LevelProgress>,
) {
    if *state.get() == GamePhase::Playing {
        progress.level_time_seconds += time.delta_seconds();
    }
}

pub fn handle_level_completed_events(
    mut events: EventReader<LevelCompletedEvent>,
    mut next_state: ResMut<NextState<GamePhase>>,
    mut progress: ResMut<LevelProgress>,
) {
    for _ in events.read() {
        progress.is_level_completed = true;
        next_state.set(GamePhase::Intermission);
    }
}

pub fn handle_intermission_input(
    keys: Res<ButtonInput<KeyCode>>,
    state: Res<State<GamePhase>>,
    mut next_state: ResMut<NextState<GamePhase>>,
    mut progress: ResMut<LevelProgress>,
) {
    if *state.get() == GamePhase::Intermission {
        if keys.just_pressed(KeyCode::Space) || keys.just_pressed(KeyCode::Enter) {
            let episode_finished = progress.advance_to_next_level();
            if episode_finished {
                next_state.set(GamePhase::MainMenu);
            } else {
                next_state.set(GamePhase::Playing);
            }
        }
    }
}

impl LevelProgress {
    pub fn compute_stats(&self) -> IntermissionStats {
        let kill_percentage = if self.total_monsters > 0 {
            ((self.kills_count as f32 / self.total_monsters as f32) * 100.0) as i32
        } else {
            100
        };

        let secret_percentage = if self.total_secrets > 0 {
            ((self.secrets_found as f32 / self.total_secrets as f32) * 100.0) as i32
        } else {
            100
        };

        IntermissionStats {
            kill_percentage: kill_percentage.clamp(0, 100),
            secret_percentage: secret_percentage.clamp(0, 100),
            time_taken_seconds: self.level_time_seconds,
            par_time_seconds: self.par_time_seconds,
        }
    }
}
