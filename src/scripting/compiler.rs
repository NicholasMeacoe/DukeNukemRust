#![allow(dead_code)]

use std::collections::HashMap;
use crate::scripting::lexer::{Lexer, Token};
use crate::scripting::types::*;

pub struct CompiledScript {
    pub bytecode: Vec<i32>,
    pub actor_script_ptrs: Vec<Option<usize>>,
    pub actor_types: Vec<u8>,
    pub symbols: HashMap<String, i32>,
    pub actions: HashMap<String, ActionDef>,
    pub moves: HashMap<String, MoveDef>,
    pub ais: HashMap<String, AiDef>,
    pub volumes: Vec<DynamicVolumeDef>,
    pub skills: Vec<DynamicSkillDef>,
    pub levels: Vec<DynamicLevelDef>,
    pub quotes: HashMap<i32, String>,
    pub sounds: Vec<DynamicSoundDef>,
}

pub struct Compiler {
    bytecode: Vec<i32>,
    actor_script_ptrs: Vec<Option<usize>>,
    actor_types: Vec<u8>,
    symbols: HashMap<String, i32>,
    actions: HashMap<String, ActionDef>,
    moves: HashMap<String, MoveDef>,
    ais: HashMap<String, AiDef>,
    states: HashMap<String, usize>,
    volumes: Vec<DynamicVolumeDef>,
    skills: Vec<DynamicSkillDef>,
    levels: Vec<DynamicLevelDef>,
    quotes: HashMap<i32, String>,
    sounds: Vec<DynamicSoundDef>,
}

impl Default for Compiler {
    fn default() -> Self {
        Self::new()
    }
}

impl Compiler {
    pub fn new() -> Self {
        let mut symbols = HashMap::new();
        // Built-in movement / AI flags
        symbols.insert("face_player".into(), move_flags::FACE_PLAYER);
        symbols.insert("face_player_slow".into(), move_flags::FACE_PLAYER_SLOW);
        symbols.insert("spin".into(), move_flags::SPIN);
        symbols.insert("face_player_smart".into(), move_flags::FACE_PLAYER_SMART);
        symbols.insert("fleeenemy".into(), move_flags::FLEE_ENEMY);
        symbols.insert("jumptoplayer".into(), move_flags::JUMP_TO_PLAYER);
        symbols.insert("seekplayer".into(), move_flags::SEEK_PLAYER);
        symbols.insert("furthestdir".into(), move_flags::FURTHEST_DIR);
        symbols.insert("dodgebullet".into(), move_flags::DODGE_BULLET);
        symbols.insert("geth".into(), move_flags::GET_H);
        symbols.insert("getv".into(), move_flags::GET_V);
        symbols.insert("random_angle".into(), move_flags::RANDOM_ANGLE);
        symbols.insert("STOPPED".into(), 0);

        Self {
            bytecode: vec![0], // Reserve index 0 so NULL_PTR / STOPPED (0) never collides with real addresses
            actor_script_ptrs: vec![None; MAX_TILES],
            actor_types: vec![0; MAX_TILES],
            symbols,
            actions: HashMap::new(),
            moves: HashMap::new(),
            ais: HashMap::new(),
            states: HashMap::new(),
            volumes: Vec::new(),
            skills: Vec::new(),
            levels: Vec::new(),
            quotes: HashMap::new(),
            sounds: Vec::new(),
        }
    }

    pub fn compile(&mut self, source: &str) -> Result<CompiledScript, String> {
        let mut lexer = Lexer::new(source);
        let tokens = lexer.tokenize()?;
        let mut pos = 0;

        while pos < tokens.len() {
            match &tokens[pos] {
                Token::Ident(keyword) => {
                    let kw = keyword.to_lowercase();
                    match kw.as_str() {
                        "define" => {
                            pos += 1;
                            let name = self.expect_ident(&tokens, &mut pos)?;
                            let val = self.expect_num_or_symbol(&tokens, &mut pos)?;
                            self.symbols.insert(name, val);
                        }
                        "action" => {
                            pos += 1;
                            let name = self.expect_ident(&tokens, &mut pos)?;
                            let start = self.expect_num_or_symbol(&tokens, &mut pos)?;
                            let num = self.expect_num_or_symbol(&tokens, &mut pos)?;
                            let view = self.expect_num_or_symbol(&tokens, &mut pos)?;
                            let inc = self.expect_num_or_symbol(&tokens, &mut pos)?;
                            let delay = self.expect_num_or_symbol(&tokens, &mut pos)?;

                            let action_addr = self.bytecode.len();
                            self.bytecode.extend_from_slice(&[start, num, view, inc, delay]);
                            self.symbols.insert(name.clone(), action_addr as i32);
                            self.actions.insert(
                                name,
                                ActionDef {
                                    start_frame: start,
                                    num_frames: num,
                                    view_type: view,
                                    inc_val: inc,
                                    delay,
                                },
                            );
                        }
                        "move" => {
                            pos += 1;
                            let name = self.expect_ident(&tokens, &mut pos)?;
                            let hvel = self.expect_num_or_symbol(&tokens, &mut pos)?;
                            let vvel = if pos < tokens.len() && self.is_value(&tokens[pos]) {
                                self.expect_num_or_symbol(&tokens, &mut pos)?
                            } else {
                                0
                            };

                            let move_addr = self.bytecode.len();
                            self.bytecode.extend_from_slice(&[hvel, vvel]);
                            self.symbols.insert(name.clone(), move_addr as i32);
                            self.moves.insert(name, MoveDef { hvel, vvel });
                        }
                        "ai" => {
                            pos += 1;
                            let name = self.expect_ident(&tokens, &mut pos)?;
                            let action_val = self.expect_num_or_symbol(&tokens, &mut pos)?;
                            let move_val = self.expect_num_or_symbol(&tokens, &mut pos)?;
                            let mut flags = 0;
                            while pos < tokens.len() && self.is_value(&tokens[pos]) {
                                flags |= self.expect_num_or_symbol(&tokens, &mut pos)?;
                            }

                            let ai_addr = self.bytecode.len();
                            self.bytecode.extend_from_slice(&[action_val, move_val, flags]);
                            self.symbols.insert(name.clone(), ai_addr as i32);
                            self.ais.insert(
                                name,
                                AiDef {
                                    action_ptr: action_val as usize,
                                    move_ptr: move_val as usize,
                                    flags,
                                },
                            );
                        }
                        "state" => {
                            pos += 1;
                            let name = self.expect_ident(&tokens, &mut pos)?;
                            let state_addr = self.bytecode.len();
                            self.states.insert(name.clone(), state_addr);
                            self.symbols.insert(name, state_addr as i32);

                            while pos < tokens.len() {
                                if let Token::Ident(s) = &tokens[pos] {
                                    if s.eq_ignore_ascii_case("ends") {
                                        pos += 1;
                                        self.bytecode.push(Opcode::EndS as i32);
                                        break;
                                    }
                                }
                                self.compile_statement(&tokens, &mut pos)?;
                            }
                        }
                        "actor" | "useractor" => {
                            let is_user = kw == "useractor";
                            pos += 1;
                            let actortype = if is_user {
                                let t = self.expect_num_or_symbol(&tokens, &mut pos)? as u8;
                                t
                            } else {
                                0
                            };

                            let picnum = self.expect_num_or_symbol(&tokens, &mut pos)? as usize;
                            let strength = if pos < tokens.len() && self.is_value(&tokens[pos]) {
                                self.expect_num_or_symbol(&tokens, &mut pos)?
                            } else {
                                0
                            };
                            let action = if pos < tokens.len() && self.is_value(&tokens[pos]) {
                                self.expect_num_or_symbol(&tokens, &mut pos)?
                            } else {
                                0
                            };
                            let mov = if pos < tokens.len() && self.is_value(&tokens[pos]) {
                                self.expect_num_or_symbol(&tokens, &mut pos)?
                            } else {
                                0
                            };
                            let mut flags = 0;
                            while pos < tokens.len() && self.is_value(&tokens[pos]) {
                                flags |= self.expect_num_or_symbol(&tokens, &mut pos)?;
                            }

                            if picnum < MAX_TILES {
                                let actor_entry = self.bytecode.len();
                                self.actor_script_ptrs[picnum] = Some(actor_entry);
                                self.actor_types[picnum] = actortype;
                                // 4-word header
                                self.bytecode.extend_from_slice(&[strength, action, mov, flags]);

                                while pos < tokens.len() {
                                    if let Token::Ident(s) = &tokens[pos] {
                                        if s.eq_ignore_ascii_case("enda") {
                                            pos += 1;
                                            self.bytecode.push(Opcode::EndA as i32);
                                            break;
                                        }
                                    }
                                    self.compile_statement(&tokens, &mut pos)?;
                                }
                            } else {
                                return Err(format!("Actor picnum out of range: {}", picnum));
                            }
                        }
                        "definevolumename" => {
                            pos += 1;
                            let vol_id = self.expect_num_or_symbol(&tokens, &mut pos)? as usize;
                            let mut title_words = Vec::new();
                            while pos < tokens.len() {
                                match &tokens[pos] {
                                    Token::Str(s) => {
                                        title_words.push(s.clone());
                                        pos += 1;
                                        break;
                                    }
                                    Token::Ident(s) => {
                                        if is_keyword(s) { break; }
                                        title_words.push(s.clone());
                                        pos += 1;
                                    }
                                    Token::Number(n) => {
                                        title_words.push(n.to_string());
                                        pos += 1;
                                    }
                                    _ => break,
                                }
                            }
                            let title = title_words.join(" ");
                            self.volumes.push(DynamicVolumeDef { volume_id: vol_id, title });
                        }
                        "defineskillname" => {
                            pos += 1;
                            let skill_id = self.expect_num_or_symbol(&tokens, &mut pos)? as usize;
                            let mut title_words = Vec::new();
                            while pos < tokens.len() {
                                match &tokens[pos] {
                                    Token::Str(s) => {
                                        title_words.push(s.clone());
                                        pos += 1;
                                        break;
                                    }
                                    Token::Ident(s) => {
                                        if is_keyword(s) { break; }
                                        title_words.push(s.clone());
                                        pos += 1;
                                    }
                                    Token::Number(n) => {
                                        title_words.push(n.to_string());
                                        pos += 1;
                                    }
                                    _ => break,
                                }
                            }
                            let title = title_words.join(" ");
                            self.skills.push(DynamicSkillDef { skill_id, title });
                        }
                        "definelevelname" => {
                            pos += 1;
                            let volume = self.expect_num_or_symbol(&tokens, &mut pos)? as usize;
                            let level = self.expect_num_or_symbol(&tokens, &mut pos)? as usize;
                            let filename = self.expect_ident_or_str(&tokens, &mut pos)?;
                            let par_time_str = self.expect_ident_or_str(&tokens, &mut pos)?;
                            let three_dr_time_str = self.expect_ident_or_str(&tokens, &mut pos)?;
                            let mut title_words = Vec::new();
                            while pos < tokens.len() {
                                match &tokens[pos] {
                                    Token::Str(s) => {
                                        title_words.push(s.clone());
                                        pos += 1;
                                        break;
                                    }
                                    Token::Ident(s) => {
                                        if is_keyword(s) { break; }
                                        title_words.push(s.clone());
                                        pos += 1;
                                    }
                                    Token::Number(n) => {
                                        title_words.push(n.to_string());
                                        pos += 1;
                                    }
                                    _ => break,
                                }
                            }
                            let title = title_words.join(" ");
                            self.levels.push(DynamicLevelDef { volume, level, filename, par_time_str, three_dr_time_str, title });
                        }
                        "definequote" => {
                            pos += 1;
                            let quote_id = self.expect_num_or_symbol(&tokens, &mut pos)?;
                            let mut text_words = Vec::new();
                            while pos < tokens.len() {
                                match &tokens[pos] {
                                    Token::Str(s) => {
                                        text_words.push(s.clone());
                                        pos += 1;
                                        break;
                                    }
                                    Token::Ident(s) => {
                                        if is_keyword(s) { break; }
                                        text_words.push(s.clone());
                                        pos += 1;
                                    }
                                    Token::Number(n) => {
                                        text_words.push(n.to_string());
                                        pos += 1;
                                    }
                                    _ => break,
                                }
                            }
                            self.quotes.insert(quote_id, text_words.join(" "));
                        }
                        "definesound" => {
                            pos += 1;
                            let sound_id = self.expect_num_or_symbol(&tokens, &mut pos)?;
                            let filename = self.expect_ident_or_str(&tokens, &mut pos)?;
                            let pitch1 = self.expect_num_or_symbol(&tokens, &mut pos)?;
                            let pitch2 = self.expect_num_or_symbol(&tokens, &mut pos)?;
                            let priority = self.expect_num_or_symbol(&tokens, &mut pos)?;
                            let sound_type = self.expect_num_or_symbol(&tokens, &mut pos)?;
                            let volume = self.expect_num_or_symbol(&tokens, &mut pos)?;
                            self.sounds.push(DynamicSoundDef { sound_id, filename, pitch1, pitch2, priority, sound_type, volume });
                        }
                        _ => {
                            // Other top level keywords
                            pos += 1;
                        }
                    }
                }
                _ => pos += 1,
            }
        }

        Ok(CompiledScript {
            bytecode: self.bytecode.clone(),
            actor_script_ptrs: self.actor_script_ptrs.clone(),
            actor_types: self.actor_types.clone(),
            symbols: self.symbols.clone(),
            actions: self.actions.clone(),
            moves: self.moves.clone(),
            ais: self.ais.clone(),
            volumes: self.volumes.clone(),
            skills: self.skills.clone(),
            levels: self.levels.clone(),
            quotes: self.quotes.clone(),
            sounds: self.sounds.clone(),
        })
    }

    fn compile_statement(&mut self, tokens: &[Token], pos: &mut usize) -> Result<(), String> {
        if *pos >= tokens.len() {
            return Ok(());
        }

        match &tokens[*pos] {
            Token::LeftBrace => {
                *pos += 1;
                while *pos < tokens.len() && tokens[*pos] != Token::RightBrace {
                    self.compile_statement(tokens, pos)?;
                }
                if *pos < tokens.len() && tokens[*pos] == Token::RightBrace {
                    *pos += 1;
                }
            }
            Token::Ident(ident) => {
                let kw = ident.to_lowercase();
                match kw.as_str() {
                    // Conditionals
                    "ifrnd" => self.compile_if_1arg(Opcode::IfRnd, tokens, pos)?,
                    "ifpdistl" => self.compile_if_1arg(Opcode::IfPDistL, tokens, pos)?,
                    "ifpdistg" => self.compile_if_1arg(Opcode::IfPDistG, tokens, pos)?,
                    "ifcount" => self.compile_if_1arg(Opcode::IfCount, tokens, pos)?,
                    "ifactioncount" => self.compile_if_1arg(Opcode::IfActionCount, tokens, pos)?,
                    "ifaction" => self.compile_if_1arg(Opcode::IfAction, tokens, pos)?,
                    "ifmove" => self.compile_if_1arg(Opcode::IfMove, tokens, pos)?,
                    "ifai" => self.compile_if_1arg(Opcode::IfAi, tokens, pos)?,
                    "ifactor" => self.compile_if_1arg(Opcode::IfActor, tokens, pos)?,
                    "ifstrength" => self.compile_if_1arg(Opcode::IfStrength, tokens, pos)?,
                    "ifwasweapon" => self.compile_if_1arg(Opcode::IfWasWeapon, tokens, pos)?,
                    "ifspawnedby" => self.compile_if_1arg(Opcode::IfSpawnedBy, tokens, pos)?,
                    "ifpinventory" => self.compile_if_2args(Opcode::IfPInventory, tokens, pos)?,
                    "ifspritepal" => self.compile_if_1arg(Opcode::IfSpritePal, tokens, pos)?,
                    "ifphealthl" => self.compile_if_1arg(Opcode::IfPHealthL, tokens, pos)?,
                    "ifangdiffl" => self.compile_if_1arg(Opcode::IfAngDiffL, tokens, pos)?,
                    "iffloordistl" => self.compile_if_1arg(Opcode::IfFloorDistL, tokens, pos)?,
                    "ifceilingdistl" => self.compile_if_1arg(Opcode::IfCeilingDistL, tokens, pos)?,
                    "ifgapzl" => self.compile_if_1arg(Opcode::IfGapZL, tokens, pos)?,
                    "ifp" => self.compile_if_flags(Opcode::IfP, tokens, pos)?,

                    // 0-argument Conditionals
                    "ifcansee" => self.compile_if_0arg(Opcode::IfCanSee, tokens, pos)?,
                    "ifcanseetarget" => self.compile_if_0arg(Opcode::IfCanSeeTarget, tokens, pos)?,
                    "ifcanshoottarget" => self.compile_if_0arg(Opcode::IfCanShootTarget, tokens, pos)?,
                    "ifhitweapon" => self.compile_if_0arg(Opcode::IfHitWeapon, tokens, pos)?,
                    "ifdead" => self.compile_if_0arg(Opcode::IfDead, tokens, pos)?,
                    "ifsquished" => self.compile_if_0arg(Opcode::IfSquished, tokens, pos)?,
                    "ifonwater" => self.compile_if_0arg(Opcode::IfOnWater, tokens, pos)?,
                    "ifinwater" => self.compile_if_0arg(Opcode::IfInWater, tokens, pos)?,
                    "ifoutside" => self.compile_if_0arg(Opcode::IfOutside, tokens, pos)?,
                    "ifmultiplayer" => self.compile_if_0arg(Opcode::IfMultiplayer, tokens, pos)?,
                    "ifinspace" => self.compile_if_0arg(Opcode::IfInSpace, tokens, pos)?,
                    "ifinouterspace" => self.compile_if_0arg(Opcode::IfInOuterSpace, tokens, pos)?,
                    "ifbulletnear" => self.compile_if_0arg(Opcode::IfBulletNear, tokens, pos)?,
                    "ifrespawn" => self.compile_if_0arg(Opcode::IfRespawn, tokens, pos)?,
                    "ifnotmoving" => self.compile_if_0arg(Opcode::IfNotMoving, tokens, pos)?,
                    "ifawayfromwall" => self.compile_if_0arg(Opcode::IfAwayFromWall, tokens, pos)?,
                    "ifnosounds" => self.compile_if_0arg(Opcode::IfNoSounds, tokens, pos)?,
                    "ifhitspace" => self.compile_if_0arg(Opcode::IfHitSpace, tokens, pos)?,
                    "ifactornotstayput" => self.compile_if_0arg(Opcode::IfActorNotStayput, tokens, pos)?,
                    "ifgotweaponce" => self.compile_if_1arg(Opcode::IfGotWeaponCe, tokens, pos)?,

                    // State / Subroutine invocation
                    "state" => {
                        *pos += 1;
                        let state_name = self.expect_ident(tokens, pos)?;
                        let state_addr = self.symbols.get(&state_name).copied().unwrap_or(0);
                        self.bytecode.push(Opcode::State as i32);
                        self.bytecode.push(state_addr);
                    }
                    "action" => {
                        *pos += 1;
                        let act_val = self.expect_num_or_symbol(tokens, pos)?;
                        self.bytecode.push(Opcode::Action as i32);
                        self.bytecode.push(act_val);
                    }
                    "move" => {
                        *pos += 1;
                        let mov_val = self.expect_num_or_symbol(tokens, pos)?;
                        let mut flags = 0;
                        while *pos < tokens.len() && self.is_value(&tokens[*pos]) {
                            flags |= self.expect_num_or_symbol(tokens, pos)?;
                        }
                        self.bytecode.push(Opcode::Move as i32);
                        self.bytecode.push(mov_val);
                        self.bytecode.push(flags);
                    }
                    "ai" => {
                        *pos += 1;
                        let ai_val = self.expect_num_or_symbol(tokens, pos)?;
                        self.bytecode.push(Opcode::Ai as i32);
                        self.bytecode.push(ai_val);
                    }

                    // 0-argument Commands
                    "killit" => { *pos += 1; self.bytecode.push(Opcode::Killit as i32); }
                    "fall" => { *pos += 1; self.bytecode.push(Opcode::Fall as i32); }
                    "break" => { *pos += 1; self.bytecode.push(Opcode::Break as i32); }
                    "resetcount" => { *pos += 1; self.bytecode.push(Opcode::ResetCount as i32); }
                    "resetactioncount" => { *pos += 1; self.bytecode.push(Opcode::ResetActionCount as i32); }
                    "resetplayer" => { *pos += 1; self.bytecode.push(Opcode::ResetPlayer as i32); }
                    "pstomp" => { *pos += 1; self.bytecode.push(Opcode::PStomp as i32); }
                    "wackplayer" => { *pos += 1; self.bytecode.push(Opcode::WackPlayer as i32); }
                    "operate" => { *pos += 1; self.bytecode.push(Opcode::Operate as i32); }
                    "respawnhitag" => { *pos += 1; self.bytecode.push(Opcode::RespawnHitag as i32); }
                    "tip" => { *pos += 1; self.bytecode.push(Opcode::Tip as i32); }
                    "getlastpal" => { *pos += 1; self.bytecode.push(Opcode::GetLastPal as i32); }
                    "pkick" => { *pos += 1; self.bytecode.push(Opcode::PKick as i32); }
                    "mikesnd" => { *pos += 1; self.bytecode.push(Opcode::MikeSnd as i32); }
                    "tossweapon" => { *pos += 1; self.bytecode.push(Opcode::TossWeapon as i32); }
                    "nullop" => { *pos += 1; self.bytecode.push(Opcode::NullOp as i32); }

                    // 1-argument Commands
                    "strength" => self.compile_1arg(Opcode::Strength, tokens, pos)?,
                    "addstrength" => self.compile_1arg(Opcode::AddStrength, tokens, pos)?,
                    "addphealth" => self.compile_1arg(Opcode::AddPHealth, tokens, pos)?,
                    "count" => self.compile_1arg(Opcode::Count, tokens, pos)?,
                    "cactor" => self.compile_1arg(Opcode::CActor, tokens, pos)?,
                    "cstat" => self.compile_1arg(Opcode::CStat, tokens, pos)?,
                    "cstator" => self.compile_1arg(Opcode::CStatOr, tokens, pos)?,
                    "spritepal" => self.compile_1arg(Opcode::SpritePal, tokens, pos)?,
                    "clipdist" => self.compile_1arg(Opcode::ClipDist, tokens, pos)?,
                    "sound" => self.compile_1arg(Opcode::Sound, tokens, pos)?,
                    "soundonce" => self.compile_1arg(Opcode::SoundOnce, tokens, pos)?,
                    "stopsound" => self.compile_1arg(Opcode::StopSound, tokens, pos)?,
                    "globalsound" => self.compile_1arg(Opcode::GlobalSound, tokens, pos)?,
                    "spawn" => self.compile_1arg(Opcode::Spawn, tokens, pos)?,
                    "shoot" => self.compile_1arg(Opcode::Shoot, tokens, pos)?,
                    "money" => self.compile_1arg(Opcode::Money, tokens, pos)?,
                    "mail" => self.compile_1arg(Opcode::Mail, tokens, pos)?,
                    "paper" => self.compile_1arg(Opcode::Paper, tokens, pos)?,
                    "lotsofglass" => self.compile_1arg(Opcode::LotsOfGlass, tokens, pos)?,
                    "sleeptime" => self.compile_1arg(Opcode::SleepTime, tokens, pos)?,
                    "quote" => self.compile_1arg(Opcode::Quote, tokens, pos)?,
                    "addkills" => self.compile_1arg(Opcode::AddKills, tokens, pos)?,
                    "endofgame" => self.compile_1arg(Opcode::EndOfGame, tokens, pos)?,
                    "debug" => self.compile_1arg(Opcode::Debug, tokens, pos)?,

                    // 2-argument Commands
                    "addammo" => self.compile_2args(Opcode::AddAmmo, tokens, pos)?,
                    "addweapon" => self.compile_2args(Opcode::AddWeapon, tokens, pos)?,
                    "addinventory" => self.compile_2args(Opcode::AddInventory, tokens, pos)?,
                    "debris" => self.compile_2args(Opcode::Debris, tokens, pos)?,
                    "guts" => self.compile_2args(Opcode::Guts, tokens, pos)?,
                    "sizeto" => self.compile_2args(Opcode::SizeTo, tokens, pos)?,
                    "sizeat" => self.compile_2args(Opcode::SizeAt, tokens, pos)?,

                    // 4-argument Commands
                    "palfrom" => {
                        *pos += 1;
                        let a1 = self.expect_num_or_symbol(tokens, pos)?;
                        let a2 = self.expect_num_or_symbol(tokens, pos)?;
                        let a3 = self.expect_num_or_symbol(tokens, pos)?;
                        let a4 = self.expect_num_or_symbol(tokens, pos)?;
                        self.bytecode.extend_from_slice(&[Opcode::PalFrom as i32, a1, a2, a3, a4]);
                    }

                    // 5-argument Commands
                    "hitradius" => {
                        *pos += 1;
                        let a1 = self.expect_num_or_symbol(tokens, pos)?;
                        let a2 = self.expect_num_or_symbol(tokens, pos)?;
                        let a3 = self.expect_num_or_symbol(tokens, pos)?;
                        let a4 = self.expect_num_or_symbol(tokens, pos)?;
                        let a5 = self.expect_num_or_symbol(tokens, pos)?;
                        self.bytecode.extend_from_slice(&[Opcode::HitRadius as i32, a1, a2, a3, a4, a5]);
                    }

                    _ => {
                        *pos += 1;
                    }
                }
            }
            _ => *pos += 1,
        }

        Ok(())
    }

    fn compile_1arg(&mut self, op: Opcode, tokens: &[Token], pos: &mut usize) -> Result<(), String> {
        *pos += 1;
        let arg = self.expect_num_or_symbol(tokens, pos)?;
        self.bytecode.push(op as i32);
        self.bytecode.push(arg);
        Ok(())
    }

    fn compile_2args(&mut self, op: Opcode, tokens: &[Token], pos: &mut usize) -> Result<(), String> {
        *pos += 1;
        let arg1 = self.expect_num_or_symbol(tokens, pos)?;
        let arg2 = self.expect_num_or_symbol(tokens, pos)?;
        self.bytecode.push(op as i32);
        self.bytecode.push(arg1);
        self.bytecode.push(arg2);
        Ok(())
    }

    fn compile_if_0arg(&mut self, op: Opcode, tokens: &[Token], pos: &mut usize) -> Result<(), String> {
        *pos += 1;
        self.bytecode.push(op as i32);
        let fail_jump_pos = self.bytecode.len();
        self.bytecode.push(0); // Placeholder for fail jump

        self.compile_statement(tokens, pos)?;

        let end_then = self.bytecode.len();
        self.bytecode[fail_jump_pos] = end_then as i32;

        self.check_else(tokens, pos, fail_jump_pos)?;
        Ok(())
    }

    fn compile_if_1arg(&mut self, op: Opcode, tokens: &[Token], pos: &mut usize) -> Result<(), String> {
        *pos += 1;
        let arg = self.expect_num_or_symbol(tokens, pos)?;
        self.bytecode.push(op as i32);
        self.bytecode.push(arg);
        let fail_jump_pos = self.bytecode.len();
        self.bytecode.push(0); // Placeholder for fail jump

        self.compile_statement(tokens, pos)?;

        let end_then = self.bytecode.len();
        self.bytecode[fail_jump_pos] = end_then as i32;

        self.check_else(tokens, pos, fail_jump_pos)?;
        Ok(())
    }

    fn compile_if_2args(&mut self, op: Opcode, tokens: &[Token], pos: &mut usize) -> Result<(), String> {
        *pos += 1;
        let arg1 = self.expect_num_or_symbol(tokens, pos)?;
        let arg2 = self.expect_num_or_symbol(tokens, pos)?;
        self.bytecode.push(op as i32);
        self.bytecode.push(arg1);
        self.bytecode.push(arg2);
        let fail_jump_pos = self.bytecode.len();
        self.bytecode.push(0); // Placeholder for fail jump

        self.compile_statement(tokens, pos)?;

        let end_then = self.bytecode.len();
        self.bytecode[fail_jump_pos] = end_then as i32;

        self.check_else(tokens, pos, fail_jump_pos)?;
        Ok(())
    }

    fn compile_if_flags(&mut self, op: Opcode, tokens: &[Token], pos: &mut usize) -> Result<(), String> {
        *pos += 1;
        let mut flags = 0;
        while *pos < tokens.len() && self.is_value(&tokens[*pos]) {
            flags |= self.expect_num_or_symbol(tokens, pos)?;
        }
        self.bytecode.push(op as i32);
        self.bytecode.push(flags);
        let fail_jump_pos = self.bytecode.len();
        self.bytecode.push(0);

        self.compile_statement(tokens, pos)?;

        let end_then = self.bytecode.len();
        self.bytecode[fail_jump_pos] = end_then as i32;

        self.check_else(tokens, pos, fail_jump_pos)?;
        Ok(())
    }

    fn check_else(&mut self, tokens: &[Token], pos: &mut usize, if_fail_pos: usize) -> Result<(), String> {
        if *pos < tokens.len() {
            if let Token::Ident(s) = &tokens[*pos] {
                if s.eq_ignore_ascii_case("else") {
                    *pos += 1;
                    let else_jump_pos = self.bytecode.len();
                    self.bytecode.push(Opcode::Else as i32);
                    self.bytecode.push(0); // Placeholder for jump past else

                    // Update IF fail jump to point right to the ELSE opcode
                    self.bytecode[if_fail_pos] = else_jump_pos as i32;

                    self.compile_statement(tokens, pos)?;

                    let end_else = self.bytecode.len();
                    self.bytecode[else_jump_pos + 1] = end_else as i32;
                }
            }
        }
        Ok(())
    }

    fn expect_ident(&self, tokens: &[Token], pos: &mut usize) -> Result<String, String> {
        if *pos < tokens.len() {
            if let Token::Ident(s) = &tokens[*pos] {
                *pos += 1;
                return Ok(s.clone());
            }
        }
        Err(format!("Expected identifier at pos {}", *pos))
    }

    fn expect_ident_or_str(&self, tokens: &[Token], pos: &mut usize) -> Result<String, String> {
        if *pos < tokens.len() {
            match &tokens[*pos] {
                Token::Ident(s) => {
                    *pos += 1;
                    return Ok(s.clone());
                }
                Token::Str(s) => {
                    *pos += 1;
                    return Ok(s.clone());
                }
                Token::Number(n) => {
                    *pos += 1;
                    return Ok(n.to_string());
                }
                _ => {}
            }
        }
        Err(format!("Expected string or identifier at pos {}", *pos))
    }

    fn expect_num_or_symbol(&self, tokens: &[Token], pos: &mut usize) -> Result<i32, String> {
        if *pos < tokens.len() {
            match &tokens[*pos] {
                Token::Number(n) => {
                    *pos += 1;
                    return Ok(*n);
                }
                Token::Ident(s) => {
                    *pos += 1;
                    if let Some(&val) = self.symbols.get(s) {
                        return Ok(val);
                    } else if let Ok(parsed) = s.parse::<i32>() {
                        return Ok(parsed);
                    } else {
                        return Ok(0); // Default to 0 for unknown symbol
                    }
                }
                _ => {}
            }
        }
        Err(format!("Expected number or symbol at pos {}", *pos))
    }

    fn is_value(&self, token: &Token) -> bool {
        match token {
            Token::Number(_) => true,
            Token::Ident(s) => !is_keyword(s),
            _ => false,
        }
    }
}

pub fn is_keyword(s: &str) -> bool {
    let kw = s.to_lowercase();
    matches!(
        kw.as_str(),
        "define" | "definevolumename" | "defineskillname" | "definelevelname" | "definequote" | "definesound"
        | "action" | "move" | "ai" | "state" | "ends" | "actor" | "useractor" | "enda"
        | "ifpdistl" | "ifpdistg" | "ifcansee" | "ifhitweapon" | "ifdead" | "sound" | "killit"
        | "else" | "{" | "}" | "ifrnd" | "ifcount" | "ifactioncount" | "ifaction" | "ifmove"
        | "ifai" | "ifactor" | "ifstrength" | "ifwasweapon" | "ifspawnedby" | "ifpinventory"
        | "ifspritepal" | "ifphealthl" | "ifangdiffl" | "iffloordistl" | "ifceilingdistl"
        | "ifgapzl" | "ifp" | "ifcanseetarget" | "ifcanshoottarget" | "ifsquished" | "ifonwater"
        | "ifinwater" | "ifoutside" | "ifmultiplayer" | "ifinspace" | "ifinouterspace"
        | "ifbulletnear" | "ifrespawn" | "ifnotmoving" | "ifawayfromwall" | "ifnosounds"
        | "ifhitspace" | "ifactornotstayput" | "ifgotweaponce" | "strength" | "addstrength"
        | "addphealth" | "count" | "cactor" | "cstat" | "cstator" | "spritepal" | "clipdist"
        | "soundonce" | "stopsound" | "globalsound" | "spawn" | "shoot" | "money" | "mail"
        | "paper" | "lotsofglass" | "sleeptime" | "quote" | "addkills" | "endofgame" | "debug"
        | "addammo" | "addweapon" | "addinventory" | "debris" | "guts" | "sizeto" | "sizeat"
        | "palfrom" | "hitradius" | "fall" | "break" | "resetcount" | "resetactioncount"
        | "resetplayer" | "pstomp" | "wackplayer" | "operate" | "respawnhitag" | "tip"
        | "getlastpal" | "pkick" | "mikesnd" | "tossweapon" | "nullop"
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_compiler_defines_and_actions() {
        let script = r#"
            define TROOP 1680
            define STRENGTH 100
            action ATROOPSTAND 0 1 5 1 1
            move TROOPVELS 32 0
            ai AITROOP ATROOPSTAND TROOPVELS seekplayer face_player

            actor TROOP STRENGTH ATROOPSTAND TROOPVELS seekplayer
                ifdead
                    killit
                else
                    sound 5
            enda
        "#;

        let mut compiler = Compiler::new();
        let compiled = compiler.compile(script).unwrap();

        assert_eq!(compiled.symbols.get("TROOP"), Some(&1680));
        assert_eq!(compiled.symbols.get("STRENGTH"), Some(&100));
        assert!(compiled.actions.contains_key("ATROOPSTAND"));
        assert!(compiled.moves.contains_key("TROOPVELS"));
        assert!(compiled.ais.contains_key("AITROOP"));

        assert!(compiled.actor_script_ptrs[1680].is_some());
    }

    #[test]
    fn test_compiler_state_subroutines() {
        let script = r#"
            define PIGCOP 2000
            state check_alive
                ifdead
                    killit
            ends

            actor PIGCOP 100
                state check_alive
            enda
        "#;

        let mut compiler = Compiler::new();
        let compiled = compiler.compile(script).unwrap();

        assert!(compiled.symbols.contains_key("check_alive"));
        assert!(compiled.actor_script_ptrs[2000].is_some());
    }

    #[test]
    fn test_compiler_useractor_and_blocks() {
        let script = r#"
            define LIZMAN 2120
            action ALIZWALK 0 4 5 1 10
            move LIZVELS 40 0

            useractor 1 LIZMAN 150 ALIZWALK LIZVELS geth getv
                ifpdistl 2048 {
                    ifcansee {
                        sound 20
                        shoot 1600
                    } else {
                        wackplayer
                    }
                }
                cactor 2121
                spritepal 2
                clipdist 64
            enda
        "#;

        let mut compiler = Compiler::new();
        let compiled = compiler.compile(script).unwrap();

        assert!(compiled.actor_script_ptrs[2120].is_some());
        assert_eq!(compiled.actor_types[2120], 1); // Enemy
    }

    #[test]
    fn test_compiler_dynamic_con_definitions() {
        let script = r#"
            definevolumename 0 L.A. MELTDOWN
            definevolumename 1 LUNAR APOCALYPSE
            defineskillname 0 PIECE OF CAKE
            defineskillname 1 LET'S ROCK
            definelevelname 0 0 E1L1.map 01:45 00:53 HOLLYWOOD HOLOCAUST
            definelevelname 0 1 E1L2.map 05:10 03:21 RED LIGHT DISTRICT
            definequote 113 CLIPPING: OFF
            definesound 78 DUKE_LOOKING_GOOD.VOC 0 0 100 4 255
        "#;

        let mut compiler = Compiler::new();
        let compiled = compiler.compile(script).unwrap();

        assert_eq!(compiled.volumes.len(), 2);
        assert_eq!(compiled.volumes[0].volume_id, 0);
        assert_eq!(compiled.volumes[0].title, "L.A. MELTDOWN");
        assert_eq!(compiled.volumes[1].title, "LUNAR APOCALYPSE");

        assert_eq!(compiled.skills.len(), 2);
        assert_eq!(compiled.skills[0].title, "PIECE OF CAKE");

        assert_eq!(compiled.levels.len(), 2);
        assert_eq!(compiled.levels[0].filename, "E1L1.map");
        assert_eq!(compiled.levels[0].par_time_str, "01:45");
        assert_eq!(compiled.levels[0].title, "HOLLYWOOD HOLOCAUST");

        assert_eq!(compiled.quotes.get(&113).map(|s| s.as_str()), Some("CLIPPING: OFF"));

        assert_eq!(compiled.sounds.len(), 1);
        assert_eq!(compiled.sounds[0].sound_id, 78);
        assert_eq!(compiled.sounds[0].filename, "DUKE_LOOKING_GOOD.VOC");
    }
}
