use bevy::prelude::*;
use bytemuck::{Pod, Zeroable};

use crate::block::{BlockId, Blocks};

#[derive(Component)]
pub struct IsChunkMesh;

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
        vertices: &mut Vec<[f32; 3]>,
        triangles: &mut Vec<u16>,
        normals: &mut Vec<[f32; 3]>,
        uv: &mut Vec<[f32; 2]>,
        neighbors: [Option<&ChunkData>; 6],
    ) {
        // Clear buffers
        vertices.clear();
        triangles.clear();
        uv.clear();

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

                    let coords = me.texture.coords();

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
                        let v = vertices.len() as u16;
                        vertices.push([x - 0.5, y + 0.5, z - 0.5]);
                        vertices.push([x + 0.5, y + 0.5, z - 0.5]);
                        vertices.push([x + 0.5, y + 0.5, z + 0.5]);
                        vertices.push([x - 0.5, y + 0.5, z + 0.5]);
                        normals.push([0.0, 1.0, 0.0]);
                        normals.push([0.0, 1.0, 0.0]);
                        normals.push([0.0, 1.0, 0.0]);
                        normals.push([0.0, 1.0, 0.0]);
                        uv.push(coords.top_left());
                        uv.push(coords.top_right());
                        uv.push(coords.bottom_right());
                        uv.push(coords.bottom_left());
                        triangles.push(v);
                        triangles.push(v + 3);
                        triangles.push(v + 1);
                        triangles.push(v + 1);
                        triangles.push(v + 3);
                        triangles.push(v + 2);
                    }
                    if me.opacity != bottom.opacity {
                        let v = vertices.len() as u16;
                        vertices.push([x + 0.5, y - 0.5, z + 0.5]);
                        vertices.push([x + 0.5, y - 0.5, z - 0.5]);
                        vertices.push([x - 0.5, y - 0.5, z + 0.5]);
                        vertices.push([x - 0.5, y - 0.5, z - 0.5]);
                        normals.push([0.0, -1.0, 0.0]);
                        normals.push([0.0, -1.0, 0.0]);
                        normals.push([0.0, -1.0, 0.0]);
                        normals.push([0.0, -1.0, 0.0]);
                        uv.push(coords.bottom_left());
                        uv.push(coords.top_left());
                        uv.push(coords.bottom_right());
                        uv.push(coords.top_right());
                        triangles.push(v);
                        triangles.push(v + 2);
                        triangles.push(v + 1);
                        triangles.push(v + 2);
                        triangles.push(v + 3);
                        triangles.push(v + 1);
                    }
                    if me.opacity != right.opacity {
                        let v = vertices.len() as u16;
                        vertices.push([x + 0.5, y - 0.5, z - 0.5]);
                        vertices.push([x + 0.5, y - 0.5, z + 0.5]);
                        vertices.push([x + 0.5, y + 0.5, z + 0.5]);
                        vertices.push([x + 0.5, y + 0.5, z - 0.5]);
                        normals.push([1.0, 0.0, 0.0]);
                        normals.push([1.0, 0.0, 0.0]);
                        normals.push([1.0, 0.0, 0.0]);
                        normals.push([1.0, 0.0, 0.0]);
                        uv.push(coords.bottom_left());
                        uv.push(coords.bottom_right());
                        uv.push(coords.top_right());
                        uv.push(coords.top_left());
                        triangles.push(v);
                        triangles.push(v + 3);
                        triangles.push(v + 1);
                        triangles.push(v + 1);
                        triangles.push(v + 3);
                        triangles.push(v + 2);
                    }
                    if me.opacity != left.opacity {
                        let v = vertices.len() as u16;
                        vertices.push([x - 0.5, y - 0.5, z + 0.5]);
                        vertices.push([x - 0.5, y - 0.5, z - 0.5]);
                        vertices.push([x - 0.5, y + 0.5, z + 0.5]);
                        vertices.push([x - 0.5, y + 0.5, z - 0.5]);
                        normals.push([-1.0, 0.0, 0.0]);
                        normals.push([-1.0, 0.0, 0.0]);
                        normals.push([-1.0, 0.0, 0.0]);
                        normals.push([-1.0, 0.0, 0.0]);
                        uv.push(coords.bottom_right());
                        uv.push(coords.bottom_left());
                        uv.push(coords.top_right());
                        uv.push(coords.top_left());
                        triangles.push(v);
                        triangles.push(v + 2);
                        triangles.push(v + 1);
                        triangles.push(v + 2);
                        triangles.push(v + 3);
                        triangles.push(v + 1);
                    }
                    if me.opacity != back.opacity {
                        let v = vertices.len() as u16;
                        vertices.push([x - 0.5, y - 0.5, z + 0.5]);
                        vertices.push([x - 0.5, y + 0.5, z + 0.5]);
                        vertices.push([x + 0.5, y + 0.5, z + 0.5]);
                        vertices.push([x + 0.5, y - 0.5, z + 0.5]);
                        normals.push([0.0, 0.0, 1.0]);
                        normals.push([0.0, 0.0, 1.0]);
                        normals.push([0.0, 0.0, 1.0]);
                        normals.push([0.0, 0.0, 1.0]);
                        uv.push(coords.bottom_left());
                        uv.push(coords.top_left());
                        uv.push(coords.top_right());
                        uv.push(coords.bottom_right());
                        triangles.push(v);
                        triangles.push(v + 3);
                        triangles.push(v + 1);
                        triangles.push(v + 1);
                        triangles.push(v + 3);
                        triangles.push(v + 2);
                    }
                    if me.opacity != front.opacity {
                        let v = vertices.len() as u16;
                        vertices.push([x - 0.5, y - 0.5, z - 0.5]);
                        vertices.push([x - 0.5, y + 0.5, z - 0.5]);
                        vertices.push([x + 0.5, y + 0.5, z - 0.5]);
                        vertices.push([x + 0.5, y - 0.5, z - 0.5]);
                        normals.push([0.0, 0.0, -1.0]);
                        normals.push([0.0, 0.0, -1.0]);
                        normals.push([0.0, 0.0, -1.0]);
                        normals.push([0.0, 0.0, -1.0]);
                        uv.push(coords.top_left());
                        uv.push(coords.bottom_left());
                        uv.push(coords.bottom_right());
                        uv.push(coords.top_right());
                        triangles.push(v);
                        triangles.push(v + 1);
                        triangles.push(v + 3);
                        triangles.push(v + 1);
                        triangles.push(v + 2);
                        triangles.push(v + 3);
                    }
                }
            }
        }
    }
}
