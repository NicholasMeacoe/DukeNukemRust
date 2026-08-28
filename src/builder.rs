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
        self.tile_sizes.get(&picnum).copied().unwrap_or((64, 64))
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
        for sector in &self.map.sectors {
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

            if tessellator
                .tessellate_path(
                    &path,
                    &FillOptions::default(),
                    &mut BuffersBuilder::new(&mut buffers, |vertex: FillVertex| {
                        [vertex.position().x, vertex.position().y]
                    }),
                )
                .is_err()
            {
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
                let floor_collider_indices: Vec<[u32; 3]> = floor_indices
                    .chunks(3)
                    .map(|c| [c[0], c[1], c[2]])
                    .collect();

                if !floor_collider_indices.is_empty() {
                    let mut entity_cmds = commands.spawn((
                        PbrBundle {
                            mesh: meshes.add(floor_mesh),
                            material: floor_mat.clone(),
                            ..default()
                        },
                        RigidBody::Fixed,
                        Collider::trimesh(floor_collider_vertices, floor_collider_indices),
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
                let ceil_collider_indices: Vec<[u32; 3]> = ceil_indices
                    .chunks(3)
                    .map(|c| [c[0], c[1], c[2]])
                    .collect();

                if !ceil_collider_indices.is_empty() {
                    let mut entity_cmds = commands.spawn((
                        PbrBundle {
                            mesh: meshes.add(ceil_mesh),
                            material: ceil_mat.clone(),
                            ..default()
                        },
                        RigidBody::Fixed,
                        Collider::trimesh(ceil_collider_vertices, ceil_collider_indices),
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
        for sector in &self.map.sectors {
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
        let collider_indices: Vec<[u32; 3]> = vec![[0, 1, 2], [0, 2, 3]];

        let mut entity_cmds = commands.spawn(PbrBundle {
            mesh: meshes.add(wall_mesh),
            material: mat.clone(),
            ..default()
        });

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
                ));

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
