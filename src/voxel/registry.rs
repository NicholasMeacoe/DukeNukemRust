use bevy::prelude::*;
use std::collections::HashMap;
use crate::names::*;
use crate::palette::Palette;
use super::kvx::KvxModel;
use super::mesh::{generate_voxel_mesh, DEFAULT_VOXEL_SCALE};

/// Configuration cvars for 3D voxel model replacement.
#[derive(Resource, Clone, Debug)]
pub struct VoxelConfig {
    pub enabled: bool,
    pub rotate_pickups: bool,
    pub bob_pickups: bool,
    pub rotation_speed: f32,
    pub bob_speed: f32,
    pub bob_amplitude: f32,
}

impl Default for VoxelConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            rotate_pickups: true,
            bob_pickups: true,
            rotation_speed: 2.2,
            bob_speed: 3.2,
            bob_amplitude: 0.08,
        }
    }
}

/// Tag component identifying an instantiated 3D voxel model in the world.
#[derive(Component, Clone, Debug)]
pub struct VoxelModelInstance {
    pub picnum: i16,
    pub rotates: bool,
    pub bobs: bool,
    pub base_y: f32,
    pub bob_phase: f32,
}

impl VoxelModelInstance {
    pub fn new_pickup(picnum: i16, base_y: f32) -> Self {
        Self {
            picnum,
            rotates: true,
            bobs: true,
            base_y,
            bob_phase: 0.0,
        }
    }
}

/// Central registry mapping tile picnums to 3D KVX voxel models and cached GPU meshes.
#[derive(Resource, Default)]
pub struct VoxelRegistry {
    models: HashMap<i16, KvxModel>,
    mesh_cache: HashMap<i16, Handle<Mesh>>,
    material_handle: Option<Handle<StandardMaterial>>,
}

impl VoxelRegistry {
    pub fn new() -> Self {
        let mut reg = Self::default();
        reg.register_default_models();
        reg
    }

    pub fn register(&mut self, picnum: i16, model: KvxModel) {
        self.models.insert(picnum, model);
        self.mesh_cache.remove(&picnum); // Invalidate cached mesh
    }

    pub fn has_voxel(&self, picnum: i16) -> bool {
        self.models.contains_key(&picnum)
    }

    pub fn get_model(&self, picnum: i16) -> Option<&KvxModel> {
        self.models.get(&picnum)
    }

    pub fn get_or_create_mesh(
        &mut self,
        picnum: i16,
        palette: &Palette,
        meshes: &mut Assets<Mesh>,
    ) -> Option<Handle<Mesh>> {
        if let Some(handle) = self.mesh_cache.get(&picnum) {
            return Some(handle.clone());
        }

        if let Some(model) = self.models.get(&picnum) {
            let mesh = generate_voxel_mesh(model, palette, DEFAULT_VOXEL_SCALE);
            let handle = meshes.add(mesh);
            self.mesh_cache.insert(picnum, handle.clone());
            Some(handle)
        } else {
            None
        }
    }

    pub fn get_or_create_material(
        &mut self,
        materials: &mut Assets<StandardMaterial>,
    ) -> Handle<StandardMaterial> {
        if let Some(ref handle) = self.material_handle {
            handle.clone()
        } else {
            let handle = materials.add(StandardMaterial {
                base_color: Color::WHITE,
                unlit: false, // Receives ambient and dynamic point lighting
                perceptual_roughness: 0.9,
                metallic: 0.0,
                reflectance: 0.1,
                ..default()
            });
            self.material_handle = Some(handle.clone());
            handle
        }
    }

    /// Populates authentic/procedural voxel models for core Duke Nukem 3D pickups and weapons.
    pub fn register_default_models(&mut self) {
        // 1. FIRSTAID / Small Medkit (Tile 53): White box with red cross
        let mut medkit = KvxModel::create_test_box(16, 12, 10, 250); // white/light-grey body
        for x in 6..10 {
            for y in 0..12 {
                medkit.set_voxel(x, y, 9, Some(240)); // Red cross vertical bar
            }
        }
        for y in 4..8 {
            for x in 3..13 {
                medkit.set_voxel(x, y, 9, Some(240)); // Red cross horizontal bar
            }
        }
        self.register(FIRSTAID, medkit);

        // 2. SHIELD / Armor Vest (Tile 54): Blue Kevlar vest
        let mut armor = KvxModel::create_test_box(16, 8, 16, 196); // Blue body
        for x in 6..10 {
            for z in 12..16 {
                for y in 0..8 {
                    armor.set_voxel(x, y, z, None); // Neck cutout
                }
            }
        }
        self.register(SHIELD, armor);

        // 3. ATOMICHEALTH (Tile 99): Glowing orange/yellow nuclear sphere
        let mut atomic = KvxModel::new(14, 14, 14, Vec3::new(7.0, 7.0, 7.0));
        for x in 0..14 {
            for y in 0..14 {
                for z in 0..14 {
                    let dx = x as f32 - 6.5;
                    let dy = y as f32 - 6.5;
                    let dz = z as f32 - 6.5;
                    let dist_sq = dx * dx + dy * dy + dz * dz;
                    if dist_sq <= 36.0 {
                        let color = if dist_sq <= 16.0 {
                            144 // Bright core yellow/white
                        } else {
                            130 // Orange radiation shell
                        };
                        atomic.set_voxel(x, y, z, Some(color));
                    }
                }
            }
        }
        self.register(ATOMICHEALTH, atomic);

        // 4. ACCESSCARD (Tile 60, 175, 176, 177): Thin keycard with metallic strip
        let mut keycard = KvxModel::create_test_box(14, 2, 9, 196); // Blue default
        for z in 3..5 {
            for x in 0..14 {
                keycard.set_voxel(x, 1, z, Some(248)); // Gold/metallic magnetic strip
            }
        }
        self.register(ACCESSCARD, keycard.clone());
        self.register(175, keycard.clone());

        // Red keycard (Tile 176)
        let mut red_card = keycard.clone();
        for x in 0..14 {
            for z in 0..9 {
                if red_card.get_voxel(x, 0, z) == Some(196) {
                    red_card.set_voxel(x, 0, z, Some(240)); // Red body
                    red_card.set_voxel(x, 1, z, Some(240));
                }
            }
        }
        self.register(176, red_card);

        // Yellow keycard (Tile 177)
        let mut yellow_card = keycard.clone();
        for x in 0..14 {
            for z in 0..9 {
                if yellow_card.get_voxel(x, 0, z) == Some(196) {
                    yellow_card.set_voxel(x, 0, z, Some(144)); // Yellow body
                    yellow_card.set_voxel(x, 1, z, Some(144));
                }
            }
        }
        self.register(177, yellow_card);

        // 5. AMMO / Pistol Clip (Tile 40): Black magazine with brass cartridges on top
        let mut ammo_clip = KvxModel::create_test_box(8, 4, 14, 0); // Black magazine
        for x in 2..6 {
            for y in 1..3 {
                ammo_clip.set_voxel(x, y, 13, Some(144)); // Brass tip
            }
        }
        self.register(AMMO, ammo_clip);

        // 6. SHOTGUNAMMO (Tile 49): Red shotgun shell box
        let shotgun_ammo = KvxModel::create_test_box(12, 10, 8, 240); // Red shell box
        self.register(SHOTGUNAMMO, shotgun_ammo);

        // 7. RPGAMMO (Tile 44): Green missile crate
        let rpg_ammo = KvxModel::create_test_box(16, 12, 8, 96); // Olive drab ammo crate
        self.register(RPGAMMO, rpg_ammo);

        // 8. SHOTGUNSPRITE (Tile 28): 3D Shotgun pickup
        let mut shotgun = KvxModel::new(26, 4, 6, Vec3::new(13.0, 2.0, 3.0));
        for x in 0..26 {
            shotgun.set_voxel(x, 1, 2, Some(0)); // Dual steel barrels
            shotgun.set_voxel(x, 2, 2, Some(0));
        }
        for x in 0..10 {
            shotgun.set_voxel(x, 1, 1, Some(160)); // Wood stock/grip
            shotgun.set_voxel(x, 2, 1, Some(160));
        }
        self.register(SHOTGUNSPRITE, shotgun);

        // 9. CHAINGUNSPRITE (Tile 22): 3D Chaingun pickup
        let mut chaingun = KvxModel::new(28, 8, 8, Vec3::new(14.0, 4.0, 4.0));
        for x in 0..14 {
            for y in 2..6 {
                for z in 2..6 {
                    chaingun.set_voxel(x, y, z, Some(24)); // Gun body
                }
            }
        }
        for x in 14..28 {
            chaingun.set_voxel(x, 3, 3, Some(0)); // Triple rotating barrels
            chaingun.set_voxel(x, 5, 3, Some(0));
            chaingun.set_voxel(x, 4, 5, Some(0));
        }
        self.register(CHAINGUNSPRITE, chaingun);

        // 10. RPGSPRITE (Tile 23): 3D RPG Launcher
        let mut rpg = KvxModel::new(32, 6, 8, Vec3::new(16.0, 3.0, 4.0));
        for x in 0..32 {
            rpg.set_voxel(x, 2, 3, Some(96)); // Olive launch tube
            rpg.set_voxel(x, 3, 3, Some(96));
            rpg.set_voxel(x, 2, 4, Some(96));
            rpg.set_voxel(x, 3, 4, Some(96));
        }
        self.register(RPGSPRITE, rpg);
    }
}

/// System that rotates and bobs 3D voxel pickups in the game world.
pub fn update_voxel_instances(
    time: Res<Time>,
    config: Res<VoxelConfig>,
    mut query: Query<(&mut Transform, &mut VoxelModelInstance, &mut Visibility)>,
) {
    let dt = time.delta_seconds();

    for (mut transform, mut instance, mut visibility) in &mut query {
        if !config.enabled {
            *visibility = Visibility::Hidden;
            continue;
        } else {
            *visibility = Visibility::Inherited;
        }

        if config.rotate_pickups && instance.rotates {
            transform.rotate_y(config.rotation_speed * dt);
        }

        if config.bob_pickups && instance.bobs {
            instance.bob_phase += config.bob_speed * dt;
            transform.translation.y =
                instance.base_y + instance.bob_phase.sin() * config.bob_amplitude;
        }
    }
}

pub struct VoxelPlugin;

impl Plugin for VoxelPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<VoxelConfig>()
            .insert_resource(VoxelRegistry::new())
            .add_systems(Update, update_voxel_instances);
    }
}

#[cfg(test)]
pub mod tests {
    use super::*;

    #[test]
    fn test_voxel_registry_registration_and_lookup() {
        let mut reg = VoxelRegistry::default();
        assert!(!reg.has_voxel(53));

        let model = KvxModel::create_test_box(4, 4, 4, 10);
        reg.register(53, model.clone());

        assert!(reg.has_voxel(53));
        assert_eq!(reg.get_model(53), Some(&model));
    }

    #[test]
    fn test_voxel_instance_rotation_and_bobbing_system() {
        let mut app = App::new();
        app.insert_resource(Time::<()>::default())
            .insert_resource(VoxelConfig {
                enabled: true,
                rotate_pickups: true,
                bob_pickups: true,
                rotation_speed: 1.0,
                bob_speed: 2.0,
                bob_amplitude: 0.1,
            })
            .add_systems(Update, update_voxel_instances);

        let entity = app.world_mut().spawn((
            Transform::from_xyz(0.0, 1.0, 0.0),
            VoxelModelInstance::new_pickup(53, 1.0),
            Visibility::Inherited,
        )).id();

        // Advance time by 0.5s
        {
            let mut time = app.world_mut().resource_mut::<Time>();
            time.advance_by(std::time::Duration::from_millis(500));
        }
        app.update();

        let transform = app.world().get::<Transform>(entity).unwrap();
        // Rotation should have changed from Quat::IDENTITY
        assert!(transform.rotation != Quat::IDENTITY);
        // Translation.y should bob around 1.0
        assert!((transform.translation.y - 1.0).abs() <= 0.11);
    }

    #[test]
    fn test_default_voxel_models_generation() {
        let reg = VoxelRegistry::new();
        assert!(reg.has_voxel(FIRSTAID));
        assert!(reg.has_voxel(SHIELD));
        assert!(reg.has_voxel(ATOMICHEALTH));
        assert!(reg.has_voxel(ACCESSCARD));
        assert!(reg.has_voxel(AMMO));
        assert!(reg.has_voxel(SHOTGUNAMMO));
        assert!(reg.has_voxel(RPGAMMO));
        assert!(reg.has_voxel(SHOTGUNSPRITE));
        assert!(reg.has_voxel(CHAINGUNSPRITE));
        assert!(reg.has_voxel(RPGSPRITE));

        let atomic = reg.get_model(ATOMICHEALTH).unwrap();
        assert!(atomic.solid_count() > 50);
    }
}
