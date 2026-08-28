#![allow(dead_code)]

pub mod font;
pub mod statusbar;

pub use font::*;
pub use statusbar::*;

use bevy::prelude::*;

pub struct DukeHudPlugin;

impl Plugin for DukeHudPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Update, toggle_hud_mode);
    }
}

pub fn toggle_hud_mode(
    keys: Res<ButtonInput<KeyCode>>,
    mut query: Query<&mut StatusbarState>,
) {
    if keys.just_pressed(KeyCode::F5) || keys.just_pressed(KeyCode::Minus) {
        for mut sbar in query.iter_mut() {
            sbar.hud_mode = match sbar.hud_mode {
                HudMode::ClassicStatusbar => HudMode::FullscreenMini,
                HudMode::FullscreenMini => HudMode::Hidden,
                HudMode::Hidden => HudMode::ClassicStatusbar,
            };
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::player::types::PlayerController;

    #[test]
    fn test_font_renderer_big_red() {
        let calls = DukeFontRenderer::layout_text(DukeFont::BigRed, "DUKE 3D", 10, 20);
        assert_eq!(calls.len(), 6); // 'D', 'U', 'K', 'E', '3', 'D' (space advances x)
        assert_eq!(calls[0].tile_id, BIGALPHANUM + 3); // 'D'
        assert_eq!(calls[0].x, 10);
        assert_eq!(calls[0].y, 20);
    }

    #[test]
    fn test_font_renderer_digital_numbers() {
        let calls = DukeFontRenderer::layout_text(DukeFont::DigitalNumbers, "100%", 0, 0);
        assert_eq!(calls.len(), 4);
        assert_eq!(calls[0].tile_id, THREE_DIGIT_BASE + 1); // '1'
        assert_eq!(calls[1].tile_id, THREE_DIGIT_BASE + 0); // '0'
        assert_eq!(calls[2].tile_id, THREE_DIGIT_BASE + 0); // '0'
        assert_eq!(calls[3].tile_id, THREE_DIGIT_BASE + 10); // '%'
    }

    #[test]
    fn test_statusbar_layout_computation() {
        let sbar = StatusbarState {
            hud_mode: HudMode::ClassicStatusbar,
            has_blue_key: true,
            has_red_key: false,
            has_yellow_key: true,
            message_text: "FOUND SECRET AREA".into(),
            message_timer: 3.0,
        };

        let player = PlayerController::default();
        let layout = sbar.compute_layout(&player);

        assert_eq!(layout.base_tile, BOTTOMSTATUSBAR);
        assert_eq!(layout.key_tiles.len(), 2); // Blue and Yellow keys
        assert!(layout.glyphs.len() > 10); // Numbers + message
    }
}
