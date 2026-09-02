#![allow(dead_code)]

use std::collections::HashMap;

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct RtsLump {
    pub name: String,
    pub data: Vec<u8>,
}

#[derive(Debug, Default)]
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
}
