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

        // 11. Explosive & Radioactive Barrels
        self.register(EXPLODINGBARREL, create_barrel_model(240, 0, Some(144), false, false));
        self.register(EXPLODINGBARREL2, create_barrel_model(242, 0, Some(144), false, false));
        self.register(FIREBARREL, create_barrel_model(24, 0, None, false, false));
        self.register(NUKEBARREL, create_barrel_model(96, 0, Some(144), false, false));
        self.register(NUKEBARRELDENTED, create_barrel_model(96, 0, Some(144), true, false));
        self.register(NUKEBARRELLEAKED, create_barrel_model(96, 0, Some(144), false, true));

        // 12. Fire Extinguisher
        self.register(FIREEXT, create_fireext_model());

        // 13. Security Camera
        self.register(CAMERA1, create_camera_model());

        // 14. Ceiling Fan (Tile 617)
        self.register(617, create_fan_model());

        // 15. Drinking Water Fountain
        let fountain = create_fountain_model();
        self.register(WATERFOUNTAIN, fountain.clone());
        self.register(565, fountain);
    }
}

/// Procedural voxel model generator for explosive, burning, and toxic waste barrels.
pub fn create_barrel_model(
    body_color: u8,
    rim_color: u8,
    stripe_color: Option<u8>,
    dented: bool,
    leaked: bool,
) -> KvxModel {
    let mut barrel = KvxModel::new(16, 16, 22, Vec3::new(8.0, 8.0, 0.0));
    for x in 0..16 {
        for y in 0..16 {
            let dx = x as f32 - 7.5;
            let dy = y as f32 - 7.5;
            let dist_sq = dx * dx + dy * dy;
            if dist_sq <= 49.0 {
                for z in 0..22 {
                    if dented && x >= 10 && y >= 10 && (6..14).contains(&z) {
                        continue; // Dent cutout
                    }
                    let color = if z == 0 || z == 1 || z == 10 || z == 11 || z == 20 || z == 21 {
                        rim_color
                    } else if let Some(sc) = stripe_color {
                        if (8..14).contains(&z) && (x + y) % 4 < 2 {
                            sc
                        } else {
                            body_color
                        }
                    } else {
                        body_color
                    };
                    barrel.set_voxel(x, y, z, Some(color));
                }
            }
        }
    }
    if leaked {
        // Toxic green puddle base
        for x in 2..14 {
            for y in 2..14 {
                let dx = x as f32 - 7.5;
                let dy = y as f32 - 7.5;
                if dx * dx + dy * dy <= 36.0 && barrel.get_voxel(x, y, 21).is_none() {
                    barrel.set_voxel(x, y, 21, Some(100)); // Toxic slime
                }
            }
        }
    }
    barrel
}

/// Procedural voxel model generator for wall-mounted red fire extinguisher.
pub fn create_fireext_model() -> KvxModel {
    let mut model = KvxModel::new(10, 10, 20, Vec3::new(5.0, 5.0, 0.0));
    for x in 0..10 {
        for y in 0..10 {
            let dx = x as f32 - 4.5;
            let dy = y as f32 - 4.5;
            let dist_sq = dx * dx + dy * dy;
            if dist_sq <= 16.0 {
                for z in 3..18 {
                    let color = if (8..12).contains(&z) && y >= 7 && (3..7).contains(&x) {
                        250 // White instruction label
                    } else {
                        240 // Red tank
                    };
                    model.set_voxel(x, y, z, Some(color));
                }
            }
        }
    }
    // Valve and handle at top
    for z in 0..3 {
        for x in 4..6 {
            for y in 4..6 {
                model.set_voxel(x, y, z, Some(0)); // Valve stem
            }
        }
    }
    for x in 3..8 {
        model.set_voxel(x, 5, 1, Some(0)); // Handle lever
    }
    model.set_voxel(6, 4, 1, Some(250)); // Pressure gauge
    // Side hose
    for z in 3..13 {
        model.set_voxel(1, 5, z, Some(0));
    }
    model
}

/// Procedural voxel model generator for security / surveillance camera.
pub fn create_camera_model() -> KvxModel {
    let mut model = KvxModel::new(12, 16, 10, Vec3::new(6.0, 2.0, 5.0));
    // Wall mount plate
    for x in 4..8 {
        for y in 0..3 {
            for z in 3..7 {
                model.set_voxel(x, y, z, Some(24));
            }
        }
    }
    // Swivel joint
    for x in 5..7 {
        for y in 2..5 {
            for z in 4..6 {
                model.set_voxel(x, y, z, Some(16));
            }
        }
    }
    // Camera main body
    for x in 2..10 {
        for y in 5..15 {
            for z in 2..8 {
                model.set_voxel(x, y, z, Some(248)); // Pale industrial beige
            }
        }
    }
    // Front lens
    for x in 4..8 {
        for z in 3..7 {
            model.set_voxel(x, 15, z, Some(0)); // Black lens bezel
        }
    }
    for x in 5..7 {
        for z in 4..6 {
            model.set_voxel(x, 15, z, Some(196)); // Blue optical glass
        }
    }
    // Status LED
    model.set_voxel(3, 15, 7, Some(240)); // Red recording LED
    model
}

/// Procedural voxel model generator for 4-blade industrial ceiling fan.
pub fn create_fan_model() -> KvxModel {
    let mut model = KvxModel::new(28, 28, 6, Vec3::new(14.0, 14.0, 5.0));
    // Motor hub
    for x in 0..28 {
        for y in 0..28 {
            let dx = x as f32 - 13.5;
            let dy = y as f32 - 13.5;
            let dist_sq = dx * dx + dy * dy;
            if dist_sq <= 12.0 {
                for z in 1..5 {
                    model.set_voxel(x, y, z, Some(24)); // Steel housing
                }
            }
            if dist_sq <= 3.0 {
                model.set_voxel(x, y, 0, Some(0)); // Downrod mount
            }
        }
    }
    // 4 Blades (+X, -X, +Y, -Y)
    for x in 17..27 {
        for y in 12..16 {
            model.set_voxel(x, y, 3, Some(160)); // Wood blade
        }
    }
    for x in 1..11 {
        for y in 12..16 {
            model.set_voxel(x, y, 3, Some(160));
        }
    }
    for y in 17..27 {
        for x in 12..16 {
            model.set_voxel(x, y, 3, Some(160));
        }
    }
    for y in 1..11 {
        for x in 12..16 {
            model.set_voxel(x, y, 3, Some(160));
        }
    }
    // Brass blade brackets
    for x in 14..18 {
        for y in 13..15 {
            model.set_voxel(x, y, 3, Some(144));
        }
    }
    for x in 10..14 {
        for y in 13..15 {
            model.set_voxel(x, y, 3, Some(144));
        }
    }
    for y in 14..18 {
        for x in 13..15 {
            model.set_voxel(x, y, 3, Some(144));
        }
    }
    for y in 10..14 {
        for x in 13..15 {
            model.set_voxel(x, y, 3, Some(144));
        }
    }
    model
}

/// Procedural voxel model generator for drinking water fountain.
pub fn create_fountain_model() -> KvxModel {
    let mut model = KvxModel::new(14, 12, 14, Vec3::new(7.0, 2.0, 7.0));
    // Wall backplate
    for x in 2..12 {
        for y in 0..2 {
            for z in 1..13 {
                model.set_voxel(x, y, z, Some(24));
            }
        }
    }
    // Basin body
    for x in 3..11 {
        for y in 2..10 {
            for z in 4..12 {
                model.set_voxel(x, y, z, Some(250)); // Brushed steel
            }
        }
    }
    // Recessed basin cutout
    for x in 4..10 {
        for y in 3..9 {
            for z in 4..7 {
                model.set_voxel(x, y, z, None);
            }
        }
    }
    // Water pool
    for x in 5..9 {
        for y in 4..8 {
            model.set_voxel(x, y, 6, Some(196)); // Water blue
        }
    }
    // Bubbler spigot
    for x in 6..8 {
        for y in 7..9 {
            for z in 2..5 {
                model.set_voxel(x, y, z, Some(248)); // Chrome
            }
        }
    }
    // Side push bar
    for z in 7..9 {
        for y in 4..8 {
            model.set_voxel(11, y, z, Some(0));
            model.set_voxel(12, y, z, Some(0));
        }
    }
    model
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

/// Tag component for motorized ceiling fans rotating continuously.
#[derive(Component, Clone, Debug)]
pub struct CeilingFanVoxel {
    pub speed: f32,
}

impl Default for CeilingFanVoxel {
    fn default() -> Self {
        Self { speed: 4.5 }
    }
}

/// Tag component for motorized security surveillance cameras sweeping back and forth.
#[derive(Component, Clone, Debug)]
pub struct SecurityCameraVoxel {
    pub base_yaw: f32,
    pub sweep_range: f32,
    pub sweep_speed: f32,
    pub timer: f32,
}

impl SecurityCameraVoxel {
    pub fn new(base_yaw: f32, sweep_range: f32, sweep_speed: f32) -> Self {
        Self {
            base_yaw,
            sweep_range,
            sweep_speed,
            timer: 0.0,
        }
    }
}

/// System that animates motorized environmental voxel props (ceiling fans and security cameras).
pub fn update_kinetic_voxel_props(
    time: Res<Time>,
    config: Res<VoxelConfig>,
    mut fans: Query<(&mut Transform, &CeilingFanVoxel), Without<SecurityCameraVoxel>>,
    mut cameras: Query<(&mut Transform, &mut SecurityCameraVoxel), Without<CeilingFanVoxel>>,
) {
    if !config.enabled {
        return;
    }
    let dt = time.delta_seconds();

    for (mut transform, fan) in &mut fans {
        transform.rotate_y(fan.speed * dt);
    }

    for (mut transform, mut camera) in &mut cameras {
        camera.timer += camera.sweep_speed * dt;
        let current_yaw = camera.base_yaw + camera.timer.sin() * camera.sweep_range;
        transform.rotation = Quat::from_rotation_y(current_yaw);
    }
}

pub struct VoxelPlugin;

impl Plugin for VoxelPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<VoxelConfig>()
            .insert_resource(VoxelRegistry::new())
            .add_systems(
                Update,
                (update_voxel_instances, update_kinetic_voxel_props),
            );
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

    #[test]
    fn test_environmental_prop_voxel_models_registration() {
        let reg = VoxelRegistry::new();

        // 1. Barrels
        assert!(reg.has_voxel(EXPLODINGBARREL), "Explosive barrel 1 must be registered");
        assert!(reg.has_voxel(EXPLODINGBARREL2), "Explosive barrel 2 must be registered");
        assert!(reg.has_voxel(FIREBARREL), "Fire barrel must be registered");
        assert!(reg.has_voxel(NUKEBARREL), "Nuke barrel must be registered");
        assert!(reg.has_voxel(NUKEBARRELDENTED), "Nuke barrel dented must be registered");
        assert!(reg.has_voxel(NUKEBARRELLEAKED), "Nuke barrel leaked must be registered");

        // 2. Fire extinguisher
        assert!(reg.has_voxel(FIREEXT), "Fire extinguisher must be registered");

        // 3. Security camera
        assert!(reg.has_voxel(CAMERA1), "Security camera must be registered");

        // 4. Ceiling fan
        assert!(reg.has_voxel(617), "Ceiling fan (tile 617) must be registered");

        // 5. Water fountain
        assert!(reg.has_voxel(WATERFOUNTAIN), "Water fountain must be registered");
        assert!(reg.has_voxel(565), "Water fountain variant 565 must be registered");

        let barrel = reg.get_model(EXPLODINGBARREL).unwrap();
        assert!(barrel.solid_count() > 100, "Barrel must have solid voxel volume");
        assert!(barrel.xsiz >= 10 && barrel.zsiz >= 16);

        let fireext = reg.get_model(FIREEXT).unwrap();
        assert!(fireext.solid_count() > 30);

        let camera = reg.get_model(CAMERA1).unwrap();
        assert!(camera.solid_count() > 20);

        let fan = reg.get_model(617).unwrap();
        assert!(fan.solid_count() > 30);

        let fountain = reg.get_model(WATERFOUNTAIN).unwrap();
        assert!(fountain.solid_count() > 50);
    }

    #[test]
    fn test_prop_voxel_surface_meshing() {
        let mut reg = VoxelRegistry::new();
        let pal = Palette::default();
        let mut meshes = Assets::<Mesh>::default();

        let barrel_mesh = reg.get_or_create_mesh(EXPLODINGBARREL, &pal, &mut meshes).unwrap();
        let mesh = meshes.get(&barrel_mesh).unwrap();
        assert!(mesh.count_vertices() > 0);

        let fan_mesh = reg.get_or_create_mesh(617, &pal, &mut meshes).unwrap();
        let mesh = meshes.get(&fan_mesh).unwrap();
        assert!(mesh.count_vertices() > 0);
    }

    #[test]
    fn test_ceiling_fan_continuous_rotation() {
        let mut app = App::new();
        app.insert_resource(Time::<()>::default())
            .insert_resource(VoxelConfig::default())
            .add_systems(Update, update_kinetic_voxel_props);

        let fan_entity = app
            .world_mut()
            .spawn((
                Transform::from_xyz(0.0, 5.0, 0.0),
                CeilingFanVoxel { speed: 4.0 },
            ))
            .id();

        // Advance time by 0.5s
        {
            let mut time = app.world_mut().resource_mut::<Time>();
            time.advance_by(std::time::Duration::from_millis(500));
        }
        app.update();

        let transform = app.world().get::<Transform>(fan_entity).unwrap();
        assert!(
            transform.rotation != Quat::IDENTITY,
            "Ceiling fan should rotate after time advances"
        );
    }

    #[test]
    fn test_security_camera_sweep_oscillation() {
        let mut app = App::new();
        app.insert_resource(Time::<()>::default())
            .insert_resource(VoxelConfig::default())
            .add_systems(Update, update_kinetic_voxel_props);

        let camera_entity = app
            .world_mut()
            .spawn((
                Transform::from_xyz(2.0, 3.0, 2.0),
                SecurityCameraVoxel::new(0.0, 0.7, 2.0),
            ))
            .id();

        // Advance time by 0.25s
        {
            let mut time = app.world_mut().resource_mut::<Time>();
            time.advance_by(std::time::Duration::from_millis(250));
        }
        app.update();

        let transform = app.world().get::<Transform>(camera_entity).unwrap();
        let camera = app.world().get::<SecurityCameraVoxel>(camera_entity).unwrap();

        assert!(camera.timer > 0.0, "Camera timer should advance");
        assert!(
            transform.rotation != Quat::IDENTITY,
            "Camera transform should oscillate away from base orientation"
        );
    }
}
