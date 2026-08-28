#![allow(dead_code)]
use std::collections::HashMap;

pub struct Palette {
    pub colors: [[u8; 4]; 256],
    pub num_shades: u16,
    pub shade_tables: Vec<[u8; 256]>,
    pub lookups: HashMap<u8, [u8; 256]>,
    pub water_palette: Option<[[u8; 4]; 256]>,
    pub slime_palette: Option<[[u8; 4]; 256]>,
    pub title_palette: Option<[[u8; 4]; 256]>,
}

impl Palette {
    pub fn from_bytes(data: &[u8]) -> Result<Self, String> {
        if data.len() < 768 {
            return Err("Palette data too short".to_string());
        }

        let mut colors = [[0u8; 4]; 256];
        for i in 0..256 {
            // Build engine palette values are 0-63 (6-bit VGA DAC)
            colors[i][0] = ((data[i * 3] as u16 * 255) / 63) as u8;
            colors[i][1] = ((data[i * 3 + 1] as u16 * 255) / 63) as u8;
            colors[i][2] = ((data[i * 3 + 2] as u16 * 255) / 63) as u8;
            colors[i][3] = if i == 255 { 0 } else { 255 }; // Index 255 is transparent
        }

        let mut num_shades = 32;
        let mut shade_tables = Vec::new();

        if data.len() >= 770 {
            num_shades = u16::from_le_bytes([data[768], data[769]]);
            let total_shade_bytes = num_shades as usize * 256;
            if data.len() >= 770 + total_shade_bytes {
                let mut offset = 770;
                for _ in 0..num_shades {
                    let mut table = [0u8; 256];
                    table.copy_from_slice(&data[offset..offset + 256]);
                    shade_tables.push(table);
                    offset += 256;
                }
            }
        }

        Ok(Palette {
            colors,
            num_shades,
            shade_tables,
            lookups: HashMap::new(),
            water_palette: None,
            slime_palette: None,
            title_palette: None,
        })
    }

    pub fn load_lookups(&mut self, data: &[u8]) -> Result<(), String> {
        if data.is_empty() {
            return Err("Lookup data empty".to_string());
        }

        let num_lookups = data[0] as usize;
        let mut offset = 1;

        for _ in 0..num_lookups {
            if offset + 257 > data.len() {
                break;
            }
            let look_pos = data[offset];
            offset += 1;
            let mut remap = [0u8; 256];
            remap.copy_from_slice(&data[offset..offset + 256]);
            offset += 256;
            self.lookups.insert(look_pos, remap);
        }

        // Full-screen palettes (5 x 768 bytes: water, slime, title, drealms, ending)
        if offset + 768 <= data.len() {
            self.water_palette = Some(Self::parse_768(&data[offset..offset + 768]));
            offset += 768;
        }
        if offset + 768 <= data.len() {
            self.slime_palette = Some(Self::parse_768(&data[offset..offset + 768]));
            offset += 768;
        }
        if offset + 768 <= data.len() {
            self.title_palette = Some(Self::parse_768(&data[offset..offset + 768]));
        }

        Ok(())
    }

    fn parse_768(data: &[u8]) -> [[u8; 4]; 256] {
        let mut colors = [[0u8; 4]; 256];
        for i in 0..256 {
            colors[i][0] = ((data[i * 3] as u16 * 255) / 63) as u8;
            colors[i][1] = ((data[i * 3 + 1] as u16 * 255) / 63) as u8;
            colors[i][2] = ((data[i * 3 + 2] as u16 * 255) / 63) as u8;
            colors[i][3] = if i == 255 { 0 } else { 255 };
        }
        colors
    }

    pub fn get_remapped_color_index(&self, color_idx: u8, pal_id: u8) -> u8 {
        if pal_id == 0 {
            color_idx
        } else if let Some(lookup) = self.lookups.get(&pal_id) {
            lookup[color_idx as usize]
        } else {
            color_idx
        }
    }

    pub fn get_color(&self, color_idx: u8, pal_id: u8, shade: i8) -> [u8; 4] {
        if color_idx == 255 {
            return [0, 0, 0, 0];
        }

        let remapped_idx = self.get_remapped_color_index(color_idx, pal_id);

        let shaded_idx = if !self.shade_tables.is_empty() {
            let shade_clamp = shade.clamp(0, (self.num_shades.saturating_sub(1)) as i8) as usize;
            if let Some(table) = self.shade_tables.get(shade_clamp) {
                table[remapped_idx as usize]
            } else {
                remapped_idx
            }
        } else {
            remapped_idx
        };

        self.colors[shaded_idx as usize]
    }

    /// Convert Build engine signed shade (-128..127) into a RGBA tint factor [0.0..1.0] for Bevy vertex color modulation.
    pub fn shade_to_tint(shade: i8) -> [f32; 4] {
        // Build shade: 0 is normal, negative is brighter, positive is darker (up to 32)
        // Shade 0 -> 1.0 intensity, Shade 32 -> 0.05 intensity, Shade -10 -> 1.25 intensity
        let factor = (1.0 - (shade as f32 / 32.0)).clamp(0.05, 1.5);
        [factor, factor, factor, 1.0]
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_palette_parse() {
        let mut data = vec![0u8; 770 + 32 * 256];
        // Set first color to full white in 6-bit DAC: 63, 63, 63
        data[0] = 63;
        data[1] = 63;
        data[2] = 63;

        // numshades = 32
        data[768] = 32;
        data[769] = 0;

        let pal = Palette::from_bytes(&data).unwrap();
        assert_eq!(pal.colors[0], [255, 255, 255, 255]);
        assert_eq!(pal.colors[255][3], 0); // Transparent
        assert_eq!(pal.num_shades, 32);
        assert_eq!(pal.shade_tables.len(), 32);
    }

    #[test]
    fn test_lookups() {
        let mut pal = Palette {
            colors: [[255; 4]; 256],
            num_shades: 32,
            shade_tables: Vec::new(),
            lookups: HashMap::new(),
            water_palette: None,
            slime_palette: None,
            title_palette: None,
        };

        // 1 lookup table for pal_id 1
        let mut lookup_data = vec![1u8]; // numlookups = 1
        lookup_data.push(1); // look_pos = 1
        for i in 0..256 {
            lookup_data.push((255 - i) as u8); // Inverted colors
        }

        pal.load_lookups(&lookup_data).unwrap();
        assert_eq!(pal.get_remapped_color_index(0, 1), 255);
        assert_eq!(pal.get_remapped_color_index(10, 1), 245);
        assert_eq!(pal.get_remapped_color_index(10, 0), 10);
    }

    #[test]
    fn test_shade_to_tint() {
        let tint0 = Palette::shade_to_tint(0);
        assert!((tint0[0] - 1.0).abs() < 0.01);

        let tint32 = Palette::shade_to_tint(32);
        assert!((tint32[0] - 0.05).abs() < 0.05);
    }
}
