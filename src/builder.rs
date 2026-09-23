use bevy::prelude::*;
use bevy::render::render_asset::RenderAssetUsages;
use bevy_rapier3d::prelude::*;
use lyon_tessellation::math::point;
use lyon_tessellation::{BuffersBuilder, FillOptions, FillTessellator, FillVertex, VertexBuffers};
use std::collections::{HashMap, HashSet};

use crate::animation::AnimatedTileMaterial;
use crate::art::PicAnm;
use crate::map::{Map, Wall};
use crate::names::*;
use crate::palette::Palette;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum MaterialAlphaMode {
    Opaque,
    Mask,
    Blend(u8), // Percentage opacity: e.g. 66 for water surface or glass, 33 for faint forcefields
}

pub fn is_water_sector(lotag: i16) -> bool {
    lotag == 1 || lotag == 2
}

pub fn is_water_surface(lotag: i16, part: crate::interactivity::SectorMeshPart) -> bool {
    (lotag == 1 && part == crate::interactivity::SectorMeshPart::Floor)
        || (lotag == 2 && part == crate::interactivity::SectorMeshPart::Ceiling)
}

pub struct MapMeshBuilder<'a> {
    pub map: &'a Map,
    pub tile_textures: &'a HashMap<i16, Handle<Image>>,
    pub tile_sizes: &'a HashMap<i16, (u32, u32)>,
    pub picanm_map: &'a HashMap<i16, PicAnm>,
    pub default_material: Handle<StandardMaterial>,
    material_cache: std::cell::RefCell<HashMap<(i16, MaterialAlphaMode), Handle<StandardMaterial>>>,
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

    pub fn get_material(
        &self,
        picnum: i16,
        alpha_mode: MaterialAlphaMode,
        materials: &mut Assets<StandardMaterial>,
    ) -> Handle<StandardMaterial> {
        let key = (picnum, alpha_mode);
        if let Some(handle) = self.material_cache.borrow().get(&key) {
            return handle.clone();
        }

        let handle = if let Some(tex_handle) = self.tile_textures.get(&picnum) {
            let (bevy_alpha, base_color) = match alpha_mode {
                MaterialAlphaMode::Opaque => (AlphaMode::Opaque, Color::WHITE),
                MaterialAlphaMode::Mask => (AlphaMode::Mask(0.5), Color::WHITE),
                MaterialAlphaMode::Blend(opacity_pct) => {
                    let alpha = (opacity_pct as f32) / 100.0;
                    (AlphaMode::Blend, Color::srgba(1.0, 1.0, 1.0, alpha))
                }
            };
            materials.add(StandardMaterial {
                base_color_texture: Some(tex_handle.clone()),
                base_color,
                alpha_mode: bevy_alpha,
                unlit: false,
                double_sided: true,
                perceptual_roughness: 1.0,
                ..default()
            })
        } else {
            println!("WARNING: Texture {} not found in tile_textures!", picnum);
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
        skill_level: u8,
        voxel_registry: Option<&mut crate::voxel::VoxelRegistry>,
        palette: Option<&crate::palette::Palette>,
    ) {
        self.build_sectors(commands, meshes, materials);
        self.build_walls(commands, meshes, materials);
        self.build_sprites(
            commands,
            meshes,
            materials,
            skill_level,
            voxel_registry,
            palette,
        );
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
                    current_loop.push(Vec2::new(wall.x as f32 / 1024.0, wall.y as f32 / 1024.0));
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
                    if poly.len() < 3 {
                        continue;
                    }
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
                        let mut bx = v[0] * 1024.0;
                        let mut by = v[1] * 1024.0;
                        
                        if sector.floorstat & 64 != 0 && sector.wallnum > 0 {
                            let w0 = &self.map.walls[sector.wallptr as usize];
                            let w1 = &self.map.walls[w0.point2 as usize];
                            let dx = (w1.x - w0.x) as f32;
                            let dy = (w1.y - w0.y) as f32;
                            let len = (dx * dx + dy * dy).sqrt();
                            if len > 0.001 {
                                let ux = dx / len;
                                let uy = dy / len;
                                let px = bx - (w0.x as f32);
                                let py = by - (w0.y as f32);
                                // Rotate relative to first wall
                                bx = px * ux + py * uy;
                                by = px * uy - py * ux; 
                            }
                        }

                        let u =
                            (bx / (floor_tw as f32 * 16.0)) + (sector.floorxpanning as f32 / 256.0);
                        let v_coord =
                            (by / (floor_th as f32 * 16.0)) + (sector.floorypanning as f32 / 256.0);
                        [u, v_coord]
                    })
                    .collect();

                let is_water = is_water_surface(sector.lotag, crate::interactivity::SectorMeshPart::Floor);
                let floor_alpha = if is_water {
                    MaterialAlphaMode::Blend(60)
                } else {
                    MaterialAlphaMode::Opaque
                };
                let mut floor_tint = Palette::authentic_shade_to_tint(sector.floorshade);
                if is_water {
                    floor_tint[0] *= 0.8;
                    floor_tint[1] *= 0.95;
                    floor_tint[2] *= 1.1;
                    floor_tint[3] = 0.60;
                }
                let floor_colors: Vec<[f32; 4]> = vec![floor_tint; floor_vertices.len()];

                let floor_indices = buffers.indices.clone();
                let floor_mat = self.get_material(sector.floorpicnum, floor_alpha, materials);

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
                        crate::interactivity::DynamicSectorMesh {
                            sector_idx: sec_idx,
                            part: crate::interactivity::SectorMeshPart::Floor,
                        },
                        crate::game_flow::LevelEntity,
                    ));

                    if is_water {
                        let water_elevation = floor_vertices
                            .first()
                            .map(|v| v[1])
                            .unwrap_or_else(|| -(sector.floorz as f32) / 16384.0);
                        entity_cmds.insert(crate::interactivity::WaterSurface {
                            sector_index: sec_idx,
                            elevation: water_elevation,
                        });
                        entity_cmds.insert(Collider::trimesh(floor_collider_vertices, floor_collider_indices));
                        entity_cmds.insert(Sensor);
                    } else {
                        entity_cmds.insert(RigidBody::Fixed);
                        entity_cmds.insert(Collider::trimesh(floor_collider_vertices, floor_collider_indices));
                    }

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
                        let mut bx = v[0] * 1024.0;
                        let mut by = v[1] * 1024.0;
                        
                        if sector.ceilingstat & 64 != 0 && sector.wallnum > 0 {
                            let w0 = &self.map.walls[sector.wallptr as usize];
                            let w1 = &self.map.walls[w0.point2 as usize];
                            let dx = (w1.x - w0.x) as f32;
                            let dy = (w1.y - w0.y) as f32;
                            let len = (dx * dx + dy * dy).sqrt();
                            if len > 0.001 {
                                let ux = dx / len;
                                let uy = dy / len;
                                let px = bx - (w0.x as f32);
                                let py = by - (w0.y as f32);
                                bx = px * ux + py * uy;
                                by = px * uy - py * ux;
                            }
                        }

                        let u = (bx / (ceil_tw as f32 * 16.0))
                            + (sector.ceilingxpanning as f32 / 256.0);
                        let v_coord = (by / (ceil_th as f32 * 16.0))
                            + (sector.ceilingypanning as f32 / 256.0);
                        [u, v_coord]
                    })
                    .collect();

                let is_water = is_water_surface(sector.lotag, crate::interactivity::SectorMeshPart::Ceiling);
                let ceil_alpha = if is_water {
                    MaterialAlphaMode::Blend(60)
                } else {
                    MaterialAlphaMode::Opaque
                };
                let mut ceil_tint = Palette::authentic_shade_to_tint(sector.ceilingshade);
                if is_water {
                    ceil_tint[0] *= 0.8;
                    ceil_tint[1] *= 0.95;
                    ceil_tint[2] *= 1.1;
                    ceil_tint[3] = 0.60;
                }
                let ceil_colors: Vec<[f32; 4]> = vec![ceil_tint; ceil_vertices.len()];

                // Reverse ceiling winding order so normals face downwards
                let ceil_indices: Vec<u32> = buffers
                    .indices
                    .chunks(3)
                    .flat_map(|c| [c[0], c[2], c[1]])
                    .collect();

                let ceil_mat = self.get_material(sector.ceilingpicnum, ceil_alpha, materials);

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
                        crate::interactivity::DynamicSectorMesh {
                            sector_idx: sec_idx,
                            part: crate::interactivity::SectorMeshPart::Ceiling,
                        },
                        crate::game_flow::LevelEntity,
                    ));

                    if is_water {
                        let water_elevation = ceil_vertices
                            .first()
                            .map(|v| v[1])
                            .unwrap_or_else(|| -(sector.ceilingz as f32) / 16384.0);
                        entity_cmds.insert(crate::interactivity::WaterSurface {
                            sector_index: sec_idx,
                            elevation: water_elevation,
                        });
                        entity_cmds.insert(Collider::trimesh(ceil_collider_vertices, ceil_collider_indices));
                        entity_cmds.insert(Sensor);
                    } else {
                        entity_cmds.insert(RigidBody::Fixed);
                        entity_cmds.insert(Collider::trimesh(ceil_collider_vertices, ceil_collider_indices));
                    }

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
                let cur_ceil_y2 =
                    sector.get_ceiling_y_at(&self.map.walls, next_wall.x, next_wall.y);

                if !wall.is_portal() {
                    // One-sided solid wall: floor to ceiling
                    self.spawn_wall_quad(
                        commands,
                        meshes,
                        materials,
                        sec_idx,
                        crate::interactivity::SectorMeshPart::MiddleWall,
                        p1,
                        p2,
                        cur_floor_y1,
                        cur_floor_y2,
                        cur_ceil_y1,
                        cur_ceil_y2,
                        wall,
                        wall.picnum,
                        true,  // solid blocking
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
                    let next_floor_y2 =
                        next_sec.get_floor_y_at(&self.map.walls, next_wall.x, next_wall.y);
                    let next_ceil_y1 = next_sec.get_ceiling_y_at(&self.map.walls, wall.x, wall.y);
                    let next_ceil_y2 =
                        next_sec.get_ceiling_y_at(&self.map.walls, next_wall.x, next_wall.y);

                    // 1. Upper Wall (Step down from ceiling)
                    let cur_sec = &self.map.sectors[sec_idx];
                    if (next_ceil_y1 < cur_ceil_y1 - 0.001 || next_ceil_y2 < cur_ceil_y2 - 0.001)
                        && !cur_sec.is_ceiling_parallax()
                        && !next_sec.is_ceiling_parallax()
                    {
                        self.spawn_wall_quad(
                            commands,
                            meshes,
                            materials,
                            sec_idx,
                            crate::interactivity::SectorMeshPart::UpperWall,
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

                    // 2. Lower Wall (Step up from floor or drop down to abyss)
                    let cur_sec = &self.map.sectors[sec_idx];
                    if (next_floor_y1 > cur_floor_y1 + 0.001 || next_floor_y2 > cur_floor_y2 + 0.001) 
                        && !cur_sec.is_floor_parallax()
                        && !next_sec.is_floor_parallax()
                    {
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
                            crate::interactivity::SectorMeshPart::LowerWall,
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

                        if mid_ceil_y1 > mid_floor_y1 + 0.001 || mid_ceil_y2 > mid_floor_y2 + 0.001
                        {
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
                                crate::interactivity::SectorMeshPart::MiddleWall,
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
        part: crate::interactivity::SectorMeshPart,
        p1: Vec2,
        p2: Vec2,
        mut bottom_y1: f32,
        mut bottom_y2: f32,
        mut top_y1: f32,
        mut top_y2: f32,
        wall: &Wall,
        picnum: i16,
        is_solid: bool,
        is_masked: bool,
    ) {
        let (tw, th) = self.get_tile_size(picnum);
        
        if is_masked {
            let actual_height = (th as f32 * 2048.0) / (wall.yrepeat.max(1) as f32 * 1024.0 * 16.0);
            if wall.align_bottom() {
                top_y1 = bottom_y1 + actual_height;
                top_y2 = bottom_y2 + actual_height;
            } else {
                bottom_y1 = top_y1 - actual_height;
                bottom_y2 = top_y2 - actual_height;
            }
        }

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
        let tint = Palette::authentic_shade_to_tint(wall.shade);
        let colors = vec![tint; 4];
        let indices = vec![0u32, 1, 2, 0, 2, 3];

        let wall_alpha = wall.alpha_mode(is_masked);

        let mat = self.get_material(picnum, wall_alpha, materials);

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
            [0, 1, 2],
            [0, 2, 3],
        ];

        let visibility = if wall.yrepeat == 0 || picnum == 79 || picnum == 89 || picnum == 97 {
            Visibility::Hidden
        } else {
            Visibility::Inherited
        };

        let mut entity_cmds = commands.spawn((
            PbrBundle {
                mesh: meshes.add(wall_mesh),
                material: mat.clone(),
                visibility,
                ..default()
            },
            crate::interactivity::DynamicSectorMesh {
                sector_idx: sec_idx,
                part,
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
        skill_level: u8,
        mut voxel_registry: Option<&mut crate::voxel::VoxelRegistry>,
        palette: Option<&crate::palette::Palette>,
    ) {
        for sprite in &self.map.sprites {
            // Editor utility sprites (picnum 1..=8: SECTOREFFECTOR, ACTIVATOR, TOUCHPLATE, etc.)
            // are logic markers and should not be rendered as visible 3D textured quads in the world.
            if (1..=8).contains(&sprite.picnum) {
                let pos = Vec3::new(
                    sprite.x as f32 / 1024.0,
                    -(sprite.z as f32) / (1024.0 * 16.0),
                    sprite.y as f32 / 1024.0,
                );
                let transform = Transform::from_translation(pos);

                match sprite.picnum {
                    MASTERSWITCH => {
                        commands.spawn((
                            SpatialBundle::from_transform(transform),
                            crate::interactivity::MasterSwitch {
                                lotag: sprite.lotag,
                                hitag: sprite.hitag,
                                delay: (sprite.extra.max(0) as f32) * 0.1,
                                timer: None,
                                is_triggered: false,
                            },
                            crate::game_flow::LevelEntity,
                        ));
                    }
                    MUSICANDSFX => {
                        let sound_id = sprite.lotag as i32;
                        let range = if sprite.hitag > 0 {
                            (sprite.hitag as f32) / 1024.0 * 16.0
                        } else {
                            20.0
                        };
                        commands.spawn((
                            SpatialBundle::from_transform(transform),
                            crate::audio::AmbientSoundEmitter {
                                sound_id,
                                range,
                                repeat_delay: 4.0,
                                timer: 0.5,
                            },
                            crate::game_flow::LevelEntity,
                        ));
                    }
                    _ => {}
                }
                continue;
            }

            // Difficulty filtering (only affects enemies, items, etc. mapped to lotag > 0)
            if sprite.lotag > 0 && sprite.lotag <= 4 && sprite.lotag > (skill_level as i16 + 1) {
                // If it's a known monster or item, we should skip it.
                // In Duke 3D, lotag 1-4 is exclusively for difficulty on these actors.
                // We'll skip spawning them entirely.
                match sprite.picnum {
                    // Enemies
                    p if p == PIGCOP || p == LIZTROOP || p == OCTABRAIN || p == ENFORCER
                        || p == 1960 || p == 2370 || p == 2710 || p == 4610 => {
                        continue;
                    }
                    // Weapons & pickups & multiplayer starts
                    21 | 22 | 23 | 27 | 28 | 29 | 33 | 37 | 40 | 44
                    | 51 | 52 | 53 | 54 | 55 | 56 | 57 | 60 | 61 | 1405 => {
                        continue;
                    }
                    _ => {} // Other things with lotag (like sector effectors) are logic IDs!
                }
            }

            let has_voxel_model = if let Some(ref reg) = voxel_registry {
                reg.has_voxel(sprite.picnum)
            } else {
                false
            };

            if self.tile_textures.contains_key(&sprite.picnum) || has_voxel_model {
                let (tw, th) = self.get_tile_size(sprite.picnum);
                let mut pos = Vec3::new(
                    sprite.x as f32 / 1024.0,
                    -(sprite.z as f32) / (1024.0 * 16.0),
                    sprite.y as f32 / 1024.0,
                );

                let is_wall_aligned = (sprite.cstat & 16) != 0;
                let is_floor_aligned = (sprite.cstat & 32) != 0;

                let divisor = 4096.0;
                let scale_x = (sprite.xrepeat as f32 * tw as f32) / divisor;
                let scale_y = (sprite.yrepeat as f32 * th as f32) / divisor;

                let is_enemy = sprite.picnum == PIGCOP;

                let is_barrel = matches!(
                    sprite.picnum,
                    EXPLODINGBARREL
                        | EXPLODINGBARREL2
                        | FIREBARREL
                        | NUKEBARREL
                        | NUKEBARRELDENTED
                        | NUKEBARRELLEAKED
                );
                let is_wall_prop = matches!(
                    sprite.picnum,
                    FIREEXT | CAMERA1 | 500 | WATERFOUNTAIN | 564 | 565
                );
                let is_ceiling_fan = sprite.picnum == 617;
                let is_env_prop = is_barrel || is_wall_prop || is_ceiling_fan;

                // In Build Engine, Z is the bottom of the sprite unless cstat & 128 is set (Centered)
                let is_centered = (sprite.cstat & 128) != 0;
                if is_ceiling_fan {
                    if let Some(sector) = self.map.sectors.get(sprite.sectnum as usize) {
                        pos.y = -(sector.ceilingz as f32) / (1024.0 * 16.0);
                    }
                } else if is_barrel {
                    // For barrels with voxel models, pivot.z == 0 means mesh bottom is at local y=0.
                    // Keep pos.y directly at the floor elevation without half-height offset.
                } else if !is_centered {
                    pos.y += scale_y / 2.0;
                }

                // Check if a 3D voxel model is available for this sprite
                let mut voxel_mesh_opt = None;
                let mut voxel_mat_opt = None;

                if let (Some(ref mut reg), Some(pal)) = (&mut voxel_registry, palette) {
                    if reg.has_voxel(sprite.picnum) {
                        voxel_mesh_opt = reg.get_or_create_mesh(sprite.picnum, pal, meshes);
                        voxel_mat_opt = Some(reg.get_or_create_material(materials));
                    }
                }

                let is_voxel = voxel_mesh_opt.is_some() && voxel_mat_opt.is_some();

                let (mesh_handle, mat_handle, transform) = if is_voxel {
                    let mut t = Transform::from_translation(pos);
                    if is_wall_aligned || is_wall_prop {
                        let angle_rad = ((512.0 - sprite.ang as f32) / 2048.0) * std::f32::consts::TAU;
                        t.rotation = Quat::from_rotation_y(angle_rad);
                        // Standoff offset along outward wall normal to avoid clipping into wall collider
                        let normal = t.rotation * Vec3::Z;
                        t.translation += normal * 0.05;
                    } else if is_ceiling_fan {
                        let angle_rad = ((512.0 - sprite.ang as f32) / 2048.0) * std::f32::consts::TAU;
                        t.rotation = Quat::from_rotation_y(angle_rad);
                    }
                    (voxel_mesh_opt.unwrap(), voxel_mat_opt.unwrap(), t)
                } else {
                    let sprite_alpha = sprite.alpha_mode();
                    let sprite_mat = self.get_material(sprite.picnum, sprite_alpha, materials);

                    let mut t = Transform::from_translation(pos);
                    if is_wall_aligned {
                        let angle_rad = ((512.0 - sprite.ang as f32) / 2048.0) * std::f32::consts::TAU;
                        t.rotation = Quat::from_rotation_y(angle_rad);
                    } else if is_floor_aligned {
                        let angle_rad = ((sprite.ang as f32 - 512.0) / 2048.0) * std::f32::consts::TAU;
                        t.rotation = Quat::from_rotation_x(-std::f32::consts::FRAC_PI_2) * Quat::from_rotation_z(angle_rad);
                    }
                    t.scale = Vec3::new(scale_x, scale_y, 1.0);

                    let mut sprite_mesh = Mesh::new(
                        bevy::render::mesh::PrimitiveTopology::TriangleList,
                        bevy::render::render_asset::RenderAssetUsages::default(),
                    );
                    sprite_mesh.insert_attribute(Mesh::ATTRIBUTE_POSITION, vec![
                        [-0.5, -0.5, 0.0],
                        [0.5, -0.5, 0.0],
                        [0.5, 0.5, 0.0],
                        [-0.5, 0.5, 0.0],
                    ]);
                    sprite_mesh.insert_attribute(Mesh::ATTRIBUTE_UV_0, vec![
                        [0.001, 0.999],
                        [0.999, 0.999],
                        [0.999, 0.001],
                        [0.001, 0.001],
                    ]);
                    sprite_mesh.insert_attribute(Mesh::ATTRIBUTE_NORMAL, vec![
                        [0.0, 0.0, 1.0],
                        [0.0, 0.0, 1.0],
                        [0.0, 0.0, 1.0],
                        [0.0, 0.0, 1.0],
                    ]);
                    let sprite_tint = Palette::authentic_shade_to_tint(sprite.shade);
                    sprite_mesh.insert_attribute(Mesh::ATTRIBUTE_COLOR, vec![sprite_tint; 4]);
                    sprite_mesh.insert_indices(bevy::render::mesh::Indices::U32(vec![
                        0, 1, 2, 0, 2, 3,
                        0, 2, 1, 0, 3, 2,
                    ]));
                    (meshes.add(sprite_mesh), sprite_mat, t)
                };

                let mut entity_cmds = commands.spawn((
                    PbrBundle {
                        mesh: mesh_handle,
                        material: mat_handle.clone(),
                        transform,
                        ..default()
                    },
                    crate::Destructible {
                        health: if is_enemy { 100 } else { 10 },
                        _picnum: sprite.picnum,
                    },
                    crate::game_flow::LevelEntity,
                ));

                let is_blocking = (sprite.cstat & 1) != 0;
                if is_blocking || is_enemy {
                    entity_cmds.insert(RigidBody::Fixed);
                    
                    if !is_wall_aligned && !is_floor_aligned {
                        entity_cmds.insert(Collider::cylinder(0.4, 0.4));
                    } else if is_wall_aligned {
                        entity_cmds.insert(Collider::cuboid(0.5, 0.5, 0.05));
                    } else {
                        entity_cmds.insert(Collider::cuboid(0.5, 0.05, 0.5));
                    }
                }

                if is_voxel {
                    if is_env_prop {
                        entity_cmds.insert(crate::voxel::VoxelModelInstance::new_prop(sprite.picnum, pos.y));
                    } else {
                        entity_cmds.insert(crate::voxel::VoxelModelInstance::new_pickup(sprite.picnum, pos.y));
                    }
                    if is_ceiling_fan {
                        entity_cmds.insert(crate::voxel::CeilingFanVoxel::default());
                    }
                } else if !is_wall_aligned && !is_floor_aligned {
                    entity_cmds.insert(crate::SpriteBillboard);
                }

                // Attach Phase 4 Interactive Components directly to visual entities!
                match sprite.picnum {
                    // SWITCHES
                    LIGHTSWITCH | SPACEDOORSWITCH | DIPSWITCH | LIGHTSWITCH2 | POWERSWITCH1
                    | HANDSWITCH | PULLSWITCH => {
                        entity_cmds.insert(crate::interactivity::InteractiveSwitch {
                            switch_type: crate::interactivity::SwitchType::LightSwitch,
                            on_tile: sprite.picnum + 1,
                            off_tile: sprite.picnum,
                            is_on: false,
                            lotag: sprite.lotag,
                            hitag: sprite.hitag,
                            sound_id: 10,
                            material_handle: Some(mat_handle.clone()),
                        });
                    }
                    // KEYCARDS (Tiles ACCESSCARD..=177: Blue, Red, Yellow)
                    ACCESSCARD => {
                        let key_type = match sprite.pal {
                            21 => 2, // Red keycard
                            23 => 3, // Yellow keycard
                            _ => 1,  // Blue keycard
                        };
                        entity_cmds.insert(crate::interactivity::KeycardPickup { key_type });
                    }
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
                    COLA => {
                        entity_cmds.insert(crate::interactivity::ItemPickup {
                            kind: crate::interactivity::PickupKind::SmallMedkit,
                            respawn_timer: None,
                        });
                    }
                    SIXPAK => {
                        entity_cmds.insert(crate::interactivity::ItemPickup {
                            kind: crate::interactivity::PickupKind::LargeMedkit,
                            respawn_timer: None,
                        });
                    }
                    ATOMICHEALTH => {
                        entity_cmds.insert(crate::interactivity::ItemPickup {
                            kind: crate::interactivity::PickupKind::AtomicHealth,
                            respawn_timer: None,
                        });
                    }
                    SHIELD => {
                        entity_cmds.insert(crate::interactivity::ItemPickup {
                            kind: crate::interactivity::PickupKind::ArmorVest,
                            respawn_timer: None,
                        });
                    }
                    // AMMO PICKUPS
                    AMMO => {
                        entity_cmds.insert(crate::interactivity::ItemPickup {
                            kind: crate::interactivity::PickupKind::PistolClip,
                            respawn_timer: None,
                        });
                    }
                    RPGAMMO => {
                        entity_cmds.insert(crate::interactivity::ItemPickup {
                            kind: crate::interactivity::PickupKind::ChaingunBox,
                            respawn_timer: None,
                        });
                    }
                    HBOMBAMMO => {
                        entity_cmds.insert(crate::interactivity::ItemPickup {
                            kind: crate::interactivity::PickupKind::RpgRocket,
                            respawn_timer: None,
                        });
                    }
                    AMMOLOTS => {
                        entity_cmds.insert(crate::interactivity::ItemPickup {
                            kind: crate::interactivity::PickupKind::PipebombBox,
                            respawn_timer: None,
                        });
                    }
                    SHOTGUNAMMO => {
                        entity_cmds.insert(crate::interactivity::ItemPickup {
                            kind: crate::interactivity::PickupKind::ShotgunBox,
                            respawn_timer: None,
                        });
                    }
                    DEVISTATORAMMO => {
                        entity_cmds.insert(crate::interactivity::ItemPickup {
                            kind: crate::interactivity::PickupKind::ShrinkerAmmo,
                            respawn_timer: None,
                        });
                    }
                    GROWAMMO => {
                        entity_cmds.insert(crate::interactivity::ItemPickup {
                            kind: crate::interactivity::PickupKind::DevastatorBox,
                            respawn_timer: None,
                        });
                    }
                    CRYSTALAMMO | FREEZEAMMO => {
                        entity_cmds.insert(crate::interactivity::ItemPickup {
                            kind: crate::interactivity::PickupKind::FreezeAmmo,
                            respawn_timer: None,
                        });
                    }
                    // INVENTORY ITEM PICKUPS
                    STEROIDS => {
                        entity_cmds.insert(crate::interactivity::ItemPickup {
                            kind: crate::interactivity::PickupKind::Steroids,
                            respawn_timer: None,
                        });
                    }
                    AIRTANK => {
                        entity_cmds.insert(crate::interactivity::ItemPickup {
                            kind: crate::interactivity::PickupKind::ScubaTank,
                            respawn_timer: None,
                        });
                    }
                    JETPACK => {
                        entity_cmds.insert(crate::interactivity::ItemPickup {
                            kind: crate::interactivity::PickupKind::Jetpack,
                            respawn_timer: None,
                        });
                    }
                    HEATSENSOR | 58 => {
                        entity_cmds.insert(crate::interactivity::ItemPickup {
                            kind: crate::interactivity::PickupKind::NightvisionGoggles,
                            respawn_timer: None,
                        });
                    }
                    BOOTS => {
                        entity_cmds.insert(crate::interactivity::ItemPickup {
                            kind: crate::interactivity::PickupKind::ProtectiveBoots,
                            respawn_timer: None,
                        });
                    }
                    HOLODUKE | 62 => {
                        entity_cmds.insert(crate::interactivity::ItemPickup {
                            kind: crate::interactivity::PickupKind::Holoduke,
                            respawn_timer: None,
                        });
                    }
                    // WEAPONS ON GROUND
                    FIRSTGUNSPRITE => {
                        entity_cmds.insert(crate::interactivity::ItemPickup {
                            kind: crate::interactivity::PickupKind::WeaponPistol,
                            respawn_timer: None,
                        });
                    }
                    CHAINGUNSPRITE => {
                        entity_cmds.insert(crate::interactivity::ItemPickup {
                            kind: crate::interactivity::PickupKind::WeaponChaingun,
                            respawn_timer: None,
                        });
                    }
                    RPGSPRITE => {
                        entity_cmds.insert(crate::interactivity::ItemPickup {
                            kind: crate::interactivity::PickupKind::WeaponRpg,
                            respawn_timer: None,
                        });
                    }
                    FREEZESPRITE => {
                        entity_cmds.insert(crate::interactivity::ItemPickup {
                            kind: crate::interactivity::PickupKind::WeaponFreezer,
                            respawn_timer: None,
                        });
                    }
                    SHRINKERSPRITE => {
                        entity_cmds.insert(crate::interactivity::ItemPickup {
                            kind: crate::interactivity::PickupKind::WeaponShrinker,
                            respawn_timer: None,
                        });
                    }
                    HEAVYHBOMB => {
                        entity_cmds.insert(crate::interactivity::ItemPickup {
                            kind: crate::interactivity::PickupKind::WeaponPipebomb,
                            respawn_timer: None,
                        });
                    }
                    TRIPBOMBSPRITE => {
                        entity_cmds.insert(crate::interactivity::ItemPickup {
                            kind: crate::interactivity::PickupKind::WeaponTripbomb,
                            respawn_timer: None,
                        });
                    }
                    SHOTGUNSPRITE => {
                        entity_cmds.insert(crate::interactivity::ItemPickup {
                            kind: crate::interactivity::PickupKind::WeaponShotgun,
                            respawn_timer: None,
                        });
                    }
                    FIRSTAID => {
                        entity_cmds.insert(crate::interactivity::ItemPickup {
                            kind: crate::interactivity::PickupKind::PortableMedkit,
                            respawn_timer: None,
                        });
                    }
                    DEVISTATORSPRITE => {
                        entity_cmds.insert(crate::interactivity::ItemPickup {
                            kind: crate::interactivity::PickupKind::WeaponDevastator,
                            respawn_timer: None,
                        });
                    }
                    GROWSPRITEICON => {
                        entity_cmds.insert(crate::interactivity::ItemPickup {
                            kind: crate::interactivity::PickupKind::WeaponExpander,
                            respawn_timer: None,
                        });
                    }
                    // WATER FOUNTAIN
                    WATERFOUNTAIN | 564 | 565 => {
                        entity_cmds.insert(crate::interactivity::WaterFountain {
                            uses_left: 10,
                            is_broken: false,
                            broken_tile: WATERFOUNTAINBROKE,
                        });
                    }
                    // TOILET / STALL
                    TOILET | STALL => {
                        entity_cmds.insert(crate::interactivity::ToiletProp {
                            is_broken: false,
                            broken_tile: if sprite.picnum == TOILET {
                                TOILETBROKE
                            } else {
                                STALLBROKE
                            },
                            water_tile: TOILETWATER,
                            last_used_time: 0.0,
                            cooldown_timer: 0.0,
                        });
                    }
                    // VIEWSCREEN CRT MONITOR
                    VIEWSCREEN2 | VIEWSCREEN => {
                        entity_cmds.insert(crate::interactivity::ViewscreenProp {
                            camera_tag: sprite.hitag,
                            is_active: true,
                            is_broken: false,
                            broken_tile: VIEWSCREENBROKE,
                            scanline_timer: 0.0,
                        });
                    }
                    // SECURITY CAMERA (CAMERA1)
                    CAMERA1 | 500 => {
                        let angle_rad = ((512.0 - sprite.ang as f32) / 2048.0) * std::f32::consts::TAU;
                        entity_cmds.insert(crate::interactivity::SecurityCamera {
                            tag: sprite.hitag,
                            sweep_angle: 0.0,
                            sweep_speed: 1.0,
                            base_yaw: sprite.ang as f32,
                        });
                        if is_voxel {
                            entity_cmds.insert(crate::voxel::SecurityCameraVoxel::new(angle_rad, 0.7, 1.2));
                        }
                    }
                    // EXPLODING BARREL & RADIOACTIVE BARRELS
                    EXPLODINGBARREL | EXPLODINGBARREL2 | FIREBARREL | NUKEBARREL | NUKEBARRELDENTED | NUKEBARRELLEAKED => {
                        entity_cmds.insert(crate::interactivity::ExplodingBarrel {
                            health: 20,
                            damage_radius: 6.0,
                            damage: 100,
                            is_exploded: false,
                        });
                    }
                    // CRACK WALL
                    CRACK1..=CRACK4 => {
                        entity_cmds.insert(crate::interactivity::CrackWall {
                            health: 30,
                            stage: 1,
                            lotag: sprite.lotag,
                            is_blown: false,
                        });
                    }
                    // BREAKABLE GLASS
                    GLASS | GLASS2 => {
                        entity_cmds.insert(crate::interactivity::BreakableGlass {
                            health: 10,
                            is_broken: false,
                            wall_idx: None,
                            sector_idx: Some(sprite.sectnum as usize),
                        });
                    }
                    // FIRE EXTINGUISHER
                    FIREEXT => {
                        entity_cmds.insert(crate::interactivity::FireExtinguisher {
                            health: 10,
                            is_exploded: false,
                        });
                    }
                    // MIRROR
                    MIRROR => {
                        entity_cmds.insert(crate::interactivity::MirrorProp::default());
                    }
                    // ENEMIES & BOSSES
                    // 1. Assault Trooper & Captain (LIZTROOP..=LIZTROOPDUCKING)
                    LIZTROOP..=LIZTROOPDUCKING => {
                        let is_captain = sprite.pal == 21;
                        let (enemy, actor_hp) = if is_captain {
                            (crate::combat::EnemyActor::new_captain(), 60)
                        } else {
                            (crate::combat::EnemyActor::new_liztroop(), 30)
                        };
                        let is_dormant = matches!(
                            sprite.picnum,
                            LIZTROOPSTAYPUT | LIZTROOPONTOILET | LIZTROOPJUSTSIT | LIZTROOPDUCKING
                        );
                        let is_jetpack = sprite.picnum == LIZTROOPJETPACK;

                        entity_cmds.insert((
                            enemy,
                            crate::scripting::ConActor::new(
                                LIZTROOP,
                                sprite.sectnum,
                                sprite.ang,
                                actor_hp,
                            ),
                            crate::combat::SituationalSpawn {
                                initial_picnum: sprite.picnum,
                                is_dormant,
                            },
                        ));
                        if is_jetpack {
                            entity_cmds.insert(crate::combat::FlyingActor::default());
                        }
                    }
                    // 2. Pigcop (PIGCOP, PIGCOPSTAYPUT, PIGCOPDIVE)
                    PIGCOP | PIGCOPSTAYPUT | PIGCOPDIVE => {
                        entity_cmds.insert((
                            crate::combat::EnemyActor::new_pigcop(),
                            crate::scripting::ConActor::new(
                                PIGCOP,
                                sprite.sectnum,
                                sprite.ang,
                                100,
                            ),
                            crate::combat::SituationalSpawn {
                                initial_picnum: sprite.picnum,
                                is_dormant: sprite.picnum == PIGCOPSTAYPUT,
                            },
                        ));
                    }
                    // 3. Pigcop Recon Car (RECON)
                    RECON => {
                        entity_cmds.insert((
                            crate::combat::EnemyActor::new_recon(),
                            crate::scripting::ConActor::new(RECON, sprite.sectnum, sprite.ang, 50),
                            crate::combat::FlyingActor::default(),
                        ));
                    }
                    // 4. Pigcop Riot Tank (TANK)
                    TANK => {
                        entity_cmds.insert((
                            crate::combat::EnemyActor::new_tank(),
                            crate::scripting::ConActor::new(TANK, sprite.sectnum, sprite.ang, 500),
                        ));
                    }
                    // 5. Octabrain (OCTABRAIN, OCTABRAINSTAYPUT)
                    OCTABRAIN | OCTABRAINSTAYPUT => {
                        entity_cmds.insert((
                            crate::combat::EnemyActor::new_octabrain(),
                            crate::scripting::ConActor::new(
                                OCTABRAIN,
                                sprite.sectnum,
                                sprite.ang,
                                175,
                            ),
                            crate::combat::SituationalSpawn {
                                initial_picnum: sprite.picnum,
                                is_dormant: sprite.picnum == OCTABRAINSTAYPUT,
                            },
                            crate::combat::FlyingActor::default(),
                        ));
                    }
                    // 6. Protozoid Egg & Slimer (EGG, GREENSLIME)
                    EGG => {
                        entity_cmds.insert((
                            crate::combat::EnemyActor::new_egg(),
                            crate::scripting::ConActor::new(EGG, sprite.sectnum, sprite.ang, 20),
                            crate::combat::SituationalSpawn {
                                initial_picnum: EGG,
                                is_dormant: true,
                            },
                        ));
                    }
                    GREENSLIME => {
                        entity_cmds.insert((
                            crate::combat::EnemyActor::new_slimer(),
                            crate::scripting::ConActor::new(
                                GREENSLIME,
                                sprite.sectnum,
                                sprite.ang,
                                1,
                            ),
                        ));
                    }
                    // 7. Enforcer (LIZMAN..=LIZMANJUMP)
                    LIZMAN..=LIZMANJUMP => {
                        entity_cmds.insert((
                            crate::combat::EnemyActor::new_enforcer(),
                            crate::scripting::ConActor::new(
                                LIZMAN,
                                sprite.sectnum,
                                sprite.ang,
                                120,
                            ),
                            crate::combat::SituationalSpawn {
                                initial_picnum: sprite.picnum,
                                is_dormant: sprite.picnum == LIZMANSTAYPUT,
                            },
                        ));
                    }
                    // 8. Assault Commander (COMMANDER, COMMANDERSTAYPUT)
                    COMMANDER | COMMANDERSTAYPUT => {
                        entity_cmds.insert((
                            crate::combat::EnemyActor::new_commander(),
                            crate::scripting::ConActor::new(
                                COMMANDER,
                                sprite.sectnum,
                                sprite.ang,
                                350,
                            ),
                            crate::combat::SituationalSpawn {
                                initial_picnum: sprite.picnum,
                                is_dormant: sprite.picnum == COMMANDERSTAYPUT,
                            },
                            crate::combat::FlyingActor::default(),
                        ));
                    }
                    // 9. Sentry Drone (DRONE)
                    DRONE => {
                        entity_cmds.insert((
                            crate::combat::EnemyActor::new_drone(),
                            crate::scripting::ConActor::new(DRONE, sprite.sectnum, sprite.ang, 150),
                            crate::combat::FlyingActor::default(),
                        ));
                    }
                    // 10. Shark (SHARK)
                    SHARK => {
                        entity_cmds.insert((
                            crate::combat::EnemyActor::new_shark(),
                            crate::scripting::ConActor::new(SHARK, sprite.sectnum, sprite.ang, 35),
                            crate::combat::FlyingActor::default(),
                        ));
                    }
                    // 11. Protector Drone (NEWBEAST, NEWBEASTSTAYPUT, NEWBEASTHANG, NEWBEASTJUMP)
                    NEWBEAST | NEWBEASTSTAYPUT | NEWBEASTHANG | NEWBEASTJUMP => {
                        entity_cmds.insert((
                            crate::combat::EnemyActor::new_protector_drone(),
                            crate::scripting::ConActor::new(
                                NEWBEAST,
                                sprite.sectnum,
                                sprite.ang,
                                300,
                            ),
                            crate::combat::SituationalSpawn {
                                initial_picnum: sprite.picnum,
                                is_dormant: sprite.picnum == NEWBEASTHANG
                                    || sprite.picnum == NEWBEASTSTAYPUT,
                            },
                        ));
                    }
                    // 12. Turret (ROTATEGUN)
                    ROTATEGUN => {
                        entity_cmds.insert((
                            crate::combat::EnemyActor::new_turret(),
                            crate::scripting::ConActor::new(
                                ROTATEGUN,
                                sprite.sectnum,
                                sprite.ang,
                                40,
                            ),
                        ));
                    }
                    // 13. Scampering Rat (RAT)
                    RAT => {
                        entity_cmds.insert((
                            crate::combat::EnemyActor::new_rat(),
                            crate::scripting::ConActor::new(RAT, sprite.sectnum, sprite.ang, 5),
                        ));
                    }
                    // 14. Toxic Slime Hazard (OOZ, OOZ2) & Fire Hazards
                    OOZ | OOZ2 | BURNING | FIRE | BURNING2 | FIRE2 | FLOORFLAME => {
                        let dmg_rate = if matches!(sprite.picnum, BURNING | FIRE | BURNING2 | FIRE2 | FLOORFLAME) {
                            30.0
                        } else {
                            20.0
                        };
                        entity_cmds.insert((
                            crate::combat::EnemyActor::new_slime_hazard(),
                            crate::player::types::HazardSector {
                                damage_per_sec: dmg_rate,
                            },
                        ));
                    }
                    // 13. Boss 1: Battlelord & Mini-Battlelord (BOSS1, BOSS1STAYPUT)
                    BOSS1 | BOSS1STAYPUT => {
                        let is_mini = sprite.pal == 21;
                        let (enemy, actor_hp) = if is_mini {
                            (crate::combat::EnemyActor::new_battlelord(true), 1000)
                        } else {
                            (crate::combat::EnemyActor::new_battlelord(false), 4500)
                        };
                        entity_cmds.insert((
                            enemy,
                            crate::scripting::ConActor::new(
                                BOSS1,
                                sprite.sectnum,
                                sprite.ang,
                                actor_hp,
                            ),
                            crate::combat::SituationalSpawn {
                                initial_picnum: sprite.picnum,
                                is_dormant: sprite.picnum == BOSS1STAYPUT,
                            },
                        ));
                    }
                    // 14. Boss 2: Overlord (BOSS2)
                    BOSS2 => {
                        entity_cmds.insert((
                            crate::combat::EnemyActor::new_overlord(),
                            crate::scripting::ConActor::new(
                                BOSS2,
                                sprite.sectnum,
                                sprite.ang,
                                4500,
                            ),
                        ));
                    }
                    // 15. Boss 3: Cycloid Emperor (BOSS3)
                    BOSS3 => {
                        entity_cmds.insert((
                            crate::combat::EnemyActor::new_cycloid(),
                            crate::scripting::ConActor::new(
                                BOSS3,
                                sprite.sectnum,
                                sprite.ang,
                                4500,
                            ),
                        ));
                    }
                    // 16. Boss 4: Alien Queen (BOSS4, BOSS4STAYPUT)
                    BOSS4 | BOSS4STAYPUT => {
                        entity_cmds.insert((
                            crate::combat::EnemyActor::new_queen(),
                            crate::scripting::ConActor::new(
                                BOSS4,
                                sprite.sectnum,
                                sprite.ang,
                                6000,
                            ),
                        ));
                    }
                    // NUKE BUTTON (Level Exit)
                    NUKEBUTTON..=145 => {
                        entity_cmds.insert(crate::interactivity::NukeExitSwitch {
                            is_activated: false,
                            is_secret: sprite.lotag != 0,
                            lotag: sprite.lotag,
                            hitag: sprite.hitag,
                        });
                    }
                    _ => {}
                }

                // Check for tile animation on sprite (only for 2D billboard sprites)
                if !is_voxel {
                    if let Some(&picanm) = self.picanm_map.get(&sprite.picnum) {
                        if picanm.num_frames > 0 && picanm.anim_type > 0 {
                            entity_cmds.insert(AnimatedTileMaterial {
                                base_picnum: sprite.picnum,
                                picanm,
                                current_offset: 0,
                                material_handle: mat_handle.clone(),
                            });
                        }
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
    use lyon_tessellation::{
        BuffersBuilder, FillOptions, FillTessellator, FillVertex, VertexBuffers,
    };

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
                            current_loop
                                .push(Vec2::new(wall.x as f32 / 1024.0, wall.y as f32 / 1024.0));
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

                    let lyon_ok = tessellator
                        .tessellate_path(
                            &path,
                            &FillOptions::default(),
                            &mut BuffersBuilder::new(&mut buffers, |vertex: FillVertex| {
                                [vertex.position().x, vertex.position().y]
                            }),
                        )
                        .is_ok();

                    if !lyon_ok || buffers.indices.is_empty() {
                        buffers.vertices.clear();
                        buffers.indices.clear();
                        for poly in &loops {
                            if poly.len() < 3 {
                                continue;
                            }
                            let base_idx = buffers.vertices.len() as u32;
                            for p in poly {
                                buffers.vertices.push([p.x, p.y]);
                            }
                            let indices = triangulate_polygon_ear_clipping(poly, base_idx);
                            buffers.indices.extend(indices);
                        }
                    }

                    if buffers.indices.is_empty() {
                        println!(
                            "Sector {} FAILED! wallptr={}, wallnum={}, loops count={}",
                            sec_idx,
                            sector.wallptr,
                            sector.wallnum,
                            loops.len()
                        );
                        failed_sectors += 1;
                    }
                }

                println!(
                    "Tessellation result: {}/{} succeeded, {} failed",
                    total_sectors - failed_sectors,
                    total_sectors,
                    failed_sectors
                );
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

    #[test]
    fn test_editor_marker_sprites_hidden() {
        let marker_picnums = [
            SECTOREFFECTOR,
            ACTIVATOR,
            TOUCHPLATE,
            ACTIVATORLOCKED,
            MUSICANDSFX,
            LOCATORS,
            CYCLER,
            MASTERSWITCH,
        ];
        for &picnum in &marker_picnums {
            assert!(
                (1..=8).contains(&picnum),
                "Sprite {picnum} must be in editor utility range 1..=8"
            );
        }
    }

    #[test]
    fn test_water_sector_detection_and_surface_classification() {
        assert!(is_water_sector(1), "Lotag 1 must be water sector");
        assert!(is_water_sector(2), "Lotag 2 must be water sector");
        assert!(!is_water_sector(0), "Lotag 0 is not water sector");
        assert!(!is_water_sector(15), "Lotag 15 is not water sector");

        // Surface plane tests
        assert!(
            is_water_surface(1, crate::interactivity::SectorMeshPart::Floor),
            "Lotag 1 Floor must be a water surface"
        );
        assert!(
            !is_water_surface(1, crate::interactivity::SectorMeshPart::Ceiling),
            "Lotag 1 Ceiling is the sky/ceiling, not water surface"
        );
        assert!(
            is_water_surface(2, crate::interactivity::SectorMeshPart::Ceiling),
            "Lotag 2 Ceiling must be underwater surface looking up"
        );
        assert!(
            !is_water_surface(2, crate::interactivity::SectorMeshPart::Floor),
            "Lotag 2 Floor is the pool floor/bed, not water surface"
        );
    }

    #[test]
    fn test_water_surface_mesh_generation_and_properties() {
        let mut app = App::new();

        let make_wall = |x: i32, y: i32, point2: i16| Wall {
            x,
            y,
            point2,
            nextwall: -1,
            nextsector: -1,
            cstat: 0,
            picnum: 336, // W_WATER
            overpicnum: 0,
            shade: 0,
            pal: 0,
            xrepeat: 8,
            yrepeat: 8,
            xpanning: 0,
            ypanning: 0,
            lotag: 0,
            hitag: 0,
            extra: 0,
        };

        // Sector 0: Above water pool (lotag 1)
        // Floor at Z=0 (Y=0.0) -> water surface!
        // Ceiling at Z=-16384 (Y=1.0)
        let sec0 = crate::map::Sector {
            wallptr: 0,
            wallnum: 4,
            ceilingz: -16384,
            floorz: 0,
            ceilingstat: 0,
            floorstat: 0,
            ceilingpicnum: 100,
            ceilingheinum: 0,
            ceilingshade: 0,
            ceilingpal: 0,
            ceilingxpanning: 0,
            ceilingypanning: 0,
            floorpicnum: 336,
            floorheinum: 0,
            floorshade: 0,
            floorpal: 0,
            floorxpanning: 0,
            floorypanning: 0,
            visibility: 0,
            _filler: 0,
            lotag: 1,
            hitag: 0,
            extra: -1,
        };

        // Sector 1: Underwater basin (lotag 2)
        // Floor at Z=16384 (Y=-1.0) -> pool bed
        // Ceiling at Z=0 (Y=0.0) -> water surface!
        let sec1 = crate::map::Sector {
            wallptr: 4,
            wallnum: 4,
            ceilingz: 0,
            floorz: 16384,
            ceilingstat: 0,
            floorstat: 0,
            ceilingpicnum: 336,
            ceilingheinum: 0,
            ceilingshade: 0,
            ceilingpal: 0,
            ceilingxpanning: 0,
            ceilingypanning: 0,
            floorpicnum: 100,
            floorheinum: 0,
            floorshade: 0,
            floorpal: 0,
            floorxpanning: 0,
            floorypanning: 0,
            visibility: 0,
            _filler: 0,
            lotag: 2,
            hitag: 0,
            extra: -1,
        };

        let walls = vec![
            // Sector 0 loop
            make_wall(0, 0, 1),
            make_wall(1024, 0, 2),
            make_wall(1024, 1024, 3),
            make_wall(0, 1024, 0),
            // Sector 1 loop
            make_wall(0, 0, 5),
            make_wall(1024, 0, 6),
            make_wall(1024, 1024, 7),
            make_wall(0, 1024, 4),
        ];

        let map = Map {
            version: 7,
            posx: 0,
            posy: 0,
            posz: 0,
            ang: 0,
            cursectnum: 0,
            sectors: vec![sec0, sec1],
            walls,
            sprites: Vec::new(),
        };

        let mut tile_textures = HashMap::new();
        let mut images = Assets::<Image>::default();
        let img = images.add(Image::default());
        tile_textures.insert(336, img.clone());
        tile_textures.insert(100, img);
        let mut tile_sizes = HashMap::new();
        tile_sizes.insert(336, (64, 64));
        tile_sizes.insert(100, (64, 64));
        let picanm_map = HashMap::new();

        let mut materials: Assets<StandardMaterial> = Assets::default();
        let default_material = materials.add(StandardMaterial::default());
        let mut meshes: Assets<Mesh> = Assets::default();

        let builder = MapMeshBuilder::new(
            &map,
            &tile_textures,
            &tile_sizes,
            &picanm_map,
            default_material,
        );

        let mut commands = app.world_mut().commands();
        builder.build(
            &mut commands,
            &mut meshes,
            &mut materials,
            1,
            None,
            None,
        );
        app.update();

        // Query WaterSurface entities
        let mut water_query = app
            .world_mut()
            .query::<(
                &crate::interactivity::WaterSurface,
                &crate::interactivity::DynamicSectorMesh,
                &Handle<StandardMaterial>,
                &Handle<Mesh>,
                &Sensor,
            )>();

        let mut water_surfaces: Vec<(crate::interactivity::WaterSurface, crate::interactivity::DynamicSectorMesh)> = Vec::new();
        for (ws, dsm, mat_h, mesh_h, _sensor) in water_query.iter(app.world()) {
            water_surfaces.push((*ws, dsm.clone()));

            let mat = materials.get(mat_h).expect("Material must exist");
            assert_eq!(mat.alpha_mode, AlphaMode::Blend);
            assert!(mat.double_sided, "Water surface material must be two-sided");
            assert!(
                (mat.base_color.to_srgba().alpha - 0.60).abs() < 0.01,
                "Water material alpha must be 60%"
            );

            let mesh = meshes.get(mesh_h).expect("Mesh must exist");
            let pos_attr = mesh.attribute(Mesh::ATTRIBUTE_POSITION).expect("Mesh positions must exist");
            let uv_attr = mesh.attribute(Mesh::ATTRIBUTE_UV_0).expect("Mesh UVs must exist");
            let normal_attr = mesh.attribute(Mesh::ATTRIBUTE_NORMAL).expect("Mesh normals must exist");
            let color_attr = mesh.attribute(Mesh::ATTRIBUTE_COLOR).expect("Mesh colors must exist");

            assert_eq!(pos_attr.len(), uv_attr.len());
            assert_eq!(pos_attr.len(), normal_attr.len());
            assert_eq!(pos_attr.len(), color_attr.len());
        }

        assert_eq!(water_surfaces.len(), 2, "Must spawn exactly 2 water surface entities (1 per water sector)");
        
        let sec0_water = water_surfaces.iter().find(|(ws, _)| ws.sector_index == 0).expect("Sector 0 water surface");
        assert_eq!(sec0_water.1.part, crate::interactivity::SectorMeshPart::Floor);
        assert!((sec0_water.0.elevation - 0.0).abs() < 0.001);

        let sec1_water = water_surfaces.iter().find(|(ws, _)| ws.sector_index == 1).expect("Sector 1 water surface");
        assert_eq!(sec1_water.1.part, crate::interactivity::SectorMeshPart::Ceiling);
        assert!((sec1_water.0.elevation - 0.0).abs() < 0.001);
    }

    #[test]
    fn test_material_alpha_mode_and_translucency_flags() {
        assert_eq!(MaterialAlphaMode::Opaque, MaterialAlphaMode::Opaque);
        assert_ne!(MaterialAlphaMode::Opaque, MaterialAlphaMode::Mask);
        assert_eq!(MaterialAlphaMode::Blend(66), MaterialAlphaMode::Blend(66));

        let eval_wall_alpha = |cstat: i16| -> MaterialAlphaMode {
            let wall = crate::map::Wall {
                x: 0,
                y: 0,
                point2: 1,
                nextwall: -1,
                nextsector: -1,
                cstat,
                picnum: 100,
                overpicnum: 0,
                shade: 0,
                pal: 0,
                xrepeat: 8,
                yrepeat: 8,
                xpanning: 0,
                ypanning: 0,
                lotag: 0,
                hitag: 0,
                extra: 0,
            };
            wall.alpha_mode(true)
        };

        assert_eq!(eval_wall_alpha(0), MaterialAlphaMode::Mask);
        assert_eq!(eval_wall_alpha(128), MaterialAlphaMode::Blend(33));
        assert_eq!(eval_wall_alpha(512), MaterialAlphaMode::Blend(66));
    }

    #[test]
    fn test_builder_translucent_material_generation() {
        let mut materials = Assets::<StandardMaterial>::default();
        let default_mat = materials.add(StandardMaterial::default());
        let map = crate::map::Map {
            version: 7,
            posx: 0,
            posy: 0,
            posz: 0,
            ang: 0,
            cursectnum: 0,
            sectors: Vec::new(),
            walls: Vec::new(),
            sprites: Vec::new(),
        };
        let mut tile_textures = std::collections::HashMap::new();
        let mut images = Assets::<Image>::default();
        let img_handle = images.add(Image::default());
        tile_textures.insert(100, img_handle);
        let tile_sizes = std::collections::HashMap::new();
        let picanm_map = std::collections::HashMap::new();

        let builder = MapMeshBuilder::new(
            &map,
            &tile_textures,
            &tile_sizes,
            &picanm_map,
            default_mat,
        );

        let blend_33_handle = builder.get_material(100, MaterialAlphaMode::Blend(33), &mut materials);
        let mat_33 = materials.get(&blend_33_handle).expect("Material 33 should exist");
        assert_eq!(mat_33.alpha_mode, AlphaMode::Blend);
        assert!((mat_33.base_color.to_srgba().alpha - 0.33).abs() < 0.01);
        assert!(mat_33.double_sided);

        let blend_66_handle = builder.get_material(100, MaterialAlphaMode::Blend(66), &mut materials);
        let mat_66 = materials.get(&blend_66_handle).expect("Material 66 should exist");
        assert_eq!(mat_66.alpha_mode, AlphaMode::Blend);
        assert!((mat_66.base_color.to_srgba().alpha - 0.66).abs() < 0.01);

        let mask_handle = builder.get_material(100, MaterialAlphaMode::Mask, &mut materials);
        let mat_mask = materials.get(&mask_handle).expect("Material mask should exist");
        assert_eq!(mat_mask.alpha_mode, AlphaMode::Mask(0.5));
    }

    #[test]
    fn test_voxel_sprite_spawning_replaces_billboard() {
        let mut app = App::new();
        let map = crate::map::Map {
            version: 7,
            posx: 0,
            posy: 0,
            posz: 0,
            ang: 0,
            cursectnum: 0,
            sectors: Vec::new(),
            walls: Vec::new(),
            sprites: vec![
                crate::map::Sprite {
                    x: 1024,
                    y: 1024,
                    z: -16384,
                    cstat: 0, // Face sprite
                    shade: 0,
                    pal: 0,
                    clipdist: 32,
                    _filler: 0,
                    xrepeat: 64,
                    yrepeat: 64,
                    xoffset: 0,
                    yoffset: 0,
                    picnum: FIRSTAID,
                    ang: 0,
                    xvel: 0,
                    yvel: 0,
                    zvel: 0,
                    owner: 0,
                    sectnum: 0,
                    statnum: 0,
                    lotag: 0,
                    hitag: 0,
                    extra: -1,
                },
            ],
        };

        let tile_textures = HashMap::new();
        let tile_sizes = HashMap::new();
        let picanm_map = HashMap::new();
        let mut materials: Assets<StandardMaterial> = Assets::default();
        let default_material = materials.add(StandardMaterial::default());
        let mut meshes: Assets<Mesh> = Assets::default();

        let builder = MapMeshBuilder::new(
            &map,
            &tile_textures,
            &tile_sizes,
            &picanm_map,
            default_material,
        );

        let mut voxel_reg = crate::voxel::VoxelRegistry::new();
        let pal = crate::palette::Palette::default();

        let mut commands = app.world_mut().commands();
        builder.build(
            &mut commands,
            &mut meshes,
            &mut materials,
            1,
            Some(&mut voxel_reg),
            Some(&pal),
        );
        app.update();

        let mut voxel_query = app.world_mut().query::<(&crate::voxel::VoxelModelInstance, Option<&crate::SpriteBillboard>)>();
        let mut found = false;
        for (instance, billboard) in voxel_query.iter(app.world()) {
            assert_eq!(instance.picnum, FIRSTAID);
            assert!(billboard.is_none(), "Voxel pickups must not have SpriteBillboard");
            found = true;
        }
        assert!(found, "Expected voxel model instance to be spawned for FIRSTAID");
    }

    #[test]
    fn test_environmental_prop_voxel_spawning_and_alignment() {
        let mut app = App::new();
        let test_sector = crate::map::Sector {
            wallptr: 0,
            wallnum: 0,
            ceilingz: -16384, // World Y = -(-16384) / 16384 = 1.0
            floorz: 0,       // World Y = 0.0
            ceilingstat: 0,
            floorstat: 0,
            ceilingpicnum: 0,
            ceilingheinum: 0,
            ceilingshade: 0,
            ceilingpal: 0,
            ceilingxpanning: 0,
            ceilingypanning: 0,
            floorpicnum: 0,
            floorheinum: 0,
            floorshade: 0,
            floorpal: 0,
            floorxpanning: 0,
            floorypanning: 0,
            visibility: 0,
            _filler: 0,
            lotag: 0,
            hitag: 0,
            extra: -1,
        };

        let map = crate::map::Map {
            version: 7,
            posx: 0,
            posy: 0,
            posz: 0,
            ang: 0,
            cursectnum: 0,
            sectors: vec![test_sector],
            walls: Vec::new(),
            sprites: vec![
                // 1. Standing explosive barrel on floor
                crate::map::Sprite {
                    x: 1024,
                    y: 1024,
                    z: 0,
                    cstat: 1, // Blocking floor prop
                    shade: 0,
                    pal: 0,
                    clipdist: 32,
                    _filler: 0,
                    xrepeat: 64,
                    yrepeat: 64,
                    xoffset: 0,
                    yoffset: 0,
                    picnum: EXPLODINGBARREL,
                    ang: 0,
                    xvel: 0,
                    yvel: 0,
                    zvel: 0,
                    owner: 0,
                    sectnum: 0,
                    statnum: 0,
                    lotag: 0,
                    hitag: 0,
                    extra: -1,
                },
                // 2. Wall-mounted fire extinguisher
                crate::map::Sprite {
                    x: 2048,
                    y: 2048,
                    z: -8192,
                    cstat: 16, // Wall-aligned
                    shade: 0,
                    pal: 0,
                    clipdist: 32,
                    _filler: 0,
                    xrepeat: 64,
                    yrepeat: 64,
                    xoffset: 0,
                    yoffset: 0,
                    picnum: FIREEXT,
                    ang: 512, // Facing South (+Z in Bevy)
                    xvel: 0,
                    yvel: 0,
                    zvel: 0,
                    owner: 0,
                    sectnum: 0,
                    statnum: 0,
                    lotag: 0,
                    hitag: 0,
                    extra: -1,
                },
                // 3. Wall-mounted security camera
                crate::map::Sprite {
                    x: 3072,
                    y: 3072,
                    z: -12288,
                    cstat: 16,
                    shade: 0,
                    pal: 0,
                    clipdist: 32,
                    _filler: 0,
                    xrepeat: 64,
                    yrepeat: 64,
                    xoffset: 0,
                    yoffset: 0,
                    picnum: CAMERA1,
                    ang: 512,
                    xvel: 0,
                    yvel: 0,
                    zvel: 0,
                    owner: 0,
                    sectnum: 0,
                    statnum: 0,
                    lotag: 0,
                    hitag: 42,
                    extra: -1,
                },
                // 4. Ceiling-anchored fan
                crate::map::Sprite {
                    x: 4096,
                    y: 4096,
                    z: 0, // Sprite z is 0, but ceiling fan should anchor to sector ceiling (-16384)
                    cstat: 32,
                    shade: 0,
                    pal: 0,
                    clipdist: 32,
                    _filler: 0,
                    xrepeat: 64,
                    yrepeat: 64,
                    xoffset: 0,
                    yoffset: 0,
                    picnum: 617,
                    ang: 0,
                    xvel: 0,
                    yvel: 0,
                    zvel: 0,
                    owner: 0,
                    sectnum: 0,
                    statnum: 0,
                    lotag: 0,
                    hitag: 0,
                    extra: -1,
                },
            ],
        };

        let tile_textures = HashMap::new();
        let tile_sizes = HashMap::new();
        let picanm_map = HashMap::new();
        let mut materials: Assets<StandardMaterial> = Assets::default();
        let default_material = materials.add(StandardMaterial::default());
        let mut meshes: Assets<Mesh> = Assets::default();

        let builder = MapMeshBuilder::new(
            &map,
            &tile_textures,
            &tile_sizes,
            &picanm_map,
            default_material,
        );

        let mut voxel_reg = crate::voxel::VoxelRegistry::new();
        let pal = crate::palette::Palette::default();

        let mut commands = app.world_mut().commands();
        builder.build(
            &mut commands,
            &mut meshes,
            &mut materials,
            1,
            Some(&mut voxel_reg),
            Some(&pal),
        );
        app.update();

        // 1. Verify Barrel: floor grounded at Y=0.0, non-bobbing/non-rotating prop
        let mut barrel_query = app
            .world_mut()
            .query::<(&crate::voxel::VoxelModelInstance, &Transform, &crate::interactivity::ExplodingBarrel)>();
        let mut found_barrel = false;
        for (inst, transform, barrel) in barrel_query.iter(app.world()) {
            assert_eq!(inst.picnum, EXPLODINGBARREL);
            assert!(!inst.rotates, "Barrels must not rotate like pickups");
            assert!(!inst.bobs, "Barrels must not bob like pickups");
            assert!(
                (transform.translation.y - 0.0).abs() < 0.001,
                "Barrel must be grounded flush on floor at Y=0.0, got {}",
                transform.translation.y
            );
            assert_eq!(barrel.health, 20);
            found_barrel = true;
        }
        assert!(found_barrel, "Exploding barrel voxel prop was not found");

        // 2. Verify Fire Extinguisher: wall mounted with standoff offset
        let mut fireext_query = app
            .world_mut()
            .query::<(&crate::voxel::VoxelModelInstance, &Transform, &crate::interactivity::FireExtinguisher)>();
        let mut found_fireext = false;
        for (inst, transform, ext) in fireext_query.iter(app.world()) {
            assert_eq!(inst.picnum, FIREEXT);
            assert!(!inst.rotates);
            assert!(!ext.is_exploded);
            // Ang 512 is facing South (+Z in Bevy). World pos was (2.0, 0.5, 2.0).
            // Normal standoff should push translation.z beyond 2.0 (by +0.05).
            assert!(
                transform.translation.z > 2.04,
                "Wall standoff offset must shift prop outward along normal: z={}",
                transform.translation.z
            );
            found_fireext = true;
        }
        assert!(found_fireext, "Fire extinguisher voxel prop was not found");

        // 3. Verify Security Camera: has SecurityCameraVoxel with sweep config
        let mut camera_query = app
            .world_mut()
            .query::<(&crate::voxel::VoxelModelInstance, &crate::interactivity::SecurityCamera, &crate::voxel::SecurityCameraVoxel)>();
        let mut found_camera = false;
        for (inst, sec_cam, vox_cam) in camera_query.iter(app.world()) {
            assert_eq!(inst.picnum, CAMERA1);
            assert_eq!(sec_cam.tag, 42);
            assert!(vox_cam.sweep_range > 0.0);
            assert!(vox_cam.sweep_speed > 0.0);
            found_camera = true;
        }
        assert!(found_camera, "Security camera voxel prop was not found");

        // 4. Verify Ceiling Fan: anchored to sector ceiling elevation Y=1.0 and has CeilingFanVoxel
        let mut fan_query = app
            .world_mut()
            .query::<(&crate::voxel::VoxelModelInstance, &Transform, &crate::voxel::CeilingFanVoxel)>();
        let mut found_fan = false;
        for (inst, transform, fan) in fan_query.iter(app.world()) {
            assert_eq!(inst.picnum, 617);
            assert!(fan.speed > 0.0);
            assert!(
                (transform.translation.y - 1.0).abs() < 0.001,
                "Ceiling fan must anchor to sector ceiling at Y=1.0, got {}",
                transform.translation.y
            );
            found_fan = true;
        }
        assert!(found_fan, "Ceiling fan voxel prop was not found");
    }
}
