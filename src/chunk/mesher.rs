use crate::block::{
    texture::{BlockMaterials, SheetId},
    Blocks,
};
use bevy::{
    prelude::*,
    render::{mesh::Indices, render_resource::PrimitiveTopology},
};
use bevy_rapier3d::prelude::*;
use fxhash::FxHashMap;

use super::{Chunk, IsChunkMesh};

#[derive(Default, Clone, PartialEq)]
pub struct Mesher {
    pub meshes: FxHashMap<SheetId, Premesh>,
    pub physics_vertices: Vec<Vec3>,
    pub physics_triangles: Vec<[u32; 3]>,
}

impl Mesher {
    pub fn clear(&mut self) {
        self.meshes.clear();
        self.physics_vertices.clear();
        self.physics_triangles.clear();
    }
}

#[derive(Default, Clone, PartialEq)]

pub struct Premesh {
    pub vertices: Vec<[f32; 3]>,
    pub triangles: Vec<u16>,
    pub normals: Vec<[f32; 3]>,
    pub uv: Vec<[f32; 2]>,
}

#[derive(Component)]
#[component(storage = "SparseSet")]
pub struct ChunkNeedsMeshing;

pub fn chunk_mesher_system(
    mut commands: Commands,
    mut chunks: Query<
        (Entity, &Chunk, Option<&mut Collider>, Option<&Children>),
        With<ChunkNeedsMeshing>,
    >,
    blocks: Res<Blocks>,
    materials: Res<BlockMaterials>,
    mut chunk_meshes: Query<
        (
            &Handle<Mesh>,
            &mut Handle<StandardMaterial>,
            &mut IsChunkMesh,
        ),
        Without<Chunk>,
    >,
    mut meshes: ResMut<Assets<Mesh>>,
) {
    //TODO: multithread meshing
    let mut mesher = Mesher::default();
    let mut recycler = Vec::new();

    for (chunk_entity, chunk, collider, children) in chunks.iter_mut() {
        mesher.clear();
        //TODO: neighbors... have separate chunk-data/hyperchunk storage?
        chunk.data.compute_mesh(&blocks, [None; 6], &mut mesher);

        if let Some(children) = children {
            for child in children {
                if let Ok((mesh, _material, sheet)) = chunk_meshes.get_mut(*child) {
                    if let Some(premesh) = mesher.meshes.remove(&sheet.0) {
                        let mesh = meshes.get_mut(&mesh).unwrap();
                        *mesh.attribute_mut(Mesh::ATTRIBUTE_POSITION).unwrap() =
                            premesh.vertices.into();
                        mesh.set_indices(Some(Indices::U16(premesh.triangles)));
                        *mesh.attribute_mut(Mesh::ATTRIBUTE_NORMAL).unwrap() =
                            premesh.normals.into();
                        *mesh.attribute_mut(Mesh::ATTRIBUTE_UV_0).unwrap() = premesh.uv.into();
                    } else {
                        recycler.push(*child)
                    }
                }
            }
        }

        let new_collider = Collider::trimesh(
            mesher.physics_vertices.clone(),
            mesher.physics_triangles.clone(),
        );
        if let Some(mut collider) = collider {
            *collider = new_collider;
        } else {
            commands.entity(chunk_entity).insert(new_collider);
        }

        commands.entity(chunk_entity).with_children(|chunk| {
            for (sheet, premesh) in mesher.meshes.drain() {
                if let Some(recycle) = recycler.pop() {
                    let (mesh, mut material, mut sheet_ix) = chunk_meshes.get_mut(recycle).unwrap();
                    let mesh = meshes.get_mut(&mesh).unwrap();
                    *mesh.attribute_mut(Mesh::ATTRIBUTE_POSITION).unwrap() =
                        premesh.vertices.into();
                    mesh.set_indices(Some(Indices::U16(premesh.triangles)));
                    *mesh.attribute_mut(Mesh::ATTRIBUTE_NORMAL).unwrap() = premesh.normals.into();
                    *mesh.attribute_mut(Mesh::ATTRIBUTE_UV_0).unwrap() = premesh.uv.into();
                    *material = materials.get_sheet_material(sheet);
                    sheet_ix.0 = sheet;
                } else {
                    let mut mesh = Mesh::new(PrimitiveTopology::TriangleList);
                    mesh.insert_attribute(Mesh::ATTRIBUTE_POSITION, premesh.vertices);
                    mesh.set_indices(Some(Indices::U16(premesh.triangles)));
                    mesh.insert_attribute(Mesh::ATTRIBUTE_NORMAL, premesh.normals);
                    mesh.insert_attribute(Mesh::ATTRIBUTE_UV_0, premesh.uv);
                    let material = materials.get_sheet_material(sheet);
                    chunk.spawn((
                        PbrBundle {
                            mesh: meshes.add(mesh),
                            material,
                            ..default()
                        },
                        IsChunkMesh(sheet),
                    ));
                }
            }
        });

        for child in recycler.drain(..) {
            commands.entity(child).despawn();
        }

        commands.entity(chunk_entity).remove::<ChunkNeedsMeshing>();
    }
}
