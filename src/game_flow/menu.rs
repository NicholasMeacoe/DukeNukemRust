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

#[derive(bevy::ecs::system::SystemParam)]
pub struct MenuEvents<'w> {
    pub sound_events: EventWriter<'w, PlaySoundEvent>,
    pub load_level_events: EventWriter<'w, LoadLevelEvent>,
    pub save_game_events: EventWriter<'w, crate::save::SaveGameEvent>,
    pub load_game_events: EventWriter<'w, crate::save::LoadGameEvent>,
    pub app_exit: EventWriter<'w, AppExit>,
}

#[derive(bevy::ecs::system::SystemParam)]
pub struct MenuEngineConfigs<'w> {
    pub game_config: Option<ResMut<'w, crate::config::GameConfig>>,
    pub crt_config: Option<ResMut<'w, crate::palette::CrtPostProcessConfig>>,
    pub voxel_config: Option<ResMut<'w, crate::voxel::registry::VoxelConfig>>,
    pub lighting_config: Option<ResMut<'w, crate::lighting::DynamicLightingConfig>>,
}

pub fn handle_menu_navigation(
    keys: Res<ButtonInput<KeyCode>>,
    mut next_state: ResMut<NextState<GamePhase>>,
    state: Res<State<GamePhase>>,
    mut cursor: ResMut<MenuCursor>,
    mut progress: ResMut<LevelProgress>,
    mut save_load_origin: Option<ResMut<SaveLoadOrigin>>,
    mut options_origin: Option<ResMut<OptionsOrigin>>,
    mut events: MenuEvents,
    mut save_mgr: Option<ResMut<crate::save::SaveManager>>,
    mut configs: MenuEngineConfigs,
) {
    let current_phase = *state.get();

    // Toggle Pause in Playing state
    if current_phase == GamePhase::Playing {
        if keys.just_pressed(KeyCode::Escape) {
            cursor.selected_index = 0;
            cursor.max_items = 6;
            events.sound_events.send(PlaySoundEvent { sound_id: 2 });
            next_state.set(GamePhase::Paused);
        }
        return;
    }

    if current_phase == GamePhase::Intermission {
        return; // Handled by handle_intermission_input
    }

    match current_phase {
        GamePhase::MainMenu | GamePhase::EpisodeSelect | GamePhase::SkillSelect => {
            cursor.max_items = 4;
        }
        GamePhase::Paused => {
            cursor.max_items = 6;
        }
        GamePhase::OptionsMenu
        | GamePhase::SoundSetup
        | GamePhase::VideoSetup
        | GamePhase::ControlsSetup => {
            cursor.max_items = 4;
        }
        GamePhase::SaveMenu | GamePhase::LoadMenu => {
            cursor.max_items = 10;
        }
        _ => {}
    }
    if cursor.selected_index >= cursor.max_items && cursor.max_items > 0 {
        cursor.selected_index = cursor.max_items - 1;
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
        events.sound_events.send(PlaySoundEvent { sound_id: 0 });
    }

    match current_phase {
        GamePhase::MainMenu => {
            cursor.max_items = 4; // 0: New Game, 1: Options, 2: Load Game, 3: Quit
            if keys.just_pressed(KeyCode::Enter) || keys.just_pressed(KeyCode::Space) {
                events.sound_events.send(PlaySoundEvent { sound_id: 2 });
                match cursor.selected_index {
                    0 => {
                        cursor.selected_index = 0;
                        next_state.set(GamePhase::EpisodeSelect);
                    }
                    1 => {
                        cursor.selected_index = 0;
                        cursor.max_items = 4;
                        if let Some(ref mut o) = options_origin {
                            o.0 = GamePhase::MainMenu;
                        }
                        next_state.set(GamePhase::OptionsMenu);
                    }
                    2 => {
                        cursor.selected_index = 0;
                        cursor.max_items = 10;
                        if let Some(ref mut o) = save_load_origin {
                            o.0 = GamePhase::MainMenu;
                        }
                        next_state.set(GamePhase::LoadMenu);
                    }
                    3 => {
                        events.app_exit.send(AppExit::Success);
                    }
                    _ => {}
                }
            }
        }
        GamePhase::EpisodeSelect => {
            cursor.max_items = 4; // E1, E2, E3, E4
            if keys.just_pressed(KeyCode::Escape) {
                cursor.selected_index = 0;
                events.sound_events.send(PlaySoundEvent { sound_id: 0 });
                next_state.set(GamePhase::MainMenu);
            } else if keys.just_pressed(KeyCode::Enter) || keys.just_pressed(KeyCode::Space) {
                events.sound_events.send(PlaySoundEvent { sound_id: 2 });
                progress.current_episode = cursor.selected_index + 1;
                cursor.selected_index = 1; // Default to "Let's Rock"
                next_state.set(GamePhase::SkillSelect);
            }
        }
        GamePhase::SkillSelect => {
            cursor.max_items = 4; // Piece of Cake, Let's Rock, Come Get Some, Damn I'm Good
            if keys.just_pressed(KeyCode::Escape) {
                cursor.selected_index = 0;
                events.sound_events.send(PlaySoundEvent { sound_id: 0 });
                next_state.set(GamePhase::EpisodeSelect);
            } else if keys.just_pressed(KeyCode::Enter) || keys.just_pressed(KeyCode::Space) {
                events.sound_events.send(PlaySoundEvent { sound_id: 2 });
                progress.skill = match cursor.selected_index {
                    0 => SkillLevel::PieceOfCake,
                    1 => SkillLevel::LetsRock,
                    2 => SkillLevel::ComeGetSome,
                    _ => SkillLevel::DamnImGood,
                };
                progress.current_level = 1;
                events.load_level_events.send(LoadLevelEvent {
                    episode: progress.current_episode,
                    level: 1,
                });
                next_state.set(GamePhase::Playing);
            }
        }
        GamePhase::Paused => {
            cursor.max_items = 6; // 0: Resume, 1: Options, 2: Save Game, 3: Load Game, 4: Main Menu, 5: Quit
            if keys.just_pressed(KeyCode::Escape) {
                events.sound_events.send(PlaySoundEvent { sound_id: 2 });
                next_state.set(GamePhase::Playing);
            } else if keys.just_pressed(KeyCode::Enter) || keys.just_pressed(KeyCode::Space) {
                events.sound_events.send(PlaySoundEvent { sound_id: 2 });
                match cursor.selected_index {
                    0 => {
                        next_state.set(GamePhase::Playing);
                    }
                    1 => {
                        cursor.selected_index = 0;
                        cursor.max_items = 4;
                        if let Some(ref mut o) = options_origin {
                            o.0 = GamePhase::Paused;
                        }
                        next_state.set(GamePhase::OptionsMenu);
                    }
                    2 => {
                        cursor.selected_index = 0;
                        cursor.max_items = 10;
                        if let Some(ref mut o) = save_load_origin {
                            o.0 = GamePhase::Paused;
                        }
                        next_state.set(GamePhase::SaveMenu);
                    }
                    3 => {
                        cursor.selected_index = 0;
                        cursor.max_items = 10;
                        if let Some(ref mut o) = save_load_origin {
                            o.0 = GamePhase::Paused;
                        }
                        next_state.set(GamePhase::LoadMenu);
                    }
                    4 => {
                        cursor.selected_index = 0;
                        cursor.max_items = 4;
                        next_state.set(GamePhase::MainMenu);
                    }
                    5 => {
                        events.app_exit.send(AppExit::Success);
                    }
                    _ => {}
                }
            }
        }
        GamePhase::SaveMenu => {
            cursor.max_items = 10;
            let origin_phase = save_load_origin.as_ref().map_or(GamePhase::Paused, |o| o.0);
            if keys.just_pressed(KeyCode::Escape) {
                events.sound_events.send(PlaySoundEvent { sound_id: 0 });
                cursor.selected_index = 0;
                cursor.max_items = if origin_phase == GamePhase::MainMenu { 4 } else { 6 };
                next_state.set(origin_phase);
            } else if keys.just_pressed(KeyCode::Enter) || keys.just_pressed(KeyCode::Space) {
                let slot = cursor.selected_index;
                if slot < 10 {
                    events.sound_events.send(PlaySoundEvent { sound_id: 2 });
                    let min = (progress.level_time_seconds / 60.0) as i32;
                    let sec = (progress.level_time_seconds % 60.0) as i32;
                    let title = format!(
                        "E{}L{} - {:02}:{:02}",
                        progress.current_episode, progress.current_level, min, sec
                    );
                    if let Some(ref mut sm) = save_mgr {
                        sm.last_saved_slot = Some(slot);
                        if slot < sm.save_slots_info.len() {
                            sm.save_slots_info[slot] = Some(title.clone());
                        }
                    }
                    events.save_game_events.send(crate::save::SaveGameEvent {
                        slot: Some(slot),
                        title,
                    });
                    next_state.set(GamePhase::Playing);
                }
            }
        }
        GamePhase::LoadMenu => {
            cursor.max_items = 10;
            let origin_phase = save_load_origin.as_ref().map_or(GamePhase::Paused, |o| o.0);
            if keys.just_pressed(KeyCode::Escape) {
                events.sound_events.send(PlaySoundEvent { sound_id: 0 });
                cursor.selected_index = 0;
                cursor.max_items = if origin_phase == GamePhase::MainMenu { 4 } else { 6 };
                next_state.set(origin_phase);
            } else if keys.just_pressed(KeyCode::Enter) || keys.just_pressed(KeyCode::Space) {
                let slot = cursor.selected_index;
                if slot < 10 {
                    let has_save = crate::save::slot_save_exists(slot)
                        || save_mgr
                            .as_ref()
                            .map_or(false, |m| m.save_slots_info[slot].is_some());
                    if has_save {
                        events.sound_events.send(PlaySoundEvent { sound_id: 2 });
                        events.load_game_events.send(crate::save::LoadGameEvent {
                            slot: Some(slot),
                        });
                        next_state.set(GamePhase::Playing);
                    }
                }
            }
        }
        GamePhase::OptionsMenu => {
            cursor.max_items = 4; // 0: Sound Setup, 1: Video & Display, 2: Gameplay & Controls, 3: Restore Defaults
            let origin_phase = options_origin.as_ref().map_or(GamePhase::MainMenu, |o| o.0);
            if keys.just_pressed(KeyCode::Escape) {
                events.sound_events.send(PlaySoundEvent { sound_id: 0 });
                cursor.selected_index = 1; // Back to OPTIONS on parent menu
                cursor.max_items = if origin_phase == GamePhase::MainMenu { 4 } else { 6 };
                if let Some(ref cfg) = configs.game_config {
                    let _ = cfg.save(crate::config::GameConfig::default_config_path());
                }
                next_state.set(origin_phase);
            } else if keys.just_pressed(KeyCode::Enter) || keys.just_pressed(KeyCode::Space) {
                events.sound_events.send(PlaySoundEvent { sound_id: 2 });
                match cursor.selected_index {
                    0 => {
                        cursor.selected_index = 0;
                        cursor.max_items = 4;
                        next_state.set(GamePhase::SoundSetup);
                    }
                    1 => {
                        cursor.selected_index = 0;
                        cursor.max_items = 4;
                        next_state.set(GamePhase::VideoSetup);
                    }
                    2 => {
                        cursor.selected_index = 0;
                        cursor.max_items = 4;
                        next_state.set(GamePhase::ControlsSetup);
                    }
                    3 => {
                        if let Some(ref mut cfg) = configs.game_config {
                            **cfg = crate::config::GameConfig::default();
                            let _ = cfg.save(crate::config::GameConfig::default_config_path());
                        }
                        if let Some(ref mut crt) = configs.crt_config {
                            crt.enabled = false;
                        }
                        if let Some(ref mut vox) = configs.voxel_config {
                            vox.enabled = true;
                        }
                        if let Some(ref mut light) = configs.lighting_config {
                            light.enabled = true;
                        }
                    }
                    _ => {}
                }
            }
        }
        GamePhase::SoundSetup => {
            cursor.max_items = 4;
            if keys.just_pressed(KeyCode::Escape) {
                events.sound_events.send(PlaySoundEvent { sound_id: 0 });
                cursor.selected_index = 0;
                cursor.max_items = 4;
                if let Some(ref cfg) = configs.game_config {
                    let _ = cfg.save(crate::config::GameConfig::default_config_path());
                }
                next_state.set(GamePhase::OptionsMenu);
            }
        }
        GamePhase::VideoSetup => {
            cursor.max_items = 4;
            if keys.just_pressed(KeyCode::Escape) {
                events.sound_events.send(PlaySoundEvent { sound_id: 0 });
                cursor.selected_index = 1;
                cursor.max_items = 4;
                if let Some(ref cfg) = configs.game_config {
                    let _ = cfg.save(crate::config::GameConfig::default_config_path());
                }
                next_state.set(GamePhase::OptionsMenu);
            }
        }
        GamePhase::ControlsSetup => {
            cursor.max_items = 4;
            if keys.just_pressed(KeyCode::Escape) {
                events.sound_events.send(PlaySoundEvent { sound_id: 0 });
                cursor.selected_index = 2;
                cursor.max_items = 4;
                if let Some(ref cfg) = configs.game_config {
                    let _ = cfg.save(crate::config::GameConfig::default_config_path());
                }
                next_state.set(GamePhase::OptionsMenu);
            }
        }
        _ => {}
    }
}
