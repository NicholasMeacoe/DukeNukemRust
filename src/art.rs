pub struct Tile {
    pub width: u32,
    pub height: u32,
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
            // Some newer ports might have version 2, but original is 1
            // return Err(format!("Unsupported ART version: {}", version));
        }

        let num_local_tiles = (local_tile_end - local_tile_start + 1) as usize;
        let mut tiles = Vec::with_capacity(num_local_tiles);
        
        let mut offset = 16;
        
        // Headers first
        let mut tile_headers = Vec::with_capacity(num_local_tiles);
        for _ in 0..num_local_tiles {
            let width = u16::from_le_bytes(data[offset..offset+2].try_into().unwrap()) as u32;
            let height = u16::from_le_bytes(data[offset+2..offset+4].try_into().unwrap()) as u32;
            // picanm is 4 bytes, skip for now
            offset += 8;
            tile_headers.push((width, height));
        }
        
        // Data second
        for (width, height) in tile_headers {
            let size = (width * height) as usize;
            if offset + size > data.len() {
                return Err("Art data truncated".to_string());
            }
            
            let mut tile_data = vec![0u8; size];
            tile_data.copy_from_slice(&data[offset..offset+size]);
            offset += size;
            
            tiles.push(Some(Tile {
                width,
                height,
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
    pub fn get_tile_rgba(&self, tile_index: u32, palette: &[[u8; 4]; 256]) -> Option<(u32, u32, Vec<u8>)> {
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
                rgba_data[dst_idx..dst_idx+4].copy_from_slice(&color);
            }
        }
        
        Some((tile.width, tile.height, rgba_data))
    }
}
