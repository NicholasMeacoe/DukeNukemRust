#![allow(dead_code)]

use bevy::prelude::*;
use crate::game_flow::state::GamePhase;

#[derive(Resource, Debug, Clone)]
pub struct AttractModeTimer {
    pub idle_seconds: f32,
    pub active_demo_index: usize,
    pub max_demos: usize,
}

impl Default for AttractModeTimer {
    fn default() -> Self {
        Self {
            idle_seconds: 0.0,
            active_demo_index: 1,
            max_demos: 4,
        }
    }
}

pub fn update_attract_mode_timer(
    time: Res<Time>,
    keys: Res<ButtonInput<KeyCode>>,
    state: Res<State<GamePhase>>,
    mut attract: ResMut<AttractModeTimer>,
) {
    if *state.get() != GamePhase::MainMenu {
        attract.idle_seconds = 0.0;
        return;
    }

    // Any key press resets idle timer
    if keys.get_just_pressed().next().is_some() {
        attract.idle_seconds = 0.0;
    } else {
        attract.idle_seconds += time.delta_seconds();
    }
}
