#![allow(dead_code)]

use crate::scripting::types::*;

pub struct PhysicsContext<'a> {
    pub sprite_x: &'a mut i32,
    pub sprite_y: &'a mut i32,
    pub sprite_z: &'a mut i32,
    pub sprite_ang: &'a mut i16,
    pub sprite_xvel: &'a mut i16,
    pub sprite_zvel: &'a mut i16,
    pub hitag_flags: i16,
    pub registers: &'a mut ActorRegisters,
    pub player_x: i32,
    pub player_y: i32,
}

pub struct TrigTables {
    pub sin_table: [i32; 2048],
}

impl Default for TrigTables {
    fn default() -> Self {
        Self::new()
    }
}

impl TrigTables {
    pub fn new() -> Self {
        let mut sin_table = [0i32; 2048];
        for i in 0..2048 {
            let rad = (i as f64) * (2.0 * std::f64::consts::PI / 2048.0);
            sin_table[i] = (rad.sin() * 16384.0).round() as i32;
        }
        Self { sin_table }
    }

    pub fn sin(&self, ang: i16) -> i32 {
        self.sin_table[(ang as usize) & 2047]
    }

    pub fn cos(&self, ang: i16) -> i32 {
        self.sin_table[((ang + 512) as usize) & 2047]
    }
}

impl<'a> PhysicsContext<'a> {
    pub fn apply_movement(&mut self, move_def: Option<&MoveDef>, trig: &TrigTables) {
        self.registers.count += 1;
        let flags = self.hitag_flags as i32;

        // 1. Angle alterations
        if (flags & move_flags::FACE_PLAYER) != 0 {
            let dx = self.player_x - *self.sprite_x;
            let dy = self.player_y - *self.sprite_y;
            let goal_ang = get_angle(dx, dy);
            let ang_diff = get_inc_angle(*self.sprite_ang, goal_ang) >> 2;
            if !(ang_diff > -8 && ang_diff < 0) {
                *self.sprite_ang = (*self.sprite_ang + ang_diff) & 2047;
            }
        }

        if (flags & move_flags::SPIN) != 0 {
            let idx = ((self.registers.count << 3) & 2047) as i16;
            *self.sprite_ang = (*self.sprite_ang + (trig.sin(idx) >> 6) as i16) & 2047;
        }

        if (flags & move_flags::JUMP_TO_PLAYER) != 0 && self.registers.count < 16 {
            let sin_idx = (512 + (self.registers.count << 4)) as i16;
            *self.sprite_zvel -= (trig.sin(sin_idx) >> 5) as i16;
        }

        // 2. Velocity smoothing toward move definition
        if let Some(m) = move_def {
            let target_hvel = m.hvel as i16;
            let target_zvel = (m.vvel << 4) as i16;

            let div = if (flags & move_flags::GET_H) != 0 { 2 } else { 5 };
            *self.sprite_xvel += (target_hvel - *self.sprite_xvel) / div;

            if *self.sprite_zvel < 648 {
                let div_z = if (flags & move_flags::GET_V) != 0 { 2 } else { 5 };
                *self.sprite_zvel += (target_zvel - *self.sprite_zvel) / div_z;
            }
        }

        // 3. Compute displacement
        let daxvel = *self.sprite_xvel as i32;
        let ang = *self.sprite_ang;
        let sin_ang = trig.sin(ang);
        let cos_ang = trig.cos(ang);

        let dx = (daxvel * cos_ang) >> 14;
        let dy = (daxvel * sin_ang) >> 14;
        let dz = *self.sprite_zvel as i32;

        *self.sprite_x += dx;
        *self.sprite_y += dy;
        *self.sprite_z += dz;
    }
}

#[inline]
pub fn get_angle(dx: i32, dy: i32) -> i16 {
    (((dy as f64).atan2(dx as f64) * (2048.0 / (2.0 * std::f64::consts::PI))) as i32 & 2047) as i16
}

#[inline]
pub fn get_inc_angle(a: i16, na: i16) -> i16 {
    let mut diff = (na & 2047) - (a & 2047);
    if diff < -1024 {
        diff += 2048;
    } else if diff > 1024 {
        diff -= 2048;
    }
    diff
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_angle_math() {
        // (1, 0) should be angle 0 (East)
        assert_eq!(get_angle(100, 0), 0);
        // (0, 1) should be angle 512 (South in Build 2D coordinates)
        assert_eq!(get_angle(0, 100), 512);
        // (-1, 0) should be angle 1024 (West)
        assert_eq!(get_angle(-100, 0), 1024);
        // (0, -1) should be angle 1536 (North)
        assert_eq!(get_angle(0, -100), 1536);
    }

    #[test]
    fn test_inc_angle() {
        assert_eq!(get_inc_angle(0, 100), 100);
        assert_eq!(get_inc_angle(100, 0), -100);
        // Wrap around 2048
        assert_eq!(get_inc_angle(2000, 50), 98);
        assert_eq!(get_inc_angle(50, 2000), -98);
    }

    #[test]
    fn test_physics_movement() {
        let trig = TrigTables::new();
        let mut reg = ActorRegisters::default();
        let mut x = 0; let mut y = 0; let mut z = 0;
        let mut ang = 0; // East
        let mut xvel = 0; let mut zvel = 0;

        let mut pctx = PhysicsContext {
            sprite_x: &mut x,
            sprite_y: &mut y,
            sprite_z: &mut z,
            sprite_ang: &mut ang,
            sprite_xvel: &mut xvel,
            sprite_zvel: &mut zvel,
            hitag_flags: move_flags::GET_H as i16,
            registers: &mut reg,
            player_x: 1000,
            player_y: 0,
        };

        let move_def = MoveDef { hvel: 100, vvel: 0 };
        pctx.apply_movement(Some(&move_def), &trig);

        // xvel should increase toward 100
        assert!(*pctx.sprite_xvel > 0);
        // x should advance east
        assert!(*pctx.sprite_x > 0);
    }
}
