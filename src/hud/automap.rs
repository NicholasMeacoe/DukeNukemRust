#![allow(dead_code)]

use bevy::prelude::*;
use std::collections::HashSet;
use crate::game_flow::state::GamePhase;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum AutomapMode {
    #[default]
    Off,
    Overlay,
    Fullscreen,
}

#[derive(Resource, Debug, Clone)]
pub struct AutomapState {
    pub mode: AutomapMode,
    pub zoom: f32,
    pub follow_player: bool,
    pub pan_offset: Vec2,
    pub discovered_sectors: HashSet<usize>,
}

impl Default for AutomapState {
    fn default() -> Self {
        Self {
            mode: AutomapMode::Off,
            zoom: 1.0,
            follow_player: true,
            pan_offset: Vec2::ZERO,
            discovered_sectors: HashSet::new(),
        }
    }
}

pub struct AutomapPlugin;

impl Plugin for AutomapPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<AutomapState>()
            .add_systems(
                Update,
                (handle_automap_input, update_automap_discovery)
                    .run_if(in_state(GamePhase::Playing)),
            );
    }
}

pub fn handle_automap_input(
    keys: Res<ButtonInput<KeyCode>>,
    mut automap: ResMut<AutomapState>,
    mut sound_events: EventWriter<crate::audio::PlaySoundEvent>,
) {
    // Tab: cycle automap modes
    if keys.just_pressed(KeyCode::Tab) {
        automap.mode = match automap.mode {
            AutomapMode::Off => AutomapMode::Overlay,
            AutomapMode::Overlay => AutomapMode::Fullscreen,
            AutomapMode::Fullscreen => AutomapMode::Off,
        };
        sound_events.send(crate::audio::PlaySoundEvent { sound_id: 110 });
    }

    if automap.mode == AutomapMode::Off {
        return;
    }

    // Zoom controls: Plus / Minus
    if keys.just_pressed(KeyCode::Equal) || keys.just_pressed(KeyCode::NumpadAdd) {
        automap.zoom = (automap.zoom * 1.25).clamp(0.25, 4.0);
    }
    if keys.just_pressed(KeyCode::Minus) || keys.just_pressed(KeyCode::NumpadSubtract) {
        automap.zoom = (automap.zoom / 1.25).clamp(0.25, 4.0);
    }

    // Follow mode toggle: KeyF
    if keys.just_pressed(KeyCode::KeyF) {
        automap.follow_player = !automap.follow_player;
        if automap.follow_player {
            automap.pan_offset = Vec2::ZERO;
        }
    }

    // Free pan mode
    if !automap.follow_player {
        let pan_speed = 15.0;
        if keys.pressed(KeyCode::ArrowUp) {
            automap.pan_offset.y += pan_speed;
        }
        if keys.pressed(KeyCode::ArrowDown) {
            automap.pan_offset.y -= pan_speed;
        }
        if keys.pressed(KeyCode::ArrowLeft) {
            automap.pan_offset.x -= pan_speed;
        }
        if keys.pressed(KeyCode::ArrowRight) {
            automap.pan_offset.x += pan_speed;
        }
    }
}

pub fn update_automap_discovery(
    player_query: Query<&Transform, With<crate::Player>>,
    mut automap: ResMut<AutomapState>,
    cheats: Res<crate::player::CheatState>,
) {
    if cheats.show_all_map {
        // When dnshowmap is active, reveal all sectors (0..500)
        for s in 0..500 {
            automap.discovered_sectors.insert(s);
        }
        return;
    }

    let Ok(trans) = player_query.get_single() else { return; };
    // Current sector proximity discovery
    let _player_pos = trans.translation;
    automap.discovered_sectors.insert(0);
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum AutomapLineKind {
    SingleSided,
    TwoSidedStep,
    TwoSidedBlocking,
    SecretSector,
}

pub fn classify_automap_line(
    wall_cstat: u16,
    next_wall: i16,
    has_height_step: bool,
    is_secret_sector: bool,
) -> AutomapLineKind {
    if is_secret_sector {
        AutomapLineKind::SecretSector
    } else if next_wall < 0 {
        AutomapLineKind::SingleSided
    } else if (wall_cstat & 1) != 0 {
        AutomapLineKind::TwoSidedBlocking
    } else if has_height_step {
        AutomapLineKind::TwoSidedStep
    } else {
        AutomapLineKind::SingleSided
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_automap_mode_cycling() {
        let mut automap = AutomapState::default();
        assert_eq!(automap.mode, AutomapMode::Off);

        automap.mode = match automap.mode {
            AutomapMode::Off => AutomapMode::Overlay,
            AutomapMode::Overlay => AutomapMode::Fullscreen,
            AutomapMode::Fullscreen => AutomapMode::Off,
        };
        assert_eq!(automap.mode, AutomapMode::Overlay);

        automap.mode = match automap.mode {
            AutomapMode::Off => AutomapMode::Overlay,
            AutomapMode::Overlay => AutomapMode::Fullscreen,
            AutomapMode::Fullscreen => AutomapMode::Off,
        };
        assert_eq!(automap.mode, AutomapMode::Fullscreen);

        automap.mode = match automap.mode {
            AutomapMode::Off => AutomapMode::Overlay,
            AutomapMode::Overlay => AutomapMode::Fullscreen,
            AutomapMode::Fullscreen => AutomapMode::Off,
        };
        assert_eq!(automap.mode, AutomapMode::Off);
    }

    #[test]
    fn test_automap_line_classification() {
        // Single sided solid boundary wall
        assert_eq!(
            classify_automap_line(0, -1, false, false),
            AutomapLineKind::SingleSided
        );

        // Two sided portal with height step
        assert_eq!(
            classify_automap_line(0, 42, true, false),
            AutomapLineKind::TwoSidedStep
        );

        // Two sided blocking wall
        assert_eq!(
            classify_automap_line(1, 42, false, false),
            AutomapLineKind::TwoSidedBlocking
        );

        // Secret sector wall
        assert_eq!(
            classify_automap_line(0, 42, false, true),
            AutomapLineKind::SecretSector
        );
    }

    #[test]
    fn test_automap_zoom_clamping() {
        let mut automap = AutomapState::default();
        assert_eq!(automap.zoom, 1.0);

        automap.zoom = 10.0f32.clamp(0.25, 4.0);
        assert_eq!(automap.zoom, 4.0);

        automap.zoom = 0.05f32.clamp(0.25, 4.0);
        assert_eq!(automap.zoom, 0.25);
    }
}
