#![allow(dead_code)]
use std::io::{Cursor, Read};

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct Sector {
    pub wallptr: i16,
    pub wallnum: i16,
    pub ceilingz: i32,
    pub floorz: i32,
    pub ceilingstat: i16,
    pub floorstat: i16,
    pub ceilingpicnum: i16,
    pub ceilingheinum: i16,
    pub ceilingshade: i8,
    pub ceilingpal: u8,
    pub ceilingxpanning: u8,
    pub ceilingypanning: u8,
    pub floorpicnum: i16,
    pub floorheinum: i16,
    pub floorshade: i8,
    pub floorpal: u8,
    pub floorxpanning: u8,
    pub floorypanning: u8,
    pub visibility: u8,
    pub _filler: u8,
    pub lotag: i16,
    pub hitag: i16,
    pub extra: i16,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct Wall {
    pub x: i32,
    pub y: i32,
    pub point2: i16,
    pub nextwall: i16,
    pub nextsector: i16,
    pub cstat: i16,
    pub picnum: i16,
    pub overpicnum: i16,
    pub shade: i8,
    pub pal: u8,
    pub xrepeat: u8,
    pub yrepeat: u8,
    pub xpanning: u8,
    pub ypanning: u8,
    pub lotag: i16,
    pub hitag: i16,
    pub extra: i16,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct Sprite {
    pub x: i32,
    pub y: i32,
    pub z: i32,
    pub cstat: i16,
    pub picnum: i16,
    pub shade: i8,
    pub pal: u8,
    pub clipdist: u8,
    pub _filler: u8,
    pub xrepeat: u8,
    pub yrepeat: u8,
    pub xoffset: i8,
    pub yoffset: i8,
    pub sectnum: i16,
    pub statnum: i16,
    pub ang: i16,
    pub owner: i16,
    pub xvel: i16,
    pub yvel: i16,
    pub zvel: i16,
    pub lotag: i16,
    pub hitag: i16,
    pub extra: i16,
}

#[derive(Debug)]
pub struct Map {
    pub version: i32,
    pub posx: i32,
    pub posy: i32,
    pub posz: i32,
    pub ang: i16,
    pub cursectnum: i16,
    pub sectors: Vec<Sector>,
    pub walls: Vec<Wall>,
    pub sprites: Vec<Sprite>,
}

fn read_i32(reader: &mut Cursor<&[u8]>) -> Result<i32, String> {
    let mut buf = [0u8; 4];
    reader
        .read_exact(&mut buf)
        .map(|_| i32::from_le_bytes(buf))
        .map_err(|e| e.to_string())
}

fn read_i16(reader: &mut Cursor<&[u8]>) -> Result<i16, String> {
    let mut buf = [0u8; 2];
    reader
        .read_exact(&mut buf)
        .map(|_| i16::from_le_bytes(buf))
        .map_err(|e| e.to_string())
}

fn read_u8(reader: &mut Cursor<&[u8]>) -> Result<u8, String> {
    let mut buf = [0u8; 1];
    reader
        .read_exact(&mut buf)
        .map(|_| buf[0])
        .map_err(|e| e.to_string())
}

fn read_i8(reader: &mut Cursor<&[u8]>) -> Result<i8, String> {
    let mut buf = [0u8; 1];
    reader
        .read_exact(&mut buf)
        .map(|_| buf[0] as i8)
        .map_err(|e| e.to_string())
}

impl Map {
    pub fn from_bytes(data: &[u8]) -> Result<Self, String> {
        let mut reader = Cursor::new(data);

        let version = read_i32(&mut reader)?;
        let posx = read_i32(&mut reader)?;
        let posy = read_i32(&mut reader)?;
        let posz = read_i32(&mut reader)?;
        let ang = read_i16(&mut reader)?;
        let cursectnum = read_i16(&mut reader)?;

        let numsectors = read_i16(&mut reader)?;
        if numsectors < 0 || numsectors > 4096 {
            return Err(format!("Invalid sector count: {}", numsectors));
        }
        let mut sectors = Vec::with_capacity(numsectors as usize);
        for _ in 0..numsectors {
            sectors.push(Sector {
                wallptr: read_i16(&mut reader)?,
                wallnum: read_i16(&mut reader)?,
                ceilingz: read_i32(&mut reader)?,
                floorz: read_i32(&mut reader)?,
                ceilingstat: read_i16(&mut reader)?,
                floorstat: read_i16(&mut reader)?,
                ceilingpicnum: read_i16(&mut reader)?,
                ceilingheinum: read_i16(&mut reader)?,
                ceilingshade: read_i8(&mut reader)?,
                ceilingpal: read_u8(&mut reader)?,
                ceilingxpanning: read_u8(&mut reader)?,
                ceilingypanning: read_u8(&mut reader)?,
                floorpicnum: read_i16(&mut reader)?,
                floorheinum: read_i16(&mut reader)?,
                floorshade: read_i8(&mut reader)?,
                floorpal: read_u8(&mut reader)?,
                floorxpanning: read_u8(&mut reader)?,
                floorypanning: read_u8(&mut reader)?,
                visibility: read_u8(&mut reader)?,
                _filler: read_u8(&mut reader)?,
                lotag: read_i16(&mut reader)?,
                hitag: read_i16(&mut reader)?,
                extra: read_i16(&mut reader)?,
            });
        }

        let numwalls = read_i16(&mut reader)?;
        if numwalls < 0 || numwalls > 16384 {
            return Err(format!("Invalid wall count: {}", numwalls));
        }
        let mut walls = Vec::with_capacity(numwalls as usize);
        for _ in 0..numwalls {
            walls.push(Wall {
                x: read_i32(&mut reader)?,
                y: read_i32(&mut reader)?,
                point2: read_i16(&mut reader)?,
                nextwall: read_i16(&mut reader)?,
                nextsector: read_i16(&mut reader)?,
                cstat: read_i16(&mut reader)?,
                picnum: read_i16(&mut reader)?,
                overpicnum: read_i16(&mut reader)?,
                shade: read_i8(&mut reader)?,
                pal: read_u8(&mut reader)?,
                xrepeat: read_u8(&mut reader)?,
                yrepeat: read_u8(&mut reader)?,
                xpanning: read_u8(&mut reader)?,
                ypanning: read_u8(&mut reader)?,
                lotag: read_i16(&mut reader)?,
                hitag: read_i16(&mut reader)?,
                extra: read_i16(&mut reader)?,
            });
        }

        let numsprites = read_i16(&mut reader)?;
        if numsprites < 0 || numsprites > 16384 {
            return Err(format!("Invalid sprite count: {}", numsprites));
        }
        let mut sprites = Vec::with_capacity(numsprites as usize);
        for _ in 0..numsprites {
            sprites.push(Sprite {
                x: read_i32(&mut reader)?,
                y: read_i32(&mut reader)?,
                z: read_i32(&mut reader)?,
                cstat: read_i16(&mut reader)?,
                picnum: read_i16(&mut reader)?,
                shade: read_i8(&mut reader)?,
                pal: read_u8(&mut reader)?,
                clipdist: read_u8(&mut reader)?,
                _filler: read_u8(&mut reader)?,
                xrepeat: read_u8(&mut reader)?,
                yrepeat: read_u8(&mut reader)?,
                xoffset: read_i8(&mut reader)?,
                yoffset: read_i8(&mut reader)?,
                sectnum: read_i16(&mut reader)?,
                statnum: read_i16(&mut reader)?,
                ang: read_i16(&mut reader)?,
                owner: read_i16(&mut reader)?,
                xvel: read_i16(&mut reader)?,
                yvel: read_i16(&mut reader)?,
                zvel: read_i16(&mut reader)?,
                lotag: read_i16(&mut reader)?,
                hitag: i16::from_le_bytes({
                    let mut b = [0u8; 2];
                    reader.read_exact(&mut b).map_err(|e| e.to_string())?;
                    b
                }), // inline read for hitag
                extra: read_i16(&mut reader)?,
            });
        }

        Ok(Map {
            version,
            posx,
            posy,
            posz,
            ang,
            cursectnum,
            sectors,
            walls,
            sprites,
        })
    }
}

impl Sector {
    pub fn is_ceiling_parallax(&self) -> bool {
        (self.ceilingstat & 1) != 0
    }

    pub fn is_floor_parallax(&self) -> bool {
        (self.floorstat & 1) != 0
    }

    pub fn is_ceiling_sloped(&self) -> bool {
        (self.ceilingstat & 2) != 0
    }

    pub fn is_floor_sloped(&self) -> bool {
        (self.floorstat & 2) != 0
    }

    pub fn get_floor_z_at(&self, walls: &[Wall], x: i32, y: i32) -> f32 {
        if !self.is_floor_sloped() || self.wallnum == 0 {
            return self.floorz as f32;
        }
        let wal_idx = self.wallptr as usize;
        if wal_idx >= walls.len() {
            return self.floorz as f32;
        }
        let wal = &walls[wal_idx];
        let wal2_idx = wal.point2 as usize;
        if wal2_idx >= walls.len() {
            return self.floorz as f32;
        }
        let wal2 = &walls[wal2_idx];

        let dx = (wal2.x - wal.x) as f32;
        let dy = (wal2.y - wal.y) as f32;
        let wall_len = (dx * dx + dy * dy).sqrt();
        if wall_len < 0.001 {
            return self.floorz as f32;
        }

        let perp_dist = (dx * ((y - wal.y) as f32) - dy * ((x - wal.x) as f32)) / wall_len;
        let z_offset = (self.floorheinum as f32 * perp_dist) / 256.0;
        self.floorz as f32 + z_offset
    }

    pub fn get_ceiling_z_at(&self, walls: &[Wall], x: i32, y: i32) -> f32 {
        if !self.is_ceiling_sloped() || self.wallnum == 0 {
            return self.ceilingz as f32;
        }
        let wal_idx = self.wallptr as usize;
        if wal_idx >= walls.len() {
            return self.ceilingz as f32;
        }
        let wal = &walls[wal_idx];
        let wal2_idx = wal.point2 as usize;
        if wal2_idx >= walls.len() {
            return self.ceilingz as f32;
        }
        let wal2 = &walls[wal2_idx];

        let dx = (wal2.x - wal.x) as f32;
        let dy = (wal2.y - wal.y) as f32;
        let wall_len = (dx * dx + dy * dy).sqrt();
        if wall_len < 0.001 {
            return self.ceilingz as f32;
        }

        let perp_dist = (dx * ((y - wal.y) as f32) - dy * ((x - wal.x) as f32)) / wall_len;
        let z_offset = (self.ceilingheinum as f32 * perp_dist) / 256.0;
        self.ceilingz as f32 + z_offset
    }

    pub fn get_floor_y_at(&self, walls: &[Wall], x: i32, y: i32) -> f32 {
        -self.get_floor_z_at(walls, x, y) / (1024.0 * 16.0)
    }

    pub fn get_ceiling_y_at(&self, walls: &[Wall], x: i32, y: i32) -> f32 {
        -self.get_ceiling_z_at(walls, x, y) / (1024.0 * 16.0)
    }
}

impl Wall {
    pub fn is_portal(&self) -> bool {
        self.nextsector >= 0 && self.nextwall >= 0
    }

    pub fn is_blocking(&self) -> bool {
        (self.cstat & 1) != 0
    }

    pub fn bottoms_swapped(&self) -> bool {
        (self.cstat & 2) != 0
    }

    pub fn align_bottom(&self) -> bool {
        (self.cstat & 4) != 0
    }

    pub fn is_x_flipped(&self) -> bool {
        (self.cstat & 8) != 0
    }

    pub fn is_masked(&self) -> bool {
        (self.cstat & 16) != 0
    }

    pub fn is_one_way(&self) -> bool {
        (self.cstat & 32) != 0
    }

    pub fn is_translucent(&self) -> bool {
        (self.cstat & 128) != 0
    }

    pub fn is_y_flipped(&self) -> bool {
        (self.cstat & 256) != 0
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_flat_sector_height() {
        let sector = Sector {
            wallptr: 0,
            wallnum: 4,
            ceilingz: -10000,
            floorz: 20000,
            ceilingstat: 0,
            floorstat: 0,
            ceilingpicnum: 0,
            ceilingheinum: 0,
            ceilingshade: 0,
            ceilingpal: 0,
            ceilingxpanning: 0,
            ceilingypanning: 0,
            floorpicnum: 0,
            floorheinum: 0,
            floorshade: 0,
            floorpal: 0,
            floorxpanning: 0,
            floorypanning: 0,
            visibility: 0,
            _filler: 0,
            lotag: 0,
            hitag: 0,
            extra: 0,
        };

        let walls = vec![
            Wall {
                x: 0,
                y: 0,
                point2: 1,
                nextwall: -1,
                nextsector: -1,
                cstat: 0,
                picnum: 0,
                overpicnum: 0,
                shade: 0,
                pal: 0,
                xrepeat: 8,
                yrepeat: 8,
                xpanning: 0,
                ypanning: 0,
                lotag: 0,
                hitag: 0,
                extra: 0,
            },
            Wall {
                x: 1024,
                y: 0,
                point2: 2,
                nextwall: -1,
                nextsector: -1,
                cstat: 0,
                picnum: 0,
                overpicnum: 0,
                shade: 0,
                pal: 0,
                xrepeat: 8,
                yrepeat: 8,
                xpanning: 0,
                ypanning: 0,
                lotag: 0,
                hitag: 0,
                extra: 0,
            },
            Wall {
                x: 1024,
                y: 1024,
                point2: 3,
                nextwall: -1,
                nextsector: -1,
                cstat: 0,
                picnum: 0,
                overpicnum: 0,
                shade: 0,
                pal: 0,
                xrepeat: 8,
                yrepeat: 8,
                xpanning: 0,
                ypanning: 0,
                lotag: 0,
                hitag: 0,
                extra: 0,
            },
            Wall {
                x: 0,
                y: 1024,
                point2: 0,
                nextwall: -1,
                nextsector: -1,
                cstat: 0,
                picnum: 0,
                overpicnum: 0,
                shade: 0,
                pal: 0,
                xrepeat: 8,
                yrepeat: 8,
                xpanning: 0,
                ypanning: 0,
                lotag: 0,
                hitag: 0,
                extra: 0,
            },
        ];

        assert_eq!(sector.get_floor_z_at(&walls, 500, 500), 20000.0);
        assert_eq!(sector.get_ceiling_z_at(&walls, 500, 500), -10000.0);
    }

    #[test]
    fn test_sloped_sector_height() {
        let sector = Sector {
            wallptr: 0,
            wallnum: 4,
            ceilingz: -10000,
            floorz: 0,
            ceilingstat: 0,
            floorstat: 2, // sloped floor
            ceilingpicnum: 0,
            ceilingheinum: 0,
            ceilingshade: 0,
            ceilingpal: 0,
            ceilingxpanning: 0,
            ceilingypanning: 0,
            floorpicnum: 0,
            floorheinum: 1024, // slope gradient
            floorshade: 0,
            floorpal: 0,
            floorxpanning: 0,
            floorypanning: 0,
            visibility: 0,
            _filler: 0,
            lotag: 0,
            hitag: 0,
            extra: 0,
        };

        // First wall along X axis: (0,0) -> (1024, 0)
        let walls = vec![
            Wall {
                x: 0,
                y: 0,
                point2: 1,
                nextwall: -1,
                nextsector: -1,
                cstat: 0,
                picnum: 0,
                overpicnum: 0,
                shade: 0,
                pal: 0,
                xrepeat: 8,
                yrepeat: 8,
                xpanning: 0,
                ypanning: 0,
                lotag: 0,
                hitag: 0,
                extra: 0,
            },
            Wall {
                x: 1024,
                y: 0,
                point2: 2,
                nextwall: -1,
                nextsector: -1,
                cstat: 0,
                picnum: 0,
                overpicnum: 0,
                shade: 0,
                pal: 0,
                xrepeat: 8,
                yrepeat: 8,
                xpanning: 0,
                ypanning: 0,
                lotag: 0,
                hitag: 0,
                extra: 0,
            },
            Wall {
                x: 1024,
                y: 1024,
                point2: 3,
                nextwall: -1,
                nextsector: -1,
                cstat: 0,
                picnum: 0,
                overpicnum: 0,
                shade: 0,
                pal: 0,
                xrepeat: 8,
                yrepeat: 8,
                xpanning: 0,
                ypanning: 0,
                lotag: 0,
                hitag: 0,
                extra: 0,
            },
            Wall {
                x: 0,
                y: 1024,
                point2: 0,
                nextwall: -1,
                nextsector: -1,
                cstat: 0,
                picnum: 0,
                overpicnum: 0,
                shade: 0,
                pal: 0,
                xrepeat: 8,
                yrepeat: 8,
                xpanning: 0,
                ypanning: 0,
                lotag: 0,
                hitag: 0,
                extra: 0,
            },
        ];

        // On the first wall (y = 0), offset is 0
        assert_eq!(sector.get_floor_z_at(&walls, 500, 0), 0.0);
        // At y = 512, perp_dist = 512, z_offset = (1024 * 512) / 256 = 2048
        assert_eq!(sector.get_floor_z_at(&walls, 500, 512), 2048.0);
    }

    #[test]
    fn test_wall_flags() {
        let solid_wall = Wall {
            x: 0,
            y: 0,
            point2: 1,
            nextwall: -1,
            nextsector: -1,
            cstat: 1,
            picnum: 100,
            overpicnum: 0,
            shade: 0,
            pal: 0,
            xrepeat: 8,
            yrepeat: 8,
            xpanning: 0,
            ypanning: 0,
            lotag: 0,
            hitag: 0,
            extra: 0,
        };
        assert!(!solid_wall.is_portal());
        assert!(solid_wall.is_blocking());

        let portal_wall = Wall {
            x: 0,
            y: 0,
            point2: 1,
            nextwall: 5,
            nextsector: 2,
            cstat: 16 | 4,
            picnum: 100,
            overpicnum: 200,
            shade: 0,
            pal: 0,
            xrepeat: 8,
            yrepeat: 8,
            xpanning: 0,
            ypanning: 0,
            lotag: 0,
            hitag: 0,
            extra: 0,
        };
        assert!(portal_wall.is_portal());
        assert!(portal_wall.is_masked());
        assert!(portal_wall.align_bottom());
    }
}
