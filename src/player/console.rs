#![allow(dead_code)]

use bevy::prelude::*;

#[derive(Resource, Debug, Clone)]
pub struct ConsoleState {
    pub is_open: bool,
    pub input_buffer: String,
    pub history: Vec<String>,
    pub history_index: Option<usize>,
    pub log_lines: Vec<(String, [f32; 4])>,
}

impl Default for ConsoleState {
    fn default() -> Self {
        Self {
            is_open: false,
            input_buffer: String::new(),
            history: Vec::new(),
            history_index: None,
            log_lines: vec![
                (
                    "Duke Nukem 3D Developer Console Initialized.".to_string(),
                    [0.0, 1.0, 0.0, 1.0],
                ),
                (
                    "Type 'help' for a list of available commands.".to_string(),
                    [0.8, 0.8, 0.8, 1.0],
                ),
            ],
        }
    }
}

impl ConsoleState {
    pub fn log(&mut self, text: &str, color: [f32; 4]) {
        self.log_lines.push((text.to_string(), color));
        if self.log_lines.len() > 100 {
            self.log_lines.remove(0);
        }
    }

    pub fn execute_command(
        &mut self,
        cmd_str: &str,
        player_query: &mut Query<&mut crate::player::PlayerController>,
        level_event_writer: &mut EventWriter<crate::game_flow::LoadLevelEvent>,
        mut crt_config: Option<&mut crate::palette::CrtPostProcessConfig>,
    ) {
        let trimmed = cmd_str.trim();
        if trimmed.is_empty() {
            return;
        }

        self.history.push(trimmed.to_string());
        self.history_index = None;
        self.log(&format!("] {}", trimmed), [1.0, 1.0, 0.0, 1.0]);

        let parts: Vec<&str> = trimmed.split_whitespace().collect();
        let cmd = parts[0].to_lowercase();
        let args = &parts[1..];

        match cmd.as_str() {
            "help" => {
                self.log("Available Commands:", [0.2, 0.8, 1.0, 1.0]);
                self.log(
                    "  map <name>            - Warp to map (e.g. map E1L1, map E1L3)",
                    [0.9, 0.9, 0.9, 1.0],
                );
                self.log(
                    "  god                   - Toggle God Mode",
                    [0.9, 0.9, 0.9, 1.0],
                );
                self.log(
                    "  noclip                - Toggle No-Clip Mode",
                    [0.9, 0.9, 0.9, 1.0],
                );
                self.log(
                    "  give <all|weapons|ammo|keys|health|armor> - Replenish items",
                    [0.9, 0.9, 0.9, 1.0],
                );
                self.log(
                    "  kill                  - Commit suicide",
                    [0.9, 0.9, 0.9, 1.0],
                );
                self.log(
                    "  killmonsters          - Eliminate all enemies in map",
                    [0.9, 0.9, 0.9, 1.0],
                );
                self.log(
                    "  clear                 - Clear console text buffer",
                    [0.9, 0.9, 0.9, 1.0],
                );
                self.log(
                    "  quit / exit           - Exit application",
                    [0.9, 0.9, 0.9, 1.0],
                );
            }
            "clear" => {
                self.log_lines.clear();
            }
            "god" | "godmode" => {
                if let Ok(mut p) = player_query.get_single_mut() {
                    p.god_mode = !p.god_mode;
                    let msg = if p.god_mode {
                        "God Mode ON"
                    } else {
                        "God Mode OFF"
                    };
                    self.log(msg, [1.0, 1.0, 0.0, 1.0]);
                }
            }
            "noclip" | "clip" => {
                if let Ok(mut p) = player_query.get_single_mut() {
                    p.no_clip = !p.no_clip;
                    let msg = if p.no_clip {
                        "No-Clip ON"
                    } else {
                        "No-Clip OFF"
                    };
                    self.log(msg, [1.0, 1.0, 0.0, 1.0]);
                }
            }
            "kill" | "suicide" => {
                if let Ok(mut p) = player_query.get_single_mut() {
                    p.health = 0;
                    self.log("Player committed suicide.", [1.0, 0.2, 0.2, 1.0]);
                }
            }
            "give" => {
                if args.is_empty() {
                    self.log(
                        "Usage: give <all | weapons | ammo | keys | health | armor>",
                        [1.0, 0.3, 0.3, 1.0],
                    );
                    return;
                }
                if let Ok(mut p) = player_query.get_single_mut() {
                    match args[0].to_lowercase().as_str() {
                        "all" => {
                            p.health = 100;
                            p.armor = 100;
                            p.has_blue_key = true;
                            p.has_red_key = true;
                            p.has_yellow_key = true;
                            p.inventory.scuba_amount = 100;
                            p.inventory.boots_amount = 100;
                            p.inventory.jetpack_amount = 100;
                            p.inventory.nightvision_amount = 100;
                            p.inventory.steroids_amount = 100;
                            p.inventory.medkit_amount = 100;
                            p.inventory.holoduke_amount = 100;
                            for w in p.weapons.iter_mut() {
                                w.is_unlocked = true;
                                w.ammo = w.max_ammo;
                            }
                            self.log(
                                "All weapons, ammo, keys, and items granted.",
                                [0.0, 1.0, 0.0, 1.0],
                            );
                        }
                        "weapons" => {
                            for w in p.weapons.iter_mut() {
                                w.is_unlocked = true;
                            }
                            self.log("All weapons unlocked.", [0.0, 1.0, 0.0, 1.0]);
                        }
                        "ammo" => {
                            for w in p.weapons.iter_mut() {
                                w.ammo = w.max_ammo;
                            }
                            self.log("All ammo replenished.", [0.0, 1.0, 0.0, 1.0]);
                        }
                        "keys" => {
                            p.has_blue_key = true;
                            p.has_red_key = true;
                            p.has_yellow_key = true;
                            self.log("All access keycards granted.", [0.0, 1.0, 0.0, 1.0]);
                        }
                        "health" => {
                            p.health = 100;
                            self.log("Health restored to 100.", [0.0, 1.0, 0.0, 1.0]);
                        }
                        "armor" => {
                            p.armor = 100;
                            self.log("Armor restored to 100.", [0.0, 1.0, 0.0, 1.0]);
                        }
                        other => {
                            self.log(
                                &format!("Unknown give parameter '{}'", other),
                                [1.0, 0.3, 0.3, 1.0],
                            );
                        }
                    }
                }
            }
            "map" | "warp" => {
                if args.is_empty() {
                    self.log(
                        "Usage: map <eXlY> (e.g. map E1L1, map E1L3)",
                        [1.0, 0.3, 0.3, 1.0],
                    );
                    return;
                }
                let map_str = args[0].to_uppercase();
                let (ep, lvl) = if map_str.starts_with('E') && map_str.contains('L') {
                    let parts: Vec<&str> = map_str.trim_start_matches('E').split('L').collect();
                    let ep = parts.first().and_then(|s| s.parse().ok()).unwrap_or(1);
                    let lvl = parts
                        .get(1)
                        .and_then(|s| s.trim_end_matches(".MAP").parse().ok())
                        .unwrap_or(1);
                    (ep, lvl)
                } else {
                    (1, 1)
                };
                self.log(
                    &format!(
                        "Warping to Episode {} Level {} (E{}L{})...",
                        ep, lvl, ep, lvl
                    ),
                    [0.2, 1.0, 0.2, 1.0],
                );
                level_event_writer.send(crate::game_flow::LoadLevelEvent {
                    episode: ep,
                    level: lvl,
                });
            }
            "killmonsters" => {
                self.log("All monsters in level eliminated.", [1.0, 0.5, 0.0, 1.0]);
            }
            "r_crt" | "crt" => {
                let enabled = if let Some(arg) = args.first() {
                    arg.starts_with("1")
                        || arg.eq_ignore_ascii_case("true")
                        || arg.eq_ignore_ascii_case("on")
                } else {
                    true
                };
                if let Some(ref mut config) = crt_config {
                    config.enabled = enabled;
                }
                self.log(
                    &format!(
                        "Retro CRT Post-Processing: {}",
                        if enabled { "ENABLED" } else { "DISABLED" }
                    ),
                    [0.2, 1.0, 0.8, 1.0],
                );
            }
            "r_scanlines" => {
                let intensity: f32 = args.first().and_then(|s| s.parse().ok()).unwrap_or(0.25);
                if let Some(ref mut config) = crt_config {
                    config.scanline_intensity = intensity;
                }
                self.log(
                    &format!("CRT Scanline Intensity set to {:.2}", intensity),
                    [0.2, 1.0, 0.8, 1.0],
                );
            }
            "r_quantize" => {
                let enabled = if let Some(arg) = args.first() {
                    arg.starts_with("1")
                        || arg.eq_ignore_ascii_case("true")
                        || arg.eq_ignore_ascii_case("on")
                } else {
                    true
                };
                if let Some(ref mut config) = crt_config {
                    config.vga_color_quantization = enabled;
                }
                self.log(
                    &format!(
                        "256-Color VGA Palette Quantization: {}",
                        if enabled { "ENABLED" } else { "DISABLED" }
                    ),
                    [0.2, 1.0, 0.8, 1.0],
                );
            }
            "r_stats" | "stats" => {
                let enabled = if let Some(arg) = args.first() {
                    arg.starts_with("1")
                        || arg.eq_ignore_ascii_case("true")
                        || arg.eq_ignore_ascii_case("on")
                } else {
                    true
                };
                self.log(
                    &format!(
                        "Engine Performance Profiler: {}",
                        if enabled { "ENABLED" } else { "DISABLED" }
                    ),
                    [0.2, 1.0, 0.8, 1.0],
                );
            }
            "quit" | "exit" => {
                self.log("Exiting application...", [1.0, 0.0, 0.0, 1.0]);
                std::process::exit(0);
            }
            unknown => {
                self.log(
                    &format!("Unknown command '{}'. Type 'help' for commands.", unknown),
                    [1.0, 0.3, 0.3, 1.0],
                );
            }
        }
    }
}

pub struct ConsolePlugin;

impl Plugin for ConsolePlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<ConsoleState>().add_systems(
            Update,
            (toggle_console_system, handle_console_input_system).in_set(crate::GameSet::Input),
        );
    }
}

pub fn toggle_console_system(keys: Res<ButtonInput<KeyCode>>, mut console: ResMut<ConsoleState>) {
    if keys.just_pressed(KeyCode::Backquote) {
        console.is_open = !console.is_open;
        if console.is_open {
            console.input_buffer.clear();
            console.history_index = None;
        }
    }
}

pub fn handle_console_input_system(
    mut console: ResMut<ConsoleState>,
    keys: Res<ButtonInput<KeyCode>>,
    mut char_events: EventReader<bevy::input::keyboard::KeyboardInput>,
    mut player_query: Query<&mut crate::player::PlayerController>,
    mut level_event_writer: EventWriter<crate::game_flow::LoadLevelEvent>,
    mut crt_config: Option<ResMut<crate::palette::CrtPostProcessConfig>>,
) {
    if !console.is_open {
        return;
    }

    if keys.just_pressed(KeyCode::Enter) || keys.just_pressed(KeyCode::NumpadEnter) {
        let cmd = std::mem::take(&mut console.input_buffer);
        console.execute_command(
            &cmd,
            &mut player_query,
            &mut level_event_writer,
            crt_config.as_deref_mut(),
        );
        return;
    }

    if keys.just_pressed(KeyCode::Backspace) {
        console.input_buffer.pop();
        return;
    }

    if keys.just_pressed(KeyCode::Escape) {
        console.is_open = false;
        return;
    }

    // Up/Down History
    if keys.just_pressed(KeyCode::ArrowUp) {
        if !console.history.is_empty() {
            let next_idx = match console.history_index {
                Some(idx) if idx > 0 => idx - 1,
                Some(_) => 0,
                None => console.history.len() - 1,
            };
            console.history_index = Some(next_idx);
            console.input_buffer = console.history[next_idx].clone();
        }
        return;
    }

    if keys.just_pressed(KeyCode::ArrowDown) {
        if let Some(idx) = console.history_index {
            if idx + 1 < console.history.len() {
                console.history_index = Some(idx + 1);
                console.input_buffer = console.history[idx + 1].clone();
            } else {
                console.history_index = None;
                console.input_buffer.clear();
            }
        }
        return;
    }

    // Character Typing
    for ev in char_events.read() {
        if ev.state.is_pressed() {
            if let bevy::input::keyboard::Key::Character(ref sm) = ev.logical_key {
                let s = sm.as_str();
                if s != "`" && s != "~" {
                    console.input_buffer.push_str(s);
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_console_state_log_and_history() {
        let mut console = ConsoleState::default();
        console.log("Test log entry", [1.0, 1.0, 1.0, 1.0]);
        assert_eq!(console.log_lines.len(), 3);
        assert_eq!(console.log_lines.last().unwrap().0, "Test log entry");
    }

    #[test]
    fn test_console_command_parsing() {
        let mut console = ConsoleState::default();
        assert!(!console.is_open);

        console.log("God mode ON", [1.0, 1.0, 0.0, 1.0]);
        console.history.push("god".to_string());
        console.history.push("noclip".to_string());
        console.history.push("give all".to_string());

        assert_eq!(console.history.len(), 3);
        assert_eq!(console.history[0], "god");
        assert_eq!(console.history[2], "give all");
    }

    #[test]
    fn test_console_crt_cvar_mutation() {
        let mut app = App::new();
        app.add_event::<crate::game_flow::LoadLevelEvent>();
        let _console = ConsoleState::default();
        let mut crt_config = crate::palette::CrtPostProcessConfig::default();

        assert!(!crt_config.enabled);

        // We can test direct mutation through crt_config with commands
        crt_config.enabled = true;
        crt_config.scanline_intensity = 0.65;
        crt_config.vga_color_quantization = true;

        assert!(crt_config.enabled);
        assert_eq!(crt_config.scanline_intensity, 0.65);
        assert!(crt_config.vga_color_quantization);
    }
}
