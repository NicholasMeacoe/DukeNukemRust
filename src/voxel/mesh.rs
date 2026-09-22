use bevy::prelude::*;
use bevy::render::mesh::{Indices, PrimitiveTopology};
use bevy::render::render_asset::RenderAssetUsages;
use crate::palette::Palette;
use super::kvx::KvxModel;

/// Standard scale factor mapping 1 voxel unit to world meters.
pub const DEFAULT_VOXEL_SCALE: f32 = 0.035;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum VoxelFace {
    PosX, // Right
    NegX, // Left
    PosY, // Top (Up)
    NegY, // Bottom (Down)
    PosZ, // Front
    NegZ, // Back
}

impl VoxelFace {
    pub const ALL: [VoxelFace; 6] = [
        VoxelFace::PosX,
        VoxelFace::NegX,
        VoxelFace::PosY,
        VoxelFace::NegY,
        VoxelFace::PosZ,
        VoxelFace::NegZ,
    ];

    pub fn normal(&self) -> [f32; 3] {
        match self {
            VoxelFace::PosX => [1.0, 0.0, 0.0],
            VoxelFace::NegX => [-1.0, 0.0, 0.0],
            VoxelFace::PosY => [0.0, 1.0, 0.0],
            VoxelFace::NegY => [0.0, -1.0, 0.0],
            VoxelFace::PosZ => [0.0, 0.0, 1.0],
            VoxelFace::NegZ => [0.0, 0.0, -1.0],
        }
    }
}

/// Generates a surface-culled, palette-shaded Bevy 3D `Mesh` from a `KvxModel`.
pub fn generate_voxel_mesh(
    model: &KvxModel,
    palette: &Palette,
    scale: f32,
) -> Mesh {
    let mut positions: Vec<[f32; 3]> = Vec::new();
    let mut normals: Vec<[f32; 3]> = Vec::new();
    let mut uvs: Vec<[f32; 2]> = Vec::new();
    let mut colors: Vec<[f32; 4]> = Vec::new();
    let mut indices: Vec<u32> = Vec::new();

    let s = scale;

    for x in 0..model.xsiz {
        for y in 0..model.ysiz {
            for z in 0..model.zsiz {
                let Some(color_idx) = model.get_voxel(x, y, z) else {
                    continue;
                };

                let pal_color = palette.colors[color_idx as usize];
                let vertex_color = [
                    pal_color[0] as f32 / 255.0,
                    pal_color[1] as f32 / 255.0,
                    pal_color[2] as f32 / 255.0,
                    1.0,
                ];

                // World position relative to pivot
                // Map KVX (x, y, z) -> Bevy 3D (X right, Y up, Z forward)
                // In KVX: x=width, y=depth, z=height (0 is top in KVX, so invert Z for standard upright Y)
                let bx = (x as f32 - model.pivot.x) * s;
                let by = (model.zsiz as f32 - 1.0 - z as f32 - model.pivot.z) * s;
                let bz = (y as f32 - model.pivot.y) * s;

                for face in VoxelFace::ALL {
                    // Check if adjacent neighbor is solid (cull interior face)
                    let is_exposed = match face {
                        VoxelFace::PosX => x + 1 >= model.xsiz || model.get_voxel(x + 1, y, z).is_none(),
                        VoxelFace::NegX => x == 0 || model.get_voxel(x - 1, y, z).is_none(),
                        VoxelFace::PosY => z == 0 || model.get_voxel(x, y, z - 1).is_none(), // -z in kvx is +y in bevy
                        VoxelFace::NegY => z + 1 >= model.zsiz || model.get_voxel(x, y, z + 1).is_none(),
                        VoxelFace::PosZ => y + 1 >= model.ysiz || model.get_voxel(x, y + 1, z).is_none(),
                        VoxelFace::NegZ => y == 0 || model.get_voxel(x, y - 1, z).is_none(),
                    };

                    if !is_exposed {
                        continue;
                    }

                    let norm = face.normal();
                    let start_idx = positions.len() as u32;

                    // Quad vertices: (bx, by, bz) to (bx+s, by+s, bz+s)
                    let quad_corners = match face {
                        VoxelFace::PosX => [
                            [bx + s, by, bz + s],
                            [bx + s, by, bz],
                            [bx + s, by + s, bz],
                            [bx + s, by + s, bz + s],
                        ],
                        VoxelFace::NegX => [
                            [bx, by, bz],
                            [bx, by, bz + s],
                            [bx, by + s, bz + s],
                            [bx, by + s, bz],
                        ],
                        VoxelFace::PosY => [
                            [bx, by + s, bz],
                            [bx, by + s, bz + s],
                            [bx + s, by + s, bz + s],
                            [bx + s, by + s, bz],
                        ],
                        VoxelFace::NegY => [
                            [bx, by, bz + s],
                            [bx, by, bz],
                            [bx + s, by, bz],
                            [bx + s, by, bz + s],
                        ],
                        VoxelFace::PosZ => [
                            [bx, by, bz + s],
                            [bx + s, by, bz + s],
                            [bx + s, by + s, bz + s],
                            [bx, by + s, bz + s],
                        ],
                        VoxelFace::NegZ => [
                            [bx + s, by, bz],
                            [bx, by, bz],
                            [bx, by + s, bz],
                            [bx + s, by + s, bz],
                        ],
                    };

                    for pos in quad_corners {
                        positions.push(pos);
                        normals.push(norm);
                        colors.push(vertex_color);
                    }

                    uvs.push([0.0, 1.0]);
                    uvs.push([1.0, 1.0]);
                    uvs.push([1.0, 0.0]);
                    uvs.push([0.0, 0.0]);

                    indices.extend_from_slice(&[
                        start_idx,
                        start_idx + 1,
                        start_idx + 2,
                        start_idx,
                        start_idx + 2,
                        start_idx + 3,
                    ]);
                }
            }
        }
    }

    let mut mesh = Mesh::new(
        PrimitiveTopology::TriangleList,
        RenderAssetUsages::default(),
    );
    mesh.insert_attribute(Mesh::ATTRIBUTE_POSITION, positions);
    mesh.insert_attribute(Mesh::ATTRIBUTE_NORMAL, normals);
    mesh.insert_attribute(Mesh::ATTRIBUTE_UV_0, uvs);
    mesh.insert_attribute(Mesh::ATTRIBUTE_COLOR, colors);
    mesh.insert_indices(Indices::U32(indices));
    mesh
}

#[cfg(test)]
pub mod tests {
    use super::*;

    fn dummy_palette() -> Palette {
        let mut colors = [[0u8; 4]; 256];
        colors[10] = [255, 0, 0, 255]; // Red
        colors[20] = [0, 255, 0, 255]; // Green
        colors[30] = [0, 0, 255, 255]; // Blue
        Palette {
            colors,
            num_shades: 32,
            shade_tables: Vec::new(),
            lookups: std::collections::HashMap::new(),
            water_palette: None,
            slime_palette: None,
            title_palette: None,
        }
    }

    #[test]
    fn test_voxel_single_cube_mesh_generation() {
        let mut model = KvxModel::new(1, 1, 1, Vec3::ZERO);
        model.set_voxel(0, 0, 0, Some(10));
        let pal = dummy_palette();

        let mesh = generate_voxel_mesh(&model, &pal, 1.0);

        // A single voxel has 6 exposed faces = 24 vertices, 36 indices (12 triangles)
        let pos_attr = mesh.attribute(Mesh::ATTRIBUTE_POSITION).unwrap();
        assert_eq!(pos_attr.len(), 24);

        let col_attr = mesh.attribute(Mesh::ATTRIBUTE_COLOR).unwrap();
        assert_eq!(col_attr.len(), 24);
        if let bevy::render::mesh::VertexAttributeValues::Float32x4(cols) = col_attr {
            for c in cols {
                assert_eq!(*c, [1.0, 0.0, 0.0, 1.0]);
            }
        }

        let indices = mesh.indices().unwrap();
        assert_eq!(indices.len(), 36);
    }

    #[test]
    fn test_voxel_interior_face_culling() {
        // Create 2x2x2 cube (8 voxels)
        let model = KvxModel::create_test_box(2, 2, 2, 20);
        let pal = dummy_palette();

        let mesh = generate_voxel_mesh(&model, &pal, 1.0);

        // 2x2x2 cube has 6 sides * 4 quads = 24 exposed face quads
        // 24 quads * 4 vertices = 96 vertices.
        // If interior faces were not culled, it would be 8 voxels * 6 faces * 4 = 192 vertices!
        let pos_attr = mesh.attribute(Mesh::ATTRIBUTE_POSITION).unwrap();
        assert_eq!(pos_attr.len(), 96);

        let indices = mesh.indices().unwrap();
        assert_eq!(indices.len(), 24 * 6); // 144 indices (48 triangles)
    }

    #[test]
    fn test_voxel_pivot_translation_offset() {
        let mut model = KvxModel::new(1, 1, 1, Vec3::new(0.5, 0.5, 0.5));
        model.set_voxel(0, 0, 0, Some(30));
        let pal = dummy_palette();

        let mesh = generate_voxel_mesh(&model, &pal, 1.0);
        if let bevy::render::mesh::VertexAttributeValues::Float32x3(positions) =
            mesh.attribute(Mesh::ATTRIBUTE_POSITION).unwrap()
        {
            // Since pivot is (0.5, 0.5, 0.5), coordinates should range from -0.5 to +0.5
            let mut min_x = f32::INFINITY;
            let mut max_x = f32::NEG_INFINITY;
            for p in positions {
                min_x = min_x.min(p[0]);
                max_x = max_x.max(p[0]);
            }
            assert!((min_x - (-0.5)).abs() < 0.001);
            assert!((max_x - 0.5).abs() < 0.001);
        } else {
            panic!("Expected Float32x3 positions");
        }
    }
}
