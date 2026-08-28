#![allow(dead_code)]

use bevy::prelude::*;
use crate::game_flow::state::*;

#[derive(Resource, Debug, Clone)]
pub struct MenuCursor {
    pub selected_index: usize,
    pub max_items: usize,
}

impl Default for MenuCursor {
    fn default() -> Self {
        Self {
            selected_index: 0,
            max_items: 5,
        }
    }
}

pub fn handle_menu_navigation(
    keys: Res<ButtonInput<KeyCode>>,
    mut next_state: ResMut<NextState<GamePhase>>,
    state: Res<State<GamePhase>>,
    mut cursor: ResMut<MenuCursor>,
    mut progress: ResMut<LevelProgress>,
) {
    if cursor.max_items > 0 {
        if keys.just_pressed(KeyCode::ArrowUp) || keys.just_pressed(KeyCode::KeyW) {
            if cursor.selected_index > 0 {
                cursor.selected_index -= 1;
            } else {
                cursor.selected_index = cursor.max_items.saturating_sub(1);
            }
        }

        if keys.just_pressed(KeyCode::ArrowDown) || keys.just_pressed(KeyCode::KeyS) {
            cursor.selected_index = (cursor.selected_index + 1) % cursor.max_items;
        }
    }

    match state.get() {
        GamePhase::MainMenu => {
            cursor.max_items = 4; // 0: New Game, 1: Options, 2: Load Game, 3: Quit
            if keys.just_pressed(KeyCode::Enter) || keys.just_pressed(KeyCode::Space) {
                match cursor.selected_index {
                    0 => {
                        cursor.selected_index = 0;
                        next_state.set(GamePhase::EpisodeSelect);
                    }
                    _ => {}
                }
            }
        }
        GamePhase::EpisodeSelect => {
            cursor.max_items = 4; // E1, E2, E3, E4
            if keys.just_pressed(KeyCode::Escape) {
                cursor.selected_index = 0;
                next_state.set(GamePhase::MainMenu);
            } else if keys.just_pressed(KeyCode::Enter) || keys.just_pressed(KeyCode::Space) {
                progress.current_episode = cursor.selected_index + 1;
                cursor.selected_index = 1; // Default to "Let's Rock"
                next_state.set(GamePhase::SkillSelect);
            }
        }
        GamePhase::SkillSelect => {
            cursor.max_items = 4; // Piece of Cake, Let's Rock, Come Get Some, Damn I'm Good
            if keys.just_pressed(KeyCode::Escape) {
                cursor.selected_index = 0;
                next_state.set(GamePhase::EpisodeSelect);
            } else if keys.just_pressed(KeyCode::Enter) || keys.just_pressed(KeyCode::Space) {
                progress.skill = match cursor.selected_index {
                    0 => SkillLevel::PieceOfCake,
                    1 => SkillLevel::LetsRock,
                    2 => SkillLevel::ComeGetSome,
                    _ => SkillLevel::DamnImGood,
                };
                progress.current_level = 1;
                next_state.set(GamePhase::Playing);
            }
        }
        _ => {}
    }
}
