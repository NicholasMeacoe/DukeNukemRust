#![allow(dead_code)]

use crate::game_flow::state::*;
use bevy::prelude::*;

use crate::audio::PlaySoundEvent;
use bevy::app::AppExit;

#[derive(Resource, Debug, Clone)]
pub struct MenuCursor {
    pub selected_index: usize,
    pub max_items: usize,
}

impl Default for MenuCursor {
    fn default() -> Self {
        Self {
            selected_index: 0,
            max_items: 4,
        }
    }
}

#[derive(Resource, Debug, Clone)]
pub struct CursorAnimTimer {
    pub timer: Timer,
    pub frame: usize,
}

impl Default for CursorAnimTimer {
    fn default() -> Self {
        Self {
            timer: Timer::from_seconds(0.12, TimerMode::Repeating),
            frame: 0,
        }
    }
}

pub fn update_cursor_animation(time: Res<Time>, mut anim: ResMut<CursorAnimTimer>) {
    if anim.timer.tick(time.delta()).just_finished() {
        anim.frame = (anim.frame + 1) % 4;
    }
}

pub fn handle_menu_navigation(
    keys: Res<ButtonInput<KeyCode>>,
    mut next_state: ResMut<NextState<GamePhase>>,
    state: Res<State<GamePhase>>,
    mut cursor: ResMut<MenuCursor>,
    mut progress: ResMut<LevelProgress>,
    mut sound_events: EventWriter<PlaySoundEvent>,
    mut load_level_events: EventWriter<LoadLevelEvent>,
    mut app_exit: EventWriter<AppExit>,
) {
    let current_phase = *state.get();

    // Toggle Pause in Playing state
    if current_phase == GamePhase::Playing {
        if keys.just_pressed(KeyCode::Escape) {
            cursor.selected_index = 0;
            cursor.max_items = 4;
            sound_events.send(PlaySoundEvent { sound_id: 34 });
            next_state.set(GamePhase::Paused);
        }
        return;
    }

    if current_phase == GamePhase::Intermission {
        return; // Handled by handle_intermission_input
    }

    let mut moved = false;
    if cursor.max_items > 0 {
        if keys.just_pressed(KeyCode::ArrowUp) || keys.just_pressed(KeyCode::KeyW) {
            if cursor.selected_index > 0 {
                cursor.selected_index -= 1;
            } else {
                cursor.selected_index = cursor.max_items.saturating_sub(1);
            }
            moved = true;
        }

        if keys.just_pressed(KeyCode::ArrowDown) || keys.just_pressed(KeyCode::KeyS) {
            cursor.selected_index = (cursor.selected_index + 1) % cursor.max_items;
            moved = true;
        }
    }

    if moved {
        sound_events.send(PlaySoundEvent { sound_id: 33 });
    }

    match current_phase {
        GamePhase::MainMenu => {
            cursor.max_items = 4; // 0: New Game, 1: Options, 2: Load Game, 3: Quit
            if keys.just_pressed(KeyCode::Enter) || keys.just_pressed(KeyCode::Space) {
                sound_events.send(PlaySoundEvent { sound_id: 34 });
                match cursor.selected_index {
                    0 => {
                        cursor.selected_index = 0;
                        next_state.set(GamePhase::EpisodeSelect);
                    }
                    3 => {
                        app_exit.send(AppExit::Success);
                    }
                    _ => {}
                }
            }
        }
        GamePhase::EpisodeSelect => {
            cursor.max_items = 4; // E1, E2, E3, E4
            if keys.just_pressed(KeyCode::Escape) {
                cursor.selected_index = 0;
                sound_events.send(PlaySoundEvent { sound_id: 33 });
                next_state.set(GamePhase::MainMenu);
            } else if keys.just_pressed(KeyCode::Enter) || keys.just_pressed(KeyCode::Space) {
                sound_events.send(PlaySoundEvent { sound_id: 34 });
                progress.current_episode = cursor.selected_index + 1;
                cursor.selected_index = 1; // Default to "Let's Rock"
                next_state.set(GamePhase::SkillSelect);
            }
        }
        GamePhase::SkillSelect => {
            cursor.max_items = 4; // Piece of Cake, Let's Rock, Come Get Some, Damn I'm Good
            if keys.just_pressed(KeyCode::Escape) {
                cursor.selected_index = 0;
                sound_events.send(PlaySoundEvent { sound_id: 33 });
                next_state.set(GamePhase::EpisodeSelect);
            } else if keys.just_pressed(KeyCode::Enter) || keys.just_pressed(KeyCode::Space) {
                sound_events.send(PlaySoundEvent { sound_id: 34 });
                progress.skill = match cursor.selected_index {
                    0 => SkillLevel::PieceOfCake,
                    1 => SkillLevel::LetsRock,
                    2 => SkillLevel::ComeGetSome,
                    _ => SkillLevel::DamnImGood,
                };
                progress.current_level = 1;
                load_level_events.send(LoadLevelEvent {
                    episode: progress.current_episode,
                    level: 1,
                });
                next_state.set(GamePhase::Playing);
            }
        }
        GamePhase::Paused => {
            cursor.max_items = 4; // 0: Resume, 1: Options, 2: Main Menu, 3: Quit
            if keys.just_pressed(KeyCode::Escape) {
                sound_events.send(PlaySoundEvent { sound_id: 34 });
                next_state.set(GamePhase::Playing);
            } else if keys.just_pressed(KeyCode::Enter) || keys.just_pressed(KeyCode::Space) {
                sound_events.send(PlaySoundEvent { sound_id: 34 });
                match cursor.selected_index {
                    0 => {
                        next_state.set(GamePhase::Playing);
                    }
                    2 => {
                        cursor.selected_index = 0;
                        next_state.set(GamePhase::MainMenu);
                    }
                    3 => {
                        app_exit.send(AppExit::Success);
                    }
                    _ => {}
                }
            }
        }
        _ => {}
    }
}
