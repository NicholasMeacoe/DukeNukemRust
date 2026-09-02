#![allow(dead_code)]

use crate::game_flow::menu::{CursorAnimTimer, MenuCursor};
use crate::game_flow::state::*;
use bevy::prelude::*;

#[derive(Component)]
pub struct MenuUiRoot;

#[derive(Component)]
pub struct MenuItemText(pub usize);

#[derive(Component)]
pub struct MenuCursorIndicator(pub usize);

#[derive(Component)]
pub struct MenuHeaderTitle;

#[derive(Component)]
pub struct MenuSubheaderText;

#[derive(Component)]
pub struct MenuFooterText;

const DUKE_GOLD: Color = Color::srgb(1.0, 0.82, 0.12);
const DUKE_RED: Color = Color::srgb(0.95, 0.22, 0.15);
const DUKE_GREY: Color = Color::srgb(0.65, 0.65, 0.70);
const DUKE_WHITE: Color = Color::srgb(1.0, 1.0, 1.0);

pub fn setup_menu_ui(mut commands: Commands) {
    commands
        .spawn((
            NodeBundle {
                style: Style {
                    width: Val::Percent(100.0),
                    height: Val::Percent(100.0),
                    position_type: PositionType::Absolute,
                    flex_direction: FlexDirection::Column,
                    align_items: AlignItems::Center,
                    justify_content: JustifyContent::Center,
                    display: Display::Flex,
                    ..default()
                },
                background_color: BackgroundColor(Color::srgba(0.05, 0.05, 0.08, 0.85)),
                ..default()
            },
            MenuUiRoot,
        ))
        .with_children(|parent| {
            // Main Title Header
            parent.spawn((
                TextBundle::from_section(
                    "DUKE NUKEM 3D",
                    TextStyle {
                        font_size: 48.0,
                        color: DUKE_GOLD,
                        ..default()
                    },
                ),
                MenuHeaderTitle,
            ));

            // Subheader (Phase title)
            parent.spawn((
                TextBundle::from_section(
                    "MAIN MENU",
                    TextStyle {
                        font_size: 26.0,
                        color: DUKE_RED,
                        ..default()
                    },
                )
                .with_style(Style {
                    margin: UiRect::new(Val::Px(0.0), Val::Px(0.0), Val::Px(10.0), Val::Px(30.0)),
                    ..default()
                }),
                MenuSubheaderText,
            ));

            // Menu items container (4 rows)
            for i in 0..4 {
                parent
                    .spawn(NodeBundle {
                        style: Style {
                            flex_direction: FlexDirection::Row,
                            align_items: AlignItems::Center,
                            justify_content: JustifyContent::FlexStart,
                            width: Val::Px(450.0),
                            height: Val::Px(42.0),
                            margin: UiRect::vertical(Val::Px(4.0)),
                            ..default()
                        },
                        ..default()
                    })
                    .with_children(|row| {
                        // Cursor symbol
                        row.spawn((
                            TextBundle::from_section(
                                "► ",
                                TextStyle {
                                    font_size: 28.0,
                                    color: DUKE_RED,
                                    ..default()
                                },
                            )
                            .with_style(Style {
                                width: Val::Px(40.0),
                                ..default()
                            }),
                            MenuCursorIndicator(i),
                        ));

                        // Item Text
                        row.spawn((
                            TextBundle::from_section(
                                "",
                                TextStyle {
                                    font_size: 26.0,
                                    color: DUKE_GREY,
                                    ..default()
                                },
                            ),
                            MenuItemText(i),
                        ));
                    });
            }

            // Footer instructions
            parent.spawn((
                TextBundle::from_section(
                    "ARROWS / W/S TO MOVE • ENTER TO SELECT • ESC TO BACK",
                    TextStyle {
                        font_size: 16.0,
                        color: Color::srgb(0.5, 0.5, 0.55),
                        ..default()
                    },
                )
                .with_style(Style {
                    margin: UiRect::top(Val::Px(40.0)),
                    ..default()
                }),
                MenuFooterText,
            ));
        });
}

pub fn update_menu_ui(
    state: Res<State<GamePhase>>,
    cursor: Res<MenuCursor>,
    cursor_anim: Res<CursorAnimTimer>,
    progress: Res<LevelProgress>,
    anim_state: Res<crate::game_flow::intermission::IntermissionAnimationState>,
    mut root_query: Query<(&mut Style, &mut BackgroundColor), With<MenuUiRoot>>,
    mut header_query: Query<
        &mut Text,
        (
            With<MenuHeaderTitle>,
            Without<MenuSubheaderText>,
            Without<MenuItemText>,
            Without<MenuCursorIndicator>,
            Without<MenuFooterText>,
        ),
    >,
    mut subheader_query: Query<
        &mut Text,
        (
            With<MenuSubheaderText>,
            Without<MenuHeaderTitle>,
            Without<MenuItemText>,
            Without<MenuCursorIndicator>,
            Without<MenuFooterText>,
        ),
    >,
    mut footer_query: Query<
        &mut Text,
        (
            With<MenuFooterText>,
            Without<MenuHeaderTitle>,
            Without<MenuSubheaderText>,
            Without<MenuItemText>,
            Without<MenuCursorIndicator>,
        ),
    >,
    mut items_query: Query<
        (&mut Text, &MenuItemText),
        (
            Without<MenuHeaderTitle>,
            Without<MenuSubheaderText>,
            Without<MenuCursorIndicator>,
            Without<MenuFooterText>,
        ),
    >,
    mut cursor_indicators: Query<
        (&mut Text, &MenuCursorIndicator),
        (
            Without<MenuHeaderTitle>,
            Without<MenuSubheaderText>,
            Without<MenuItemText>,
            Without<MenuFooterText>,
        ),
    >,
) {
    let current_phase = *state.get();
    let is_menu_active = current_phase != GamePhase::Playing;

    let Ok((mut root_style, mut root_bg)) = root_query.get_single_mut() else {
        return;
    };

    if !is_menu_active {
        root_style.display = Display::None;
        return;
    }

    root_style.display = Display::Flex;

    // Semi-transparent backdrop for pause/intermission vs dark for main menu
    if current_phase == GamePhase::Paused || current_phase == GamePhase::Intermission {
        root_bg.0 = Color::srgba(0.02, 0.02, 0.04, 0.82);
    } else {
        root_bg.0 = Color::srgba(0.04, 0.04, 0.07, 0.95);
    }

    // Update Header & Subheader
    if let Ok(mut header) = header_query.get_single_mut() {
        header.sections[0].value = match current_phase {
            GamePhase::Intermission => format!(
                "E{}L{}: LEVEL COMPLETED",
                progress.current_episode, progress.current_level
            ),
            GamePhase::Paused => "PAUSED".to_string(),
            _ => "DUKE NUKEM 3D".to_string(),
        };
    }

    if let Ok(mut subheader) = subheader_query.get_single_mut() {
        subheader.sections[0].value = match current_phase {
            GamePhase::MainMenu => "MAIN MENU".to_string(),
            GamePhase::EpisodeSelect => "SELECT AN EPISODE".to_string(),
            GamePhase::SkillSelect => "CHOOSE SKILL LEVEL".to_string(),
            GamePhase::Paused => "OPTIONS & STATUS".to_string(),
            GamePhase::Intermission => "MISSION STATISTICS".to_string(),
            _ => "".to_string(),
        };
    }

    if let Ok(mut footer) = footer_query.get_single_mut() {
        footer.sections[0].value = match current_phase {
            GamePhase::Intermission => "PRESS SPACE OR ENTER TO CONTINUE TO NEXT LEVEL".to_string(),
            GamePhase::Paused => "PRESS ESC TO RESUME • ENTER TO SELECT".to_string(),
            GamePhase::MainMenu => "USE ARROWS / W/S TO MOVE • ENTER TO SELECT".to_string(),
            _ => "USE ARROWS / W/S TO MOVE • ENTER TO SELECT • ESC TO GO BACK".to_string(),
        };
    }

    // Get item labels
    let stats = progress.compute_stats();
    let min = (stats.time_taken_seconds / 60.0) as i32;
    let sec = (stats.time_taken_seconds % 60.0) as i32;
    let par_min = (stats.par_time_seconds / 60.0) as i32;
    let par_sec = (stats.par_time_seconds % 60.0) as i32;

    let item_labels: [&str; 4] = match current_phase {
        GamePhase::MainMenu => ["NEW GAME", "OPTIONS", "LOAD GAME", "QUIT"],
        GamePhase::EpisodeSelect => [
            "1: L.A. MELTDOWN",
            "2: LUNAR APOCALYPSE",
            "3: SHRAPNEL CITY",
            "4: THE PLUTONIUM PAK",
        ],
        GamePhase::SkillSelect => [
            "PIECE OF CAKE",
            "LET'S ROCK",
            "COME GET SOME",
            "DAMN I'M GOOD",
        ],
        GamePhase::Paused => ["RESUME GAME", "OPTIONS", "MAIN MENU", "QUIT TO DESKTOP"],
        GamePhase::Intermission => ["", "", "", ""],
        _ => ["", "", "", ""],
    };

    // Update item texts & colors
    for (mut text, item) in items_query.iter_mut() {
        let idx = item.0;
        if current_phase == GamePhase::Intermission {
            text.sections[0].value = match idx {
                0 => {
                    if anim_state.stage as u8
                        >= crate::game_flow::intermission::IntermissionStage::KillsTally as u8
                    {
                        format!("KILLS:    {:>3}%", anim_state.displayed_kills)
                    } else {
                        "".to_string()
                    }
                }
                1 => {
                    if anim_state.stage as u8
                        >= crate::game_flow::intermission::IntermissionStage::SecretsTally as u8
                    {
                        format!("SECRETS:  {:>3}%", anim_state.displayed_secrets)
                    } else {
                        "".to_string()
                    }
                }
                2 => {
                    if anim_state.stage as u8
                        >= crate::game_flow::intermission::IntermissionStage::TimeReveal as u8
                    {
                        format!(
                            "TIME:     {:02}:{:02} (PAR {:02}:{:02})",
                            min, sec, par_min, par_sec
                        )
                    } else {
                        "".to_string()
                    }
                }
                _ => "".to_string(),
            };
            text.sections[0].style.color = DUKE_WHITE;
        } else if idx < item_labels.len() {
            text.sections[0].value = item_labels[idx].to_string();
            if idx == cursor.selected_index {
                text.sections[0].style.color = DUKE_GOLD;
            } else {
                text.sections[0].style.color = DUKE_GREY;
            }
        }
    }

    // Update animated cursor indicators
    let cursor_frames = ["► ", " ►", "► ", "  ►"];
    let cur_frame = cursor_frames[cursor_anim.frame % cursor_frames.len()];

    for (mut text, indicator) in cursor_indicators.iter_mut() {
        let idx = indicator.0;
        if current_phase != GamePhase::Intermission && idx == cursor.selected_index {
            text.sections[0].value = cur_frame.to_string();
            text.sections[0].style.color = DUKE_RED;
        } else {
            text.sections[0].value = "".to_string();
        }
    }
}
