use bevy::prelude::*;
use bevy::render::render_asset::RenderAssetUsages;
use bevy_rapier3d::prelude::*;
use lyon_tessellation::math::point;
use lyon_tessellation::{
    BuffersBuilder, FillOptions, FillTessellator, FillVertex, VertexBuffers,
};
use std::collections::{HashMap, HashSet};

use crate::animation::AnimatedTileMaterial;
use crate::art::PicAnm;
use crate::map::{Map, Wall};
use crate::palette::Palette;

pub struct MapMeshBuilder<'a> {
    pub map: &'a Map,
    pub tile_textures: &'a HashMap<i16, Handle<Image>>,
    pub tile_sizes: &'a HashMap<i16, (u32, u32)>,
    pub picanm_map: &'a HashMap<i16, PicAnm>,
    pub default_material: Handle<StandardMaterial>,
    material_cache: std::cell::RefCell<HashMap<(i16, bool), Handle<StandardMaterial>>>,
}

impl<'a> MapMeshBuilder<'a> {
    pub fn new(
        map: &'a Map,
        tile_textures: &'a HashMap<i16, Handle<Image>>,
        tile_sizes: &'a HashMap<i16, (u32, u32)>,
        picanm_map: &'a HashMap<i16, PicAnm>,
        default_material: Handle<StandardMaterial>,
    ) -> Self {
        Self {
            map,
            tile_textures,
            tile_sizes,
            picanm_map,
            default_material,
            material_cache: std::cell::RefCell::new(HashMap::new()),
        }
    }

    fn get_material(
        &self,
        picnum: i16,
        is_transparent: bool,
        materials: &mut Assets<StandardMaterial>,
    ) -> Handle<StandardMaterial> {
        let key = (picnum, is_transparent);
        if let Some(handle) = self.material_cache.borrow().get(&key) {
            return handle.clone();
        }

        let handle = if let Some(tex_handle) = self.tile_textures.get(&picnum) {
            materials.add(StandardMaterial {
                base_color_texture: Some(tex_handle.clone()),
                alpha_mode: if is_transparent {
                    AlphaMode::Mask(0.5)
                } else {
                    AlphaMode::Opaque
                },
                unlit: true,
                double_sided: true,
                ..default()
            })
        } else {
            self.default_material.clone()
        };

        self.material_cache.borrow_mut().insert(key, handle.clone());
        handle
    }

    fn get_tile_size(&self, picnum: i16) -> (u32, u32) {
        let (w, h) = self.tile_sizes.get(&picnum).copied().unwrap_or((64, 64));
        (w.max(1), h.max(1))
    }

    pub fn build(
        &self,
        commands: &mut Commands,
        meshes: &mut Assets<Mesh>,
        materials: &mut Assets<StandardMaterial>,
    ) {
        self.build_sectors(commands, meshes, materials);
        self.build_walls(commands, meshes, materials);
        self.build_sprites(commands, meshes, materials);
    }

    fn build_sectors(
        &self,
        commands: &mut Commands,
        meshes: &mut Assets<Mesh>,
        materials: &mut Assets<StandardMaterial>,
    ) {
        for (sec_idx, sector) in self.map.sectors.iter().enumerate() {
            let mut loops = Vec::new();
            let mut current_loop = Vec::new();
            let mut visited_walls = HashSet::new();

            for i in 0..sector.wallnum {
                let wall_idx = (sector.wallptr + i) as usize;
                if visited_walls.contains(&wall_idx) || wall_idx >= self.map.walls.len() {
                    continue;
                }

                let mut w = wall_idx;
                loop {
                    if visited_walls.contains(&w) || w >= self.map.walls.len() {
                        break;
                    }
                    visited_walls.insert(w);
                    let wall = &self.map.walls[w];
                    current_loop.push(Vec2::new(
                        wall.x as f32 / 1024.0,
                        wall.y as f32 / 1024.0,
                    ));
                    let next_w = wall.point2 as usize;
                    if next_w == wall_idx {
                        break;
                    }
                    if next_w < sector.wallptr as usize
                        || next_w >= (sector.wallptr + sector.wallnum) as usize
                    {
                        break;
                    }
                    w = next_w;
                }
                if !current_loop.is_empty() {
                    loops.push(std::mem::take(&mut current_loop));
                }
            }

            if loops.is_empty() {
                continue;
            }

            let mut tessellator = FillTessellator::new();
            let mut buffers: VertexBuffers<[f32; 2], u32> = VertexBuffers::new();

            let mut path_builder = lyon_tessellation::path::Path::builder();
            for poly_points in &loops {
                if poly_points.len() < 3 {
                    continue;
                }
                path_builder.begin(point(poly_points[0].x, poly_points[0].y));
                for p in poly_points.iter().skip(1) {
                    path_builder.line_to(point(p.x, p.y));
                }
                path_builder.end(true);
            }
            let path = path_builder.build();

            let lyon_ok = tessellator
                .tessellate_path(
                    &path,
                    &FillOptions::default(),
                    &mut BuffersBuilder::new(&mut buffers, |vertex: FillVertex| {
                        [vertex.position().x, vertex.position().y]
                    }),
                )
                .is_ok();

            // Robust Fallback: if Lyon produced 0 triangles or failed on complex/self-intersecting loops,
            // use guaranteed ear-clipping and fan triangulation across the loops
            if !lyon_ok || buffers.indices.is_empty() {
                buffers.vertices.clear();
                buffers.indices.clear();
                for poly in &loops {
                    if poly.len() < 3 { continue; }
                    let base_idx = buffers.vertices.len() as u32;
                    for p in poly {
                        buffers.vertices.push([p.x, p.y]);
                    }
                    let indices = triangulate_polygon_ear_clipping(poly, base_idx);
                    buffers.indices.extend(indices);
                }
            }

            if buffers.indices.is_empty() {
                continue;
            }

            let (floor_tw, floor_th) = self.get_tile_size(sector.floorpicnum);
            let (ceil_tw, ceil_th) = self.get_tile_size(sector.ceilingpicnum);

            // 1. Generate Floor Mesh
            if !sector.is_floor_parallax() {
                let floor_vertices: Vec<[f32; 3]> = buffers
                    .vertices
                    .iter()
                    .map(|v| {
                        let bx = (v[0] * 1024.0).round() as i32;
                        let by = (v[1] * 1024.0).round() as i32;
                        let y = sector.get_floor_y_at(&self.map.walls, bx, by);
                        [v[0], y, v[1]]
                    })
                    .collect();

                let floor_uvs: Vec<[f32; 2]> = buffers
                    .vertices
                    .iter()
                    .map(|v| {
                        let bx = v[0] * 1024.0;
                        let by = v[1] * 1024.0;
                        let u = (bx / (floor_tw as f32 * 16.0))
                            + (sector.floorxpanning as f32 / 256.0);
                        let v_coord = (by / (floor_th as f32 * 16.0))
                            + (sector.floorypanning as f32 / 256.0);
                        [u, v_coord]
                    })
                    .collect();

                let floor_tint = Palette::shade_to_tint(sector.floorshade);
                let floor_colors: Vec<[f32; 4]> = vec![floor_tint; floor_vertices.len()];

                let floor_indices = buffers.indices.clone();
                let floor_mat = self.get_material(sector.floorpicnum, false, materials);

                let mut floor_mesh = Mesh::new(
                    bevy::render::mesh::PrimitiveTopology::TriangleList,
                    RenderAssetUsages::default(),
                );
                floor_mesh.insert_attribute(Mesh::ATTRIBUTE_POSITION, floor_vertices.clone());
                floor_mesh.insert_attribute(Mesh::ATTRIBUTE_UV_0, floor_uvs);
                floor_mesh.insert_attribute(Mesh::ATTRIBUTE_COLOR, floor_colors);
                floor_mesh.insert_indices(bevy::render::mesh::Indices::U32(floor_indices.clone()));
                floor_mesh.duplicate_vertices();
                floor_mesh.compute_flat_normals();

                let floor_collider_vertices: Vec<Vect> = floor_vertices
                    .iter()
                    .map(|v| Vect::new(v[0], v[1], v[2]))
                    .collect();
                let mut floor_collider_indices: Vec<[u32; 3]> = Vec::new();
                for c in floor_indices.chunks(3) {
                    floor_collider_indices.push([c[0], c[1], c[2]]);
                }

                if !floor_collider_indices.is_empty() {
                    let mut entity_cmds = commands.spawn((
                        PbrBundle {
                            mesh: meshes.add(floor_mesh),
                            material: floor_mat.clone(),
                            ..default()
                        },
                        RigidBody::Fixed,
                        Collider::trimesh(floor_collider_vertices, floor_collider_indices),
                        crate::interactivity::DynamicSectorMesh {
                            sector_idx: sec_idx,
                            orig_translation: Vec3::ZERO,
                        },
                        crate::game_flow::LevelEntity,
                    ));

                    // Check for tile animation on floor
                    if let Some(&picanm) = self.picanm_map.get(&sector.floorpicnum) {
                        if picanm.num_frames > 0 && picanm.anim_type > 0 {
                            entity_cmds.insert(AnimatedTileMaterial {
                                base_picnum: sector.floorpicnum,
                                picanm,
                                current_offset: 0,
                                material_handle: floor_mat,
                            });
                        }
                    }
                }
            }

            // 2. Generate Ceiling Mesh
            if !sector.is_ceiling_parallax() {
                let ceil_vertices: Vec<[f32; 3]> = buffers
                    .vertices
                    .iter()
                    .map(|v| {
                        let bx = (v[0] * 1024.0).round() as i32;
                        let by = (v[1] * 1024.0).round() as i32;
                        let y = sector.get_ceiling_y_at(&self.map.walls, bx, by);
                        [v[0], y, v[1]]
                    })
                    .collect();

                let ceil_uvs: Vec<[f32; 2]> = buffers
                    .vertices
                    .iter()
                    .map(|v| {
                        let bx = v[0] * 1024.0;
                        let by = v[1] * 1024.0;
                        let u = (bx / (ceil_tw as f32 * 16.0))
                            + (sector.ceilingxpanning as f32 / 256.0);
                        let v_coord = (by / (ceil_th as f32 * 16.0))
                            + (sector.ceilingypanning as f32 / 256.0);
                        [u, v_coord]
                    })
                    .collect();

                let ceil_tint = Palette::shade_to_tint(sector.ceilingshade);
                let ceil_colors: Vec<[f32; 4]> = vec![ceil_tint; ceil_vertices.len()];

                // Reverse ceiling winding order so normals face downwards
                let ceil_indices: Vec<u32> = buffers
                    .indices
                    .chunks(3)
                    .flat_map(|c| [c[0], c[2], c[1]])
                    .collect();

                let ceil_mat = self.get_material(sector.ceilingpicnum, false, materials);

                let mut ceil_mesh = Mesh::new(
                    bevy::render::mesh::PrimitiveTopology::TriangleList,
                    RenderAssetUsages::default(),
                );
                ceil_mesh.insert_attribute(Mesh::ATTRIBUTE_POSITION, ceil_vertices.clone());
                ceil_mesh.insert_attribute(Mesh::ATTRIBUTE_UV_0, ceil_uvs);
                ceil_mesh.insert_attribute(Mesh::ATTRIBUTE_COLOR, ceil_colors);
                ceil_mesh.insert_indices(bevy::render::mesh::Indices::U32(ceil_indices.clone()));
                ceil_mesh.duplicate_vertices();
                ceil_mesh.compute_flat_normals();

                let ceil_collider_vertices: Vec<Vect> = ceil_vertices
                    .iter()
                    .map(|v| Vect::new(v[0], v[1], v[2]))
                    .collect();
                let mut ceil_collider_indices: Vec<[u32; 3]> = Vec::new();
                for c in ceil_indices.chunks(3) {
                    ceil_collider_indices.push([c[0], c[1], c[2]]);
                    ceil_collider_indices.push([c[0], c[2], c[1]]);
                }

                if !ceil_collider_indices.is_empty() {
                    let mut entity_cmds = commands.spawn((
                        PbrBundle {
                            mesh: meshes.add(ceil_mesh),
                            material: ceil_mat.clone(),
                            ..default()
                        },
                        RigidBody::Fixed,
                        Collider::trimesh(ceil_collider_vertices, ceil_collider_indices),
                        crate::interactivity::DynamicSectorMesh {
                            sector_idx: sec_idx,
                            orig_translation: Vec3::ZERO,
                        },
                        crate::game_flow::LevelEntity,
                    ));

                    // Check for tile animation on ceiling
                    if let Some(&picanm) = self.picanm_map.get(&sector.ceilingpicnum) {
                        if picanm.num_frames > 0 && picanm.anim_type > 0 {
                            entity_cmds.insert(AnimatedTileMaterial {
                                base_picnum: sector.ceilingpicnum,
                                picanm,
                                current_offset: 0,
                                material_handle: ceil_mat,
                            });
                        }
                    }
                }
            }
        }
    }

    fn build_walls(
        &self,
        commands: &mut Commands,
        meshes: &mut Assets<Mesh>,
        materials: &mut Assets<StandardMaterial>,
    ) {
        for (sec_idx, sector) in self.map.sectors.iter().enumerate() {
            for i in 0..sector.wallnum {
                let wall_idx = (sector.wallptr + i) as usize;
                if wall_idx >= self.map.walls.len() {
                    continue;
                }
                let wall = &self.map.walls[wall_idx];
                let next_wall_idx = wall.point2 as usize;
                if next_wall_idx >= self.map.walls.len() {
                    continue;
                }
                let next_wall = &self.map.walls[next_wall_idx];

                let p1 = Vec2::new(wall.x as f32 / 1024.0, wall.y as f32 / 1024.0);
                let p2 = Vec2::new(next_wall.x as f32 / 1024.0, next_wall.y as f32 / 1024.0);
                let wall_len = (p2 - p1).length();
                if wall_len < 0.001 {
                    continue;
                }

                let cur_floor_y1 = sector.get_floor_y_at(&self.map.walls, wall.x, wall.y);
                let cur_floor_y2 = sector.get_floor_y_at(&self.map.walls, next_wall.x, next_wall.y);
                let cur_ceil_y1 = sector.get_ceiling_y_at(&self.map.walls, wall.x, wall.y);
                let cur_ceil_y2 = sector.get_ceiling_y_at(&self.map.walls, next_wall.x, next_wall.y);

                if !wall.is_portal() {
                    // One-sided solid wall: floor to ceiling
                    self.spawn_wall_quad(
                        commands,
                        meshes,
                        materials,
                        sec_idx,
                        p1,
                        p2,
                        cur_floor_y1,
                        cur_floor_y2,
                        cur_ceil_y1,
                        cur_ceil_y2,
                        wall,
                        wall.picnum,
                        true, // solid blocking
                        false, // opaque
                    );
                } else {
                    // Two-sided portal wall
                    let next_sec_idx = wall.nextsector as usize;
                    if next_sec_idx >= self.map.sectors.len() {
                        continue;
                    }
                    let next_sec = &self.map.sectors[next_sec_idx];

                    let next_floor_y1 = next_sec.get_floor_y_at(&self.map.walls, wall.x, wall.y);
                    let next_floor_y2 = next_sec.get_floor_y_at(&self.map.walls, next_wall.x, next_wall.y);
                    let next_ceil_y1 = next_sec.get_ceiling_y_at(&self.map.walls, wall.x, wall.y);
                    let next_ceil_y2 = next_sec.get_ceiling_y_at(&self.map.walls, next_wall.x, next_wall.y);

                    // 1. Upper Wall (Step down from ceiling)
                    if next_ceil_y1 < cur_ceil_y1 - 0.001 || next_ceil_y2 < cur_ceil_y2 - 0.001 {
                        self.spawn_wall_quad(
                            commands,
                            meshes,
                            materials,
                            sec_idx,
                            p1,
                            p2,
                            next_ceil_y1,
                            next_ceil_y2,
                            cur_ceil_y1,
                            cur_ceil_y2,
                            wall,
                            wall.picnum,
                            true,
                            false,
                        );
                    }

                    // 2. Lower Wall (Step up from floor)
                    if next_floor_y1 > cur_floor_y1 + 0.001 || next_floor_y2 > cur_floor_y2 + 0.001 {
                        let lower_picnum = if wall.bottoms_swapped() {
                            wall.picnum
                        } else if (wall.nextwall as usize) < self.map.walls.len() {
                            self.map.walls[wall.nextwall as usize].picnum
                        } else {
                            wall.picnum
                        };

                        self.spawn_wall_quad(
                            commands,
                            meshes,
                            materials,
                            sec_idx,
                            p1,
                            p2,
                            cur_floor_y1,
                            cur_floor_y2,
                            next_floor_y1,
                            next_floor_y2,
                            wall,
                            lower_picnum,
                            true,
                            false,
                        );
                    }

                    // 3. Masked / Transparent Middle Wall
                    if wall.is_masked() || wall.is_one_way() {
                        let mid_floor_y1 = cur_floor_y1.max(next_floor_y1);
                        let mid_floor_y2 = cur_floor_y2.max(next_floor_y2);
                        let mid_ceil_y1 = cur_ceil_y1.min(next_ceil_y1);
                        let mid_ceil_y2 = cur_ceil_y2.min(next_ceil_y2);

                        if mid_ceil_y1 > mid_floor_y1 + 0.001 || mid_ceil_y2 > mid_floor_y2 + 0.001 {
                            let masked_picnum = if wall.overpicnum != 0 {
                                wall.overpicnum
                            } else {
                                wall.picnum
                            };

                            self.spawn_wall_quad(
                                commands,
                                meshes,
                                materials,
                                sec_idx,
                                p1,
                                p2,
                                mid_floor_y1,
                                mid_floor_y2,
                                mid_ceil_y1,
                                mid_ceil_y2,
                                wall,
                                masked_picnum,
                                wall.is_blocking(),
                                true, // transparent/masked
                            );
                        }
                    }
                }
            }
        }
    }

    fn spawn_wall_quad(
        &self,
        commands: &mut Commands,
        meshes: &mut Assets<Mesh>,
        materials: &mut Assets<StandardMaterial>,
        sec_idx: usize,
        p1: Vec2,
        p2: Vec2,
        bottom_y1: f32,
        bottom_y2: f32,
        top_y1: f32,
        top_y2: f32,
        wall: &Wall,
        picnum: i16,
        is_solid: bool,
        is_masked: bool,
    ) {
        let (tw, th) = self.get_tile_size(picnum);
        let wall_len = (p2 - p1).length();
        let avg_height = ((top_y1 - bottom_y1) + (top_y2 - bottom_y2)) / 2.0;
        if wall_len < 0.001 || avg_height.abs() < 0.001 {
            return;
        }

        let u_span = (wall.xrepeat as f32 * 8.0) / (tw as f32);
        let u0_raw = (wall.xpanning as f32) / (tw as f32);
        let u1_raw = u0_raw + u_span;

        let (u0, u1) = if wall.is_x_flipped() {
            (u1_raw, u0_raw)
        } else {
            (u0_raw, u1_raw)
        };

        let build_height = avg_height.abs() * 1024.0 * 16.0;
        let v_span = (build_height * wall.yrepeat as f32) / (th as f32 * 2048.0);
        let v_pan = (wall.ypanning as f32) / (th as f32);

        let (mut v_top, mut v_bottom) = if wall.align_bottom() {
            (1.0 + v_pan - v_span, 1.0 + v_pan)
        } else {
            (v_pan, v_pan + v_span)
        };

        if wall.is_y_flipped() {
            std::mem::swap(&mut v_top, &mut v_bottom);
        }

        let v0_pos = [p1.x, bottom_y1, p1.y];
        let v1_pos = [p2.x, bottom_y2, p2.y];
        let v2_pos = [p2.x, top_y2, p2.y];
        let v3_pos = [p1.x, top_y1, p1.y];

        let positions = vec![v0_pos, v1_pos, v2_pos, v3_pos];
        // v0/v1 are bottom vertices, v2/v3 are top vertices
        let uvs = vec![[u0, v_bottom], [u1, v_bottom], [u1, v_top], [u0, v_top]];
        let tint = Palette::shade_to_tint(wall.shade);
        let colors = vec![tint; 4];
        let indices = vec![0u32, 1, 2, 0, 2, 3];

        let mat = self.get_material(picnum, is_masked, materials);

        let mut wall_mesh = Mesh::new(
            bevy::render::mesh::PrimitiveTopology::TriangleList,
            RenderAssetUsages::default(),
        );
        wall_mesh.insert_attribute(Mesh::ATTRIBUTE_POSITION, positions.clone());
        wall_mesh.insert_attribute(Mesh::ATTRIBUTE_UV_0, uvs);
        wall_mesh.insert_attribute(Mesh::ATTRIBUTE_COLOR, colors);
        wall_mesh.insert_indices(bevy::render::mesh::Indices::U32(indices.clone()));
        wall_mesh.duplicate_vertices();
        wall_mesh.compute_flat_normals();

        let collider_vertices: Vec<Vect> = positions
            .iter()
            .map(|v| Vect::new(v[0], v[1], v[2]))
            .collect();
        let collider_indices: Vec<[u32; 3]> = vec![
            [0, 1, 2], [0, 2, 3],
            [0, 2, 1], [0, 3, 2], // Double-sided wall collision
        ];

        let mut entity_cmds = commands.spawn((
            PbrBundle {
                mesh: meshes.add(wall_mesh),
                material: mat.clone(),
                ..default()
            },
            crate::interactivity::DynamicSectorMesh {
                sector_idx: sec_idx,
                orig_translation: Vec3::ZERO,
            },
            crate::game_flow::LevelEntity,
        ));

        if is_solid {
            entity_cmds.insert((
                RigidBody::Fixed,
                Collider::trimesh(collider_vertices, collider_indices),
            ));
        }

        // Check for tile animation on wall
        if let Some(&picanm) = self.picanm_map.get(&picnum) {
            if picanm.num_frames > 0 && picanm.anim_type > 0 {
                entity_cmds.insert(AnimatedTileMaterial {
                    base_picnum: picnum,
                    picanm,
                    current_offset: 0,
                    material_handle: mat,
                });
            }
        }
    }

    fn build_sprites(
        &self,
        commands: &mut Commands,
        meshes: &mut Assets<Mesh>,
        materials: &mut Assets<StandardMaterial>,
    ) {
        for sprite in &self.map.sprites {
            if self.tile_textures.contains_key(&sprite.picnum) {
                let (tw, th) = self.get_tile_size(sprite.picnum);
                let pos = Vec3::new(
                    sprite.x as f32 / 1024.0,
                    -(sprite.z as f32) / (1024.0 * 16.0),
                    sprite.y as f32 / 1024.0,
                );

                let scale_x = (sprite.xrepeat as f32 * tw as f32) / 4096.0 * 3.0;
                let scale_y = (sprite.yrepeat as f32 * th as f32) / 4096.0 * 3.0;
                let is_enemy = sprite.picnum == 2000; // PIGCOP

                let sprite_mat = self.get_material(sprite.picnum, true, materials);

                let mut entity_cmds = commands.spawn((
                    PbrBundle {
                        mesh: meshes.add(Rectangle::new(scale_x, scale_y)),
                        material: sprite_mat.clone(),
                        transform: Transform::from_translation(pos),
                        ..default()
                    },
                    crate::SpriteBillboard,
                    RigidBody::Fixed,
                    Collider::cuboid(scale_x / 2.0, scale_y / 2.0, 0.1),
                    crate::Destructible {
                        health: if is_enemy { 100 } else { 10 },
                        _picnum: sprite.picnum,
                    },
                    crate::game_flow::LevelEntity,
                ));

                // Attach Phase 4 Interactive Components directly to visual entities!
                match sprite.picnum {
                    // SWITCHES
                    134 | 136 | 162 | 712 | 860 | 1111 | 1122 => {
                        entity_cmds.insert(crate::interactivity::InteractiveSwitch {
                            switch_type: crate::interactivity::SwitchType::LightSwitch,
                            on_tile: sprite.picnum + 1,
                            off_tile: sprite.picnum,
                            is_on: false,
                            lotag: sprite.lotag,
                            hitag: sprite.hitag,
                            sound_id: 10,
                            material_handle: Some(sprite_mat.clone()),
                        });
                    }
                    // KEYCARDS (Tiles 175 = Blue, 176 = Red, 177 = Yellow)
                    175 => {
                        entity_cmds.insert(crate::interactivity::KeycardPickup { key_type: 1 });
                    }
                    176 => {
                        entity_cmds.insert(crate::interactivity::KeycardPickup { key_type: 2 });
                    }
                    177 => {
                        entity_cmds.insert(crate::interactivity::KeycardPickup { key_type: 3 });
                    }
                    // HEALTH & ARMOR PICKUPS
                    51 => {
                        entity_cmds.insert(crate::interactivity::ItemPickup {
                            kind: crate::interactivity::PickupKind::SmallMedkit,
                            respawn_timer: None,
                        });
                    }
                    52 => {
                        entity_cmds.insert(crate::interactivity::ItemPickup {
                            kind: crate::interactivity::PickupKind::LargeMedkit,
                            respawn_timer: None,
                        });
                    }
                    55 => {
                        entity_cmds.insert(crate::interactivity::ItemPickup {
                            kind: crate::interactivity::PickupKind::AtomicHealth,
                            respawn_timer: None,
                        });
                    }
                    56 => {
                        entity_cmds.insert(crate::interactivity::ItemPickup {
                            kind: crate::interactivity::PickupKind::ArmorVest,
                            respawn_timer: None,
                        });
                    }
                    // AMMO PICKUPS
                    40 => {
                        entity_cmds.insert(crate::interactivity::ItemPickup {
                            kind: crate::interactivity::PickupKind::PistolClip,
                            respawn_timer: None,
                        });
                    }
                    44 => {
                        entity_cmds.insert(crate::interactivity::ItemPickup {
                            kind: crate::interactivity::PickupKind::ChaingunBox,
                            respawn_timer: None,
                        });
                    }
                    47 => {
                        entity_cmds.insert(crate::interactivity::ItemPickup {
                            kind: crate::interactivity::PickupKind::RpgRocket,
                            respawn_timer: None,
                        });
                    }
                    48 => {
                        entity_cmds.insert(crate::interactivity::ItemPickup {
                            kind: crate::interactivity::PickupKind::PipebombBox,
                            respawn_timer: None,
                        });
                    }
                    49 => {
                        entity_cmds.insert(crate::interactivity::ItemPickup {
                            kind: crate::interactivity::PickupKind::ShotgunBox,
                            respawn_timer: None,
                        });
                    }
                    42 => {
                        entity_cmds.insert(crate::interactivity::ItemPickup {
                            kind: crate::interactivity::PickupKind::ShrinkerAmmo,
                            respawn_timer: None,
                        });
                    }
                    45 => {
                        entity_cmds.insert(crate::interactivity::ItemPickup {
                            kind: crate::interactivity::PickupKind::DevastatorBox,
                            respawn_timer: None,
                        });
                    }
                    46 => {
                        entity_cmds.insert(crate::interactivity::ItemPickup {
                            kind: crate::interactivity::PickupKind::FreezeAmmo,
                            respawn_timer: None,
                        });
                    }
                    // INVENTORY ITEM PICKUPS
                    57 => {
                        entity_cmds.insert(crate::interactivity::ItemPickup {
                            kind: crate::interactivity::PickupKind::Steroids,
                            respawn_timer: None,
                        });
                    }
                    58 => {
                        entity_cmds.insert(crate::interactivity::ItemPickup {
                            kind: crate::interactivity::PickupKind::Jetpack,
                            respawn_timer: None,
                        });
                    }
                    59 => {
                        entity_cmds.insert(crate::interactivity::ItemPickup {
                            kind: crate::interactivity::PickupKind::ScubaTank,
                            respawn_timer: None,
                        });
                    }
                    60 => {
                        entity_cmds.insert(crate::interactivity::ItemPickup {
                            kind: crate::interactivity::PickupKind::NightvisionGoggles,
                            respawn_timer: None,
                        });
                    }
                    61 => {
                        entity_cmds.insert(crate::interactivity::ItemPickup {
                            kind: crate::interactivity::PickupKind::ProtectiveBoots,
                            respawn_timer: None,
                        });
                    }
                    62 => {
                        entity_cmds.insert(crate::interactivity::ItemPickup {
                            kind: crate::interactivity::PickupKind::Holoduke,
                            respawn_timer: None,
                        });
                    }
                    // WEAPONS ON GROUND
                    21 => {
                        entity_cmds.insert(crate::interactivity::ItemPickup {
                            kind: crate::interactivity::PickupKind::WeaponPistol,
                            respawn_timer: None,
                        });
                    }
                    22 => {
                        entity_cmds.insert(crate::interactivity::ItemPickup {
                            kind: crate::interactivity::PickupKind::WeaponShotgun,
                            respawn_timer: None,
                        });
                    }
                    23 => {
                        entity_cmds.insert(crate::interactivity::ItemPickup {
                            kind: crate::interactivity::PickupKind::WeaponChaingun,
                            respawn_timer: None,
                        });
                    }
                    24 => {
                        entity_cmds.insert(crate::interactivity::ItemPickup {
                            kind: crate::interactivity::PickupKind::WeaponRpg,
                            respawn_timer: None,
                        });
                    }
                    25 => {
                        entity_cmds.insert(crate::interactivity::ItemPickup {
                            kind: crate::interactivity::PickupKind::WeaponPipebomb,
                            respawn_timer: None,
                        });
                    }
                    26 => {
                        entity_cmds.insert(crate::interactivity::ItemPickup {
                            kind: crate::interactivity::PickupKind::WeaponShrinker,
                            respawn_timer: None,
                        });
                    }
                    27 => {
                        entity_cmds.insert(crate::interactivity::ItemPickup {
                            kind: crate::interactivity::PickupKind::WeaponDevastator,
                            respawn_timer: None,
                        });
                    }
                    28 => {
                        entity_cmds.insert(crate::interactivity::ItemPickup {
                            kind: crate::interactivity::PickupKind::WeaponTripbomb,
                            respawn_timer: None,
                        });
                    }
                    29 => {
                        entity_cmds.insert(crate::interactivity::ItemPickup {
                            kind: crate::interactivity::PickupKind::WeaponFreezer,
                            respawn_timer: None,
                        });
                    }
                    // WATER FOUNTAIN
                    564 | 565 => {
                        entity_cmds.insert(crate::interactivity::WaterFountain {
                            uses_left: 10,
                            is_broken: false,
                            broken_tile: 567,
                        });
                    }
                    // TOILET / STALL
                    569 | 571 => {
                        entity_cmds.insert(crate::interactivity::ToiletProp {
                            is_broken: false,
                            broken_tile: if sprite.picnum == 569 { 615 } else { 573 },
                            water_tile: 921,
                            last_used_time: 0.0,
                            cooldown_timer: 0.0,
                        });
                    }
                    // VIEWSCREEN CRT MONITOR
                    499 | 502 => {
                        entity_cmds.insert(crate::interactivity::ViewscreenProp {
                            camera_tag: sprite.hitag,
                            is_active: true,
                            is_broken: false,
                            broken_tile: 501,
                            scanline_timer: 0.0,
                        });
                    }
                    // SECURITY CAMERA (CAMERA1)
                    500 => {
                        entity_cmds.insert(crate::interactivity::SecurityCamera {
                            tag: sprite.hitag,
                            sweep_angle: 0.0,
                            sweep_speed: 1.0,
                            base_yaw: sprite.ang as f32,
                        });
                    }
                    // EXPLODING BARREL
                    1238 | 1240 | 1242 => {
                        entity_cmds.insert(crate::interactivity::ExplodingBarrel {
                            health: 20,
                            damage_radius: 6.0,
                            damage: 100,
                            is_exploded: false,
                        });
                    }
                    // CRACK WALL
                    546..=549 => {
                        entity_cmds.insert(crate::interactivity::CrackWall {
                            health: 30,
                            stage: 1,
                            lotag: sprite.lotag,
                            is_blown: false,
                        });
                    }
                    // BREAKABLE GLASS
                    503 => {
                        entity_cmds.insert(crate::interactivity::BreakableGlass {
                            health: 10,
                            is_broken: false,
                            wall_idx: None,
                            sector_idx: Some(sprite.sectnum as usize),
                        });
                    }
                    // ENEMIES & BOSSES
                    // 1. Assault Trooper & Captain (1680..=1744)
                    1680..=1744 => {
                        let is_captain = sprite.pal == 21;
                        let (enemy, actor_hp) = if is_captain {
                            (crate::combat::EnemyActor::new_captain(), 60)
                        } else {
                            (crate::combat::EnemyActor::new_liztroop(), 30)
                        };
                        let is_dormant = matches!(sprite.picnum, 1682 | 1741 | 1742 | 1744);
                        let is_jetpack = sprite.picnum == 1725;

                        entity_cmds.insert((
                            enemy,
                            crate::scripting::ConActor::new(1680, sprite.sectnum, sprite.ang, actor_hp),
                            crate::combat::SituationalSpawn {
                                initial_picnum: sprite.picnum,
                                is_dormant,
                            },
                        ));
                        if is_jetpack {
                            entity_cmds.insert(crate::combat::FlyingActor::default());
                        }
                    }
                    // 2. Pigcop (2000, 2001, 2045)
                    2000 | 2001 | 2045 => {
                        entity_cmds.insert((
                            crate::combat::EnemyActor::new_pigcop(),
                            crate::scripting::ConActor::new(2000, sprite.sectnum, sprite.ang, 100),
                            crate::combat::SituationalSpawn {
                                initial_picnum: sprite.picnum,
                                is_dormant: sprite.picnum == 2001,
                            },
                        ));
                    }
                    // 3. Pigcop Recon Car (1960)
                    1960 => {
                        entity_cmds.insert((
                            crate::combat::EnemyActor::new_recon(),
                            crate::scripting::ConActor::new(1960, sprite.sectnum, sprite.ang, 50),
                            crate::combat::FlyingActor::default(),
                        ));
                    }
                    // 4. Pigcop Riot Tank (1975)
                    1975 => {
                        entity_cmds.insert((
                            crate::combat::EnemyActor::new_tank(),
                            crate::scripting::ConActor::new(1975, sprite.sectnum, sprite.ang, 500),
                        ));
                    }
                    // 5. Octabrain (1820, 1821)
                    1820 | 1821 => {
                        entity_cmds.insert((
                            crate::combat::EnemyActor::new_octabrain(),
                            crate::scripting::ConActor::new(1820, sprite.sectnum, sprite.ang, 175),
                            crate::combat::SituationalSpawn {
                                initial_picnum: sprite.picnum,
                                is_dormant: sprite.picnum == 1821,
                            },
                            crate::combat::FlyingActor::default(),
                        ));
                    }
                    // 6. Protozoid Egg & Slimer (675, 2370)
                    675 => {
                        entity_cmds.insert((
                            crate::combat::EnemyActor::new_egg(),
                            crate::scripting::ConActor::new(675, sprite.sectnum, sprite.ang, 20),
                            crate::combat::SituationalSpawn {
                                initial_picnum: 675,
                                is_dormant: true,
                            },
                        ));
                    }
                    2370 => {
                        entity_cmds.insert((
                            crate::combat::EnemyActor::new_slimer(),
                            crate::scripting::ConActor::new(2370, sprite.sectnum, sprite.ang, 1),
                        ));
                    }
                    // 7. Enforcer (2120, 2121, 2150, 2160, 2165)
                    2120..=2165 => {
                        entity_cmds.insert((
                            crate::combat::EnemyActor::new_enforcer(),
                            crate::scripting::ConActor::new(2120, sprite.sectnum, sprite.ang, 120),
                            crate::combat::SituationalSpawn {
                                initial_picnum: sprite.picnum,
                                is_dormant: sprite.picnum == 2121,
                            },
                        ));
                    }
                    // 8. Assault Commander (1920, 1921)
                    1920 | 1921 => {
                        entity_cmds.insert((
                            crate::combat::EnemyActor::new_commander(),
                            crate::scripting::ConActor::new(1920, sprite.sectnum, sprite.ang, 350),
                            crate::combat::SituationalSpawn {
                                initial_picnum: sprite.picnum,
                                is_dormant: sprite.picnum == 1921,
                            },
                            crate::combat::FlyingActor::default(),
                        ));
                    }
                    // 9. Sentry Drone (1880)
                    1880 => {
                        entity_cmds.insert((
                            crate::combat::EnemyActor::new_drone(),
                            crate::scripting::ConActor::new(1880, sprite.sectnum, sprite.ang, 150),
                            crate::combat::FlyingActor::default(),
                        ));
                    }
                    // 10. Shark (1550)
                    1550 => {
                        entity_cmds.insert((
                            crate::combat::EnemyActor::new_shark(),
                            crate::scripting::ConActor::new(1550, sprite.sectnum, sprite.ang, 35),
                            crate::combat::FlyingActor::default(),
                        ));
                    }
                    // 11. Protector Drone (4610, 4611, 4670, 4690)
                    4610 | 4611 | 4670 | 4690 => {
                        entity_cmds.insert((
                            crate::combat::EnemyActor::new_protector_drone(),
                            crate::scripting::ConActor::new(4610, sprite.sectnum, sprite.ang, 300),
                            crate::combat::SituationalSpawn {
                                initial_picnum: sprite.picnum,
                                is_dormant: sprite.picnum == 4670 || sprite.picnum == 4611,
                            },
                        ));
                    }
                    // 12. Turret (2360)
                    2360 => {
                        entity_cmds.insert((
                            crate::combat::EnemyActor::new_turret(),
                            crate::scripting::ConActor::new(2360, sprite.sectnum, sprite.ang, 40),
                        ));
                    }
                    // 13. Boss 1: Battlelord & Mini-Battlelord (2630, 2631)
                    2630 | 2631 => {
                        let is_mini = sprite.pal == 21;
                        let (enemy, actor_hp) = if is_mini {
                            (crate::combat::EnemyActor::new_battlelord(true), 1000)
                        } else {
                            (crate::combat::EnemyActor::new_battlelord(false), 4500)
                        };
                        entity_cmds.insert((
                            enemy,
                            crate::scripting::ConActor::new(2630, sprite.sectnum, sprite.ang, actor_hp),
                            crate::combat::SituationalSpawn {
                                initial_picnum: sprite.picnum,
                                is_dormant: sprite.picnum == 2631,
                            },
                        ));
                    }
                    // 14. Boss 2: Overlord (2710)
                    2710 => {
                        entity_cmds.insert((
                            crate::combat::EnemyActor::new_overlord(),
                            crate::scripting::ConActor::new(2710, sprite.sectnum, sprite.ang, 4500),
                        ));
                    }
                    // 15. Boss 3: Cycloid Emperor (2760)
                    2760 => {
                        entity_cmds.insert((
                            crate::combat::EnemyActor::new_cycloid(),
                            crate::scripting::ConActor::new(2760, sprite.sectnum, sprite.ang, 4500),
                        ));
                    }
                    // 16. Boss 4: Alien Queen (4740)
                    4740 => {
                        entity_cmds.insert((
                            crate::combat::EnemyActor::new_queen(),
                            crate::scripting::ConActor::new(4740, sprite.sectnum, sprite.ang, 6000),
                        ));
                    }
                    // NUKE BUTTON (Level Exit)
                    142..=145 => {
                        entity_cmds.insert(crate::interactivity::NukeExitSwitch {
                            is_activated: false,
                            is_secret: sprite.lotag != 0,
                            lotag: sprite.lotag,
                            hitag: sprite.hitag,
                        });
                    }
                    _ => {}
                }

                // Check for tile animation on sprite
                if let Some(&picanm) = self.picanm_map.get(&sprite.picnum) {
                    if picanm.num_frames > 0 && picanm.anim_type > 0 {
                        entity_cmds.insert(AnimatedTileMaterial {
                            base_picnum: sprite.picnum,
                            picanm,
                            current_offset: 0,
                            material_handle: sprite_mat,
                        });
                    }
                }
            }
        }
    }
}

fn triangulate_polygon_ear_clipping(points: &[Vec2], base_idx: u32) -> Vec<u32> {
    let mut indices = Vec::new();
    let n = points.len();
    if n < 3 {
        return indices;
    }
    if n == 3 {
        return vec![base_idx, base_idx + 1, base_idx + 2];
    }

    let mut vertex_indices: Vec<usize> = (0..n).collect();
    let mut count = 0;
    while vertex_indices.len() > 2 && count < n * 4 {
        count += 1;
        let mut ear_found = false;
        let len = vertex_indices.len();
        for i in 0..len {
            let prev = vertex_indices[(i + len - 1) % len];
            let curr = vertex_indices[i];
            let next = vertex_indices[(i + 1) % len];

            let a = points[prev];
            let b = points[curr];
            let c = points[next];

            let cross = (b.x - a.x) * (c.y - a.y) - (b.y - a.y) * (c.x - a.x);
            if cross.abs() > 0.00001 {
                let mut contains_other = false;
                for &idx in &vertex_indices {
                    if idx == prev || idx == curr || idx == next {
                        continue;
                    }
                    let p = points[idx];
                    if point_in_triangle_2d(p, a, b, c) {
                        contains_other = true;
                        break;
                    }
                }

                if !contains_other {
                    indices.push(base_idx + prev as u32);
                    indices.push(base_idx + curr as u32);
                    indices.push(base_idx + next as u32);
                    vertex_indices.remove(i);
                    ear_found = true;
                    break;
                }
            }
        }
        if !ear_found {
            // Fan fallback for remaining vertices
            let first = vertex_indices[0];
            for i in 1..vertex_indices.len() - 1 {
                indices.push(base_idx + first as u32);
                indices.push(base_idx + vertex_indices[i] as u32);
                indices.push(base_idx + vertex_indices[i + 1] as u32);
            }
            break;
        }
    }
    indices
}

fn point_in_triangle_2d(p: Vec2, a: Vec2, b: Vec2, c: Vec2) -> bool {
    let d1 = (p.x - b.x) * (a.y - b.y) - (a.x - b.x) * (p.y - b.y);
    let d2 = (p.x - c.x) * (b.y - c.y) - (b.x - c.x) * (p.y - c.y);
    let d3 = (p.x - a.x) * (c.y - a.y) - (c.x - a.x) * (p.y - a.y);
    let has_neg = (d1 < 0.0) || (d2 < 0.0) || (d3 < 0.0);
    let has_pos = (d1 > 0.0) || (d2 > 0.0) || (d3 > 0.0);
    !(has_neg && has_pos)
}

#[cfg(test)]
mod tests {
    use super::*;
    use lyon_tessellation::math::point;
    use lyon_tessellation::{BuffersBuilder, FillOptions, FillTessellator, FillVertex, VertexBuffers};

    #[test]
    fn test_e1l1_all_sectors_tessellation() {
        if let Ok(grp) = crate::grp::Grp::open("dukenukem3d/duke3d.grp") {
            if let Ok(map_data) = grp.read_file("E1L1.MAP") {
                let map = Map::from_bytes(&map_data).unwrap();
                println!("Testing E1L1.MAP with {} sectors...", map.sectors.len());
                let mut failed_sectors = 0;
                let total_sectors = map.sectors.len();

                for (sec_idx, sector) in map.sectors.iter().enumerate() {
                    let mut loops = Vec::new();
                    let mut current_loop = Vec::new();
                    let mut visited_walls = HashSet::new();

                    for i in 0..sector.wallnum {
                        let wall_idx = (sector.wallptr + i) as usize;
                        if visited_walls.contains(&wall_idx) || wall_idx >= map.walls.len() {
                            continue;
                        }

                        let mut w = wall_idx;
                        loop {
                            if visited_walls.contains(&w) || w >= map.walls.len() {
                                break;
                            }
                            visited_walls.insert(w);
                            let wall = &map.walls[w];
                            current_loop.push(Vec2::new(
                                wall.x as f32 / 1024.0,
                                wall.y as f32 / 1024.0,
                            ));
                            let next_w = wall.point2 as usize;
                            if next_w == wall_idx {
                                break;
                            }
                            if next_w < sector.wallptr as usize
                                || next_w >= (sector.wallptr + sector.wallnum) as usize
                            {
                                break;
                            }
                            w = next_w;
                        }
                        if !current_loop.is_empty() {
                            loops.push(std::mem::take(&mut current_loop));
                        }
                    }

                    if loops.is_empty() {
                        println!("Sector {} has NO loops!", sec_idx);
                        failed_sectors += 1;
                        continue;
                    }

                    let mut tessellator = FillTessellator::new();
                    let mut buffers: VertexBuffers<[f32; 2], u32> = VertexBuffers::new();
                    let mut path_builder = lyon_tessellation::path::Path::builder();
                    for poly_points in &loops {
                        if poly_points.len() < 3 {
                            continue;
                        }
                        path_builder.begin(point(poly_points[0].x, poly_points[0].y));
                        for p in poly_points.iter().skip(1) {
                            path_builder.line_to(point(p.x, p.y));
                        }
                        path_builder.end(true);
                    }
                    let path = path_builder.build();

                    let lyon_ok = tessellator.tessellate_path(
                        &path,
                        &FillOptions::default(),
                        &mut BuffersBuilder::new(&mut buffers, |vertex: FillVertex| {
                            [vertex.position().x, vertex.position().y]
                        }),
                    ).is_ok();

                    if !lyon_ok || buffers.indices.is_empty() {
                        buffers.vertices.clear();
                        buffers.indices.clear();
                        for poly in &loops {
                            if poly.len() < 3 { continue; }
                            let base_idx = buffers.vertices.len() as u32;
                            for p in poly {
                                buffers.vertices.push([p.x, p.y]);
                            }
                            let indices = triangulate_polygon_ear_clipping(poly, base_idx);
                            buffers.indices.extend(indices);
                        }
                    }

                    if buffers.indices.is_empty() {
                        println!("Sector {} FAILED! wallptr={}, wallnum={}, loops count={}", sec_idx, sector.wallptr, sector.wallnum, loops.len());
                        failed_sectors += 1;
                    }
                }

                println!("Tessellation result: {}/{} succeeded, {} failed", total_sectors - failed_sectors, total_sectors, failed_sectors);
                assert_eq!(failed_sectors, 0, "All sectors must succeed!");
            }
        }
    }

    #[test]
    fn test_e1l1_player_start_sector_floor_collider() {
        if let Ok(grp) = crate::grp::Grp::open("dukenukem3d/duke3d.grp") {
            if let Ok(map_data) = grp.read_file("E1L1.MAP") {
                let map = Map::from_bytes(&map_data).unwrap();
                let sec_idx = map.cursectnum as usize;
                let sector = &map.sectors[sec_idx];
                let floor_y = sector.get_floor_y_at(&map.walls, map.posx, map.posy);
                assert!((floor_y - 9.0625).abs() < 0.001);
                assert_eq!(sector.floorstat, 100);
            }
        }
    }
}
