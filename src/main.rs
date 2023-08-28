use std::time::Duration;

use bevy::{
    asset::ChangeWatcher,
    diagnostic::FrameTimeDiagnosticsPlugin,
    prelude::*,
    render::{mesh::Indices, render_resource::PrimitiveTopology},
};
use bevy_egui::{EguiPlugin, EguiSettings};
use blocks::Blocks;
use chunk::{ChunkData, IsChunk};
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
        .run()
}

fn setup_environment(
    mut commands: Commands,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut blocks: ResMut<Blocks>,
    asset_server: Res<AssetServer>,
) {
    let mut chunk = ChunkData {
        blocks: [[[0; 16]; 16]; 16],
    };

    let mut rng = rand::thread_rng();
    for x in 0..16 {
        for z in 0..16 {
            let y = rng.gen_range(7..=9);
            chunk.blocks[x][y][z] = 2;
            for y in 0..y {
                chunk.blocks[x][y][z] = 1;
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

    let atlas_texture: Handle<Image> = asset_server.load("atlas.png");
    let atlas: Handle<StandardMaterial> = materials.add(StandardMaterial {
        base_color_texture: Some(atlas_texture),
        perceptual_roughness: 1.0,
        reflectance: 0.0,
        unlit: false,
        ..default()
    });

    let white_ore_texture: Handle<Image> = asset_server.load("white_ore.png");
    let white_ore: Handle<StandardMaterial> = materials.add(StandardMaterial {
        base_color_texture: Some(white_ore_texture),
        perceptual_roughness: 1.0,
        reflectance: 0.0,
        unlit: false,
        ..default()
    });

    let coords_texture: Handle<Image> = asset_server.load("coords.png");
    let coords: Handle<StandardMaterial> = materials.add(StandardMaterial {
        base_color_texture: Some(coords_texture),
        perceptual_roughness: 1.0,
        reflectance: 0.0,
        unlit: false,
        ..default()
    });

    blocks.white_ore = white_ore.clone();
    blocks.coords = coords.clone();

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

    commands.spawn((
        PbrBundle {
            mesh: chunk_mesh,
            material: atlas.clone(),
            transform: Transform::from_translation(Vec3::new(0.0, 0.0, -10.0)),
            ..default()
        },
        IsChunk,
    ));
}
