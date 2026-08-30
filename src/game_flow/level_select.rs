#![allow(dead_code)]

use bevy::prelude::*;
use crate::campaign::episodes::{ALL_CAMPAIGN_MAPS, CampaignMapInfo};
use crate::game_flow::LoadLevelEvent;

#[derive(Resource, Debug, Clone)]
pub struct LevelSelectState {
    pub is_open: bool,
    pub selected_episode: usize,
    pub selected_level: usize,
}

impl Default for LevelSelectState {
    fn default() -> Self {
        Self {
            is_open: false,
            selected_episode: 1,
            selected_level: 1,
        }
    }
}

impl LevelSelectState {
    pub fn next_episode(&mut self) {
        self.selected_episode = if self.selected_episode >= 4 { 1 } else { self.selected_episode + 1 };
        self.selected_level = 1;
    }

    pub fn prev_episode(&mut self) {
        self.selected_episode = if self.selected_episode <= 1 { 4 } else { self.selected_episode - 1 };
        self.selected_level = 1;
    }

    pub fn next_level(&mut self) {
        let max_levels = self.get_levels_for_episode(self.selected_episode).len();
        if max_levels > 0 {
            self.selected_level = if self.selected_level >= max_levels { 1 } else { self.selected_level + 1 };
        }
    }

    pub fn prev_level(&mut self) {
        let max_levels = self.get_levels_for_episode(self.selected_episode).len();
        if max_levels > 0 {
            self.selected_level = if self.selected_level <= 1 { max_levels } else { self.selected_level - 1 };
        }
    }

    pub fn get_levels_for_episode(&self, ep: usize) -> Vec<&'static CampaignMapInfo> {
        ALL_CAMPAIGN_MAPS.iter().filter(|m| m.episode == ep).collect()
    }

    pub fn get_current_map_info(&self) -> Option<&'static CampaignMapInfo> {
        ALL_CAMPAIGN_MAPS.iter().find(|m| m.episode == self.selected_episode && m.level == self.selected_level)
    }

    pub fn warp_to_selected(&mut self, level_event_writer: &mut EventWriter<LoadLevelEvent>) {
        level_event_writer.send(LoadLevelEvent {
            episode: self.selected_episode,
            level: self.selected_level,
        });
        self.is_open = false;
    }
}

pub struct LevelSelectPlugin;

impl Plugin for LevelSelectPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<LevelSelectState>()
            .add_systems(
                Update,
                handle_level_select_input.in_set(crate::GameSet::Input),
            );
    }
}

pub fn handle_level_select_input(
    keys: Res<ButtonInput<KeyCode>>,
    mut level_select: ResMut<LevelSelectState>,
    mut level_event_writer: EventWriter<LoadLevelEvent>,
) {
    if keys.just_pressed(KeyCode::F3) {
        level_select.is_open = !level_select.is_open;
    }

    if !level_select.is_open {
        return;
    }

    if keys.just_pressed(KeyCode::ArrowUp) {
        level_select.prev_episode();
    } else if keys.just_pressed(KeyCode::ArrowDown) {
        level_select.next_episode();
    } else if keys.just_pressed(KeyCode::ArrowLeft) {
        level_select.prev_level();
    } else if keys.just_pressed(KeyCode::ArrowRight) {
        level_select.next_level();
    } else if keys.just_pressed(KeyCode::Enter) || keys.just_pressed(KeyCode::NumpadEnter) {
        level_select.warp_to_selected(&mut level_event_writer);
    } else if keys.just_pressed(KeyCode::Escape) {
        level_select.is_open = false;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_level_select_navigation_and_maps() {
        let mut ls = LevelSelectState::default();
        assert_eq!(ls.selected_episode, 1);
        assert_eq!(ls.selected_level, 1);

        let info = ls.get_current_map_info().unwrap();
        assert_eq!(info.map_filename, "E1L1.MAP");

        ls.next_level();
        assert_eq!(ls.selected_level, 2);
        assert_eq!(ls.get_current_map_info().unwrap().map_filename, "E1L2.MAP");

        ls.next_episode();
        assert_eq!(ls.selected_episode, 2);
        assert_eq!(ls.selected_level, 1);
        assert_eq!(ls.get_current_map_info().unwrap().map_filename, "E2L1.MAP");
    }
}
