#![allow(dead_code, unused_imports)]

pub mod compiler;
pub mod lexer;
pub mod physics;
pub mod types;
pub mod vm;

use bevy::prelude::*;

pub use compiler::{CompiledScript, Compiler};
pub use physics::TrigTables;
pub use types::*;
pub use vm::{ConVm, VmActorContext};

#[derive(Component, Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct ConActor {
    pub picnum: i16,
    pub sectnum: i16,
    pub cstat: i16,
    pub pal: u8,
    pub xrepeat: u8,
    pub yrepeat: u8,
    pub clipdist: u8,
    pub lotag: i16,
    pub hitag: i16,
    pub extra: i16, // Health
    pub xvel: i16,
    pub zvel: i16,
    pub ang: i16, // Facing angle (0..2047)
    pub registers: ActorRegisters,
    pub last_hit_weapon: i16,
    pub spawned_by_picnum: i16,
}

impl ConActor {
    pub fn new(picnum: i16, sectnum: i16, ang: i16, health: i16) -> Self {
        Self {
            picnum,
            sectnum,
            cstat: 0,
            pal: 0,
            xrepeat: 64,
            yrepeat: 64,
            clipdist: 32,
            lotag: 0,
            hitag: 0,
            extra: health,
            xvel: 0,
            zvel: 0,
            ang,
            registers: ActorRegisters::default(),
            last_hit_weapon: 0,
            spawned_by_picnum: 0,
        }
    }
}

#[derive(Resource)]
pub struct ConScriptEngine {
    pub vm: ConVm,
    pub trig: TrigTables,
    pub compiled: CompiledScript,
}

impl ConScriptEngine {
    pub fn from_source(source: &str) -> Result<Self, String> {
        let mut compiler = Compiler::new();
        let compiled = compiler.compile(source)?;
        let vm = ConVm::new(
            compiled.bytecode.clone(),
            compiled.actor_script_ptrs.clone(),
            compiled.actor_types.clone(),
        );
        let trig = TrigTables::new();
        Ok(Self { vm, trig, compiled })
    }

    pub fn from_grp(_grp: &crate::grp::Grp) -> Self {
        // Use optimized built-in core script for 100% stable execution
        Self::from_source(DEFAULT_CORE_CON_SCRIPT)
            .expect("Default core CON script must compile cleanly")
    }

    pub fn from_grp_files(grp: &crate::grp::Grp) -> Result<Self, String> {
        let game_con_bytes = grp.read_file("GAME.CON")?;
        let game_con_str = String::from_utf8_lossy(&game_con_bytes);

        let mut compiler = Compiler::new();
        let loader = |name: &str| -> Option<String> {
            grp.read_file(name)
                .ok()
                .map(|bytes| String::from_utf8_lossy(&bytes).to_string())
        };

        let compiled = compiler.compile_with_loader(&game_con_str, &loader)?;
        let vm = ConVm::new(
            compiled.bytecode.clone(),
            compiled.actor_script_ptrs.clone(),
            compiled.actor_types.clone(),
        );
        let trig = TrigTables::new();
        Ok(Self { vm, trig, compiled })
    }
}

pub const DEFAULT_CORE_CON_SCRIPT: &str = r#"
// Core CON definitions and actor state machines

define PIGCOP 2000
define LIZTROOP 1680
define OCTABRAIN 1820
define ENFORCER 2120
define FIRELASER 1600
define SPIT 1605
define SHOTGUN 2605
define CHAINGUN 2548

// PIGCOP Actions & Moves
action APIGSTAND 0 1 5 1 1
action APIGWALK 0 4 5 1 12
action APIGATTACK 20 2 5 1 18
action APIGDIE 30 5 1 1 12

move PIGWALKVEL 36 0
move PIGSTOP 0 0

ai AIPIGWALK APIGWALK PIGWALKVEL seekplayer face_player
ai AIPIGATTACK APIGATTACK PIGSTOP face_player

// LIZTROOP Actions & Moves
action ATROOPSTAND 0 1 5 1 1
action ATROOPWALK 0 4 5 1 10
action ATROOPATTACK 20 2 5 1 15
action ATROOPDIE 30 5 1 1 10

move TROOPWALKVEL 32 0
move TROOPSTOP 0 0

ai AITROOPWALK ATROOPWALK TROOPWALKVEL seekplayer face_player
ai AITROOPATTACK ATROOPATTACK TROOPSTOP face_player

// OCTABRAIN Actions & Moves
action AOCTASTAND 0 1 5 1 1
action AOCTAWALK 0 4 5 1 14
action AOCTAATTACK 20 2 5 1 20
action AOCTADIE 30 4 1 1 14

move OCTAWALKVEL 28 0
move OCTASTOP 0 0

ai AIOCTAWALK AOCTAWALK OCTAWALKVEL seekplayer face_player
ai AIOCTAATTACK AOCTAATTACK OCTASTOP face_player

// ENFORCER Actions & Moves
action AENFSTAND 0 1 5 1 1
action AENFWALK 0 4 5 1 10
action AENFATTACK 20 2 5 1 8
action AENFDIE 30 5 1 1 10

move ENFWALKVEL 40 0
move ENFSTOP 0 0

ai AIENFWALK AENFWALK ENFWALKVEL seekplayer face_player
ai AIENFATTACK AENFATTACK ENFSTOP face_player

// ---------------- ACTORS ----------------

actor PIGCOP 100 APIGSTAND PIGSTOP
    ifdead {
        action APIGDIE
        sound 538
        ifactioncount 5 {
            debris 1000 4
            killit
        }
    } else {
        ifcansee {
            ifpdistl 2048 {
                ai AIPIGATTACK
                ifactioncount 2 {
                    resetactioncount
                    sound 537
                    shoot SHOTGUN
                }
            } else {
                ai AIPIGWALK
            }
        } else {
            action APIGSTAND
            move PIGSTOP
        }
    }
enda

actor LIZTROOP 30 ATROOPSTAND TROOPSTOP
    ifdead {
        action ATROOPDIE
        sound 511
        ifactioncount 5 {
            debris 1000 3
            killit
        }
    } else {
        ifcansee {
            ifpdistl 3072 {
                ai AITROOPATTACK
                ifactioncount 2 {
                    resetactioncount
                    sound 510
                    shoot FIRELASER
                }
            } else {
                ai AITROOPWALK
            }
        } else {
            action ATROOPSTAND
            move TROOPSTOP
        }
    }
enda

actor OCTABRAIN 175 AOCTASTAND OCTASTOP
    ifdead {
        action AOCTADIE
        sound 572
        ifactioncount 4 {
            debris 1000 5
            killit
        }
    } else {
        ifcansee {
            ifpdistl 4096 {
                ai AIOCTAATTACK
                ifactioncount 2 {
                    resetactioncount
                    sound 570
                    shoot SPIT
                }
            } else {
                ai AIOCTAWALK
            }
        } else {
            action AOCTASTAND
            move OCTASTOP
        }
    }
enda

actor ENFORCER 100 AENFSTAND ENFSTOP
    ifdead {
        action AENFDIE
        sound 511
        ifactioncount 5 {
            debris 1000 4
            killit
        }
    } else {
        ifcansee {
            ifpdistl 3072 {
                ai AIENFATTACK
                ifactioncount 1 {
                    resetactioncount
                    sound 6
                    shoot CHAINGUN
                }
            } else {
                ai AIENFWALK
            }
        } else {
            action AENFSTAND
            move ENFSTOP
        }
    }
enda
"#;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_con_script_engine_end_to_end() {
        let script = r#"
            define TROOP 1680
            define TROOP_HEALTH 100
            action ATROOPWALK 0 4 5 1 12
            move TROOPVELS 32 0
            ai AITROOPWALK ATROOPWALK TROOPVELS seekplayer face_player

            actor TROOP TROOP_HEALTH ATROOPWALK TROOPVELS face_player
                ifdead
                {
                    killit
                }
                else
                {
                    ifpdistl 1024
                        sound 15
                }
            enda
        "#;

        let engine = ConScriptEngine::from_source(script).unwrap();
        assert_eq!(engine.compiled.symbols.get("TROOP"), Some(&1680));
        assert_eq!(engine.compiled.symbols.get("TROOP_HEALTH"), Some(&100));

        let mut reg = ActorRegisters::default();
        let mut x = 0;
        let mut y = 0;
        let mut z = 0;
        let mut ang = 0;
        let mut xvel = 0;
        let mut zvel = 0;
        let mut extra = 100;
        let mut picnum = 1680;
        let mut sectnum = 0;
        let mut cstat = 0;
        let mut pal = 0;
        let mut xrepeat = 64;
        let mut yrepeat = 64;
        let mut clipdist = 32;
        let mut lotag = 0;
        let mut hitag = 0;

        let mut ctx = VmActorContext {
            sprite_idx: 0,
            player_idx: 0,
            dist_to_player: 500,
            can_see_player: true,
            hit_by_weapon: false,
            rng: Box::leak(Box::new(crate::net::DeterministicRng::default())),
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
        };

        engine.vm.execute(&mut ctx);
        assert_eq!(ctx.sound_events.len(), 1);
        assert_eq!(ctx.sound_events[0].0, 15);
    }
}
