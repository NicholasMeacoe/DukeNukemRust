#![allow(dead_code)]
use crate::art::PicAnm;
use bevy::prelude::*;
use std::collections::HashMap;

#[derive(Resource)]
pub struct EngineClock {
    pub total_clock_120hz: u32,
    pub timer: Timer,
}

impl Default for EngineClock {
    fn default() -> Self {
        Self {
            total_clock_120hz: 0,
            timer: Timer::from_seconds(1.0 / 120.0, TimerMode::Repeating),
        }
    }
}

#[derive(Component)]
pub struct AnimatedTileMaterial {
    pub base_picnum: i16,
    pub picanm: PicAnm,
    pub current_offset: i32,
    pub material_handle: Handle<StandardMaterial>,
}

#[derive(Resource, Default)]
pub struct TileAnmRegistry {
    pub picanm_map: HashMap<i16, PicAnm>,
}

pub fn update_engine_clock(time: Res<Time>, mut clock: ResMut<EngineClock>) {
    clock.timer.tick(time.delta());
    let ticks = clock.timer.times_finished_this_tick();
    clock.total_clock_120hz = clock.total_clock_120hz.wrapping_add(ticks);
}

pub fn update_tile_animations(
    clock: Res<EngineClock>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut query: Query<&mut AnimatedTileMaterial>,
    assets: Res<crate::GameAssets>,
) {
    if !clock.is_changed() {
        return;
    }

    for mut anim in query.iter_mut() {
        let offset = anim.picanm.get_frame_offset(clock.total_clock_120hz);
        if offset != anim.current_offset {
            anim.current_offset = offset;
            let target_picnum = anim.base_picnum + offset as i16;
            if let Some(tex_handle) = assets.tile_textures.get(&target_picnum) {
                if let Some(mat) = materials.get_mut(&anim.material_handle) {
                    mat.base_color_texture = Some(tex_handle.clone());
                }
            }
        }
    }
}
