#![allow(dead_code)]

use std::collections::HashMap;

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct RtsLump {
    pub name: String,
    pub data: Vec<u8>,
}

#[derive(Debug, Clone, Default)]
pub struct DukeRts {
    pub lumps: HashMap<String, Vec<u8>>,
}

impl DukeRts {
    pub fn parse(bytes: &[u8]) -> Result<Self, &'static str> {
        if bytes.len() < 12 {
            return Err("RTS file too short");
        }

        // RTS files use standard WAD-style header:
        // [0..4]: Identification (e.g. "IWAD" or "PWAD")
        // [4..8]: numlumps (i32)
        // [8..12]: infotableoffset (i32)
        let numlumps = i32::from_le_bytes(bytes[4..8].try_into().unwrap()) as usize;
        let infotable_offset = i32::from_le_bytes(bytes[8..12].try_into().unwrap()) as usize;

        if infotable_offset + numlumps * 16 > bytes.len() {
            return Err("Corrupt RTS infotable offset");
        }

        let mut lumps = HashMap::with_capacity(numlumps);

        for i in 0..numlumps {
            let entry_offset = infotable_offset + i * 16;
            let filepos =
                i32::from_le_bytes(bytes[entry_offset..entry_offset + 4].try_into().unwrap())
                    as usize;
            let size = i32::from_le_bytes(
                bytes[entry_offset + 4..entry_offset + 8]
                    .try_into()
                    .unwrap(),
            ) as usize;

            let name_bytes = &bytes[entry_offset + 8..entry_offset + 16];
            let name_str = String::from_utf8_lossy(name_bytes)
                .trim_matches('\0')
                .trim()
                .to_uppercase();

            if let Some(end_pos) = filepos.checked_add(size) {
                if end_pos <= bytes.len() {
                    let lump_data = bytes[filepos..end_pos].to_vec();
                    lumps.insert(name_str, lump_data);
                }
            }
        }

        Ok(Self { lumps })
    }

    pub fn get_sound(&self, name: &str) -> Option<&[u8]> {
        self.lumps.get(&name.to_uppercase()).map(|v| v.as_slice())
    }

    /// Sample a random speech taunt lump from parsed DUKE.RTS
    pub fn sample_random_taunt(&self) -> Option<(&str, &[u8])> {
        if self.lumps.is_empty() {
            return None;
        }
        let mut keys: Vec<&String> = self.lumps.keys().collect();
        keys.sort();
        let idx = rand::random::<usize>() % keys.len();
        let key = keys[idx];
        self.lumps.get(key).map(|v| (key.as_str(), v.as_slice()))
    }

    /// List all lump names available in the RTS archive
    pub fn lump_names(&self) -> Vec<String> {
        let mut names: Vec<String> = self.lumps.keys().cloned().collect();
        names.sort();
        names
    }

    /// Helper to convert lump bytes (VOC or WAV) into a Bevy AudioSource
    pub fn lump_to_audio_source(data: &[u8]) -> bevy::audio::AudioSource {
        let wav_bytes = if data.starts_with(b"Creative Voice File\x1A") {
            if let Ok(voc) = crate::audio::voc::VocSound::parse("rts", data) {
                voc.to_wav_bytes()
            } else {
                data.to_vec()
            }
        } else if data.starts_with(b"RIFF") {
            data.to_vec()
        } else {
            data.to_vec()
        };
        bevy::audio::AudioSource {
            bytes: wav_bytes.into(),
        }
    }
}
