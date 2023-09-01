use fxhash::FxHashMap;
use crate::block::texture::SheetId;

#[derive(Default, Clone, PartialEq)]
pub struct Mesher {
    pub meshes: FxHashMap<SheetId, Mesh>,
}

#[derive(Default, Clone, PartialEq)]

pub struct Mesh {
    pub vertices: Vec<[f32; 3]>,
    pub triangles: Vec<u16>,
    pub normals: Vec<[f32; 3]>,
    pub uv: Vec<[f32; 2]>,
}
