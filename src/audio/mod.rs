#![allow(dead_code)]

pub mod midi;
pub mod rts;

pub use midi::*;
pub use rts::*;

use bevy::prelude::*;

pub struct DukeAudioPlugin;

impl Plugin for DukeAudioPlugin {
    fn build(&self, _app: &mut App) {
        // Audio playback systems can be registered here
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_midi_track_mapping() {
        assert_eq!(LevelMidiTrack::for_level(1, 1), LevelMidiTrack::E1L1Stalker);
        assert_eq!(LevelMidiTrack::for_level(1, 2), LevelMidiTrack::E1L2Dethtoll);
        assert_eq!(LevelMidiTrack::for_level(1, 3), LevelMidiTrack::E1L3Streets);
        assert_eq!(LevelMidiTrack::for_level(1, 4), LevelMidiTrack::E1L4Watrwrld);
        assert_eq!(LevelMidiTrack::for_level(1, 5), LevelMidiTrack::E1L5Snake1);
        assert_eq!(LevelMidiTrack::for_level(1, 6), LevelMidiTrack::E1L6TheCall);
        assert_eq!(LevelMidiTrack::E1L1Stalker.filename(), "STALKER.MID");
    }

    #[test]
    fn test_rts_wad_parsing() {
        // Construct mock RTS WAD binary
        let mut wad_bytes = Vec::new();
        wad_bytes.extend_from_slice(b"IWAD");
        wad_bytes.extend_from_slice(&1i32.to_le_bytes()); // 1 lump
        wad_bytes.extend_from_slice(&16i32.to_le_bytes()); // infotable at offset 16

        // Mock sound data at offset 12..16 (4 bytes)
        wad_bytes.extend_from_slice(b"TEST");

        // Directory entry at offset 16..32
        wad_bytes.extend_from_slice(&12i32.to_le_bytes()); // filepos = 12
        wad_bytes.extend_from_slice(&4i32.to_le_bytes());  // size = 4
        let mut name = [0u8; 8];
        name[0..4].copy_from_slice(b"DUKE");
        wad_bytes.extend_from_slice(&name);

        let rts = DukeRts::parse(&wad_bytes).expect("Should parse mock RTS");
        let sound = rts.get_sound("DUKE").expect("Should find lump DUKE");
        assert_eq!(sound, b"TEST");
    }
}
