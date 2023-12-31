use bevy::prelude::*;
use bytemuck::{Pod, Zeroable};

use crate::block::{texture::SheetId, BlockId, Blocks};

use self::mesher::Mesher;

pub mod mesher;

#[derive(Component)]
pub struct IsChunkMesh(pub SheetId);

#[derive(Component)]
pub struct Chunk {
    pub data: ChunkData,
}

#[derive(Debug, Copy, Clone, Eq, PartialEq, Hash, Ord, PartialOrd, Pod, Zeroable)]
#[repr(C)]
pub struct ChunkData {
    pub blocks: [[[BlockId; 16]; 16]; 16],
}

impl ChunkData {
    // pub fn blocks(&self) -> &[u16] {
    //     bytemuck::cast_slice(&self.blocks)
    // }

    // pub fn blocks_mut(&mut self) -> &mut [u16] {
    //     bytemuck::cast_slice_mut(&mut self.blocks)
    // }

    pub fn compute_mesh(
        &self,
        blocks: &Blocks,
        neighbors: [Option<&ChunkData>; 6],
        mesher: &mut Mesher,
    ) {
        let mut buffer = [[[BlockId::default(); 18]; 18]; 18];
        for x in 0..16 {
            for y in 0..16 {
                for z in 0..16 {
                    // This will be replaced with a "transparency class" mapping later
                    buffer[x + 1][y + 1][z + 1] = self.blocks[x][y][z]
                }
            }
        }
        if let Some(top) = neighbors[0] {
            // +x
            for x in 0..16 {
                for z in 0..16 {
                    buffer[x + 1][17][z + 1] = top.blocks[x][0][z]
                }
            }
        }
        if let Some(bottom) = neighbors[1] {
            // -x
            for x in 0..16 {
                for z in 0..16 {
                    buffer[x + 1][0][z + 1] = bottom.blocks[x][15][z]
                }
            }
        }
        if let Some(right) = neighbors[2] {
            // +y
            for y in 0..16 {
                for z in 0..16 {
                    buffer[17][y + 1][z + 1] = right.blocks[0][y][z]
                }
            }
        }
        if let Some(left) = neighbors[3] {
            // -y
            for y in 0..16 {
                for z in 0..16 {
                    buffer[0][y + 1][z + 1] = left.blocks[17][y][z]
                }
            }
        }
        if let Some(back) = neighbors[4] {
            // +z
            for x in 0..16 {
                for y in 0..16 {
                    buffer[x + 1][y + 1][17] = back.blocks[x][y][0]
                }
            }
        }
        if let Some(front) = neighbors[5] {
            // -z
            for x in 0..16 {
                for y in 0..16 {
                    buffer[x + 1][y + 1][0] = front.blocks[x][y][17]
                }
            }
        }

        for x in 0..16 {
            for y in 0..16 {
                for z in 0..16 {
                    // top face
                    let me = blocks.get_meshing_data(buffer[x + 1][y + 1][z + 1]);

                    if me.opacity == 0 {
                        continue;
                    }

                    let top = blocks.get_meshing_data(buffer[x + 1][y + 2][z + 1]);
                    let bottom = blocks.get_meshing_data(buffer[x + 1][y][z + 1]);
                    let right = blocks.get_meshing_data(buffer[x + 2][y + 1][z + 1]);
                    let left = blocks.get_meshing_data(buffer[x][y + 1][z + 1]);
                    let back = blocks.get_meshing_data(buffer[x + 1][y + 1][z + 2]);
                    let front = blocks.get_meshing_data(buffer[x + 1][y + 1][z]);

                    // Center of block coordinates
                    let x = x as f32 - 8.0;
                    let y = y as f32 - 8.0;
                    let z = z as f32 - 8.0;

                    //TODO: shared texture optimization?
                    if me.opacity != top.opacity {
                        let mesh = mesher.meshes.entry(me.textures[0].sheet()).or_default();
                        let coords = me.textures[0].coords();
                        let v = mesh.vertices.len() as u16;
                        mesh.vertices.push([x - 0.5, y + 0.5, z - 0.5]);
                        mesh.vertices.push([x + 0.5, y + 0.5, z - 0.5]);
                        mesh.vertices.push([x + 0.5, y + 0.5, z + 0.5]);
                        mesh.vertices.push([x - 0.5, y + 0.5, z + 0.5]);
                        mesh.normals.push([0.0, 1.0, 0.0]);
                        mesh.normals.push([0.0, 1.0, 0.0]);
                        mesh.normals.push([0.0, 1.0, 0.0]);
                        mesh.normals.push([0.0, 1.0, 0.0]);
                        mesh.uv.push(coords.top_left());
                        mesh.uv.push(coords.top_right());
                        mesh.uv.push(coords.bottom_right());
                        mesh.uv.push(coords.bottom_left());
                        mesh.triangles.push(v);
                        mesh.triangles.push(v + 3);
                        mesh.triangles.push(v + 1);
                        mesh.triangles.push(v + 1);
                        mesh.triangles.push(v + 3);
                        mesh.triangles.push(v + 2);
                    }
                    if me.solid > top.solid {
                        let v = mesher.physics_vertices.len() as u32;
                        mesher.physics_vertices.extend([
                            Vec3::from([x - 0.5, y + 0.5, z - 0.5]),
                            [x + 0.5, y + 0.5, z - 0.5].into(),
                            [x + 0.5, y + 0.5, z + 0.5].into(),
                            [x - 0.5, y + 0.5, z + 0.5].into(),
                        ]);
                        mesher
                            .physics_triangles
                            .extend([[v, v + 3, v + 1], [v + 1, v + 3, v + 2]]);
                    }
                    if me.opacity != bottom.opacity {
                        let mesh = mesher.meshes.entry(me.textures[1].sheet()).or_default();
                        let coords = me.textures[1].coords();
                        let v = mesh.vertices.len() as u16;
                        mesh.vertices.push([x + 0.5, y - 0.5, z + 0.5]);
                        mesh.vertices.push([x + 0.5, y - 0.5, z - 0.5]);
                        mesh.vertices.push([x - 0.5, y - 0.5, z + 0.5]);
                        mesh.vertices.push([x - 0.5, y - 0.5, z - 0.5]);
                        mesh.normals.push([0.0, -1.0, 0.0]);
                        mesh.normals.push([0.0, -1.0, 0.0]);
                        mesh.normals.push([0.0, -1.0, 0.0]);
                        mesh.normals.push([0.0, -1.0, 0.0]);
                        mesh.uv.push(coords.bottom_left());
                        mesh.uv.push(coords.top_left());
                        mesh.uv.push(coords.bottom_right());
                        mesh.uv.push(coords.top_right());
                        mesh.triangles.push(v);
                        mesh.triangles.push(v + 2);
                        mesh.triangles.push(v + 1);
                        mesh.triangles.push(v + 2);
                        mesh.triangles.push(v + 3);
                        mesh.triangles.push(v + 1);
                    }
                    if me.solid > bottom.solid {
                        let v = mesher.physics_vertices.len() as u32;
                        mesher.physics_vertices.extend([
                            Vec3::from([x + 0.5, y - 0.5, z + 0.5]),
                            [x + 0.5, y - 0.5, z - 0.5].into(),
                            [x - 0.5, y - 0.5, z + 0.5].into(),
                            [x - 0.5, y - 0.5, z - 0.5].into(),
                        ]);
                        mesher
                            .physics_triangles
                            .extend([[v, v + 2, v + 1], [v + 2, v + 3, v + 1]]);
                    }
                    if me.opacity != right.opacity {
                        let mesh = mesher.meshes.entry(me.textures[2].sheet()).or_default();
                        let coords = me.textures[2].coords();
                        let v = mesh.vertices.len() as u16;
                        mesh.vertices.push([x + 0.5, y - 0.5, z - 0.5]);
                        mesh.vertices.push([x + 0.5, y - 0.5, z + 0.5]);
                        mesh.vertices.push([x + 0.5, y + 0.5, z + 0.5]);
                        mesh.vertices.push([x + 0.5, y + 0.5, z - 0.5]);
                        mesh.normals.push([1.0, 0.0, 0.0]);
                        mesh.normals.push([1.0, 0.0, 0.0]);
                        mesh.normals.push([1.0, 0.0, 0.0]);
                        mesh.normals.push([1.0, 0.0, 0.0]);
                        mesh.uv.push(coords.bottom_left());
                        mesh.uv.push(coords.bottom_right());
                        mesh.uv.push(coords.top_right());
                        mesh.uv.push(coords.top_left());
                        mesh.triangles.push(v);
                        mesh.triangles.push(v + 3);
                        mesh.triangles.push(v + 1);
                        mesh.triangles.push(v + 1);
                        mesh.triangles.push(v + 3);
                        mesh.triangles.push(v + 2);
                    }
                    if me.solid > right.solid {
                        let v = mesher.physics_vertices.len() as u32;
                        mesher.physics_vertices.extend([
                            Vec3::from([x + 0.5, y - 0.5, z - 0.5]),
                            [x + 0.5, y - 0.5, z + 0.5].into(),
                            [x + 0.5, y + 0.5, z + 0.5].into(),
                            [x + 0.5, y + 0.5, z - 0.5].into(),
                        ]);
                        mesher
                            .physics_triangles
                            .extend([[v, v + 3, v + 1], [v + 1, v + 3, v + 2]]);
                    }
                    if me.opacity != left.opacity {
                        let mesh = mesher.meshes.entry(me.textures[3].sheet()).or_default();
                        let coords = me.textures[3].coords();
                        let v = mesh.vertices.len() as u16;
                        mesh.vertices.push([x - 0.5, y - 0.5, z + 0.5]);
                        mesh.vertices.push([x - 0.5, y - 0.5, z - 0.5]);
                        mesh.vertices.push([x - 0.5, y + 0.5, z + 0.5]);
                        mesh.vertices.push([x - 0.5, y + 0.5, z - 0.5]);
                        mesh.normals.push([-1.0, 0.0, 0.0]);
                        mesh.normals.push([-1.0, 0.0, 0.0]);
                        mesh.normals.push([-1.0, 0.0, 0.0]);
                        mesh.normals.push([-1.0, 0.0, 0.0]);
                        mesh.uv.push(coords.bottom_right());
                        mesh.uv.push(coords.bottom_left());
                        mesh.uv.push(coords.top_right());
                        mesh.uv.push(coords.top_left());
                        mesh.triangles.push(v);
                        mesh.triangles.push(v + 2);
                        mesh.triangles.push(v + 1);
                        mesh.triangles.push(v + 2);
                        mesh.triangles.push(v + 3);
                        mesh.triangles.push(v + 1);
                    }
                    if me.solid > left.solid {
                        let v = mesher.physics_vertices.len() as u32;
                        mesher.physics_vertices.extend([
                            Vec3::from([x - 0.5, y - 0.5, z + 0.5]),
                            [x - 0.5, y - 0.5, z - 0.5].into(),
                            [x - 0.5, y + 0.5, z + 0.5].into(),
                            [x - 0.5, y + 0.5, z - 0.5].into(),
                        ]);
                        mesher
                            .physics_triangles
                            .extend([[v, v + 2, v + 1], [v + 2, v + 3, v + 1]]);
                    }
                    if me.opacity != back.opacity {
                        let mesh = mesher.meshes.entry(me.textures[4].sheet()).or_default();
                        let coords = me.textures[4].coords();
                        let v = mesh.vertices.len() as u16;
                        mesh.vertices.push([x - 0.5, y - 0.5, z + 0.5]);
                        mesh.vertices.push([x - 0.5, y + 0.5, z + 0.5]);
                        mesh.vertices.push([x + 0.5, y + 0.5, z + 0.5]);
                        mesh.vertices.push([x + 0.5, y - 0.5, z + 0.5]);
                        mesh.normals.push([0.0, 0.0, 1.0]);
                        mesh.normals.push([0.0, 0.0, 1.0]);
                        mesh.normals.push([0.0, 0.0, 1.0]);
                        mesh.normals.push([0.0, 0.0, 1.0]);
                        mesh.uv.push(coords.bottom_left());
                        mesh.uv.push(coords.top_left());
                        mesh.uv.push(coords.top_right());
                        mesh.uv.push(coords.bottom_right());
                        mesh.triangles.push(v);
                        mesh.triangles.push(v + 3);
                        mesh.triangles.push(v + 1);
                        mesh.triangles.push(v + 1);
                        mesh.triangles.push(v + 3);
                        mesh.triangles.push(v + 2);
                    }
                    if me.solid > back.solid {
                        let v = mesher.physics_vertices.len() as u32;
                        mesher.physics_vertices.extend([
                            Vec3::from([x - 0.5, y - 0.5, z + 0.5]),
                            [x - 0.5, y + 0.5, z + 0.5].into(),
                            [x + 0.5, y + 0.5, z + 0.5].into(),
                            [x + 0.5, y - 0.5, z + 0.5].into(),
                        ]);
                        mesher
                            .physics_triangles
                            .extend([[v, v + 3, v + 1], [v + 1, v + 3, v + 2]]);
                    }
                    if me.opacity != front.opacity {
                        let mesh = mesher.meshes.entry(me.textures[5].sheet()).or_default();
                        let coords = me.textures[5].coords();
                        let v = mesh.vertices.len() as u16;
                        mesh.vertices.push([x - 0.5, y - 0.5, z - 0.5]);
                        mesh.vertices.push([x - 0.5, y + 0.5, z - 0.5]);
                        mesh.vertices.push([x + 0.5, y + 0.5, z - 0.5]);
                        mesh.vertices.push([x + 0.5, y - 0.5, z - 0.5]);
                        mesh.normals.push([0.0, 0.0, -1.0]);
                        mesh.normals.push([0.0, 0.0, -1.0]);
                        mesh.normals.push([0.0, 0.0, -1.0]);
                        mesh.normals.push([0.0, 0.0, -1.0]);
                        mesh.uv.push(coords.top_left());
                        mesh.uv.push(coords.bottom_left());
                        mesh.uv.push(coords.bottom_right());
                        mesh.uv.push(coords.top_right());
                        mesh.triangles.push(v);
                        mesh.triangles.push(v + 1);
                        mesh.triangles.push(v + 3);
                        mesh.triangles.push(v + 1);
                        mesh.triangles.push(v + 2);
                        mesh.triangles.push(v + 3);
                    }
                    if me.solid > front.solid {
                        let v = mesher.physics_vertices.len() as u32;
                        mesher.physics_vertices.extend([
                            Vec3::from([x - 0.5, y - 0.5, z - 0.5]),
                            [x - 0.5, y + 0.5, z - 0.5].into(),
                            [x + 0.5, y + 0.5, z - 0.5].into(),
                            [x + 0.5, y - 0.5, z - 0.5].into(),
                        ]);
                        mesher
                            .physics_triangles
                            .extend([[v, v + 1, v + 3], [v + 1, v + 2, v + 3]]);
                    }
                }
            }
        }
    }
}
