#![allow(dead_code)]

pub mod types;
pub mod lexer;
pub mod compiler;
pub mod vm;
pub mod physics;

pub use types::*;
pub use compiler::{Compiler, CompiledScript};
pub use vm::{ConVm, VmActorContext};
pub use physics::TrigTables;

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
            compiled.actor_script_ptrs,
            compiled.actor_types,
        );
        let trig = TrigTables::new();
        Ok(Self { vm, trig, compiled })
    }
}

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
        let mut x = 0; let mut y = 0; let mut z = 0;
        let mut ang = 0; let mut xvel = 0; let mut zvel = 0;
        let mut extra = 100;
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

        engine.vm.execute(&mut ctx);
        assert_eq!(ctx.sound_events.len(), 1);
        assert_eq!(ctx.sound_events[0].0, 15);
    }
}
