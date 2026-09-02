#![allow(dead_code)]

use bevy::prelude::*;
use crate::hud::font::{DukeFont, DukeFontRenderer, GlyphDrawCall};
use crate::player::types::PlayerController;

pub const BOTTOMSTATUSBAR: i16 = 2462;
pub const KEY_BLUE: i16 = 175;
pub const KEY_RED: i16 = 176;
pub const KEY_YELLOW: i16 = 177;

#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub enum HudMode {
    ClassicStatusbar,
    FullscreenMini,
    Hidden,
}

#[derive(Component, Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct StatusbarState {
    pub hud_mode: HudMode,
    pub has_blue_key: bool,
    pub has_red_key: bool,
    pub has_yellow_key: bool,
    pub message_text: String,
    pub message_timer: f32,
}

impl Default for StatusbarState {
    fn default() -> Self {
        Self {
            hud_mode: HudMode::ClassicStatusbar,
            has_blue_key: false,
            has_red_key: false,
            has_yellow_key: false,
            message_text: String::new(),
            message_timer: 0.0,
        }
    }
}

pub struct StatusbarLayout {
    pub base_tile: i16,
    pub glyphs: Vec<GlyphDrawCall>,
    pub key_tiles: Vec<(i16, i32, i32)>,
}

impl StatusbarState {
    pub fn compute_layout(&self, player: &PlayerController) -> StatusbarLayout {
        let mut glyphs = Vec::new();
        let mut key_tiles = Vec::new();

        match self.hud_mode {
            HudMode::ClassicStatusbar => {
                // Health (3 digits at x: 24, y: 176)
                let health_str = format!("{:>3}", player.health.clamp(0, 999));
                glyphs.extend(DukeFontRenderer::layout_text(DukeFont::DigitalNumbers, &health_str, 24, 176));

                // Armor (3 digits at x: 64, y: 176)
                let armor_str = format!("{:>3}", player.armor.clamp(0, 999));
                glyphs.extend(DukeFontRenderer::layout_text(DukeFont::DigitalNumbers, &armor_str, 64, 176));

                // Current Weapon Ammo (3 digits at x: 224, y: 176)
                let cur_idx = player.current_weapon as usize;
                let ammo = if cur_idx < player.weapons.len() { player.weapons[cur_idx].ammo } else { 0 };
                let ammo_str = format!("{:>3}", ammo.clamp(0, 999));
                glyphs.extend(DukeFontRenderer::layout_text(DukeFont::DigitalNumbers, &ammo_str, 224, 176));

                // Keycards
                if self.has_blue_key {
                    key_tiles.push((KEY_BLUE, 194, 172));
                }
                if self.has_red_key {
                    key_tiles.push((KEY_RED, 194, 180));
                }
                if self.has_yellow_key {
                    key_tiles.push((KEY_YELLOW, 194, 188));
                }

                // Message text console at top of screen
                if self.message_timer > 0.0 && !self.message_text.is_empty() {
                    glyphs.extend(DukeFontRenderer::layout_text(DukeFont::SmallBlue, &self.message_text, 10, 10));
                }

                StatusbarLayout {
                    base_tile: BOTTOMSTATUSBAR,
                    glyphs,
                    key_tiles,
                }
            }
            HudMode::FullscreenMini => {
                // Minimalist floating numbers
                let health_str = format!("H:{}", player.health);
                glyphs.extend(DukeFontRenderer::layout_text(DukeFont::SmallBlue, &health_str, 10, 185));

                let armor_str = format!("A:{}", player.armor);
                glyphs.extend(DukeFontRenderer::layout_text(DukeFont::SmallBlue, &armor_str, 70, 185));

                let cur_idx = player.current_weapon as usize;
                let ammo = if cur_idx < player.weapons.len() { player.weapons[cur_idx].ammo } else { 0 };
                let ammo_str = format!("AMMO:{}", ammo);
                glyphs.extend(DukeFontRenderer::layout_text(DukeFont::SmallBlue, &ammo_str, 240, 185));

                StatusbarLayout {
                    base_tile: 0, // No full background
                    glyphs,
                    key_tiles,
                }
            }
            HudMode::Hidden => StatusbarLayout {
                base_tile: 0,
                glyphs: Vec::new(),
                key_tiles: Vec::new(),
            },
        }
    }
}
