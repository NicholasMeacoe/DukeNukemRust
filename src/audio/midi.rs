#![allow(dead_code)]

use std::collections::HashMap;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum LevelMidiTrack {
    TitleGrabbag,
    E1L1Stalker,
    E1L2Dethtoll,
    E1L3Streets,
    E1L4Watrwrld,
    E1L5Snake1,
    E1L6TheCall,
    E1L7Ahgeez,
}

impl LevelMidiTrack {
    pub fn filename(&self) -> &'static str {
        match self {
            LevelMidiTrack::TitleGrabbag => "GRABBAG.MID",
            LevelMidiTrack::E1L1Stalker => "STALKER.MID",
            LevelMidiTrack::E1L2Dethtoll => "DETHTOLL.MID",
            LevelMidiTrack::E1L3Streets => "STREETS.MID",
            LevelMidiTrack::E1L4Watrwrld => "watrwld1.mid",
            LevelMidiTrack::E1L5Snake1 => "SNAKE1.MID",
            LevelMidiTrack::E1L6TheCall => "THECALL.MID",
            LevelMidiTrack::E1L7Ahgeez => "ahgeez.mid",
        }
    }

    pub fn for_level(episode: usize, level: usize) -> Self {
        match (episode, level) {
            (1, 1) => LevelMidiTrack::E1L1Stalker,
            (1, 2) => LevelMidiTrack::E1L2Dethtoll,
            (1, 3) => LevelMidiTrack::E1L3Streets,
            (1, 4) => LevelMidiTrack::E1L4Watrwrld,
            (1, 5) => LevelMidiTrack::E1L5Snake1,
            (1, 6) => LevelMidiTrack::E1L6TheCall,
            (1, 7) => LevelMidiTrack::E1L7Ahgeez,
            _ => LevelMidiTrack::TitleGrabbag,
        }
    }
}

pub struct MidiPlaylist {
    pub current_track: LevelMidiTrack,
    pub is_playing: bool,
    pub volume: f32,
    pub tracks: HashMap<LevelMidiTrack, Vec<u8>>,
}

impl Default for MidiPlaylist {
    fn default() -> Self {
        Self {
            current_track: LevelMidiTrack::TitleGrabbag,
            is_playing: true,
            volume: 0.8,
            tracks: HashMap::new(),
        }
    }
}

impl MidiPlaylist {
    pub fn play_track(&mut self, track: LevelMidiTrack) {
        self.current_track = track;
        self.is_playing = true;
    }

    pub fn stop(&mut self) {
        self.is_playing = false;
    }
}
