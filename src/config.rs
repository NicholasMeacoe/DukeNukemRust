#![allow(dead_code)]

use bevy::prelude::*;
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

pub struct ConfigPlugin;

impl Plugin for ConfigPlugin {
    fn build(&self, app: &mut App) {
        let config = load_config_from_disk(&get_default_config_path());
        app.insert_resource(config);
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
}
