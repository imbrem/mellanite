use crate::block::texture::SheetId;
use bevy::prelude::Vec3;
use fxhash::FxHashMap;

#[derive(Default, Clone, PartialEq)]
pub struct Mesher {
    pub meshes: FxHashMap<SheetId, Mesh>,
    pub physics_vertices: Vec<Vec3>,
    pub physics_triangles: Vec<[u32; 3]>,
}

#[derive(Default, Clone, PartialEq)]

pub struct Mesh {
    pub vertices: Vec<[f32; 3]>,
    pub triangles: Vec<u16>,
    pub normals: Vec<[f32; 3]>,
    pub uv: Vec<[f32; 2]>,
}
