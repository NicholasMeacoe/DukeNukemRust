#![allow(dead_code)]

use std::fs::File;
use std::io::Read;
use std::path::Path;
use crate::demo::format::*;

#[derive(Debug, Clone, Default, serde::Serialize, serde::Deserialize)]
pub struct DemoPlayer {
    pub is_playing: bool,
    pub header: Option<DemoHeader>,
    pub inputs: Vec<GameticInput>,
    pub current_tic: usize,
}

impl DemoPlayer {
    pub fn load_from_file(&mut self, path: &Path) -> Result<(), &'static str> {
        let mut file = File::open(path).map_err(|_| "Failed to open demo file")?;
        let mut buffer = Vec::new();
        file.read_to_end(&mut buffer).map_err(|_| "Failed to read demo file")?;

        if buffer.len() < 8 {
            return Err("Demo file too small");
        }

        let header = DemoHeader::from_bytes(&buffer[0..8])?;
        let mut inputs = Vec::with_capacity(header.total_tics as usize);

        let mut offset = 8;
        while offset + 10 <= buffer.len() {
            let input = GameticInput::from_bytes(&buffer[offset..offset + 10])?;
            inputs.push(input);
            offset += 10;
        }

        self.is_playing = true;
        self.header = Some(header);
        self.inputs = inputs;
        self.current_tic = 0;
        Ok(())
    }

    pub fn next_input(&mut self) -> Option<GameticInput> {
        if !self.is_playing {
            return None;
        }
        if self.current_tic < self.inputs.len() {
            let input = self.inputs[self.current_tic];
            self.current_tic += 1;
            Some(input)
        } else {
            self.is_playing = false;
            None
        }
    }

    pub fn stop(&mut self) {
        self.is_playing = false;
        self.current_tic = 0;
    }
}
