use std::time::Duration;

use bevy::{
    asset::ChangeWatcher,
    diagnostic::FrameTimeDiagnosticsPlugin,
    prelude::*,
    render::{mesh::Indices, render_resource::PrimitiveTopology},
};
use bevy_egui::{EguiPlugin, EguiSettings, egui::color_picker::Alpha};
use mellanite::block::{
    texture::{blit_loaded_textures, BlockTextures, SheetId},
    BlockId, Blocks,
};
use mellanite::chunk::{ChunkData, IsChunkMesh};
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
        .insert_resource(BlockTextures::default())
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
    mut textures: ResMut<BlockTextures>,
    mut images: ResMut<Assets<Image>>,
    asset_server: Res<AssetServer>,
) {
    let mut chunk = ChunkData {
        blocks: [[[BlockId::default(); 16]; 16]; 16],
    };

    let coords_texture = textures.new_texture(&mut materials).unwrap();
    let dirt_texture = textures.new_texture(&mut materials).unwrap();
    let stone_texture: mellanite::block::texture::BlockTextureId =
        textures.new_texture(&mut materials).unwrap();
    let white_ore_texture = textures.new_texture(&mut materials).unwrap();
    let glass_texture = textures.new_texture(&mut materials).unwrap();
    let _coords = blocks.new_block(coords_texture, u32::MAX).unwrap();
    let dirt = blocks.new_block(dirt_texture, u32::MAX).unwrap();
    let _stone = blocks.new_block(stone_texture, u32::MAX).unwrap();
    let _white_ore = blocks.new_block(white_ore_texture, u32::MAX).unwrap();
    let glass = blocks.new_block(glass_texture, 1).unwrap();

    let mut rng = rand::thread_rng();
    for x in 0..16 {
        for z in 0..16 {
            let y = rng.gen_range(7..=9);
            chunk.blocks[x][y][z] = dirt;
            for y in 0..y {
                chunk.blocks[x][y][z] = glass;
            }
        }
    }

    let mut vertices = Vec::new();
    let mut triangles = Vec::new();
    let mut normals = Vec::new();
    let mut uv = Vec::new();

    chunk.compute_mesh(
        &blocks,
        &mut vertices,
        &mut triangles,
        &mut normals,
        &mut uv,
        [None, None, None, None, None, None],
    );

    let dirt_image: Handle<Image> = asset_server.load("dirt.png");
    textures
        .set_block_texture(dirt_texture, dirt_image, &mut images, &mut materials)
        .unwrap();

    let stone_image: Handle<Image> = asset_server.load("stone.png");
    textures
        .set_block_texture(stone_texture, stone_image, &mut images, &mut materials)
        .unwrap();

    let white_ore_image: Handle<Image> = asset_server.load("white_ore.png");
    textures
        .set_block_texture(
            white_ore_texture,
            white_ore_image,
            &mut images,
            &mut materials,
        )
        .unwrap();

    let glass_image: Handle<Image> = asset_server.load("glass.png");
    textures
        .set_block_texture(glass_texture, glass_image, &mut images, &mut materials)
        .unwrap();

    let coords_image: Handle<Image> = asset_server.load("coords.png");
    textures
        .set_block_texture(coords_texture, coords_image, &mut images, &mut materials)
        .unwrap();

    let mut mesh = Mesh::new(PrimitiveTopology::TriangleList);
    mesh.insert_attribute(Mesh::ATTRIBUTE_POSITION, vertices);
    mesh.set_indices(Some(Indices::U16(triangles)));
    mesh.insert_attribute(Mesh::ATTRIBUTE_NORMAL, normals);
    mesh.insert_attribute(Mesh::ATTRIBUTE_UV_0, uv);

    let chunk_mesh = meshes.add(mesh);

    commands.spawn(PointLightBundle {
        point_light: PointLight {
            intensity: 1000.0,
            range: 100.0,
            ..default()
        },
        transform: Transform::from_translation(Vec3::new(0.0, 5.0, 0.0)),
        ..default()
    });

    let atlas = textures.get_sheet_material(SheetId(0));

    commands.spawn((
        PbrBundle {
            mesh: chunk_mesh,
            material: atlas.clone(),
            transform: Transform::from_translation(Vec3::new(0.0, 0.0, -10.0)),
            ..default()
        },
        IsChunkMesh,
    ));
}
