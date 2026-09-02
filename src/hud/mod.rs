#![allow(dead_code)]

pub mod automap;
pub mod font;
pub mod statusbar;

pub use automap::*;
pub use font::*;
pub use statusbar::*;

use bevy::prelude::*;

#[derive(Component)]
pub struct HudRoot;

#[derive(Component)]
pub struct HudHealthText;

#[derive(Component)]
pub struct HudArmorText;

#[derive(Component)]
pub struct HudAmmoText;

#[derive(Component)]
pub struct HudMessageText;

#[derive(Component)]
pub struct HudKeysText;

#[derive(Resource, Debug, Clone, PartialEq)]
pub struct EngineProfilerMetrics {
    pub fps: f32,
    pub frame_time_ms: f32,
    pub actor_count: usize,
    pub projectile_count: usize,
    pub decal_count: usize,
    pub is_visible: bool,
}

impl Default for EngineProfilerMetrics {
    fn default() -> Self {
        Self {
            fps: 60.0,
            frame_time_ms: 16.6,
            actor_count: 0,
            projectile_count: 0,
            decal_count: 0,
            is_visible: false,
        }
    }
}

#[derive(Component)]
pub struct ScreenTintOverlay;

#[derive(Resource, Default)]
pub struct ScreenTintState {
    pub current_color: Color,
    pub target_color: Color,
}

pub struct DukeHudPlugin;

impl Plugin for DukeHudPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(automap::AutomapPlugin)
            .init_resource::<EngineProfilerMetrics>()
            .init_resource::<ScreenTintState>()
            .add_systems(Startup, setup_hud_ui)
            .add_systems(
                Update,
                (
                    toggle_hud_mode,
                    update_hud_display,
                    update_profiler_metrics_system,
                    update_screen_tint,
                )
                    .run_if(in_state(crate::game_flow::GamePhase::Playing)),
            );
    }
}

pub fn update_screen_tint(
    time: Res<Time>,
    mut state: ResMut<ScreenTintState>,
    mut query: Query<&mut BackgroundColor, With<ScreenTintOverlay>>,
) {
    // Fade out target color towards transparent
    if state.target_color.alpha() > 0.0 {
        let alpha = (state.target_color.alpha() - time.delta_seconds() * 1.5).max(0.0);
        state.target_color.set_alpha(alpha);
    }

    // Lerp current towards target
    // We can just set current to target since we are fading the target itself
    state.current_color = state.target_color;

    for mut bg in &mut query {
        bg.0 = state.current_color;
    }
}

pub fn update_profiler_metrics_system(
    time: Res<Time>,
    mut profiler: ResMut<EngineProfilerMetrics>,
    actors: Query<&crate::combat::types::EnemyActor>,
    projectiles: Query<&crate::combat::types::Projectile>,
    decals: Query<&crate::combat::decals::SurfaceDecal>,
) {
    let dt = time.delta_seconds();
    if dt > 0.0001 {
        let current_fps = 1.0 / dt;
        profiler.fps = profiler.fps * 0.9 + current_fps * 0.1;
        profiler.frame_time_ms = dt * 1000.0;
    }
    profiler.actor_count = actors.iter().count();
    profiler.projectile_count = projectiles.iter().count();
    profiler.decal_count = decals.iter().count();
}

pub fn setup_hud_ui(mut commands: Commands) {
    // 0. Screen Tint Overlay (Full screen, transparent by default)
    commands.spawn((
        NodeBundle {
            style: Style {
                width: Val::Percent(100.0),
                height: Val::Percent(100.0),
                position_type: PositionType::Absolute,
                ..default()
            },
            background_color: BackgroundColor(Color::NONE),
            z_index: ZIndex::Global(-1), // Behind HUD text but above 3D game
            ..default()
        },
        ScreenTintOverlay,
    ));
    commands
        .spawn((
            NodeBundle {
                style: Style {
                    position_type: PositionType::Absolute,
                    left: Val::Px(0.0),
                    right: Val::Px(0.0),
                    bottom: Val::Px(0.0),
                    height: Val::Px(50.0),
                    justify_content: JustifyContent::SpaceBetween,
                    align_items: AlignItems::Center,
                    padding: UiRect::horizontal(Val::Px(20.0)),
                    display: Display::Flex,
                    ..default()
                },
                background_color: BackgroundColor(Color::srgba(0.05, 0.05, 0.08, 0.85)),
                ..default()
            },
            HudRoot,
        ))
        .with_children(|parent| {
            // Health
            parent.spawn((
                TextBundle::from_section(
                    "HEALTH: 100",
                    TextStyle {
                        font_size: 24.0,
                        color: Color::srgb(1.0, 0.2, 0.2),
                        ..default()
                    },
                ),
                HudHealthText,
            ));

            // Armor
            parent.spawn((
                TextBundle::from_section(
                    "ARMOR: 0",
                    TextStyle {
                        font_size: 24.0,
                        color: Color::srgb(0.2, 0.6, 1.0),
                        ..default()
                    },
                ),
                HudArmorText,
            ));

            // Ammo
            parent.spawn((
                TextBundle::from_section(
                    "AMMO: 48",
                    TextStyle {
                        font_size: 24.0,
                        color: Color::srgb(1.0, 0.9, 0.2),
                        ..default()
                    },
                ),
                HudAmmoText,
            ));

            // Keys
            parent.spawn((
                TextBundle::from_section(
                    "KEYS: -",
                    TextStyle {
                        font_size: 20.0,
                        color: Color::srgb(0.9, 0.9, 0.9),
                        ..default()
                    },
                ),
                HudKeysText,
            ));
        });

    // Top message text
    commands
        .spawn((NodeBundle {
            style: Style {
                position_type: PositionType::Absolute,
                left: Val::Px(20.0),
                top: Val::Px(20.0),
                display: Display::Flex,
                ..default()
            },
            ..default()
        },))
        .with_children(|parent| {
            parent.spawn((
                TextBundle::from_section(
                    "",
                    TextStyle {
                        font_size: 22.0,
                        color: Color::srgb(0.4, 0.8, 1.0),
                        ..default()
                    },
                ),
                HudMessageText,
            ));
        });

    commands.spawn(StatusbarState::default());
}

pub fn update_hud_display(
    mut sbar_query: Query<&mut StatusbarState>,
    player_query: Query<&crate::player::PlayerController>,
    mut health_text: Query<
        &mut Text,
        (
            With<HudHealthText>,
            Without<HudArmorText>,
            Without<HudAmmoText>,
            Without<HudKeysText>,
            Without<HudMessageText>,
        ),
    >,
    mut armor_text: Query<
        &mut Text,
        (
            With<HudArmorText>,
            Without<HudHealthText>,
            Without<HudAmmoText>,
            Without<HudKeysText>,
            Without<HudMessageText>,
        ),
    >,
    mut ammo_text: Query<
        &mut Text,
        (
            With<HudAmmoText>,
            Without<HudHealthText>,
            Without<HudArmorText>,
            Without<HudKeysText>,
            Without<HudMessageText>,
        ),
    >,
    mut keys_text: Query<
        &mut Text,
        (
            With<HudKeysText>,
            Without<HudHealthText>,
            Without<HudArmorText>,
            Without<HudAmmoText>,
            Without<HudMessageText>,
        ),
    >,
    mut msg_text: Query<
        &mut Text,
        (
            With<HudMessageText>,
            Without<HudHealthText>,
            Without<HudArmorText>,
            Without<HudAmmoText>,
            Without<HudKeysText>,
        ),
    >,
    mut hud_root: Query<&mut Style, With<HudRoot>>,
    time: Res<Time>,
) {
    let Ok(player) = player_query.get_single() else {
        return;
    };
    let Ok(mut sbar) = sbar_query.get_single_mut() else {
        return;
    };

    // Sync keys
    sbar.has_blue_key = player.has_blue_key;
    sbar.has_red_key = player.has_red_key;
    sbar.has_yellow_key = player.has_yellow_key;

    if sbar.message_timer > 0.0 {
        sbar.message_timer -= time.delta_seconds();
    }

    if let Ok(mut style) = hud_root.get_single_mut() {
        style.display = match sbar.hud_mode {
            HudMode::ClassicStatusbar | HudMode::FullscreenMini => Display::Flex,
            HudMode::Hidden => Display::None,
        };
    }

    if let Ok(mut txt) = health_text.get_single_mut() {
        txt.sections[0].value = format!("HEALTH: {:>3}", player.health.clamp(0, 999));
    }

    if let Ok(mut txt) = armor_text.get_single_mut() {
        txt.sections[0].value = format!("ARMOR: {:>3}", player.armor.clamp(0, 999));
    }

    if let Ok(mut txt) = ammo_text.get_single_mut() {
        let cur_idx = player.current_weapon as usize;
        let ammo = if cur_idx < player.weapons.len() {
            player.weapons[cur_idx].ammo
        } else {
            0
        };
        txt.sections[0].value = format!("AMMO: {:>3}", ammo.clamp(0, 999));
    }

    if let Ok(mut txt) = keys_text.get_single_mut() {
        let mut keys_str = String::new();
        if player.has_blue_key {
            keys_str.push_str("[B] ");
        }
        if player.has_red_key {
            keys_str.push_str("[R] ");
        }
        if player.has_yellow_key {
            keys_str.push_str("[Y] ");
        }
        if keys_str.is_empty() {
            keys_str = "-".to_string();
        }
        txt.sections[0].value = format!("KEYS: {}", keys_str);
    }

    if let Ok(mut txt) = msg_text.get_single_mut() {
        if sbar.message_timer > 0.0 {
            txt.sections[0].value = sbar.message_text.clone();
        } else {
            txt.sections[0].value.clear();
        }
    }
}

pub fn toggle_hud_mode(keys: Res<ButtonInput<KeyCode>>, mut query: Query<&mut StatusbarState>) {
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

    #[test]
    fn test_engine_profiler_metrics_computation() {
        let mut profiler = EngineProfilerMetrics::default();
        assert_eq!(profiler.fps, 60.0);
        assert!(!profiler.is_visible);

        profiler.actor_count = 14;
        profiler.projectile_count = 5;
        profiler.decal_count = 12;
        profiler.is_visible = true;

        assert_eq!(profiler.actor_count, 14);
        assert_eq!(profiler.projectile_count, 5);
        assert_eq!(profiler.decal_count, 12);
        assert!(profiler.is_visible);
    }
}
