pub struct Palette {
    pub colors: [[u8; 4]; 256],
}

impl Palette {
    pub fn from_bytes(data: &[u8]) -> Result<Self, String> {
        if data.len() < 768 {
            return Err("Palette data too short".to_string());
        }
        
        let mut colors = [[0u8; 4]; 256];
        for i in 0..256 {
            // Build engine palette values are 0-63
            colors[i][0] = data[i * 3] << 2;
            colors[i][1] = data[i * 3 + 1] << 2;
            colors[i][2] = data[i * 3 + 2] << 2;
            colors[i][3] = if i == 255 { 0 } else { 255 }; // Simple transparency for index 255
        }
        
        Ok(Palette { colors })
    }
}
