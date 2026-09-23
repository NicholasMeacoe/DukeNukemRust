use crate::art::PicAnm;
use bevy::prelude::*;

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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_water_tile_picanm_frame_progression() {
        // Water caustics: 4 frames total (num_frames = 3), forward loop (anim_type = 2), speed = 3 (8 ticks per frame)
        let water_picanm = PicAnm {
            num_frames: 3,
            anim_type: 2,
            x_offset: 0,
            y_offset: 0,
            speed: 3,
        };

        assert_eq!(water_picanm.get_frame_offset(0), 0);
        assert_eq!(water_picanm.get_frame_offset(7), 0);
        assert_eq!(water_picanm.get_frame_offset(8), 1);
        assert_eq!(water_picanm.get_frame_offset(15), 1);
        assert_eq!(water_picanm.get_frame_offset(16), 2);
        assert_eq!(water_picanm.get_frame_offset(23), 2);
        assert_eq!(water_picanm.get_frame_offset(24), 3);
        assert_eq!(water_picanm.get_frame_offset(31), 3);
        assert_eq!(water_picanm.get_frame_offset(32), 0); // Wraps back to frame 0!

        // Slime: 4 frames oscillation (anim_type = 1, speed = 3)
        let slime_picanm = PicAnm {
            num_frames: 3,
            anim_type: 1,
            x_offset: 0,
            y_offset: 0,
            speed: 3,
        };
        assert_eq!(slime_picanm.get_frame_offset(0), 0);
        assert_eq!(slime_picanm.get_frame_offset(8), 1);
        assert_eq!(slime_picanm.get_frame_offset(16), 2);
        assert_eq!(slime_picanm.get_frame_offset(24), 3);
        assert_eq!(slime_picanm.get_frame_offset(32), 2); // Ping-pong back to 2!
        assert_eq!(slime_picanm.get_frame_offset(40), 1); // Ping-pong back to 1!
        assert_eq!(slime_picanm.get_frame_offset(48), 0); // Ping-pong back to 0!
    }

    #[test]
    fn test_water_surface_animated_tile_material_swapping() {
        let mut app = App::new();
        app.init_resource::<EngineClock>();

        let mut images = Assets::<Image>::default();
        let h0 = images.add(Image::default());
        let h1 = images.add(Image::default());
        let h2 = images.add(Image::default());
        let h3 = images.add(Image::default());

        let mut tile_textures = std::collections::HashMap::new();
        tile_textures.insert(336, h0.clone());
        tile_textures.insert(337, h1.clone());
        tile_textures.insert(338, h2.clone());
        tile_textures.insert(339, h3.clone());

        app.insert_resource(crate::GameAssets {
            tile_textures,
            tile_sizes: std::collections::HashMap::new(),
            picanm_map: std::collections::HashMap::new(),
            default_material: Handle::default(),
            spark_material: Handle::default(),
            spark_mesh: Handle::default(),
            grp_path: String::new(),
        });

        let mut materials = Assets::<StandardMaterial>::default();
        let mat_handle = materials.add(StandardMaterial {
            base_color_texture: Some(h0.clone()),
            ..default()
        });
        app.insert_resource(materials);

        let water_picanm = PicAnm {
            num_frames: 3,
            anim_type: 2,
            x_offset: 0,
            y_offset: 0,
            speed: 3,
        };

        app.world_mut().spawn(AnimatedTileMaterial {
            base_picnum: 336,
            picanm: water_picanm,
            current_offset: 0,
            material_handle: mat_handle.clone(),
        });

        app.add_systems(Update, update_tile_animations);

        // Frame 0 at clock 0
        app.update();
        let mats = app.world().resource::<Assets<StandardMaterial>>();
        assert_eq!(mats.get(&mat_handle).unwrap().base_color_texture, Some(h0.clone()));

        // Advance clock by 8 ticks -> offset 1 (tile 337)
        {
            let mut clock = app.world_mut().resource_mut::<EngineClock>();
            clock.total_clock_120hz = 8;
        }
        app.update();
        let mats = app.world().resource::<Assets<StandardMaterial>>();
        assert_eq!(mats.get(&mat_handle).unwrap().base_color_texture, Some(h1.clone()));

        // Advance clock to 24 ticks -> offset 3 (tile 339)
        {
            let mut clock = app.world_mut().resource_mut::<EngineClock>();
            clock.total_clock_120hz = 24;
        }
        app.update();
        let mats = app.world().resource::<Assets<StandardMaterial>>();
        assert_eq!(mats.get(&mat_handle).unwrap().base_color_texture, Some(h3.clone()));

        // Advance clock to 32 ticks -> wraps back to offset 0 (tile 336)
        {
            let mut clock = app.world_mut().resource_mut::<EngineClock>();
            clock.total_clock_120hz = 32;
        }
        app.update();
        let mats = app.world().resource::<Assets<StandardMaterial>>();
        assert_eq!(mats.get(&mat_handle).unwrap().base_color_texture, Some(h0.clone()));
    }
}
