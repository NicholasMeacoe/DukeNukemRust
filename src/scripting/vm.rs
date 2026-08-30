#![allow(dead_code)]

use crate::scripting::types::*;

pub const MAX_CALL_DEPTH: usize = 64;

pub struct ConVm {
    pub bytecode: Vec<i32>,
    pub actor_script_ptrs: Vec<Option<usize>>,
    pub actor_types: Vec<u8>,
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
    pub end_of_game: Option<i32>,

    // --- Player state (populated from PlayerController before VM execution) ---
    pub player_health: i32,
    pub player_ang: i16,
    pub player_on_ground: bool,
    pub player_jumping_counter: i32,
    pub player_posz_velocity: i32,
    pub player_crouching: bool,
    pub player_xvel: i32,
    pub player_running: bool,
    pub player_quick_kick: i32,
    pub player_shrunk: bool,
    pub player_jetpack_on: bool,
    pub player_steroids_active: bool,
    pub player_dead: bool,
    pub player_weapon: i32,
    pub player_kickback: i32,
    pub player_facing_actor: bool,

    // --- Inventory amounts (for ifpinventory) ---
    pub player_steroids_amount: i32,
    pub player_shield_amount: i32,
    pub player_scuba_amount: i32,
    pub player_holoduke_amount: i32,
    pub player_jetpack_amount: i32,
    pub player_heat_amount: i32,
    pub player_firstaid_amount: i32,
    pub player_boot_amount: i32,
    pub player_got_access: i32,

    // --- World state ---
    pub sector_lotag: i32,
    pub sector_ceilingstat: i32,
    pub is_multiplayer: bool,
    pub hit_space_pressed: bool,

    // --- Tracking fields ---
    pub spawned_by_picnum: i16,
    pub last_hit_weapon: i16,

    // --- AI Conditionals ---
    pub can_shoot_target: bool,
    pub bullet_near: bool,
    pub not_moving: bool,

    // --- Output: directional shoot events (separate from spawned_sprites) ---
    /// (tile, x, y, z, ang) — the consuming ECS system uses ang to fire projectiles
    pub shoot_events: Vec<(i16, i32, i32, i32, i16)>,
}


impl ConVm {
    pub fn new(
        bytecode: Vec<i32>,
        actor_script_ptrs: Vec<Option<usize>>,
        actor_types: Vec<u8>,
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
        let script_entry = match self.actor_script_ptrs.get(picnum).copied().flatten() {
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

                if num_frames > 0 && inc_val != 0 {
                    if (ctx.registers.frame_offset).abs() >= (num_frames * inc_val).abs() {
                        ctx.registers.frame_offset = 0;
                    }
                }
            }
        }

        // 3. Instruction interpretation loop
        let mut ip = script_entry + 4; // Skip 4-word header
        ctx.killit_flag = false;

        let mut call_stack: Vec<usize> = Vec::with_capacity(8);
        let mut instruction_count = 0usize;
        const MAX_INSTRUCTIONS_PER_TICK: usize = 10_000;

        while ip < self.bytecode.len() {
            instruction_count += 1;
            if instruction_count > MAX_INSTRUCTIONS_PER_TICK {
                break;
            }

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
                    // Original GAMEDEF.C case 78: parseifelse(sprite[ps[g_p].i].extra < *insptr)
                    let threshold = self.get_word(ip + 1).unwrap_or(0);
                    let fail_target = self.get_word(ip + 2).unwrap_or(0) as usize;
                    let cond = ctx.player_health < threshold;
                    self.handle_if_else(cond, &mut ip, 3, fail_target);
                }

                Opcode::IfAngDiffL => {
                    // Original GAMEDEF.C case 111: j = klabs(getincangle(ps[g_p].ang, g_sp->ang)); parseifelse(j <= *insptr);
                    let max_diff = self.get_word(ip + 1).unwrap_or(0);
                    let fail_target = self.get_word(ip + 2).unwrap_or(0) as usize;
                    let ang_diff = getincangle(ctx.player_ang, *ctx.sprite_ang).abs() as i32;
                    let cond = ang_diff <= max_diff;
                    self.handle_if_else(cond, &mut ip, 3, fail_target);
                }

                Opcode::IfP => {
                    // Original GAMEDEF.C case 51 (L2722-2778)
                    let flags = self.get_word(ip + 1).unwrap_or(0);
                    let fail_target = self.get_word(ip + 2).unwrap_or(0) as usize;
                    let mut j = false;

                    if (flags & 1) != 0 && ctx.player_xvel >= 0 && ctx.player_xvel < 8 { j = true; }
                    else if (flags & 2) != 0 && ctx.player_xvel >= 8 && !ctx.player_running { j = true; }
                    else if (flags & 4) != 0 && ctx.player_xvel >= 8 && ctx.player_running { j = true; }
                    else if (flags & 8) != 0 && ctx.player_on_ground && ctx.player_crouching { j = true; }
                    else if (flags & 16) != 0 && !ctx.player_on_ground && ctx.player_posz_velocity > 2048 { j = true; }
                    else if (flags & 32) != 0 && ctx.player_jumping_counter > 348 { j = true; }
                    else if (flags & 64) != 0 && ctx.player_health > 0 { j = true; }
                    else if (flags & 128) != 0 && ctx.player_xvel <= -8 && !ctx.player_running { j = true; }
                    else if (flags & 256) != 0 && ctx.player_xvel <= -8 && ctx.player_running { j = true; }
                    else if (flags & 512) != 0 && (ctx.player_quick_kick > 0 || (ctx.player_weapon == 0 && ctx.player_kickback > 0)) { j = true; }
                    else if (flags & 1024) != 0 && ctx.player_shrunk { j = true; }
                    else if (flags & 2048) != 0 && ctx.player_jetpack_on { j = true; }
                    else if (flags & 4096) != 0 && ctx.player_steroids_active { j = true; }
                    else if (flags & 8192) != 0 && ctx.player_on_ground { j = true; }
                    else if (flags & 16384) != 0 && !ctx.player_shrunk && ctx.player_health > 0 { j = true; }
                    else if (flags & 32768) != 0 && ctx.player_dead { j = true; }
                    else if (flags & 65536) != 0 && ctx.player_facing_actor { j = true; }

                    self.handle_if_else(j, &mut ip, 3, fail_target);
                }

                Opcode::IfPInventory => {
                    // Original GAMEDEF.C case 75 (L2898-2929)
                    let item = self.get_word(ip + 1).unwrap_or(0);
                    let amount = self.get_word(ip + 2).unwrap_or(0);
                    let fail_target = self.get_word(ip + 3).unwrap_or(0) as usize;
                    let cond = match item {
                        0 => ctx.player_steroids_amount != amount,
                        1 => ctx.player_shield_amount != 100,
                        2 => ctx.player_scuba_amount != amount,
                        3 => ctx.player_holoduke_amount != amount,
                        4 => ctx.player_jetpack_amount != amount,
                        6 => match *ctx.sprite_pal {
                            0 => (ctx.player_got_access & 1) != 0,
                            21 => (ctx.player_got_access & 2) != 0,
                            23 => (ctx.player_got_access & 4) != 0,
                            _ => false,
                        },
                        7 => ctx.player_heat_amount != amount,
                        9 => ctx.player_firstaid_amount != amount,
                        10 => ctx.player_boot_amount != amount,
                        _ => false,
                    };
                    self.handle_if_else(cond, &mut ip, 4, fail_target);
                }

                Opcode::IfWasWeapon => {
                    // Original GAMEDEF.C case 33: parseifelse(hittype[g_i].picnum == *insptr);
                    let expected = self.get_word(ip + 1).unwrap_or(0) as i16;
                    let fail_target = self.get_word(ip + 2).unwrap_or(0) as usize;
                    let cond = ctx.last_hit_weapon == expected;
                    self.handle_if_else(cond, &mut ip, 3, fail_target);
                }

                Opcode::IfSpawnedBy => {
                    // Original GAMEDEF.C case 59: parseifelse(hittype[g_i].picnum == *insptr);
                    let expected = self.get_word(ip + 1).unwrap_or(0) as i16;
                    let fail_target = self.get_word(ip + 2).unwrap_or(0) as usize;
                    let cond = ctx.spawned_by_picnum == expected;
                    self.handle_if_else(cond, &mut ip, 3, fail_target);
                }

                Opcode::IfGotWeaponCe => {
                    // TODO: Requires coop weapon recording system
                    let fail_target = self.get_word(ip + 2).unwrap_or(0) as usize;
                    self.handle_if_else(false, &mut ip, 3, fail_target);
                }

                Opcode::IfOnWater => {
                    // Original GAMEDEF.C case 43: klabs(g_sp->z - sector[g_sp->sectnum].floorz) < (32<<8) && sector[g_sp->sectnum].lotag == 1
                    let fail_target = self.get_word(ip + 1).unwrap_or(0) as usize;
                    let floor_dist = (ctx.registers.floor_z - *ctx.sprite_z).abs();
                    let cond = floor_dist <= (32 << 8) && ctx.sector_lotag == 1;
                    self.handle_if_else(cond, &mut ip, 2, fail_target);
                }

                Opcode::IfInWater => {
                    // Original GAMEDEF.C case 44: sector[g_sp->sectnum].lotag == 2
                    let fail_target = self.get_word(ip + 1).unwrap_or(0) as usize;
                    let cond = ctx.sector_lotag == 2;
                    self.handle_if_else(cond, &mut ip, 2, fail_target);
                }

                Opcode::IfOutside => {
                    // Original GAMEDEF.C case 64: sector[g_sp->sectnum].ceilingstat & 1
                    let fail_target = self.get_word(ip + 1).unwrap_or(0) as usize;
                    let cond = (ctx.sector_ceilingstat & 1) != 0;
                    self.handle_if_else(cond, &mut ip, 2, fail_target);
                }

                Opcode::IfMultiplayer => {
                    // Original GAMEDEF.C case 65: ud.multimode > 1
                    let fail_target = self.get_word(ip + 1).unwrap_or(0) as usize;
                    let cond = ctx.is_multiplayer;
                    self.handle_if_else(cond, &mut ip, 2, fail_target);
                }

                Opcode::IfHitSpace => {
                    // Original GAMEDEF.C case 63: sync[g_p].bits & (1<<29)
                    let fail_target = self.get_word(ip + 1).unwrap_or(0) as usize;
                    let cond = ctx.hit_space_pressed;
                    self.handle_if_else(cond, &mut ip, 2, fail_target);
                }

                Opcode::IfNotMoving => {
                    // Original GAMEDEF.C case 82: (hittype[g_i].movflag & 49152) > 16384
                    let fail_target = self.get_word(ip + 1).unwrap_or(0) as usize;
                    let cond = ctx.not_moving || (ctx.registers.mov_flag as i32 & 49152) > 16384;
                    self.handle_if_else(cond, &mut ip, 2, fail_target);
                }

                Opcode::IfActorNotStayput => {
                    // Original GAMEDEF.C case 49: hittype[g_i].actorstayput == -1
                    let fail_target = self.get_word(ip + 1).unwrap_or(0) as usize;
                    let cond = ctx.registers.actor_stay_put == -1;
                    self.handle_if_else(cond, &mut ip, 2, fail_target);
                }

                Opcode::IfCanShootTarget => {
                    // Check line of sight and firing angle alignment
                    let fail_target = self.get_word(ip + 1).unwrap_or(0) as usize;
                    let cond = ctx.can_shoot_target;
                    self.handle_if_else(cond, &mut ip, 2, fail_target);
                }

                Opcode::IfBulletNear => {
                    // Check if player projectiles are within dodging radius
                    let fail_target = self.get_word(ip + 1).unwrap_or(0) as usize;
                    let cond = ctx.bullet_near;
                    self.handle_if_else(cond, &mut ip, 2, fail_target);
                }

                Opcode::IfAwayFromWall => {
                    // TODO: Requires updatesector() wall proximity check
                    let fail_target = self.get_word(ip + 1).unwrap_or(0) as usize;
                    self.handle_if_else(true, &mut ip, 2, fail_target);
                }

                Opcode::IfNoSounds => {
                    // TODO: Requires tracking sound ownership per-sprite
                    let fail_target = self.get_word(ip + 1).unwrap_or(0) as usize;
                    self.handle_if_else(true, &mut ip, 2, fail_target);
                }

                Opcode::IfSquished => {
                    // TODO: Requires floor/ceiling crush detection
                    let fail_target = self.get_word(ip + 1).unwrap_or(0) as usize;
                    self.handle_if_else(false, &mut ip, 2, fail_target);
                }

                Opcode::IfInSpace | Opcode::IfInOuterSpace | Opcode::IfRespawn => {
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
                    ctx.shoot_events.push((tile, *ctx.sprite_x, *ctx.sprite_y, *ctx.sprite_z, *ctx.sprite_ang));
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

                Opcode::SizeTo => {
                    let target_xr = self.get_word(ip + 1).unwrap_or(64) as i32;
                    let target_yr = self.get_word(ip + 2).unwrap_or(64) as i32;

                    let dx = (target_xr - *ctx.sprite_xrepeat as i32) << 1;
                    if dx > 0 {
                        *ctx.sprite_xrepeat = ctx.sprite_xrepeat.saturating_add(1);
                    } else if dx < 0 {
                        *ctx.sprite_xrepeat = ctx.sprite_xrepeat.saturating_sub(1);
                    }

                    let dy = (target_yr - *ctx.sprite_yrepeat as i32) << 1;
                    if dy > 0 {
                        *ctx.sprite_yrepeat = ctx.sprite_yrepeat.saturating_add(1);
                    } else if dy < 0 {
                        *ctx.sprite_yrepeat = ctx.sprite_yrepeat.saturating_sub(1);
                    }

                    ip += 3;
                }

                Opcode::SizeAt => {
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

                Opcode::EndOfGame => {
                    let delay = self.get_word(ip + 1).unwrap_or(52);
                    ctx.end_of_game = Some(delay as i32);
                    ip += 2;
                }

                Opcode::SleepTime | Opcode::AddKills | Opcode::Debug => {
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

    fn create_test_context<'a>(
        reg: &'a mut ActorRegisters,
        x: &'a mut i32,
        y: &'a mut i32,
        z: &'a mut i32,
        ang: &'a mut i16,
        xvel: &'a mut i16,
        zvel: &'a mut i16,
        extra: &'a mut i16,
        picnum: &'a mut i16,
        sectnum: &'a mut i16,
        cstat: &'a mut i16,
        pal: &'a mut u8,
        xrepeat: &'a mut u8,
        yrepeat: &'a mut u8,
        clipdist: &'a mut u8,
        lotag: &'a mut i16,
        hitag: &'a mut i16,
    ) -> VmActorContext<'a> {
        VmActorContext {
            sprite_idx: 0,
            player_idx: 0,
            dist_to_player: 500,
            can_see_player: true,
            hit_by_weapon: false,
            registers: reg,
            sprite_x: x,
            sprite_y: y,
            sprite_z: z,
            sprite_ang: ang,
            sprite_xvel: xvel,
            sprite_zvel: zvel,
            sprite_extra: extra,
            sprite_picnum: picnum,
            sprite_sectnum: sectnum,
            sprite_cstat: cstat,
            sprite_pal: pal,
            sprite_xrepeat: xrepeat,
            sprite_yrepeat: yrepeat,
            sprite_clipdist: clipdist,
            sprite_lotag: lotag,
            sprite_hitag: hitag,
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
            end_of_game: None,
            player_health: 100,
            player_ang: 0,
            player_on_ground: true,
            player_jumping_counter: 0,
            player_posz_velocity: 0,
            player_crouching: false,
            player_xvel: 0,
            player_running: false,
            player_quick_kick: 0,
            player_shrunk: false,
            player_jetpack_on: false,
            player_steroids_active: false,
            player_dead: false,
            player_weapon: 1,
            player_kickback: 0,
            player_facing_actor: false,
            player_steroids_amount: 0,
            player_shield_amount: 0,
            player_scuba_amount: 0,
            player_holoduke_amount: 0,
            player_jetpack_amount: 0,
            player_heat_amount: 0,
            player_firstaid_amount: 0,
            player_boot_amount: 0,
            player_got_access: 0,
            sector_lotag: 0,
            sector_ceilingstat: 0,
            is_multiplayer: false,
            hit_space_pressed: false,
            spawned_by_picnum: 0,
            last_hit_weapon: 0,
            can_shoot_target: false,
            bullet_near: false,
            not_moving: false,
            shoot_events: Vec::new(),
        }
    }

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

        let mut ctx = create_test_context(
            &mut reg, &mut x, &mut y, &mut z, &mut ang, &mut xvel, &mut zvel,
            &mut extra, &mut picnum, &mut sectnum, &mut cstat, &mut pal,
            &mut xrepeat, &mut yrepeat, &mut clipdist, &mut lotag, &mut hitag,
        );

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

        let mut ctx = create_test_context(
            &mut reg, &mut x, &mut y, &mut z, &mut ang, &mut xvel, &mut zvel,
            &mut extra, &mut picnum, &mut sectnum, &mut cstat, &mut pal,
            &mut xrepeat, &mut yrepeat, &mut clipdist, &mut lotag, &mut hitag,
        );
        ctx.dist_to_player = 500; // < 1024

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

        let mut ctx = create_test_context(
            &mut reg, &mut x, &mut y, &mut z, &mut ang, &mut xvel, &mut zvel,
            &mut extra, &mut picnum, &mut sectnum, &mut cstat, &mut pal,
            &mut xrepeat, &mut yrepeat, &mut clipdist, &mut lotag, &mut hitag,
        );

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
                sizeat 80 80
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

        let mut ctx = create_test_context(
            &mut reg, &mut x, &mut y, &mut z, &mut ang, &mut xvel, &mut zvel,
            &mut extra, &mut picnum, &mut sectnum, &mut cstat, &mut pal,
            &mut xrepeat, &mut yrepeat, &mut clipdist, &mut lotag, &mut hitag,
        );

        vm.execute(&mut ctx);
        // Multi-arg opcodes synchronized properly
        assert_eq!(ctx.player_ammo_deltas.len(), 1);
        assert_eq!(ctx.player_ammo_deltas[0], (1, 50));
        assert_eq!(ctx.debris_events.len(), 1);
        assert_eq!(ctx.debris_events[0], (1000, 4));
        assert_eq!(*ctx.sprite_xrepeat, 80);
        assert_eq!(*ctx.sprite_yrepeat, 80);
        assert_eq!(ctx.hitradius_events.len(), 1);
        assert_eq!(ctx.pal_flashes.len(), 1);

        // Break prevented sound 999
        assert_eq!(ctx.sound_events.len(), 0);
    }

    #[test]
    fn test_vm_instruction_limit_guard() {
        let script = r#"
            define TESTACTOR 2700
            state infiniteloop
                state infiniteloop
            ends
            actor TESTACTOR 100
                state infiniteloop
            enda
        "#;
        let mut compiler = Compiler::new();
        let compiled = compiler.compile(script).unwrap();
        let vm = ConVm::new(compiled.bytecode, compiled.actor_script_ptrs, compiled.actor_types);

        let mut reg = ActorRegisters::default();
        let mut x = 0; let mut y = 0; let mut z = 0;
        let mut ang = 0; let mut xvel = 0; let mut zvel = 0;
        let mut extra = 100;
        let mut picnum = 2700;
        let mut sectnum = 0;
        let mut cstat = 0; let mut pal = 0;
        let mut xrepeat = 64; let mut yrepeat = 64;
        let mut clipdist = 32; let mut lotag = 0; let mut hitag = 0;

        let mut ctx = create_test_context(
            &mut reg, &mut x, &mut y, &mut z, &mut ang, &mut xvel, &mut zvel,
            &mut extra, &mut picnum, &mut sectnum, &mut cstat, &mut pal,
            &mut xrepeat, &mut yrepeat, &mut clipdist, &mut lotag, &mut hitag,
        );

        // VM terminates safely without hanging the process
        vm.execute(&mut ctx);
        assert_eq!(*ctx.sprite_picnum, 2700);
    }

    #[test]
    fn test_vm_ifphealthl_checks_actual_health() {
        let script = r#"
            define DOCTOR 2800
            actor DOCTOR 100
                ifphealthl 50
                    sound 10
                else
                    sound 20
            enda
        "#;
        let mut compiler = Compiler::new();
        let compiled = compiler.compile(script).unwrap();
        let vm = ConVm::new(compiled.bytecode, compiled.actor_script_ptrs, compiled.actor_types);

        let mut reg = ActorRegisters::default();
        let mut x = 0; let mut y = 0; let mut z = 0;
        let mut ang = 0; let mut xvel = 0; let mut zvel = 0;
        let mut extra = 100;
        let mut picnum = 2800;
        let mut sectnum = 0;
        let mut cstat = 0; let mut pal = 0;
        let mut xrepeat = 64; let mut yrepeat = 64;
        let mut clipdist = 32; let mut lotag = 0; let mut hitag = 0;

        let mut ctx = create_test_context(
            &mut reg, &mut x, &mut y, &mut z, &mut ang, &mut xvel, &mut zvel,
            &mut extra, &mut picnum, &mut sectnum, &mut cstat, &mut pal,
            &mut xrepeat, &mut yrepeat, &mut clipdist, &mut lotag, &mut hitag,
        );

        // Health 30 < 50 -> should play sound 10
        ctx.player_health = 30;
        vm.execute(&mut ctx);
        assert_eq!(ctx.sound_events.len(), 1);
        assert_eq!(ctx.sound_events[0].0, 10);

        // Health 80 >= 50 -> should play sound 20
        ctx.sound_events.clear();
        ctx.player_health = 80;
        vm.execute(&mut ctx);
        assert_eq!(ctx.sound_events.len(), 1);
        assert_eq!(ctx.sound_events[0].0, 20);
    }

    #[test]
    fn test_vm_ifangdiffl_uses_build_angles() {
        let script = r#"
            define WATCHER 2801
            actor WATCHER 100
                ifangdiffl 128
                    sound 100
                else
                    sound 200
            enda
        "#;
        let mut compiler = Compiler::new();
        let compiled = compiler.compile(script).unwrap();
        let vm = ConVm::new(compiled.bytecode, compiled.actor_script_ptrs, compiled.actor_types);

        let mut reg = ActorRegisters::default();
        let mut x = 0; let mut y = 0; let mut z = 0;
        let mut ang = 50; // Actor facing angle 50
        let mut xvel = 0; let mut zvel = 0;
        let mut extra = 100;
        let mut picnum = 2801;
        let mut sectnum = 0;
        let mut cstat = 0; let mut pal = 0;
        let mut xrepeat = 64; let mut yrepeat = 64;
        let mut clipdist = 32; let mut lotag = 0; let mut hitag = 0;

        let mut ctx = create_test_context(
            &mut reg, &mut x, &mut y, &mut z, &mut ang, &mut xvel, &mut zvel,
            &mut extra, &mut picnum, &mut sectnum, &mut cstat, &mut pal,
            &mut xrepeat, &mut yrepeat, &mut clipdist, &mut lotag, &mut hitag,
        );

        // Player angle 80 -> diff is 30 <= 128 -> sound 100
        ctx.player_ang = 80;
        vm.execute(&mut ctx);
        assert_eq!(ctx.sound_events.len(), 1);
        assert_eq!(ctx.sound_events[0].0, 100);

        // Player angle 1024 -> diff is 974 > 128 -> sound 200
        ctx.sound_events.clear();
        ctx.player_ang = 1024;
        vm.execute(&mut ctx);
        assert_eq!(ctx.sound_events.len(), 1);
        assert_eq!(ctx.sound_events[0].0, 200);
    }

    #[test]
    fn test_vm_ifp_player_state_flags() {
        let script = r#"
            define JETPACKACTOR 2802
            actor JETPACKACTOR 100
                ifp 2048 // Jetpack active flag
                    sound 50
                else
                    sound 60
            enda
        "#;
        let mut compiler = Compiler::new();
        let compiled = compiler.compile(script).unwrap();
        let vm = ConVm::new(compiled.bytecode, compiled.actor_script_ptrs, compiled.actor_types);

        let mut reg = ActorRegisters::default();
        let mut x = 0; let mut y = 0; let mut z = 0;
        let mut ang = 0; let mut xvel = 0; let mut zvel = 0;
        let mut extra = 100;
        let mut picnum = 2802;
        let mut sectnum = 0;
        let mut cstat = 0; let mut pal = 0;
        let mut xrepeat = 64; let mut yrepeat = 64;
        let mut clipdist = 32; let mut lotag = 0; let mut hitag = 0;

        let mut ctx = create_test_context(
            &mut reg, &mut x, &mut y, &mut z, &mut ang, &mut xvel, &mut zvel,
            &mut extra, &mut picnum, &mut sectnum, &mut cstat, &mut pal,
            &mut xrepeat, &mut yrepeat, &mut clipdist, &mut lotag, &mut hitag,
        );

        ctx.player_jetpack_on = true;
        vm.execute(&mut ctx);
        assert_eq!(ctx.sound_events.len(), 1);
        assert_eq!(ctx.sound_events[0].0, 50);

        ctx.sound_events.clear();
        ctx.player_jetpack_on = false;
        vm.execute(&mut ctx);
        assert_eq!(ctx.sound_events.len(), 1);
        assert_eq!(ctx.sound_events[0].0, 60);
    }

    #[test]
    fn test_vm_ifpinventory_checks_items() {
        let script = r#"
            define LOCKACTOR 2803
            actor LOCKACTOR 100
                ifpinventory 6 0 // Check blue keycard (pal 0, bitmask 1)
                    sound 11
                else
                    sound 22
            enda
        "#;
        let mut compiler = Compiler::new();
        let compiled = compiler.compile(script).unwrap();
        let vm = ConVm::new(compiled.bytecode, compiled.actor_script_ptrs, compiled.actor_types);

        let mut reg = ActorRegisters::default();
        let mut x = 0; let mut y = 0; let mut z = 0;
        let mut ang = 0; let mut xvel = 0; let mut zvel = 0;
        let mut extra = 100;
        let mut picnum = 2803;
        let mut sectnum = 0;
        let mut cstat = 0; let mut pal = 0; // Blue keycard
        let mut xrepeat = 64; let mut yrepeat = 64;
        let mut clipdist = 32; let mut lotag = 0; let mut hitag = 0;

        let mut ctx = create_test_context(
            &mut reg, &mut x, &mut y, &mut z, &mut ang, &mut xvel, &mut zvel,
            &mut extra, &mut picnum, &mut sectnum, &mut cstat, &mut pal,
            &mut xrepeat, &mut yrepeat, &mut clipdist, &mut lotag, &mut hitag,
        );

        // Player has blue keycard (got_access = 1)
        ctx.player_got_access = 1;
        vm.execute(&mut ctx);
        assert_eq!(ctx.sound_events.len(), 1);
        assert_eq!(ctx.sound_events[0].0, 11);

        // Player doesn't have keycard
        ctx.sound_events.clear();
        ctx.player_got_access = 0;
        vm.execute(&mut ctx);
        assert_eq!(ctx.sound_events.len(), 1);
        assert_eq!(ctx.sound_events[0].0, 22);
    }

    #[test]
    fn test_vm_water_and_outside_conditionals() {
        let script = r#"
            define AQUAACTOR 2804
            actor AQUAACTOR 100
                ifinwater
                    sound 300
                else ifoutside
                    sound 400
            enda
        "#;
        let mut compiler = Compiler::new();
        let compiled = compiler.compile(script).unwrap();
        let vm = ConVm::new(compiled.bytecode, compiled.actor_script_ptrs, compiled.actor_types);

        let mut reg = ActorRegisters::default();
        let mut x = 0; let mut y = 0; let mut z = 0;
        let mut ang = 0; let mut xvel = 0; let mut zvel = 0;
        let mut extra = 100;
        let mut picnum = 2804;
        let mut sectnum = 0;
        let mut cstat = 0; let mut pal = 0;
        let mut xrepeat = 64; let mut yrepeat = 64;
        let mut clipdist = 32; let mut lotag = 0; let mut hitag = 0;

        let mut ctx = create_test_context(
            &mut reg, &mut x, &mut y, &mut z, &mut ang, &mut xvel, &mut zvel,
            &mut extra, &mut picnum, &mut sectnum, &mut cstat, &mut pal,
            &mut xrepeat, &mut yrepeat, &mut clipdist, &mut lotag, &mut hitag,
        );

        // Sector lotag = 2 -> in water
        ctx.sector_lotag = 2;
        vm.execute(&mut ctx);
        assert_eq!(ctx.sound_events.len(), 1);
        assert_eq!(ctx.sound_events[0].0, 300);

        // Sector ceilingstat = 1 -> outside
        ctx.sound_events.clear();
        ctx.sector_lotag = 0;
        ctx.sector_ceilingstat = 1;
        vm.execute(&mut ctx);
        assert_eq!(ctx.sound_events.len(), 1);
        assert_eq!(ctx.sound_events[0].0, 400);
    }

    #[test]
    fn test_vm_shoot_produces_directional_events() {
        let script = r#"
            define SHOOTER 2805
            actor SHOOTER 100
                shoot 1600
            enda
        "#;
        let mut compiler = Compiler::new();
        let compiled = compiler.compile(script).unwrap();
        let vm = ConVm::new(compiled.bytecode, compiled.actor_script_ptrs, compiled.actor_types);

        let mut reg = ActorRegisters::default();
        let mut x = 100; let mut y = 200; let mut z = 300;
        let mut ang = 512; // Facing right (512 in Build 0-2047)
        let mut xvel = 0; let mut zvel = 0;
        let mut extra = 100;
        let mut picnum = 2805;
        let mut sectnum = 0;
        let mut cstat = 0; let mut pal = 0;
        let mut xrepeat = 64; let mut yrepeat = 64;
        let mut clipdist = 32; let mut lotag = 0; let mut hitag = 0;

        let mut ctx = create_test_context(
            &mut reg, &mut x, &mut y, &mut z, &mut ang, &mut xvel, &mut zvel,
            &mut extra, &mut picnum, &mut sectnum, &mut cstat, &mut pal,
            &mut xrepeat, &mut yrepeat, &mut clipdist, &mut lotag, &mut hitag,
        );

        vm.execute(&mut ctx);
        assert_eq!(ctx.spawned_sprites.len(), 0); // Not a passive spawn
        assert_eq!(ctx.shoot_events.len(), 1);
        assert_eq!(ctx.shoot_events[0], (1600, 100, 200, 300, 512));
    }

    #[test]
    fn test_vm_sizeto_gradual_interpolation() {
        let script = r#"
            define GROWACTOR 2806
            actor GROWACTOR 100
                sizeto 80 80
            enda
        "#;
        let mut compiler = Compiler::new();
        let compiled = compiler.compile(script).unwrap();
        let vm = ConVm::new(compiled.bytecode, compiled.actor_script_ptrs, compiled.actor_types);

        let mut reg = ActorRegisters::default();
        let mut x = 0; let mut y = 0; let mut z = 0;
        let mut ang = 0; let mut xvel = 0; let mut zvel = 0;
        let mut extra = 100;
        let mut picnum = 2806;
        let mut sectnum = 0;
        let mut cstat = 0; let mut pal = 0;
        let mut xrepeat = 64; let mut yrepeat = 64;
        let mut clipdist = 32; let mut lotag = 0; let mut hitag = 0;

        let mut ctx = create_test_context(
            &mut reg, &mut x, &mut y, &mut z, &mut ang, &mut xvel, &mut zvel,
            &mut extra, &mut picnum, &mut sectnum, &mut cstat, &mut pal,
            &mut xrepeat, &mut yrepeat, &mut clipdist, &mut lotag, &mut hitag,
        );

        // Tick 1: increments from 64 to 65
        vm.execute(&mut ctx);
        assert_eq!(*ctx.sprite_xrepeat, 65);
        assert_eq!(*ctx.sprite_yrepeat, 65);

        // Tick 2: increments from 65 to 66
        vm.execute(&mut ctx);
        assert_eq!(*ctx.sprite_xrepeat, 66);
        assert_eq!(*ctx.sprite_yrepeat, 66);
    }

    #[test]
    fn test_vm_ifwasweapon_and_ifspawnedby() {
        let script = r#"
            define TARGETACTOR 2807
            actor TARGETACTOR 100
                ifwasweapon 21 // RPG
                    sound 77
                else ifspawnedby 2000 // Pigcop
                    sound 88
            enda
        "#;
        let mut compiler = Compiler::new();
        let compiled = compiler.compile(script).unwrap();
        let vm = ConVm::new(compiled.bytecode, compiled.actor_script_ptrs, compiled.actor_types);

        let mut reg = ActorRegisters::default();
        let mut x = 0; let mut y = 0; let mut z = 0;
        let mut ang = 0; let mut xvel = 0; let mut zvel = 0;
        let mut extra = 100;
        let mut picnum = 2807;
        let mut sectnum = 0;
        let mut cstat = 0; let mut pal = 0;
        let mut xrepeat = 64; let mut yrepeat = 64;
        let mut clipdist = 32; let mut lotag = 0; let mut hitag = 0;

        let mut ctx = create_test_context(
            &mut reg, &mut x, &mut y, &mut z, &mut ang, &mut xvel, &mut zvel,
            &mut extra, &mut picnum, &mut sectnum, &mut cstat, &mut pal,
            &mut xrepeat, &mut yrepeat, &mut clipdist, &mut lotag, &mut hitag,
        );

        // Hit by RPG
        ctx.last_hit_weapon = 21;
        vm.execute(&mut ctx);
        assert_eq!(ctx.sound_events.len(), 1);
        assert_eq!(ctx.sound_events[0].0, 77);

        // Spawned by Pigcop
        ctx.sound_events.clear();
        ctx.last_hit_weapon = 0;
        ctx.spawned_by_picnum = 2000;
        vm.execute(&mut ctx);
        assert_eq!(ctx.sound_events.len(), 1);
        assert_eq!(ctx.sound_events[0].0, 88);
    }

    #[test]
    fn test_vm_ai_raycast_and_dodge_conditionals() {
        let script = r#"
            define ENEMY 2808
            actor ENEMY 100
                ifcanshoottarget
                    sound 101
                else ifbulletnear
                    sound 202
                else ifnotmoving
                    sound 303
            enda
        "#;
        let mut compiler = Compiler::new();
        let compiled = compiler.compile(script).unwrap();
        let vm = ConVm::new(compiled.bytecode, compiled.actor_script_ptrs, compiled.actor_types);

        let mut reg = ActorRegisters::default();
        let mut x = 0; let mut y = 0; let mut z = 0;
        let mut ang = 0; let mut xvel = 0; let mut zvel = 0;
        let mut extra = 100;
        let mut picnum = 2808;
        let mut sectnum = 0;
        let mut cstat = 0; let mut pal = 0;
        let mut xrepeat = 64; let mut yrepeat = 64;
        let mut clipdist = 32; let mut lotag = 0; let mut hitag = 0;

        let mut ctx = create_test_context(
            &mut reg, &mut x, &mut y, &mut z, &mut ang, &mut xvel, &mut zvel,
            &mut extra, &mut picnum, &mut sectnum, &mut cstat, &mut pal,
            &mut xrepeat, &mut yrepeat, &mut clipdist, &mut lotag, &mut hitag,
        );

        // 1. can_shoot_target = true -> sound 101
        ctx.can_shoot_target = true;
        vm.execute(&mut ctx);
        assert_eq!(ctx.sound_events.len(), 1);
        assert_eq!(ctx.sound_events[0].0, 101);

        // 2. bullet_near = true -> sound 202
        ctx.sound_events.clear();
        ctx.can_shoot_target = false;
        ctx.bullet_near = true;
        vm.execute(&mut ctx);
        assert_eq!(ctx.sound_events.len(), 1);
        assert_eq!(ctx.sound_events[0].0, 202);

        // 3. not_moving = true -> sound 303
        ctx.sound_events.clear();
        ctx.bullet_near = false;
        ctx.not_moving = true;
        vm.execute(&mut ctx);
        assert_eq!(ctx.sound_events.len(), 1);
        assert_eq!(ctx.sound_events[0].0, 303);
    }

    #[test]
    fn test_vm_endofgame_opcode() {
        let script = r#"
            define BOSS 2630
            actor BOSS 4500
                ifdead
                    endofgame 52
            enda
        "#;
        let mut compiler = Compiler::new();
        let compiled = compiler.compile(script).unwrap();
        let vm = ConVm::new(compiled.bytecode, compiled.actor_script_ptrs, compiled.actor_types);

        let mut reg = ActorRegisters::default();
        let mut x = 0; let mut y = 0; let mut z = 0;
        let mut ang = 0; let mut xvel = 0; let mut zvel = 0;
        let mut extra = 0; // dead
        let mut picnum = 2630;
        let mut sectnum = 0;
        let mut cstat = 0; let mut pal = 0;
        let mut xrepeat = 64; let mut yrepeat = 64;
        let mut clipdist = 32; let mut lotag = 0; let mut hitag = 0;

        let mut ctx = create_test_context(
            &mut reg, &mut x, &mut y, &mut z, &mut ang, &mut xvel, &mut zvel,
            &mut extra, &mut picnum, &mut sectnum, &mut cstat, &mut pal,
            &mut xrepeat, &mut yrepeat, &mut clipdist, &mut lotag, &mut hitag,
        );

        vm.execute(&mut ctx);
        assert_eq!(ctx.end_of_game, Some(52));
    }
}

