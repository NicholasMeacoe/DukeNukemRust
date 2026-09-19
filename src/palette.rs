#![allow(dead_code)]
use bevy::prelude::*;
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

    /// Compute distance-based depth shade matching Build engine visibility decay
    pub fn compute_depth_shade(base_shade: i8, distance: f32, visibility: f32) -> i8 {
        let decay = (distance * visibility * 0.25) as i32;
        let final_shade = (base_shade as i32 + decay).clamp(0, 31);
        final_shade as i8
    }

    /// Convert Build engine signed shade (-128..127) into a RGBA tint factor [0.0..1.0] for Bevy vertex color modulation.
    pub fn shade_to_tint(shade: i8) -> [f32; 4] {
        // Build shade: 0 is normal, positive is darker
        // Duke 3D's software renderer doesn't darken as aggressively as a linear multiplier.
        // We clamp it to a very forgiving 0.5 so even the darkest sectors are fully visible.
        let factor = (1.0 - (shade as f32 / 128.0)).clamp(0.5, 1.5);
        [factor, factor, factor, 1.0]
    }

    /// Calculate authentic Build engine distance-attenuated shade table index (0..31).
    /// In the original Build engine: `shade = curshade + ((dist * visibility) >> 8)`
    pub fn calculate_build_distance_shade(base_shade: i8, distance_world: f32, visibility: f32) -> i8 {
        let build_dist = distance_world * 100.0;
        let shade_delta = ((build_dist * visibility) / 256.0) as i32;
        (base_shade as i32 + shade_delta).clamp(0, 31) as i8
    }

    /// Convert a 32-level Build shade value into an authentic photometric light multiplier.
    /// In authentic Build, shade 0 is 100% full brightness, shade 31 is near pitch-black (~2-5% light),
    /// and negative shades provide overbrightening (e.g. muzzle flashes and bright lamps).
    pub fn build_shade_to_light_multiplier(shade: i8) -> f32 {
        if shade < 0 {
            (1.0 + (-shade as f32 / 32.0) * 0.5).min(1.75)
        } else {
            let clamped = shade.min(31) as f32;
            let normalized = 1.0 - (clamped / 31.0);
            (normalized.powf(1.4) * 0.97 + 0.03).clamp(0.02, 1.0)
        }
    }

    /// Convert Build shade to authentic 4-component RGBA light tint with true dark levels.
    pub fn authentic_shade_to_tint(shade: i8) -> [f32; 4] {
        let factor = Self::build_shade_to_light_multiplier(shade);
        [factor, factor, factor, 1.0]
    }
}

#[derive(Resource, Debug, Clone, PartialEq)]
pub struct PaletteFlashState {
    pub red_flash: f32,
    pub yellow_flash: f32,
    pub blue_tint: f32,
    pub green_tint: f32,
    pub amber_glow: f32,
}

impl Default for PaletteFlashState {
    fn default() -> Self {
        Self {
            red_flash: 0.0,
            yellow_flash: 0.0,
            blue_tint: 0.0,
            green_tint: 0.0,
            amber_glow: 0.0,
        }
    }
}

impl PaletteFlashState {
    pub fn compute_screen_tint(&self) -> [f32; 4] {
        let r = 1.0 + self.red_flash * 0.8 + self.yellow_flash * 0.4 + self.amber_glow * 0.5;
        let g = 1.0 + self.green_tint * 0.9 + self.yellow_flash * 0.4 + self.amber_glow * 0.3;
        let b = 1.0 + self.blue_tint * 0.8;
        let a = (self.red_flash
            + self.yellow_flash
            + self.blue_tint
            + self.green_tint
            + self.amber_glow)
            .min(0.85);
        [r.clamp(0.0, 2.0), g.clamp(0.0, 2.0), b.clamp(0.0, 2.0), a]
    }

    pub fn tick(&mut self, dt: f32) {
        self.red_flash = (self.red_flash - 2.5 * dt).max(0.0);
        self.yellow_flash = (self.yellow_flash - 3.0 * dt).max(0.0);
        self.amber_glow = (self.amber_glow - 1.5 * dt).max(0.0);
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
        assert!((tint32[0] - 0.75).abs() < 0.05);
    }

    #[test]
    fn test_build_shade_table_clamping() {
        // Negative shade should be overbright (> 1.0)
        let overbright = Palette::build_shade_to_light_multiplier(-16);
        assert!(overbright > 1.0 && overbright <= 1.5, "Expected overbright > 1.0, got {}", overbright);

        // Neutral shade (0) should be 1.0
        let neutral = Palette::build_shade_to_light_multiplier(0);
        assert!((neutral - 1.0).abs() < 0.01, "Expected ~1.0, got {}", neutral);

        // Medium shade (16) should be moderately dark (~0.3-0.5)
        let med = Palette::build_shade_to_light_multiplier(16);
        assert!(med > 0.25 && med < 0.55, "Expected medium shade in [0.25, 0.55], got {}", med);

        // Maximum dark shade (31) and beyond should be near pitch black (<= 0.05)
        let max_dark = Palette::build_shade_to_light_multiplier(31);
        assert!(max_dark <= 0.05, "Expected shade 31 <= 0.05, got {}", max_dark);

        let beyond_dark = Palette::build_shade_to_light_multiplier(64);
        assert_eq!(beyond_dark, max_dark, "Shades beyond 31 should clamp to shade 31 value");
    }

    #[test]
    fn test_build_distance_falloff_formula() {
        let base_shade = 0i8;
        let visibility = 1.0f32; // Normal visibility

        // Distance 0 should yield base shade
        let shade_close = Palette::calculate_build_distance_shade(base_shade, 0.0, visibility);
        assert_eq!(shade_close, 0);

        // Moderate distance (5 meters) should increase shade
        let shade_mid = Palette::calculate_build_distance_shade(base_shade, 5.0, visibility);
        assert!(shade_mid > 0 && shade_mid < 15, "Expected shade_mid in 1..15, got {}", shade_mid);

        // Long distance (100 meters) should hit maximum dark shade (31)
        let shade_far = Palette::calculate_build_distance_shade(base_shade, 100.0, visibility);
        assert_eq!(shade_far, 31);
    }

    #[test]
    fn test_build_authentic_shade_to_tint_levels() {
        let tint_bright = Palette::authentic_shade_to_tint(-10);
        assert!(tint_bright[0] > 1.0);
        assert_eq!(tint_bright[3], 1.0);

        let tint_normal = Palette::authentic_shade_to_tint(0);
        assert!((tint_normal[0] - 1.0).abs() < 0.02);

        let tint_dark = Palette::authentic_shade_to_tint(31);
        assert!(tint_dark[0] <= 0.05);
    }

    #[test]
    fn test_crt_post_process_config() {
        let mut config = CrtPostProcessConfig::default();
        assert!(!config.enabled);
        assert_eq!(config.scanline_intensity, 0.25);

        config.enabled = true;
        config.scanline_intensity = 0.5;
        assert!(config.enabled);
        assert_eq!(config.scanline_intensity, 0.5);
    }
}

#[derive(Resource, Debug, Clone, PartialEq)]
pub struct CrtPostProcessConfig {
    pub enabled: bool,
    pub scanline_intensity: f32,
    pub shadow_mask: bool,
    pub vga_color_quantization: bool,
}

impl Default for CrtPostProcessConfig {
    fn default() -> Self {
        Self {
            enabled: false,
            scanline_intensity: 0.25,
            shadow_mask: true,
            vga_color_quantization: false,
        }
    }
}
