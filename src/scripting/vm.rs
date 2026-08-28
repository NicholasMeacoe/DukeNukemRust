#![allow(dead_code)]

use crate::scripting::types::*;

pub const MAX_CALL_DEPTH: usize = 64;

pub struct ConVm {
    pub bytecode: Vec<i32>,
    pub actor_script_ptrs: [Option<usize>; MAX_TILES],
    pub actor_types: [u8; MAX_TILES],
}

pub struct VmActorContext<'a> {
    pub sprite_idx: usize,
    pub player_idx: usize,
    pub dist_to_player: i32,
    pub can_see_player: bool,
    pub hit_by_weapon: bool,
    pub registers: &'a mut ActorRegisters,
    pub sprite_x: &'a mut i32,
    pub sprite_y: &'a mut i32,
    pub sprite_z: &'a mut i32,
    pub sprite_ang: &'a mut i16,
    pub sprite_xvel: &'a mut i16,
    pub sprite_zvel: &'a mut i16,
    pub sprite_extra: &'a mut i16,
    pub sprite_picnum: &'a mut i16,
    pub sprite_sectnum: &'a mut i16,
    pub sprite_cstat: &'a mut i16,
    pub sprite_pal: &'a mut u8,
    pub sprite_xrepeat: &'a mut u8,
    pub sprite_yrepeat: &'a mut u8,
    pub sprite_clipdist: &'a mut u8,
    pub sprite_lotag: &'a mut i16,
    pub sprite_hitag: &'a mut i16,
    pub killit_flag: bool,
    pub spawned_sprites: Vec<(i16, i32, i32, i32)>,
    pub sound_events: Vec<(i32, bool)>,
    pub quotes_displayed: Vec<i32>,
    pub pal_flashes: Vec<(i32, i32, i32, i32)>,
    pub player_health_delta: i32,
    pub player_ammo_deltas: Vec<(i32, i32)>,
    pub player_inventory_deltas: Vec<(i32, i32)>,
    pub debris_events: Vec<(i16, i32)>,
    pub hitradius_events: Vec<(i32, i32, i32, i32, i32)>,
}

impl ConVm {
    pub fn new(
        bytecode: Vec<i32>,
        actor_script_ptrs: [Option<usize>; MAX_TILES],
        actor_types: [u8; MAX_TILES],
    ) -> Self {
        Self {
            bytecode,
            actor_script_ptrs,
            actor_types,
        }
    }

    #[inline(always)]
    fn get_word(&self, ip: usize) -> Option<i32> {
        self.bytecode.get(ip).copied()
    }

    /// Executes one tick for an actor.
    pub fn execute(&self, ctx: &mut VmActorContext) {
        let picnum = *ctx.sprite_picnum as usize;
        if picnum >= MAX_TILES {
            return;
        }

        let script_entry = match self.actor_script_ptrs[picnum] {
            Some(entry) => entry,
            None => return,
        };

        // 1. Sector bounds check
        if *ctx.sprite_sectnum < 0 {
            ctx.killit_flag = true;
            return;
        }

        // 2. Action animation timer update (using ActorRegisters::action_delay_timer to protect sprite.lotag)
        if let Some(action_ptr) = ctx.registers.action_ptr {
            if action_ptr + 4 < self.bytecode.len() {
                let num_frames = self.bytecode[action_ptr + 1];
                let inc_val    = self.bytecode[action_ptr + 3];
                let delay      = self.bytecode[action_ptr + 4];

                ctx.registers.action_delay_timer += TICSPERFRAME as i16;
                if (ctx.registers.action_delay_timer as i32) > delay {
                    ctx.registers.action_count += 1;
                    ctx.registers.action_delay_timer = 0;
                    ctx.registers.frame_offset += inc_val;
                }

                if (ctx.registers.frame_offset).abs() >= (num_frames * inc_val).abs() {
                    ctx.registers.frame_offset = 0;
                }
            }
        }

        // 3. Instruction interpretation loop
        let mut ip = script_entry + 4; // Skip 4-word header
        ctx.killit_flag = false;

        let mut call_stack: Vec<usize> = Vec::with_capacity(8);

        while ip < self.bytecode.len() {
            let opcode = Opcode::from(self.bytecode[ip]);

            match opcode {
                // Return from Subroutine (State)
                Opcode::EndS => {
                    if let Some(ret_ip) = call_stack.pop() {
                        ip = ret_ip;
                    } else {
                        break;
                    }
                }

                // Explicit Actor End
                Opcode::EndA => {
                    break;
                }

                // Break: In Duke3D, break halts execution for the current frame tick immediately
                Opcode::Break => {
                    break;
                }

                // Subroutine Invocation
                Opcode::State => {
                    let target_state = self.get_word(ip + 1).unwrap_or(0) as usize;
                    if call_stack.len() < MAX_CALL_DEPTH && target_state > 0 && target_state < self.bytecode.len() {
                        call_stack.push(ip + 2);
                        ip = target_state;
                    } else {
                        // Exceeded call stack or invalid state pointer; step past instruction
                        ip += 2;
                    }
                }

                Opcode::LeftBrace | Opcode::RightBrace | Opcode::NullOp => {
                    ip += 1;
                }

                // ------------------ CONDITIONALS ------------------
                Opcode::IfRnd => {
                    let threshold = self.get_word(ip + 1).unwrap_or(0);
                    let fail_target = self.get_word(ip + 2).unwrap_or(0) as usize;
                    let rand_val = (rand::random::<u16>() >> 8) as i32;
                    let cond = rand_val >= (255 - threshold);
                    self.handle_if_else(cond, &mut ip, 3, fail_target);
                }

                Opcode::IfPDistL => {
                    let max_dist = self.get_word(ip + 1).unwrap_or(0);
                    let fail_target = self.get_word(ip + 2).unwrap_or(0) as usize;
                    let cond = ctx.dist_to_player < max_dist;
                    self.handle_if_else(cond, &mut ip, 3, fail_target);
                }

                Opcode::IfPDistG => {
                    let min_dist = self.get_word(ip + 1).unwrap_or(0);
                    let fail_target = self.get_word(ip + 2).unwrap_or(0) as usize;
                    let cond = ctx.dist_to_player > min_dist;
                    self.handle_if_else(cond, &mut ip, 3, fail_target);
                }

                Opcode::IfCanSee | Opcode::IfCanSeeTarget => {
                    let fail_target = self.get_word(ip + 1).unwrap_or(0) as usize;
                    let cond = ctx.can_see_player;
                    self.handle_if_else(cond, &mut ip, 2, fail_target);
                }

                Opcode::IfHitWeapon => {
                    let fail_target = self.get_word(ip + 1).unwrap_or(0) as usize;
                    let cond = ctx.hit_by_weapon;
                    self.handle_if_else(cond, &mut ip, 2, fail_target);
                }

                Opcode::IfCount => {
                    let target_count = self.get_word(ip + 1).unwrap_or(0);
                    let fail_target = self.get_word(ip + 2).unwrap_or(0) as usize;
                    let cond = ctx.registers.count >= target_count;
                    self.handle_if_else(cond, &mut ip, 3, fail_target);
                }

                Opcode::IfActionCount => {
                    let target_count = self.get_word(ip + 1).unwrap_or(0);
                    let fail_target = self.get_word(ip + 2).unwrap_or(0) as usize;
                    let cond = ctx.registers.action_count >= target_count;
                    self.handle_if_else(cond, &mut ip, 3, fail_target);
                }

                Opcode::IfAction => {
                    let expected_action = self.get_word(ip + 1).unwrap_or(0) as usize;
                    let fail_target = self.get_word(ip + 2).unwrap_or(0) as usize;
                    let cond = ctx.registers.action_ptr == Some(expected_action);
                    self.handle_if_else(cond, &mut ip, 3, fail_target);
                }

                Opcode::IfMove => {
                    let expected_move = self.get_word(ip + 1).unwrap_or(0) as usize;
                    let fail_target = self.get_word(ip + 2).unwrap_or(0) as usize;
                    let cond = ctx.registers.move_ptr == Some(expected_move);
                    self.handle_if_else(cond, &mut ip, 3, fail_target);
                }

                Opcode::IfAi => {
                    let expected_ai = self.get_word(ip + 1).unwrap_or(0) as usize;
                    let fail_target = self.get_word(ip + 2).unwrap_or(0) as usize;
                    let cond = ctx.registers.ai_ptr == Some(expected_ai);
                    self.handle_if_else(cond, &mut ip, 3, fail_target);
                }

                Opcode::IfDead => {
                    let fail_target = self.get_word(ip + 1).unwrap_or(0) as usize;
                    let cond = *ctx.sprite_extra <= 0;
                    self.handle_if_else(cond, &mut ip, 2, fail_target);
                }

                Opcode::IfActor => {
                    let target_picnum = self.get_word(ip + 1).unwrap_or(0) as i16;
                    let fail_target = self.get_word(ip + 2).unwrap_or(0) as usize;
                    let cond = *ctx.sprite_picnum == target_picnum;
                    self.handle_if_else(cond, &mut ip, 3, fail_target);
                }

                Opcode::IfStrength => {
                    let threshold = self.get_word(ip + 1).unwrap_or(0) as i16;
                    let fail_target = self.get_word(ip + 2).unwrap_or(0) as usize;
                    let cond = *ctx.sprite_extra <= threshold;
                    self.handle_if_else(cond, &mut ip, 3, fail_target);
                }

                Opcode::IfFloorDistL => {
                    let max_dist = self.get_word(ip + 1).unwrap_or(0);
                    let fail_target = self.get_word(ip + 2).unwrap_or(0) as usize;
                    let cond = (ctx.registers.floor_z - *ctx.sprite_z) <= (max_dist << 8);
                    self.handle_if_else(cond, &mut ip, 3, fail_target);
                }

                Opcode::IfCeilingDistL => {
                    let max_dist = self.get_word(ip + 1).unwrap_or(0);
                    let fail_target = self.get_word(ip + 2).unwrap_or(0) as usize;
                    let cond = (*ctx.sprite_z - ctx.registers.ceiling_z) <= (max_dist << 8);
                    self.handle_if_else(cond, &mut ip, 3, fail_target);
                }

                Opcode::IfGapZL => {
                    let min_gap = self.get_word(ip + 1).unwrap_or(0);
                    let fail_target = self.get_word(ip + 2).unwrap_or(0) as usize;
                    let cond = ((ctx.registers.floor_z - ctx.registers.ceiling_z) >> 8) < min_gap;
                    self.handle_if_else(cond, &mut ip, 3, fail_target);
                }

                Opcode::IfSpritePal => {
                    let pal = self.get_word(ip + 1).unwrap_or(0) as u8;
                    let fail_target = self.get_word(ip + 2).unwrap_or(0) as usize;
                    let cond = *ctx.sprite_pal == pal;
                    self.handle_if_else(cond, &mut ip, 3, fail_target);
                }

                Opcode::IfPHealthL => {
                    let health = self.get_word(ip + 1).unwrap_or(0);
                    let fail_target = self.get_word(ip + 2).unwrap_or(0) as usize;
                    let cond = health > 0;
                    self.handle_if_else(cond, &mut ip, 3, fail_target);
                }

                Opcode::IfAngDiffL => {
                    let max_diff = self.get_word(ip + 1).unwrap_or(0);
                    let fail_target = self.get_word(ip + 2).unwrap_or(0) as usize;
                    let cond = max_diff > 0;
                    self.handle_if_else(cond, &mut ip, 3, fail_target);
                }

                Opcode::IfP => {
                    let _flags = self.get_word(ip + 1).unwrap_or(0);
                    let fail_target = self.get_word(ip + 2).unwrap_or(0) as usize;
                    let cond = true;
                    self.handle_if_else(cond, &mut ip, 3, fail_target);
                }

                Opcode::IfPInventory => {
                    let _item = self.get_word(ip + 1).unwrap_or(0);
                    let _amount = self.get_word(ip + 2).unwrap_or(0);
                    let fail_target = self.get_word(ip + 3).unwrap_or(0) as usize;
                    let cond = true;
                    self.handle_if_else(cond, &mut ip, 4, fail_target);
                }

                Opcode::IfWasWeapon | Opcode::IfSpawnedBy | Opcode::IfGotWeaponCe => {
                    let fail_target = self.get_word(ip + 2).unwrap_or(0) as usize;
                    self.handle_if_else(false, &mut ip, 3, fail_target);
                }

                Opcode::IfSquished | Opcode::IfOnWater | Opcode::IfInWater | Opcode::IfOutside
                | Opcode::IfMultiplayer | Opcode::IfInSpace | Opcode::IfInOuterSpace | Opcode::IfBulletNear
                | Opcode::IfRespawn | Opcode::IfNotMoving | Opcode::IfAwayFromWall | Opcode::IfNoSounds
                | Opcode::IfHitSpace | Opcode::IfActorNotStayput | Opcode::IfCanShootTarget => {
                    let fail_target = self.get_word(ip + 1).unwrap_or(0) as usize;
                    self.handle_if_else(false, &mut ip, 2, fail_target);
                }

                // ------------------ ACTIONS & COMMANDS ------------------
                Opcode::Action => {
                    let action_ptr = self.get_word(ip + 1).unwrap_or(0) as usize;
                    ctx.registers.action_ptr = Some(action_ptr);
                    ctx.registers.action_count = 0;
                    ctx.registers.frame_offset = 0;
                    ctx.registers.action_delay_timer = 0;
                    ip += 2;
                }

                Opcode::Move => {
                    let move_ptr = self.get_word(ip + 1).unwrap_or(0) as usize;
                    let flags = self.get_word(ip + 2).unwrap_or(0) as i16;
                    ctx.registers.move_ptr = Some(move_ptr);
                    *ctx.sprite_hitag = flags;
                    ctx.registers.count = 0;
                    if (flags as i32 & move_flags::RANDOM_ANGLE) != 0 {
                        *ctx.sprite_ang = (rand::random::<u16>() & 2047) as i16;
                    }
                    ip += 3;
                }

                Opcode::Ai => {
                    let ai_ptr = self.get_word(ip + 1).unwrap_or(0) as usize;
                    ctx.registers.ai_ptr = Some(ai_ptr);
                    if ai_ptr + 2 < self.bytecode.len() {
                        ctx.registers.action_ptr = Some(self.bytecode[ai_ptr] as usize);
                        ctx.registers.move_ptr   = Some(self.bytecode[ai_ptr + 1] as usize);
                        let flags = self.bytecode[ai_ptr + 2] as i16;
                        *ctx.sprite_hitag = flags;
                        ctx.registers.count = 0;
                        ctx.registers.action_count = 0;
                        ctx.registers.frame_offset = 0;
                        ctx.registers.action_delay_timer = 0;
                        if (flags as i32 & move_flags::RANDOM_ANGLE) != 0 {
                            *ctx.sprite_ang = (rand::random::<u16>() & 2047) as i16;
                        }
                    }
                    ip += 2;
                }

                Opcode::Count => {
                    ctx.registers.count = self.get_word(ip + 1).unwrap_or(0);
                    ip += 2;
                }

                Opcode::ResetCount => {
                    ctx.registers.count = 0;
                    ip += 1;
                }

                Opcode::ResetActionCount => {
                    ctx.registers.action_count = 0;
                    ip += 1;
                }

                Opcode::Strength => {
                    *ctx.sprite_extra = self.get_word(ip + 1).unwrap_or(0) as i16;
                    ip += 2;
                }

                Opcode::AddStrength => {
                    *ctx.sprite_extra += self.get_word(ip + 1).unwrap_or(0) as i16;
                    ip += 2;
                }

                Opcode::AddPHealth => {
                    let health = self.get_word(ip + 1).unwrap_or(0);
                    ctx.player_health_delta += health;
                    ip += 2;
                }

                Opcode::AddAmmo => {
                    let weapon = self.get_word(ip + 1).unwrap_or(0);
                    let amt = self.get_word(ip + 2).unwrap_or(0);
                    ctx.player_ammo_deltas.push((weapon, amt));
                    ip += 3;
                }

                Opcode::AddWeapon => {
                    let weapon = self.get_word(ip + 1).unwrap_or(0);
                    let ammo = self.get_word(ip + 2).unwrap_or(0);
                    ctx.player_ammo_deltas.push((weapon, ammo));
                    ip += 3;
                }

                Opcode::AddInventory => {
                    let item = self.get_word(ip + 1).unwrap_or(0);
                    let amt = self.get_word(ip + 2).unwrap_or(0);
                    ctx.player_inventory_deltas.push((item, amt));
                    ip += 3;
                }

                Opcode::CStat => {
                    *ctx.sprite_cstat = self.get_word(ip + 1).unwrap_or(0) as i16;
                    ip += 2;
                }

                Opcode::CStatOr => {
                    *ctx.sprite_cstat |= self.get_word(ip + 1).unwrap_or(0) as i16;
                    ip += 2;
                }

                Opcode::ClipDist => {
                    *ctx.sprite_clipdist = self.get_word(ip + 1).unwrap_or(0) as u8;
                    ip += 2;
                }

                Opcode::SpritePal => {
                    *ctx.sprite_pal = self.get_word(ip + 1).unwrap_or(0) as u8;
                    ip += 2;
                }

                Opcode::CActor => {
                    *ctx.sprite_picnum = self.get_word(ip + 1).unwrap_or(0) as i16;
                    ip += 2;
                }

                Opcode::Sound => {
                    let sound_id = self.get_word(ip + 1).unwrap_or(0);
                    ctx.sound_events.push((sound_id, false));
                    ip += 2;
                }

                Opcode::SoundOnce => {
                    let sound_id = self.get_word(ip + 1).unwrap_or(0);
                    ctx.sound_events.push((sound_id, true));
                    ip += 2;
                }

                Opcode::StopSound | Opcode::GlobalSound => {
                    let sound_id = self.get_word(ip + 1).unwrap_or(0);
                    ctx.sound_events.push((sound_id, false));
                    ip += 2;
                }

                Opcode::Spawn => {
                    let tile = self.get_word(ip + 1).unwrap_or(0) as i16;
                    ctx.spawned_sprites.push((tile, *ctx.sprite_x, *ctx.sprite_y, *ctx.sprite_z));
                    ip += 2;
                }

                Opcode::Shoot => {
                    let tile = self.get_word(ip + 1).unwrap_or(0) as i16;
                    ctx.spawned_sprites.push((tile, *ctx.sprite_x, *ctx.sprite_y, *ctx.sprite_z));
                    ip += 2;
                }

                Opcode::Quote => {
                    let quote_id = self.get_word(ip + 1).unwrap_or(0);
                    ctx.quotes_displayed.push(quote_id);
                    ip += 2;
                }

                Opcode::Debris | Opcode::Guts => {
                    let tile = self.get_word(ip + 1).unwrap_or(0) as i16;
                    let count = self.get_word(ip + 2).unwrap_or(0);
                    ctx.debris_events.push((tile, count));
                    ip += 3;
                }

                Opcode::SizeTo | Opcode::SizeAt => {
                    let xr = self.get_word(ip + 1).unwrap_or(64) as u8;
                    let yr = self.get_word(ip + 2).unwrap_or(64) as u8;
                    *ctx.sprite_xrepeat = xr;
                    *ctx.sprite_yrepeat = yr;
                    ip += 3;
                }

                Opcode::PalFrom => {
                    let time = self.get_word(ip + 1).unwrap_or(0);
                    let r = self.get_word(ip + 2).unwrap_or(0);
                    let g = self.get_word(ip + 3).unwrap_or(0);
                    let b = self.get_word(ip + 4).unwrap_or(0);
                    ctx.pal_flashes.push((time, r, g, b));
                    ip += 5;
                }

                Opcode::HitRadius => {
                    let r = self.get_word(ip + 1).unwrap_or(0);
                    let d1 = self.get_word(ip + 2).unwrap_or(0);
                    let d2 = self.get_word(ip + 3).unwrap_or(0);
                    let d3 = self.get_word(ip + 4).unwrap_or(0);
                    let d4 = self.get_word(ip + 5).unwrap_or(0);
                    ctx.hitradius_events.push((r, d1, d2, d3, d4));
                    ip += 6;
                }

                Opcode::Money | Opcode::Mail | Opcode::Paper | Opcode::LotsOfGlass => {
                    let count = self.get_word(ip + 1).unwrap_or(0);
                    ctx.debris_events.push((*ctx.sprite_picnum, count));
                    ip += 2;
                }

                Opcode::SleepTime | Opcode::AddKills | Opcode::EndOfGame | Opcode::Debug => {
                    ip += 2;
                }

                Opcode::Fall | Opcode::ResetPlayer | Opcode::PStomp | Opcode::WackPlayer
                | Opcode::Operate | Opcode::RespawnHitag | Opcode::Tip | Opcode::GetLastPal
                | Opcode::PKick | Opcode::MikeSnd | Opcode::TossWeapon => {
                    ip += 1;
                }

                Opcode::Killit => {
                    ctx.killit_flag = true;
                    break;
                }

                Opcode::Else => {
                    let else_skip_target = self.get_word(ip + 1).unwrap_or(0) as usize;
                    ip = else_skip_target;
                }

                _ => {
                    ip += 1;
                }
            }
        }
    }

    #[inline(always)]
    fn handle_if_else(&self, condition: bool, ip: &mut usize, advance_if_true: usize, fail_target: usize) {
        if condition {
            *ip += advance_if_true;
        } else {
            *ip = fail_target;
            if *ip < self.bytecode.len() && self.bytecode[*ip] == (Opcode::Else as i32) {
                *ip += 2;
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::scripting::compiler::Compiler;

    #[test]
    fn test_vm_execution_death_and_killit() {
        let script = r#"
            define TROOP 1680
            actor TROOP 100
                ifdead
                    killit
                else
                    addstrength -10
            enda
        "#;

        let mut compiler = Compiler::new();
        let compiled = compiler.compile(script).unwrap();
        let vm = ConVm::new(compiled.bytecode, compiled.actor_script_ptrs, compiled.actor_types);

        let mut reg = ActorRegisters::default();
        let mut x = 0; let mut y = 0; let mut z = 0;
        let mut ang = 0; let mut xvel = 0; let mut zvel = 0;
        let mut extra = 100; // Alive
        let mut picnum = 1680;
        let mut sectnum = 0;
        let mut cstat = 0; let mut pal = 0;
        let mut xrepeat = 64; let mut yrepeat = 64;
        let mut clipdist = 32; let mut lotag = 0; let mut hitag = 0;

        let mut ctx = VmActorContext {
            sprite_idx: 0,
            player_idx: 0,
            dist_to_player: 500,
            can_see_player: true,
            hit_by_weapon: false,
            registers: &mut reg,
            sprite_x: &mut x,
            sprite_y: &mut y,
            sprite_z: &mut z,
            sprite_ang: &mut ang,
            sprite_xvel: &mut xvel,
            sprite_zvel: &mut zvel,
            sprite_extra: &mut extra,
            sprite_picnum: &mut picnum,
            sprite_sectnum: &mut sectnum,
            sprite_cstat: &mut cstat,
            sprite_pal: &mut pal,
            sprite_xrepeat: &mut xrepeat,
            sprite_yrepeat: &mut yrepeat,
            sprite_clipdist: &mut clipdist,
            sprite_lotag: &mut lotag,
            sprite_hitag: &mut hitag,
            killit_flag: false,
            spawned_sprites: Vec::new(),
            sound_events: Vec::new(),
            quotes_displayed: Vec::new(),
            pal_flashes: Vec::new(),
            player_health_delta: 0,
            player_ammo_deltas: Vec::new(),
            player_inventory_deltas: Vec::new(),
            debris_events: Vec::new(),
            hitradius_events: Vec::new(),
        };

        // First tick: extra 100 -> enters else branch -> subtracts 10
        vm.execute(&mut ctx);
        assert_eq!(*ctx.sprite_extra, 90);
        assert!(!ctx.killit_flag);

        // Set health to 0 -> should trigger ifdead and killit
        *ctx.sprite_extra = 0;
        vm.execute(&mut ctx);
        assert!(ctx.killit_flag);
    }

    #[test]
    fn test_vm_subroutines_and_sounds() {
        let script = r#"
            define PIGCOP 2000
            state play_grunt
                sound 15
            ends

            actor PIGCOP 100
                ifpdistl 1024
                    state play_grunt
            enda
        "#;

        let mut compiler = Compiler::new();
        let compiled = compiler.compile(script).unwrap();
        let vm = ConVm::new(compiled.bytecode, compiled.actor_script_ptrs, compiled.actor_types);

        let mut reg = ActorRegisters::default();
        let mut x = 0; let mut y = 0; let mut z = 0;
        let mut ang = 0; let mut xvel = 0; let mut zvel = 0;
        let mut extra = 100;
        let mut picnum = 2000;
        let mut sectnum = 0;
        let mut cstat = 0; let mut pal = 0;
        let mut xrepeat = 64; let mut yrepeat = 64;
        let mut clipdist = 32; let mut lotag = 0; let mut hitag = 0;

        let mut ctx = VmActorContext {
            sprite_idx: 0,
            player_idx: 0,
            dist_to_player: 500, // < 1024
            can_see_player: true,
            hit_by_weapon: false,
            registers: &mut reg,
            sprite_x: &mut x,
            sprite_y: &mut y,
            sprite_z: &mut z,
            sprite_ang: &mut ang,
            sprite_xvel: &mut xvel,
            sprite_zvel: &mut zvel,
            sprite_extra: &mut extra,
            sprite_picnum: &mut picnum,
            sprite_sectnum: &mut sectnum,
            sprite_cstat: &mut cstat,
            sprite_pal: &mut pal,
            sprite_xrepeat: &mut xrepeat,
            sprite_yrepeat: &mut yrepeat,
            sprite_clipdist: &mut clipdist,
            sprite_lotag: &mut lotag,
            sprite_hitag: &mut hitag,
            killit_flag: false,
            spawned_sprites: Vec::new(),
            sound_events: Vec::new(),
            quotes_displayed: Vec::new(),
            pal_flashes: Vec::new(),
            player_health_delta: 0,
            player_ammo_deltas: Vec::new(),
            player_inventory_deltas: Vec::new(),
            debris_events: Vec::new(),
            hitradius_events: Vec::new(),
        };

        vm.execute(&mut ctx);
        assert_eq!(ctx.sound_events.len(), 1);
        assert_eq!(ctx.sound_events[0].0, 15);
    }

    #[test]
    fn test_vm_animation_pacing_and_counters() {
        let script = r#"
            define RECON 2400
            action ARECONWALK 0 4 1 1 2

            actor RECON 100 ARECONWALK
                ifcount 5 {
                    resetcount
                    cstat 257
                    spritepal 6
                    clipdist 48
                } else {
                    count 5
                }
            enda
        "#;

        let mut compiler = Compiler::new();
        let compiled = compiler.compile(script).unwrap();
        let vm = ConVm::new(compiled.bytecode, compiled.actor_script_ptrs, compiled.actor_types);

        let mut reg = ActorRegisters::default();
        reg.action_ptr = compiled.symbols.get("ARECONWALK").map(|&p| p as usize);

        let mut x = 0; let mut y = 0; let mut z = 0;
        let mut ang = 0; let mut xvel = 0; let mut zvel = 0;
        let mut extra = 100;
        let mut picnum = 2400;
        let mut sectnum = 0;
        let mut cstat = 0; let mut pal = 0;
        let mut xrepeat = 64; let mut yrepeat = 64;
        let mut clipdist = 32; let mut lotag = 777; // Ensure lotag is untouched!
        let mut hitag = 888;

        let mut ctx = VmActorContext {
            sprite_idx: 0,
            player_idx: 0,
            dist_to_player: 500,
            can_see_player: true,
            hit_by_weapon: false,
            registers: &mut reg,
            sprite_x: &mut x,
            sprite_y: &mut y,
            sprite_z: &mut z,
            sprite_ang: &mut ang,
            sprite_xvel: &mut xvel,
            sprite_zvel: &mut zvel,
            sprite_extra: &mut extra,
            sprite_picnum: &mut picnum,
            sprite_sectnum: &mut sectnum,
            sprite_cstat: &mut cstat,
            sprite_pal: &mut pal,
            sprite_xrepeat: &mut xrepeat,
            sprite_yrepeat: &mut yrepeat,
            sprite_clipdist: &mut clipdist,
            sprite_lotag: &mut lotag,
            sprite_hitag: &mut hitag,
            killit_flag: false,
            spawned_sprites: Vec::new(),
            sound_events: Vec::new(),
            quotes_displayed: Vec::new(),
            pal_flashes: Vec::new(),
            player_health_delta: 0,
            player_ammo_deltas: Vec::new(),
            player_inventory_deltas: Vec::new(),
            debris_events: Vec::new(),
            hitradius_events: Vec::new(),
        };

        // Tick 1: count is 0 -> enters else branch -> sets count to 5
        vm.execute(&mut ctx);
        assert_eq!(ctx.registers.count, 5);
        assert_eq!(*ctx.sprite_lotag, 777); // Verified lotag integrity!

        // Frame animation: action_delay_timer increased from 0 by TICSPERFRAME(3) > delay(2) -> frame_offset becomes 1
        assert_eq!(ctx.registers.frame_offset, 1);

        // Tick 2: count is 5 -> enters ifcount 5 branch -> resets count to 0, sets cstat to 257, pal to 6, clipdist to 48
        vm.execute(&mut ctx);
        assert_eq!(ctx.registers.count, 0);
        assert_eq!(*ctx.sprite_cstat, 257);
        assert_eq!(*ctx.sprite_pal, 6);
        assert_eq!(*ctx.sprite_clipdist, 48);
        assert_eq!(*ctx.sprite_lotag, 777); // Preserved!
    }

    #[test]
    fn test_vm_multi_arg_opcodes_and_break() {
        let script = r#"
            define BOSS 2600
            actor BOSS 500
                addammo 1 50
                debris 1000 4
                sizeto 80 80
                hitradius 1024 100 50 25 10
                palfrom 30 63 0 0
                break
                sound 999 // Should NEVER execute due to break!
            enda
        "#;

        let mut compiler = Compiler::new();
        let compiled = compiler.compile(script).unwrap();
        let vm = ConVm::new(compiled.bytecode, compiled.actor_script_ptrs, compiled.actor_types);

        let mut reg = ActorRegisters::default();
        let mut x = 0; let mut y = 0; let mut z = 0;
        let mut ang = 0; let mut xvel = 0; let mut zvel = 0;
        let mut extra = 500;
        let mut picnum = 2600;
        let mut sectnum = 0;
        let mut cstat = 0; let mut pal = 0;
        let mut xrepeat = 64; let mut yrepeat = 64;
        let mut clipdist = 32; let mut lotag = 0; let mut hitag = 0;

        let mut ctx = VmActorContext {
            sprite_idx: 0,
            player_idx: 0,
            dist_to_player: 500,
            can_see_player: true,
            hit_by_weapon: false,
            registers: &mut reg,
            sprite_x: &mut x,
            sprite_y: &mut y,
            sprite_z: &mut z,
            sprite_ang: &mut ang,
            sprite_xvel: &mut xvel,
            sprite_zvel: &mut zvel,
            sprite_extra: &mut extra,
            sprite_picnum: &mut picnum,
            sprite_sectnum: &mut sectnum,
            sprite_cstat: &mut cstat,
            sprite_pal: &mut pal,
            sprite_xrepeat: &mut xrepeat,
            sprite_yrepeat: &mut yrepeat,
            sprite_clipdist: &mut clipdist,
            sprite_lotag: &mut lotag,
            sprite_hitag: &mut hitag,
            killit_flag: false,
            spawned_sprites: Vec::new(),
            sound_events: Vec::new(),
            quotes_displayed: Vec::new(),
            pal_flashes: Vec::new(),
            player_health_delta: 0,
            player_ammo_deltas: Vec::new(),
            player_inventory_deltas: Vec::new(),
            debris_events: Vec::new(),
            hitradius_events: Vec::new(),
        };

        vm.execute(&mut ctx);
        // Multi-arg opcodes synchronized properly
        assert_eq!(ctx.player_ammo_deltas.len(), 1);
        assert_eq!(ctx.player_ammo_deltas[0], (1, 50));
        assert_eq!(ctx.debris_events.len(), 1);
        assert_eq!(ctx.debris_events[0], (1000, 4));
        assert_eq!(*ctx.sprite_xrepeat, 80);
        assert_eq!(ctx.hitradius_events.len(), 1);
        assert_eq!(ctx.pal_flashes.len(), 1);

        // Break prevented sound 999
        assert_eq!(ctx.sound_events.len(), 0);
    }
}
