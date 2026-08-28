#![allow(dead_code)]
use std::io::{Read, Cursor};

#[derive(Debug, Clone)]
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

#[derive(Debug, Clone)]
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

#[derive(Debug, Clone)]
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
    reader.read_exact(&mut buf).map(|_| i32::from_le_bytes(buf)).map_err(|e| e.to_string())
}

fn read_i16(reader: &mut Cursor<&[u8]>) -> Result<i16, String> {
    let mut buf = [0u8; 2];
    reader.read_exact(&mut buf).map(|_| i16::from_le_bytes(buf)).map_err(|e| e.to_string())
}

fn read_u8(reader: &mut Cursor<&[u8]>) -> Result<u8, String> {
    let mut buf = [0u8; 1];
    reader.read_exact(&mut buf).map(|_| buf[0]).map_err(|e| e.to_string())
}

fn read_i8(reader: &mut Cursor<&[u8]>) -> Result<i8, String> {
    let mut buf = [0u8; 1];
    reader.read_exact(&mut buf).map(|_| buf[0] as i8).map_err(|e| e.to_string())
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
                hitag: i16::from_le_bytes({let mut b=[0u8;2]; reader.read_exact(&mut b).map_err(|e| e.to_string())?; b}), // inline read for hitag
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
