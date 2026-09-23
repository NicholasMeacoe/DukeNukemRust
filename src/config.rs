use bevy::prelude::*;
use serde::{Deserialize, Serialize};
use std::fs::File;
use std::io::{Read, Write};
use std::path::{Path, PathBuf};

#[derive(Resource, Debug, Clone, PartialEq)]
pub struct DukeConfig {
    pub screen_width: u32,
    pub screen_height: u32,
    pub fx_volume: u8,
    pub music_volume: u8,
    pub sound_toggle: bool,
    pub music_toggle: bool,
    pub voice_toggle: bool,
    pub ambience_toggle: bool,
    pub mouse_sensitivity: f32,
    pub mouse_aiming: bool,
    pub invert_mouse: bool,
    pub run_mode: bool,
    pub crosshairs: bool,
}

impl Default for DukeConfig {
    fn default() -> Self {
        Self {
            screen_width: 1280,
            screen_height: 720,
            fx_volume: 192,
            music_volume: 128,
            sound_toggle: true,
            music_toggle: true,
            voice_toggle: true,
            ambience_toggle: true,
            mouse_sensitivity: 1.0,
            mouse_aiming: true,
            invert_mouse: false,
            run_mode: true,
            crosshairs: false,
        }
    }
}

impl DukeConfig {
    pub fn parse_ini(content: &str) -> Self {
        let mut cfg = Self::default();
        let mut current_section = String::new();

        for line in content.lines() {
            let line = line.trim();
            if line.is_empty() || line.starts_with(';') || line.starts_with('#') {
                continue;
            }

            if line.starts_with('[') && line.ends_with(']') {
                current_section = line[1..line.len() - 1].trim().to_lowercase();
                continue;
            }

            if let Some((key, val)) = line.split_once('=') {
                let key = key.trim().to_lowercase();
                let val = val.split(';').next().unwrap_or("").trim();

                match (current_section.as_str(), key.as_str()) {
                    ("screen setup", "screenwidth") => {
                        if let Ok(w) = val.parse::<u32>() {
                            cfg.screen_width = w;
                        }
                    }
                    ("screen setup", "screenheight") => {
                        if let Ok(h) = val.parse::<u32>() {
                            cfg.screen_height = h;
                        }
                    }
                    ("sound setup", "fxvolume") => {
                        if let Ok(v) = val.parse::<u8>() {
                            cfg.fx_volume = v;
                        }
                    }
                    ("sound setup", "musicvolume") => {
                        if let Ok(v) = val.parse::<u8>() {
                            cfg.music_volume = v;
                        }
                    }
                    ("sound setup", "soundtoggle") => {
                        cfg.sound_toggle = val == "1" || val.eq_ignore_ascii_case("true");
                    }
                    ("sound setup", "musictoggle") => {
                        cfg.music_toggle = val == "1" || val.eq_ignore_ascii_case("true");
                    }
                    ("sound setup", "voicetoggle") => {
                        cfg.voice_toggle = val == "1" || val.eq_ignore_ascii_case("true");
                    }
                    ("sound setup", "ambiencetoggle") => {
                        cfg.ambience_toggle = val == "1" || val.eq_ignore_ascii_case("true");
                    }
                    ("controls", "mousesensitivity") => {
                        if let Ok(s) = val.parse::<f32>() {
                            cfg.mouse_sensitivity = s;
                        }
                    }
                    ("controls", "mouseaiming") => {
                        cfg.mouse_aiming = val == "1" || val.eq_ignore_ascii_case("true");
                    }
                    ("controls", "mouseaimingflipped") => {
                        cfg.invert_mouse = val == "1" || val.eq_ignore_ascii_case("true");
                    }
                    ("misc", "runmode") => {
                        cfg.run_mode = val == "1" || val.eq_ignore_ascii_case("true");
                    }
                    ("misc", "crosshairs") => {
                        cfg.crosshairs = val == "1" || val.eq_ignore_ascii_case("true");
                    }
                    _ => {}
                }
            }
        }

        cfg
    }

    pub fn to_ini(&self) -> String {
        format!(
            "[Screen Setup]\n\
            ScreenWidth = {}\n\
            ScreenHeight = {}\n\
            \n\
            [Sound Setup]\n\
            FXVolume = {}\n\
            MusicVolume = {}\n\
            SoundToggle = {}\n\
            MusicToggle = {}\n\
            VoiceToggle = {}\n\
            AmbienceToggle = {}\n\
            \n\
            [Controls]\n\
            MouseSensitivity = {:.2}\n\
            MouseAiming = {}\n\
            MouseAimingFlipped = {}\n\
            \n\
            [Misc]\n\
            RunMode = {}\n\
            Crosshairs = {}\n",
            self.screen_width,
            self.screen_height,
            self.fx_volume,
            self.music_volume,
            if self.sound_toggle { 1 } else { 0 },
            if self.music_toggle { 1 } else { 0 },
            if self.voice_toggle { 1 } else { 0 },
            if self.ambience_toggle { 1 } else { 0 },
            self.mouse_sensitivity,
            if self.mouse_aiming { 1 } else { 0 },
            if self.invert_mouse { 1 } else { 0 },
            if self.run_mode { 1 } else { 0 },
            if self.crosshairs { 1 } else { 0 },
        )
    }
}

pub fn get_default_config_path() -> PathBuf {
    PathBuf::from("duke3d.cfg")
}

pub fn load_config_from_disk(path: &Path) -> DukeConfig {
    if let Ok(mut file) = File::open(path) {
        let mut content = String::new();
        if file.read_to_string(&mut content).is_ok() {
            return DukeConfig::parse_ini(&content);
        }
    }
    DukeConfig::default()
}

pub fn save_config_to_disk(path: &Path, config: &DukeConfig) -> Result<(), std::io::Error> {
    let mut file = File::create(path)?;
    file.write_all(config.to_ini().as_bytes())?;
    file.flush()?;
    Ok(())
}

/// Window display mode setting.
#[derive(Serialize, Deserialize, Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum WindowModeSetting {
    #[default]
    Windowed,
    BorderlessFullscreen,
}

/// Sound volume and audio channel settings.
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub struct SoundConfig {
    pub master_volume: f32,
    pub sfx_volume: f32,
    pub music_volume: f32,
    pub voice_volume: f32,
}

impl Default for SoundConfig {
    fn default() -> Self {
        Self {
            master_volume: 1.0,
            sfx_volume: 1.0,
            music_volume: 0.7,
            voice_volume: 1.0,
        }
    }
}

impl SoundConfig {
    pub fn clamp(&mut self) {
        self.master_volume = self.master_volume.clamp(0.0, 1.0);
        self.sfx_volume = self.sfx_volume.clamp(0.0, 1.0);
        self.music_volume = self.music_volume.clamp(0.0, 1.0);
        self.voice_volume = self.voice_volume.clamp(0.0, 1.0);
    }
}

/// Video rendering and display settings.
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub struct VideoConfig {
    pub crt_enabled: bool,
    pub voxels_enabled: bool,
    pub dynamic_lighting_enabled: bool,
    pub window_mode: WindowModeSetting,
}

impl Default for VideoConfig {
    fn default() -> Self {
        Self {
            crt_enabled: false,
            voxels_enabled: true,
            dynamic_lighting_enabled: true,
            window_mode: WindowModeSetting::Windowed,
        }
    }
}

/// Player gameplay and controls settings.
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub struct ControlsConfig {
    pub mouse_sensitivity: f32,
    pub invert_mouse_y: bool,
    pub auto_switch_weapon: bool,
    pub view_bobbing: bool,
}

impl Default for ControlsConfig {
    fn default() -> Self {
        Self {
            mouse_sensitivity: 1.0,
            invert_mouse_y: false,
            auto_switch_weapon: true,
            view_bobbing: true,
        }
    }
}

impl ControlsConfig {
    pub fn clamp(&mut self) {
        self.mouse_sensitivity = self.mouse_sensitivity.clamp(0.5, 3.0);
    }
}

/// Top-level persistent game configuration.
#[derive(Resource, Serialize, Deserialize, Debug, Clone, PartialEq, Default)]
pub struct GameConfig {
    pub sound: SoundConfig,
    pub video: VideoConfig,
    pub controls: ControlsConfig,
}

impl GameConfig {
    pub const DEFAULT_FILE_NAME: &'static str = "config.json";

    pub fn default_config_path() -> PathBuf {
        PathBuf::from(Self::DEFAULT_FILE_NAME)
    }

    pub fn load_or_default(path: impl AsRef<Path>) -> Self {
        let path = path.as_ref();
        if path.exists() {
            if let Ok(contents) = std::fs::read_to_string(path) {
                match serde_json::from_str::<GameConfig>(&contents) {
                    Ok(mut cfg) => {
                        cfg.sound.clamp();
                        cfg.controls.clamp();
                        return cfg;
                    }
                    Err(e) => {
                        warn!("Failed to parse config file {:?}: {}. Using defaults.", path, e);
                    }
                }
            } else {
                warn!("Failed to read config file {:?}. Using defaults.", path);
            }
        }
        Self::default()
    }

    pub fn save(&self, path: impl AsRef<Path>) -> Result<(), std::io::Error> {
        let path = path.as_ref();
        if let Some(parent) = path.parent() {
            if !parent.as_os_str().is_empty() {
                std::fs::create_dir_all(parent)?;
            }
        }
        let json_str = serde_json::to_string_pretty(self)
            .map_err(|e| std::io::Error::new(std::io::ErrorKind::InvalidData, e))?;
        std::fs::write(path, json_str)?;
        Ok(())
    }
}

pub fn sync_window_mode_system(
    game_config: Option<Res<GameConfig>>,
    mut windows: Query<&mut Window, With<bevy::window::PrimaryWindow>>,
) {
    let Some(config) = game_config else { return };
    if !config.is_changed() {
        return;
    }
    let Ok(mut window) = windows.get_single_mut() else {
        return;
    };
    let target_mode = match config.video.window_mode {
        WindowModeSetting::Windowed => bevy::window::WindowMode::Windowed,
        WindowModeSetting::BorderlessFullscreen => bevy::window::WindowMode::BorderlessFullscreen,
    };
    if window.mode != target_mode {
        window.mode = target_mode;
    }
}

pub struct ConfigPlugin;

impl Plugin for ConfigPlugin {
    fn build(&self, app: &mut App) {
        let game_config = GameConfig::load_or_default(GameConfig::default_config_path());
        app.insert_resource(crate::palette::CrtPostProcessConfig {
            enabled: game_config.video.crt_enabled,
            ..Default::default()
        });
        app.insert_resource(crate::voxel::registry::VoxelConfig {
            enabled: game_config.video.voxels_enabled,
            ..Default::default()
        });
        app.insert_resource(crate::lighting::DynamicLightingConfig {
            enabled: game_config.video.dynamic_lighting_enabled,
            ..Default::default()
        });
        app.insert_resource(game_config);

        let duke_config = load_config_from_disk(&get_default_config_path());
        app.insert_resource(duke_config);

        app.add_systems(Update, sync_window_mode_system);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_config_ini_roundtrip() {
        let mut config = DukeConfig::default();
        config.screen_width = 1920;
        config.screen_height = 1080;
        config.fx_volume = 220;
        config.music_volume = 160;
        config.mouse_sensitivity = 2.5;
        config.invert_mouse = true;
        config.crosshairs = true;

        let ini_str = config.to_ini();
        assert!(ini_str.contains("ScreenWidth = 1920"));
        assert!(ini_str.contains("FXVolume = 220"));
        assert!(ini_str.contains("Crosshairs = 1"));

        let loaded = DukeConfig::parse_ini(&ini_str);
        assert_eq!(loaded.screen_width, 1920);
        assert_eq!(loaded.screen_height, 1080);
        assert_eq!(loaded.fx_volume, 220);
        assert_eq!(loaded.music_volume, 160);
        assert!((loaded.mouse_sensitivity - 2.5).abs() < 0.01);
        assert!(loaded.invert_mouse);
        assert!(loaded.crosshairs);
    }

    #[test]
    fn test_default_game_config_values() {
        let config = GameConfig::default();
        // Sound defaults
        assert_eq!(config.sound.master_volume, 1.0);
        assert_eq!(config.sound.sfx_volume, 1.0);
        assert_eq!(config.sound.music_volume, 0.7);
        assert_eq!(config.sound.voice_volume, 1.0);

        // Video defaults
        assert!(!config.video.crt_enabled);
        assert!(config.video.voxels_enabled);
        assert!(config.video.dynamic_lighting_enabled);
        assert_eq!(config.video.window_mode, WindowModeSetting::Windowed);

        // Controls defaults
        assert_eq!(config.controls.mouse_sensitivity, 1.0);
        assert!(!config.controls.invert_mouse_y);
        assert!(config.controls.auto_switch_weapon);
        assert!(config.controls.view_bobbing);
    }

    #[test]
    fn test_game_config_json_roundtrip() {
        let mut config = GameConfig::default();
        config.sound.master_volume = 0.8;
        config.sound.sfx_volume = 0.6;
        config.sound.music_volume = 0.5;
        config.sound.voice_volume = 0.9;
        config.video.crt_enabled = true;
        config.video.voxels_enabled = false;
        config.video.dynamic_lighting_enabled = false;
        config.video.window_mode = WindowModeSetting::BorderlessFullscreen;
        config.controls.mouse_sensitivity = 2.25;
        config.controls.invert_mouse_y = true;
        config.controls.auto_switch_weapon = false;
        config.controls.view_bobbing = false;

        let json_str = serde_json::to_string_pretty(&config).expect("Serialize to JSON");
        assert!(json_str.contains("\"master_volume\": 0.8"));
        assert!(json_str.contains("\"crt_enabled\": true"));
        assert!(json_str.contains("\"BorderlessFullscreen\""));

        let deserialized: GameConfig = serde_json::from_str(&json_str).expect("Deserialize from JSON");
        assert_eq!(config, deserialized);

        let temp_dir = std::env::temp_dir();
        let temp_file = temp_dir.join(format!("dn3d_test_config_{}.json", std::process::id()));

        // Test save to disk
        config.save(&temp_file).expect("Save config to file");
        assert!(temp_file.exists());

        // Test load from disk
        let loaded = GameConfig::load_or_default(&temp_file);
        assert_eq!(config, loaded);

        // Clean up
        let _ = std::fs::remove_file(&temp_file);
    }

    #[test]
    fn test_game_config_missing_or_corrupted_file_fallback() {
        let non_existent = PathBuf::from("non_existent_config_path_12345.json");
        let fallback = GameConfig::load_or_default(&non_existent);
        assert_eq!(fallback, GameConfig::default());

        let temp_dir = std::env::temp_dir();
        let corrupt_file = temp_dir.join(format!("dn3d_corrupt_config_{}.json", std::process::id()));
        std::fs::write(&corrupt_file, "{ invalid_json: definitely not valid }").expect("Write corrupt file");

        let fallback_corrupt = GameConfig::load_or_default(&corrupt_file);
        assert_eq!(fallback_corrupt, GameConfig::default());

        // Clean up
        let _ = std::fs::remove_file(&corrupt_file);
    }

    #[test]
    fn test_sync_window_mode_system() {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins);
        let mut game_config = GameConfig::default();
        game_config.video.window_mode = WindowModeSetting::BorderlessFullscreen;
        app.insert_resource(game_config);
        app.world_mut().spawn((
            Window {
                mode: bevy::window::WindowMode::Windowed,
                ..default()
            },
            bevy::window::PrimaryWindow,
        ));
        app.add_systems(Update, sync_window_mode_system);
        app.update();

        let mut window_query = app.world_mut().query_filtered::<&Window, With<bevy::window::PrimaryWindow>>();
        let window = window_query.single(app.world());
        assert_eq!(window.mode, bevy::window::WindowMode::BorderlessFullscreen);

        // Mutate back to Windowed
        app.world_mut().resource_mut::<GameConfig>().video.window_mode = WindowModeSetting::Windowed;
        app.update();

        let window = window_query.single(app.world());
        assert_eq!(window.mode, bevy::window::WindowMode::Windowed);
    }
}

