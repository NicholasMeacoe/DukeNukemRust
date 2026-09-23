use bevy::prelude::*;
use bevy::render::camera::Viewport;
pub use crate::player::types::{PlayerCamera, PlayerId};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum SplitscreenLayout {
    #[default]
    SinglePlayer,
    TwoPlayerHorizontal,
    FourPlayerQuadrant,
}

#[derive(Resource, Debug, Clone)]
pub struct SplitscreenConfig {
    pub active_players: usize,
    pub layout: SplitscreenLayout,
}

impl Default for SplitscreenConfig {
    fn default() -> Self {
        Self {
            active_players: 1,
            layout: SplitscreenLayout::SinglePlayer,
        }
    }
}

impl SplitscreenConfig {
    pub fn new(active_players: usize) -> Self {
        let layout = match active_players {
            0 | 1 => SplitscreenLayout::SinglePlayer,
            2 => SplitscreenLayout::TwoPlayerHorizontal,
            _ => SplitscreenLayout::FourPlayerQuadrant,
        };
        Self {
            active_players: active_players.max(1),
            layout,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ViewportRect {
    pub x: u32,
    pub y: u32,
    pub width: u32,
    pub height: u32,
}

impl ViewportRect {
    pub fn to_viewport(&self) -> Viewport {
        Viewport {
            physical_position: UVec2::new(self.x, self.y),
            physical_size: UVec2::new(self.width, self.height),
            depth: 0.0..1.0,
        }
    }
}

pub fn compute_viewport_rect(
    player_idx: usize,
    total_players: usize,
    window_width: u32,
    window_height: u32,
) -> ViewportRect {
    if total_players <= 1 {
        return ViewportRect {
            x: 0,
            y: 0,
            width: window_width,
            height: window_height,
        };
    }

    if total_players == 2 {
        let half_h = window_height / 2;
        if player_idx == 0 {
            // Player 1: Top half
            ViewportRect {
                x: 0,
                y: 0,
                width: window_width,
                height: half_h,
            }
        } else {
            // Player 2: Bottom half
            ViewportRect {
                x: 0,
                y: half_h,
                width: window_width,
                height: window_height.saturating_sub(half_h),
            }
        }
    } else {
        // 3 or 4 players: Quadrants
        let half_w = window_width / 2;
        let half_h = window_height / 2;
        match player_idx {
            0 => ViewportRect {
                x: 0,
                y: 0,
                width: half_w,
                height: half_h,
            },
            1 => ViewportRect {
                x: half_w,
                y: 0,
                width: window_width.saturating_sub(half_w),
                height: half_h,
            },
            2 => ViewportRect {
                x: 0,
                y: half_h,
                width: half_w,
                height: window_height.saturating_sub(half_h),
            },
            _ => ViewportRect {
                x: half_w,
                y: half_h,
                width: window_width.saturating_sub(half_w),
                height: window_height.saturating_sub(half_h),
            },
        }
    }
}

#[derive(Debug, Clone)]
pub struct PlayerInputConfig {
    pub forward: KeyCode,
    pub backward: KeyCode,
    pub strafe_left: KeyCode,
    pub strafe_right: KeyCode,
    pub jump: KeyCode,
    pub crouch: KeyCode,
    pub fire: KeyCode,
    pub use_action: KeyCode,
    pub prev_weapon: KeyCode,
    pub next_weapon: KeyCode,
}

pub fn get_player_input_config(player_id: usize) -> PlayerInputConfig {
    match player_id {
        0 => PlayerInputConfig {
            forward: KeyCode::KeyW,
            backward: KeyCode::KeyS,
            strafe_left: KeyCode::KeyA,
            strafe_right: KeyCode::KeyD,
            jump: KeyCode::Space,
            crouch: KeyCode::KeyC,
            fire: KeyCode::ControlLeft,
            use_action: KeyCode::KeyE,
            prev_weapon: KeyCode::Digit1,
            next_weapon: KeyCode::Digit2,
        },
        1 => PlayerInputConfig {
            forward: KeyCode::ArrowUp,
            backward: KeyCode::ArrowDown,
            strafe_left: KeyCode::ArrowLeft,
            strafe_right: KeyCode::ArrowRight,
            jump: KeyCode::Numpad0,
            crouch: KeyCode::NumpadDecimal,
            fire: KeyCode::ControlRight,
            use_action: KeyCode::Enter,
            prev_weapon: KeyCode::BracketLeft,
            next_weapon: KeyCode::BracketRight,
        },
        2 => PlayerInputConfig {
            forward: KeyCode::KeyI,
            backward: KeyCode::KeyK,
            strafe_left: KeyCode::KeyJ,
            strafe_right: KeyCode::KeyL,
            jump: KeyCode::KeyM,
            crouch: KeyCode::KeyN,
            fire: KeyCode::KeyU,
            use_action: KeyCode::KeyO,
            prev_weapon: KeyCode::Digit8,
            next_weapon: KeyCode::Digit9,
        },
        _ => PlayerInputConfig {
            forward: KeyCode::KeyT,
            backward: KeyCode::KeyG,
            strafe_left: KeyCode::KeyF,
            strafe_right: KeyCode::KeyH,
            jump: KeyCode::KeyB,
            crouch: KeyCode::KeyV,
            fire: KeyCode::KeyR,
            use_action: KeyCode::KeyY,
            prev_weapon: KeyCode::Digit5,
            next_weapon: KeyCode::Digit6,
        },
    }
}

pub fn update_splitscreen_viewports(
    windows: Query<&Window>,
    splitscreen: Option<Res<SplitscreenConfig>>,
    mut camera_query: Query<(&PlayerCamera, &mut Camera)>,
) {
    let Ok(window) = windows.get_single() else {
        return;
    };
    let width = window.physical_width().max(1);
    let height = window.physical_height().max(1);
    let total_players = splitscreen.as_ref().map_or(1, |c| c.active_players);

    for (player_cam, mut camera) in camera_query.iter_mut() {
        let rect = compute_viewport_rect(player_cam.0, total_players, width, height);
        camera.viewport = Some(rect.to_viewport());
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_compute_viewport_rect_single_player() {
        let rect = compute_viewport_rect(0, 1, 1920, 1080);
        assert_eq!(
            rect,
            ViewportRect {
                x: 0,
                y: 0,
                width: 1920,
                height: 1080
            }
        );
    }

    #[test]
    fn test_compute_viewport_rect_two_players_horizontal_split() {
        let p0 = compute_viewport_rect(0, 2, 1920, 1080);
        let p1 = compute_viewport_rect(1, 2, 1920, 1080);

        // Player 0 should be top half
        assert_eq!(
            p0,
            ViewportRect {
                x: 0,
                y: 0,
                width: 1920,
                height: 540
            }
        );
        // Player 1 should be bottom half
        assert_eq!(
            p1,
            ViewportRect {
                x: 0,
                y: 540,
                width: 1920,
                height: 540
            }
        );
    }

    #[test]
    fn test_compute_viewport_rect_four_players_quadrants() {
        let p0 = compute_viewport_rect(0, 4, 1920, 1080);
        let p1 = compute_viewport_rect(1, 4, 1920, 1080);
        let p2 = compute_viewport_rect(2, 4, 1920, 1080);
        let p3 = compute_viewport_rect(3, 4, 1920, 1080);

        assert_eq!(
            p0,
            ViewportRect {
                x: 0,
                y: 0,
                width: 960,
                height: 540
            }
        );
        assert_eq!(
            p1,
            ViewportRect {
                x: 960,
                y: 0,
                width: 960,
                height: 540
            }
        );
        assert_eq!(
            p2,
            ViewportRect {
                x: 0,
                y: 540,
                width: 960,
                height: 540
            }
        );
        assert_eq!(
            p3,
            ViewportRect {
                x: 960,
                y: 540,
                width: 960,
                height: 540
            }
        );
    }

    #[test]
    fn test_player_input_configs_are_distinct() {
        let p0 = get_player_input_config(0);
        let p1 = get_player_input_config(1);

        assert_ne!(p0.forward, p1.forward);
        assert_ne!(p0.backward, p1.backward);
        assert_ne!(p0.strafe_left, p1.strafe_left);
        assert_ne!(p0.strafe_right, p1.strafe_right);
        assert_ne!(p0.fire, p1.fire);
        assert_ne!(p0.jump, p1.jump);
    }

    #[test]
    fn test_camera_and_player_id_association() {
        let mut app = App::new();
        // Spawn 2 players and 2 cameras
        let p0_entity = app.world_mut().spawn(PlayerId(0)).id();
        let p1_entity = app.world_mut().spawn(PlayerId(1)).id();
        let cam0_entity = app.world_mut().spawn(PlayerCamera(0)).id();
        let cam1_entity = app.world_mut().spawn(PlayerCamera(1)).id();

        assert_eq!(app.world().entity(p0_entity).get::<PlayerId>().unwrap().0, 0);
        assert_eq!(app.world().entity(p1_entity).get::<PlayerId>().unwrap().0, 1);
        assert_eq!(app.world().entity(cam0_entity).get::<PlayerCamera>().unwrap().0, 0);
        assert_eq!(app.world().entity(cam1_entity).get::<PlayerCamera>().unwrap().0, 1);
    }

    #[test]
    fn test_splitscreen_multiplayer_weapon_cycling_and_movement() {
        let mut app = App::new();
        app.add_plugins(bevy::time::TimePlugin);
        app.init_resource::<ButtonInput<KeyCode>>();
        app.add_event::<crate::audio::PlaySoundEvent>();
        app.add_systems(Update, crate::player::weapons::handle_weapon_selection);

        // Spawn P1 with Shotgun unlocked
        let mut p1_ctrl = crate::player::PlayerController::default();
        p1_ctrl.weapons[crate::player::WeaponType::Shotgun as usize].is_unlocked = true;
        let p1_entity = app.world_mut().spawn((p1_ctrl, PlayerId(1))).id();

        // Simulate Player 1 pressing ']' (next_weapon)
        let mut keys = app.world_mut().resource_mut::<ButtonInput<KeyCode>>();
        keys.press(KeyCode::BracketRight);

        app.update();

        let updated_p1 = app.world().entity(p1_entity).get::<crate::player::PlayerController>().unwrap();
        // Weapon should have advanced from Pistol (1) to Shotgun (2)
        assert_eq!(updated_p1.current_weapon, crate::player::WeaponType::Shotgun);
    }
}
