#![allow(dead_code)]
use crate::palette::Palette;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct PicAnm {
    pub num_frames: u8, // bits 0..5 (0..63)
    pub anim_type: u8,  // 0: None, 1: Oscillation, 2: Forward, 3: Backward
    pub x_offset: i8,   // signed horizontal offset
    pub y_offset: i8,   // signed vertical offset
    pub speed: u8,      // clock shift divisor (2^speed ticks at 120Hz)
}

impl PicAnm {
    pub fn from_u32(raw: u32) -> Self {
        let num_frames = (raw & 0x3F) as u8;
        let anim_type = ((raw & 0xC0) >> 6) as u8;
        let x_offset = ((raw >> 8) & 0xFF) as i8;
        let y_offset = ((raw >> 16) & 0xFF) as i8;
        let speed = ((raw >> 24) & 0x0F) as u8;
        Self {
            num_frames,
            anim_type,
            x_offset,
            y_offset,
            speed,
        }
    }

    /// Calculate the frame offset given a 120 Hz engine clock counter
    pub fn get_frame_offset(&self, clock_120hz: u32) -> i32 {
        if self.num_frames == 0 || self.anim_type == 0 {
            return 0;
        }

        let i = clock_120hz >> (self.speed as u32);
        let n = self.num_frames as u32;

        match self.anim_type {
            1 => {
                // Oscillation (ping-pong: 0, 1, ..., n, n-1, ..., 1)
                let period = n * 2;
                if period == 0 {
                    return 0;
                }
                let k = i % period;
                if k <= n {
                    k as i32
                } else {
                    (period - k) as i32
                }
            }
            2 => {
                // Forward loop (0, 1, ..., n, 0, 1, ...)
                (i % (n + 1)) as i32
            }
            3 => {
                // Backward loop (0, -1, ..., -n, 0, -1, ...)
                -((i % (n + 1)) as i32)
            }
            _ => 0,
        }
    }
}

pub struct Tile {
    pub width: u32,
    pub height: u32,
    pub picanm: PicAnm,
    pub data: Vec<u8>, // 8-bit palette indices
}

pub struct Art {
    pub tiles: Vec<Option<Tile>>,
    pub local_tile_start: u32,
    pub local_tile_end: u32,
}

impl Art {
    pub fn from_bytes(data: &[u8]) -> Result<Self, String> {
        if data.len() < 16 {
            return Err("Art data too short".to_string());
        }

        let version = u32::from_le_bytes(data[0..4].try_into().unwrap());
        let _num_tiles = u32::from_le_bytes(data[4..8].try_into().unwrap());
        let local_tile_start = u32::from_le_bytes(data[8..12].try_into().unwrap());
        let local_tile_end = u32::from_le_bytes(data[12..16].try_into().unwrap());

        if version != 1 {
            // Original is version 1
        }

        let num_local_tiles = (local_tile_end - local_tile_start + 1) as usize;
        let mut tiles = Vec::with_capacity(num_local_tiles);

        let mut offset = 16;

        // Headers first
        let mut tile_headers = Vec::with_capacity(num_local_tiles);
        for _ in 0..num_local_tiles {
            let width = u16::from_le_bytes(data[offset..offset + 2].try_into().unwrap()) as u32;
            let height =
                u16::from_le_bytes(data[offset + 2..offset + 4].try_into().unwrap()) as u32;
            let picanm_raw =
                u32::from_le_bytes(data[offset + 4..offset + 8].try_into().unwrap());
            let picanm = PicAnm::from_u32(picanm_raw);
            offset += 8;
            tile_headers.push((width, height, picanm));
        }

        // Data second
        for (width, height, picanm) in tile_headers {
            let size = (width * height) as usize;
            if offset + size > data.len() {
                return Err("Art data truncated".to_string());
            }

            let mut tile_data = vec![0u8; size];
            tile_data.copy_from_slice(&data[offset..offset + size]);
            offset += size;

            tiles.push(Some(Tile {
                width,
                height,
                picanm,
                data: tile_data,
            }));
        }

        Ok(Art {
            tiles,
            local_tile_start,
            local_tile_end,
        })
    }

    // Convert 8-bit column-major tile to 32-bit row-major RGBA texture data
    pub fn get_tile_rgba(
        &self,
        tile_index: u32,
        palette: &[[u8; 4]; 256],
    ) -> Option<(u32, u32, Vec<u8>)> {
        if tile_index < self.local_tile_start || tile_index > self.local_tile_end {
            return None;
        }

        let local_index = (tile_index - self.local_tile_start) as usize;
        let tile = self.tiles.get(local_index)?.as_ref()?;

        let mut rgba_data = vec![0u8; (tile.width * tile.height * 4) as usize];

        for x in 0..tile.width {
            for y in 0..tile.height {
                // Input is column-major: index = x * height + y
                let src_idx = (x * tile.height + y) as usize;
                let pal_idx = tile.data[src_idx] as usize;
                let color = palette[pal_idx];

                // Output is row-major: index = (y * width + x) * 4
                let dst_idx = ((y * tile.width + x) * 4) as usize;
                rgba_data[dst_idx..dst_idx + 4].copy_from_slice(&color);
            }
        }

        Some((tile.width, tile.height, rgba_data))
    }

    // Convert with palette lookup remapping (e.g. pal 1=blue, 2=red, 6=green/nightvision)
    pub fn get_tile_rgba_with_pal(
        &self,
        tile_index: u32,
        palette: &Palette,
        pal_id: u8,
    ) -> Option<(u32, u32, Vec<u8>)> {
        if tile_index < self.local_tile_start || tile_index > self.local_tile_end {
            return None;
        }

        let local_index = (tile_index - self.local_tile_start) as usize;
        let tile = self.tiles.get(local_index)?.as_ref()?;

        let mut rgba_data = vec![0u8; (tile.width * tile.height * 4) as usize];

        for x in 0..tile.width {
            for y in 0..tile.height {
                let src_idx = (x * tile.height + y) as usize;
                let raw_pal_idx = tile.data[src_idx];
                let color = palette.get_color(raw_pal_idx, pal_id, 0);

                let dst_idx = ((y * tile.width + x) * 4) as usize;
                rgba_data[dst_idx..dst_idx + 4].copy_from_slice(&color);
            }
        }

        Some((tile.width, tile.height, rgba_data))
    }

    pub fn get_picanm(&self, tile_index: u32) -> Option<PicAnm> {
        if tile_index < self.local_tile_start || tile_index > self.local_tile_end {
            return None;
        }
        let local_index = (tile_index - self.local_tile_start) as usize;
        Some(self.tiles.get(local_index)?.as_ref()?.picanm)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_picanm_bitfield() {
        // num = 3 (frames 0, 1, 2, 3)
        // type = 2 (forward loop -> 128 -> bits 6..7 = 2)
        // xoffset = 5
        // yoffset = -10
        // speed = 2
        let raw = 3 | (2 << 6) | (5 << 8) | (((-10i8 as u8) as u32) << 16) | (2 << 24);
        let anm = PicAnm::from_u32(raw);

        assert_eq!(anm.num_frames, 3);
        assert_eq!(anm.anim_type, 2);
        assert_eq!(anm.x_offset, 5);
        assert_eq!(anm.y_offset, -10);
        assert_eq!(anm.speed, 2);

        // At speed 2 (shifts clock by 2, so frame advances every 4 ticks)
        assert_eq!(anm.get_frame_offset(0), 0);
        assert_eq!(anm.get_frame_offset(4), 1);
        assert_eq!(anm.get_frame_offset(8), 2);
        assert_eq!(anm.get_frame_offset(12), 3);
        assert_eq!(anm.get_frame_offset(16), 0); // Loops back to 0
    }

    #[test]
    fn test_picanm_oscillation() {
        // Oscillation (type = 1), num = 2 (sequence: 0, 1, 2, 1, 0, 1, 2, ...)
        let raw = 2 | (1 << 6) | (0 << 24); // speed = 0 (1 tick per frame)
        let anm = PicAnm::from_u32(raw);

        assert_eq!(anm.get_frame_offset(0), 0);
        assert_eq!(anm.get_frame_offset(1), 1);
        assert_eq!(anm.get_frame_offset(2), 2);
        assert_eq!(anm.get_frame_offset(3), 1);
        assert_eq!(anm.get_frame_offset(4), 0);
    }
}
