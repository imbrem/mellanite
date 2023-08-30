use std::time::Duration;

use bevy::{
    asset::ChangeWatcher,
    diagnostic::FrameTimeDiagnosticsPlugin,
    prelude::*,
    render::{mesh::Indices, render_resource::PrimitiveTopology},
};
use bevy_egui::{EguiPlugin, EguiSettings};
use blocks::{blit_loaded_textures, BlockId, Blocks, SheetId};
use chunk::{ChunkData, IsChunkMesh};
use rand::Rng;

mod blocks;
mod chunk;
mod player;
mod ui;

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
        .insert_resource(blocks::Blocks::default())
        .add_systems(Startup, player::setup_player)
        .add_systems(Startup, setup_environment)
        .add_systems(Update, ui::ui_system)
        .add_systems(Update, player::player_control)
        .add_systems(Update, blit_loaded_textures)
        .run()
}

fn setup_environment(
    mut commands: Commands,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut blocks: ResMut<Blocks>,
    mut images: ResMut<Assets<Image>>,
    asset_server: Res<AssetServer>,
) {
    let mut chunk = ChunkData {
        blocks: [[[BlockId(0); 16]; 16]; 16],
    };

    let mut rng = rand::thread_rng();
    for x in 0..16 {
        for z in 0..16 {
            let y = rng.gen_range(7..=9);
            chunk.blocks[x][y][z] = BlockId(2);
            for y in 0..y {
                chunk.blocks[x][y][z] = BlockId(1);
            }
        }
    }

    let mut vertices = Vec::new();
    let mut triangles = Vec::new();
    let mut normals = Vec::new();
    let mut uv = Vec::new();

    chunk.compute_mesh(
        &mut vertices,
        &mut triangles,
        &mut normals,
        &mut uv,
        [None, None, None, None, None, None],
    );

    // Register air
    blocks.new_block(&mut images, &mut materials).unwrap();

    let white_ore_texture: Handle<Image> = asset_server.load("white_ore.png");
    let white_ore = blocks.new_block(&mut images, &mut materials).unwrap();
    blocks
        .set_block_texture(white_ore, white_ore_texture, &mut images, &mut materials)
        .unwrap();

    let coords_texture: Handle<Image> = asset_server.load("coords.png");
    let coords = blocks.new_block(&mut images, &mut materials).unwrap();
    blocks
        .set_block_texture(coords, coords_texture, &mut images, &mut materials)
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

    let atlas = blocks.get_sheet_material(SheetId(0));

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
