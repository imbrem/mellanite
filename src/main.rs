use std::time::Duration;

use bevy::{
    asset::ChangeWatcher,
    diagnostic::FrameTimeDiagnosticsPlugin,
    prelude::*,
    render::{mesh::Indices, render_resource::PrimitiveTopology},
};
use bevy_egui::{EguiPlugin, EguiSettings};
use mellanite::chunk::{ChunkData, IsChunkMesh};
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
        .insert_resource(EguiSettings {
            scale_factor: 1.0,
            ..EguiSettings::default()
        })
        .insert_resource(Blocks::default())
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

    let solid_material = block_materials
        .new_material(materials.add(StandardMaterial {
            base_color_texture: None,
            perceptual_roughness: 1.0,
            reflectance: 0.0,
            unlit: false,
            alpha_mode: AlphaMode::Mask(0.5),
            ..default()
        }))
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
        .new_texture(solid_material, &mut materials)
        .unwrap();
    let dirt_texture = block_materials
        .new_texture(solid_material, &mut materials)
        .unwrap();
    let stone_texture: mellanite::block::texture::BlockTextureId = block_materials
        .new_texture(solid_material, &mut materials)
        .unwrap();
    let white_ore_texture = block_materials
        .new_texture(solid_material, &mut materials)
        .unwrap();
    let glass_texture = block_materials
        .new_texture(glassy_material, &mut materials)
        .unwrap();
    let coords = blocks.new_block(coords_texture, u32::MAX).unwrap();
    let dirt = blocks.new_block(dirt_texture, u32::MAX).unwrap();
    let stone = blocks.new_block(stone_texture, u32::MAX).unwrap();
    let white_ore = blocks.new_block(white_ore_texture, u32::MAX).unwrap();
    let glass = blocks.new_block(glass_texture, 1).unwrap();

    let mut rng = rand::thread_rng();
    for x in 0..16 {
        for z in 0..16 {
            let y = rng.gen_range(7..=9);
            chunk.blocks[x][y][z] = dirt;
            for y in 0..y {
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

    let coords_image: Handle<Image> = asset_server.load("coords.png");
    block_materials
        .set_block_texture(coords_texture, coords_image, &mut images, &mut materials)
        .unwrap();

    for (sheet, mesh) in mesher.meshes {
        let mut chunk_mesh = Mesh::new(PrimitiveTopology::TriangleList);
        chunk_mesh.insert_attribute(Mesh::ATTRIBUTE_POSITION, mesh.vertices);
        chunk_mesh.set_indices(Some(Indices::U16(mesh.triangles)));
        chunk_mesh.insert_attribute(Mesh::ATTRIBUTE_NORMAL, mesh.normals);
        chunk_mesh.insert_attribute(Mesh::ATTRIBUTE_UV_0, mesh.uv);

        let chunk_mesh = meshes.add(chunk_mesh);

        commands.spawn((
            PbrBundle {
                mesh: chunk_mesh,
                material: block_materials.get_sheet_material(sheet),
                transform: Transform::from_translation(Vec3::new(0.0, 0.0, -10.0)),
                ..default()
            },
            IsChunkMesh,
        ));
    }

    commands.spawn(PointLightBundle {
        point_light: PointLight {
            intensity: 1000.0,
            range: 100.0,
            ..default()
        },
        transform: Transform::from_translation(Vec3::new(0.0, 5.0, 0.0)),
        ..default()
    });
}
