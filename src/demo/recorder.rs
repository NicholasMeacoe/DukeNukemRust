#![allow(dead_code)]

use std::fs::File;
use std::io::Write;
use std::path::Path;
use crate::demo::format::*;

#[derive(Debug, Clone, Default, serde::Serialize, serde::Deserialize)]
pub struct DemoRecorder {
    pub is_recording: bool,
    pub episode: u8,
    pub level: u8,
    pub skill: u8,
    pub inputs: Vec<GameticInput>,
}

impl DemoRecorder {
    pub fn start(&mut self, episode: u8, level: u8, skill: u8) {
        self.is_recording = true;
        self.episode = episode;
        self.level = level;
        self.skill = skill;
        self.inputs.clear();
    }

    pub fn record_frame(&mut self, input: GameticInput) {
        if self.is_recording {
            self.inputs.push(input);
        }
    }

    pub fn save_to_file(&mut self, path: &Path) -> Result<(), std::io::Error> {
        self.is_recording = false;
        let header = DemoHeader {
            total_tics: self.inputs.len() as u32,
            version: DEMO_BYTEVERSION,
            episode: self.episode,
            level: self.level,
            skill: self.skill,
        };

        let mut file = File::create(path)?;
        file.write_all(&header.to_bytes())?;
        for input in &self.inputs {
            file.write_all(&input.to_bytes())?;
        }
        file.flush()?;
        Ok(())
    }
}
