use std::time::Duration;

use bevy::{
    asset::ChangeWatcher,
    diagnostic::FrameTimeDiagnosticsPlugin,
    prelude::*,
    render::{mesh::Indices, render_resource::PrimitiveTopology},
};
use bevy_egui::{EguiPlugin, EguiSettings};
use bevy_rapier3d::prelude::*;
use mellanite::chunk::{ChunkData, IsChunk, IsChunkMesh};
use mellanite::{
    block::{
        texture::{blit_loaded_textures, BlockMaterials},
        BlockId, Blocks,
    },
    chunk::mesher::Mesher,
};
use rand::Rng;

fn main() {
    App::new()
        .add_plugins(
            DefaultPlugins
                .set(ImagePlugin::default_nearest())
                .set(AssetPlugin {
                    watch_for_changes: ChangeWatcher::with_delay(Duration::from_millis(200)),
                    ..default()
                }),
        )
        .add_plugins(EguiPlugin)
        .add_plugins(FrameTimeDiagnosticsPlugin)
        .add_plugins(RapierPhysicsPlugin::<NoUserData>::default())
        // .add_plugins(RapierDebugRenderPlugin::default())
        .insert_resource(EguiSettings {
            scale_factor: 1.0,
            ..EguiSettings::default()
        })
        .insert_resource(Blocks::default())
        .insert_resource(ClearColor(Color::ALICE_BLUE))
        .insert_resource(BlockMaterials::default())
        .add_systems(Startup, mellanite::player::setup_player)
        .add_systems(Startup, setup_environment)
        .add_systems(Update, mellanite::ui::ui_system)
        .add_systems(Update, mellanite::player::player_control)
        .add_systems(Update, blit_loaded_textures)
        .run()
}

fn setup_environment(
    mut commands: Commands,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut blocks: ResMut<Blocks>,
    mut block_materials: ResMut<BlockMaterials>,
    mut images: ResMut<Assets<Image>>,
    asset_server: Res<AssetServer>,
) {
    let mut chunk = ChunkData {
        blocks: [[[BlockId::default(); 16]; 16]; 16],
    };

    let solid_material = materials.add(StandardMaterial {
        base_color_texture: None,
        perceptual_roughness: 1.0,
        reflectance: 0.0,
        unlit: false,
        alpha_mode: AlphaMode::Mask(0.5),
        ..default()
    });
    let solid_block_material = block_materials
        .new_material(solid_material.clone())
        .unwrap();

    let glassy_material = block_materials
        .new_material(materials.add(StandardMaterial {
            base_color_texture: None,
            perceptual_roughness: 1.0,
            reflectance: 0.5,
            unlit: false,
            alpha_mode: AlphaMode::Blend,
            ..default()
        }))
        .unwrap();

    let coords_texture = block_materials
        .new_texture(solid_block_material, &mut materials)
        .unwrap();
    let dirt_texture = block_materials
        .new_texture(solid_block_material, &mut materials)
        .unwrap();
    let stone_texture: mellanite::block::texture::BlockTextureId = block_materials
        .new_texture(solid_block_material, &mut materials)
        .unwrap();
    let white_ore_texture = block_materials
        .new_texture(solid_block_material, &mut materials)
        .unwrap();
    let glass_texture = block_materials
        .new_texture(glassy_material, &mut materials)
        .unwrap();
    let grass_top_texture = block_materials
        .new_texture(solid_block_material, &mut materials)
        .unwrap();
    let grass_side_texture = block_materials
        .new_texture(solid_block_material, &mut materials)
        .unwrap();
    let coords = blocks
        .new_block([coords_texture; 6], u32::MAX, true)
        .unwrap();
    let dirt = blocks.new_block([dirt_texture; 6], u32::MAX, true).unwrap();
    let stone = blocks
        .new_block([stone_texture; 6], u32::MAX, true)
        .unwrap();
    let white_ore = blocks
        .new_block([white_ore_texture; 6], u32::MAX, true)
        .unwrap();
    let glass = blocks.new_block([glass_texture; 6], 1, true).unwrap();
    let grass = blocks
        .new_block(
            [
                grass_top_texture,
                dirt_texture,
                grass_side_texture,
                grass_side_texture,
                grass_side_texture,
                grass_side_texture,
            ],
            u32::MAX,
            true,
        )
        .unwrap();

    let mut rng = rand::thread_rng();
    for x in 0..16 {
        for z in 0..16 {
            let y = rng.gen_range(7..=9);
            chunk.blocks[x][y][z] = grass;
            chunk.blocks[x][y - 1][z] = dirt;
            for y in 0..y - 1 {
                if rng.gen_bool(0.1) {
                    chunk.blocks[x][y][z] = white_ore;
                } else if rng.gen_bool(0.1) {
                    chunk.blocks[x][y][z] = coords;
                } else if rng.gen_bool(0.5) {
                    chunk.blocks[x][y][z] = stone;
                } else {
                    chunk.blocks[x][y][z] = glass;
                }
            }
        }
    }

    let mut mesher = Mesher::default();

    chunk.compute_mesh(&blocks, [None, None, None, None, None, None], &mut mesher);

    let dirt_image: Handle<Image> = asset_server.load("dirt.png");
    block_materials
        .set_block_texture(dirt_texture, dirt_image, &mut images, &mut materials)
        .unwrap();

    let stone_image: Handle<Image> = asset_server.load("stone.png");
    block_materials
        .set_block_texture(stone_texture, stone_image, &mut images, &mut materials)
        .unwrap();

    let white_ore_image: Handle<Image> = asset_server.load("white_ore.png");
    block_materials
        .set_block_texture(
            white_ore_texture,
            white_ore_image,
            &mut images,
            &mut materials,
        )
        .unwrap();

    let glass_image: Handle<Image> = asset_server.load("glass.png");
    block_materials
        .set_block_texture(glass_texture, glass_image, &mut images, &mut materials)
        .unwrap();

    let grass_side: Handle<Image> = asset_server.load("grass_side.png");
    block_materials
        .set_block_texture(grass_side_texture, grass_side, &mut images, &mut materials)
        .unwrap();

    let grass_top_image: Handle<Image> = asset_server.load("grass_top.png");
    block_materials
        .set_block_texture(
            grass_top_texture,
            grass_top_image,
            &mut images,
            &mut materials,
        )
        .unwrap();

    let coords_image: Handle<Image> = asset_server.load("coords.png");
    block_materials
        .set_block_texture(coords_texture, coords_image, &mut images, &mut materials)
        .unwrap();

    let collider = Collider::trimesh(mesher.physics_vertices, mesher.physics_triangles);
    commands
        .spawn((
            IsChunk,
            collider,
            SpatialBundle::from_transform(Transform::from_translation(Vec3::new(0.0, 0.0, -10.0))),
        ))
        .with_children(|chunk| {
            for (sheet, mesh) in mesher.meshes {
                let mut chunk_mesh = Mesh::new(PrimitiveTopology::TriangleList);

                chunk_mesh.insert_attribute(Mesh::ATTRIBUTE_POSITION, mesh.vertices);
                chunk_mesh.set_indices(Some(Indices::U16(mesh.triangles)));
                chunk_mesh.insert_attribute(Mesh::ATTRIBUTE_NORMAL, mesh.normals);
                chunk_mesh.insert_attribute(Mesh::ATTRIBUTE_UV_0, mesh.uv);

                let chunk_mesh = meshes.add(chunk_mesh);

                chunk.spawn((
                    PbrBundle {
                        mesh: chunk_mesh,
                        material: block_materials.get_sheet_material(sheet),
                        ..default()
                    },
                    IsChunkMesh,
                ));
            }
        });

    commands.spawn((
        RigidBody::Dynamic,
        Collider::ball(0.5),
        Restitution::coefficient(0.7),
        PbrBundle {
            mesh: meshes.add(
                shape::Icosphere {
                    radius: 0.5,
                    subdivisions: 2,
                }
                .try_into()
                .unwrap(),
            ),
            transform: Transform::from_xyz(0.0, 30.0, -15.0),
            material: solid_material,
            ..default()
        },
    ));

    commands.spawn(PointLightBundle {
        point_light: PointLight {
            intensity: 1000.0,
            range: 100.0,
            color: Color::ORANGE,
            ..default()
        },
        transform: Transform::from_translation(Vec3::new(0.0, 5.0, 0.0)),
        ..default()
    });

    commands.spawn(DirectionalLightBundle {
        directional_light: DirectionalLight {
            illuminance: 10000.0,
            shadows_enabled: true,
            ..default()
        },
        ..default()
    });
}
