use std::f32::consts::PI;

use bevy::{
    diagnostic::{DiagnosticsStore, FrameTimeDiagnosticsPlugin},
    input::mouse::MouseMotion,
    prelude::*,
    render::{mesh::Indices, render_resource::PrimitiveTopology},
    window::{CursorGrabMode, PrimaryWindow, WindowMode},
};
use bevy_egui::{
    egui::{self},
    EguiContexts, EguiPlugin, EguiSettings,
};
use bytemuck::{Pod, Zeroable};
use rand::Rng;

#[derive(Debug, Copy, Clone, Eq, PartialEq, Hash, Ord, PartialOrd, Pod, Zeroable)]
#[repr(C)]
pub struct ChunkData {
    pub blocks: [[[u32; 16]; 16]; 16],
}

impl ChunkData {
    pub fn blocks(&self) -> &[u16] {
        bytemuck::cast_slice(&self.blocks)
    }

    pub fn blocks_mut(&mut self) -> &mut [u16] {
        bytemuck::cast_slice_mut(&mut self.blocks)
    }

    pub fn compute_mesh(
        &self,
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

        let mut buffer = [[[0; 18]; 18]; 18];
        for x in 0..16 {
            for y in 0..16 {
                for z in 0..16 {
                    // This will be replaced with a "transparency class" mapping later
                    buffer[x + 1][y + 1][z + 1] = (self.blocks[x][y][z] != 0) as u8 * 255
                }
            }
        }
        if let Some(top) = neighbors[0] {
            // +x
            for x in 0..16 {
                for z in 0..16 {
                    buffer[x + 1][17][z + 1] = (top.blocks[x][0][z] != 0) as u8 * 255
                }
            }
        }
        if let Some(bottom) = neighbors[1] {
            // -x
            for x in 0..16 {
                for z in 0..16 {
                    buffer[x + 1][0][z + 1] = (bottom.blocks[x][15][z] != 0) as u8 * 255
                }
            }
        }
        if let Some(right) = neighbors[2] {
            // +y
            for y in 0..16 {
                for z in 0..16 {
                    buffer[17][y + 1][z + 1] = (right.blocks[0][y][z] != 0) as u8 * 255
                }
            }
        }
        if let Some(left) = neighbors[3] {
            // -y
            for y in 0..16 {
                for z in 0..16 {
                    buffer[0][y + 1][z + 1] = (left.blocks[17][y][z] != 0) as u8 * 255
                }
            }
        }
        if let Some(back) = neighbors[4] {
            // +z
            for x in 0..16 {
                for y in 0..16 {
                    buffer[x + 1][y + 1][17] = (back.blocks[x][y][0] != 0) as u8 * 255
                }
            }
        }
        if let Some(front) = neighbors[5] {
            // -z
            for x in 0..16 {
                for y in 0..16 {
                    buffer[x + 1][y + 1][0] = (front.blocks[x][y][17] != 0) as u8 * 255
                }
            }
        }

        for x in 0..16 {
            for y in 0..16 {
                for z in 0..16 {
                    // top face
                    let me = buffer[x + 1][y + 1][z + 1];

                    if me == 0 {
                        continue;
                    }

                    let top = buffer[x + 1][y + 2][z + 1];
                    let bottom = buffer[x + 1][y][z + 1];
                    let right = buffer[x + 2][y + 1][z + 1];
                    let left = buffer[x][y + 1][z + 1];
                    let back = buffer[x + 1][y + 1][z + 2];
                    let front = buffer[x + 1][y + 1][z];

                    // Center of block coordinates
                    let x = x as f32 - 8.0;
                    let y = y as f32 - 8.0;
                    let z = z as f32 - 8.0;

                    //TODO: shared texture optimization?
                    if me > top {
                        let v = vertices.len() as u16;
                        vertices.push([x - 0.5, y + 0.5, z - 0.5]);
                        vertices.push([x + 0.5, y + 0.5, z - 0.5]);
                        vertices.push([x + 0.5, y + 0.5, z + 0.5]);
                        vertices.push([x - 0.5, y + 0.5, z + 0.5]);
                        normals.push([0.0, 1.0, 0.0]);
                        normals.push([0.0, 1.0, 0.0]);
                        normals.push([0.0, 1.0, 0.0]);
                        normals.push([0.0, 1.0, 0.0]);
                        uv.push([0.0, 0.0]);
                        uv.push([1.0, 0.0]);
                        uv.push([1.0, 1.0]);
                        uv.push([0.0, 1.0]);
                        triangles.push(v);
                        triangles.push(v + 3);
                        triangles.push(v + 1);
                        triangles.push(v + 1);
                        triangles.push(v + 3);
                        triangles.push(v + 2);
                    }
                    if me > bottom {
                        let v = vertices.len() as u16;
                        vertices.push([x + 0.5, y - 0.5, z + 0.5]);
                        vertices.push([x + 0.5, y - 0.5, z - 0.5]);
                        vertices.push([x - 0.5, y - 0.5, z + 0.5]);
                        vertices.push([x - 0.5, y - 0.5, z - 0.5]);
                        normals.push([0.0, -1.0, 0.0]);
                        normals.push([0.0, -1.0, 0.0]);
                        normals.push([0.0, -1.0, 0.0]);
                        normals.push([0.0, -1.0, 0.0]);
                        uv.push([0.0, 1.0]);
                        uv.push([0.0, 0.0]);
                        uv.push([1.0, 1.0]);
                        uv.push([1.0, 0.0]);
                        triangles.push(v);
                        triangles.push(v + 2);
                        triangles.push(v + 1);
                        triangles.push(v + 2);
                        triangles.push(v + 3);
                        triangles.push(v + 1);
                    }
                    if me > right {
                        let v = vertices.len() as u16;
                        vertices.push([x + 0.5, y - 0.5, z - 0.5]);
                        vertices.push([x + 0.5, y - 0.5, z + 0.5]);
                        vertices.push([x + 0.5, y + 0.5, z + 0.5]);
                        vertices.push([x + 0.5, y + 0.5, z - 0.5]);
                        normals.push([1.0, 0.0, 0.0]);
                        normals.push([1.0, 0.0, 0.0]);
                        normals.push([1.0, 0.0, 0.0]);
                        normals.push([1.0, 0.0, 0.0]);
                        uv.push([0.0, 1.0]);
                        uv.push([1.0, 1.0]);
                        uv.push([1.0, 0.0]);
                        uv.push([0.0, 0.0]);
                        triangles.push(v);
                        triangles.push(v + 3);
                        triangles.push(v + 1);
                        triangles.push(v + 1);
                        triangles.push(v + 3);
                        triangles.push(v + 2);
                    }
                    if me > left {
                        let v = vertices.len() as u16;
                        vertices.push([x - 0.5, y - 0.5, z + 0.5]);
                        vertices.push([x - 0.5, y - 0.5, z - 0.5]);
                        vertices.push([x - 0.5, y + 0.5, z + 0.5]);
                        vertices.push([x - 0.5, y + 0.5, z - 0.5]);
                        normals.push([-1.0, 0.0, 0.0]);
                        normals.push([-1.0, 0.0, 0.0]);
                        normals.push([-1.0, 0.0, 0.0]);
                        normals.push([-1.0, 0.0, 0.0]);
                        uv.push([1.0, 1.0]);
                        uv.push([0.0, 1.0]);
                        uv.push([1.0, 0.0]);
                        uv.push([0.0, 0.0]);
                        triangles.push(v);
                        triangles.push(v + 2);
                        triangles.push(v + 1);
                        triangles.push(v + 2);
                        triangles.push(v + 3);
                        triangles.push(v + 1);
                    }
                    if me > back {
                        let v = vertices.len() as u16;
                        vertices.push([x - 0.5, y - 0.5, z + 0.5]);
                        vertices.push([x - 0.5, y + 0.5, z + 0.5]);
                        vertices.push([x + 0.5, y + 0.5, z + 0.5]);
                        vertices.push([x + 0.5, y - 0.5, z + 0.5]);
                        normals.push([0.0, 0.0, 1.0]);
                        normals.push([0.0, 0.0, 1.0]);
                        normals.push([0.0, 0.0, 1.0]);
                        normals.push([0.0, 0.0, 1.0]);
                        uv.push([0.0, 1.0]);
                        uv.push([0.0, 0.0]);
                        uv.push([1.0, 0.0]);
                        uv.push([1.0, 1.0]);
                        triangles.push(v);
                        triangles.push(v + 3);
                        triangles.push(v + 1);
                        triangles.push(v + 1);
                        triangles.push(v + 3);
                        triangles.push(v + 2);
                    }
                    if me > front {
                        let v = vertices.len() as u16;
                        vertices.push([x - 0.5, y - 0.5, z - 0.5]);
                        vertices.push([x - 0.5, y + 0.5, z - 0.5]);
                        vertices.push([x + 0.5, y + 0.5, z - 0.5]);
                        vertices.push([x + 0.5, y - 0.5, z - 0.5]);
                        normals.push([0.0, 0.0, -1.0]);
                        normals.push([0.0, 0.0, -1.0]);
                        normals.push([0.0, 0.0, -1.0]);
                        normals.push([0.0, 0.0, -1.0]);
                        uv.push([0.0, 0.0]);
                        uv.push([0.0, 1.0]);
                        uv.push([1.0, 1.0]);
                        uv.push([1.0, 0.0]);
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

fn main() {
    App::new()
        .add_plugins(DefaultPlugins.set(ImagePlugin::default_nearest()))
        .add_plugins(EguiPlugin)
        .add_plugins(FrameTimeDiagnosticsPlugin)
        .insert_resource(EguiSettings {
            scale_factor: 1.0,
            ..EguiSettings::default()
        })
        .insert_resource(Blocks::default())
        .add_systems(Startup, setup_player)
        .add_systems(Startup, setup_environment)
        .add_systems(Update, ui_system)
        .add_systems(Update, player_control)
        .run()
}

#[derive(Component)]
struct PlayerCamera;

#[derive(Component)]
struct IsChunk;

fn setup_player(mut commands: Commands) {
    commands.spawn((Camera3dBundle::default(), PlayerCamera));
}

#[derive(Resource, Default)]
struct Blocks {
    coords: Handle<StandardMaterial>,
    white_ore: Handle<StandardMaterial>,
    swapped: bool,
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
            for y in 0..=y {
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
    blocks.coords = coords;

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
            material: white_ore.clone(),
            transform: Transform::from_translation(Vec3::new(0.0, 0.0, -10.0)),
            ..default()
        },
        IsChunk,
    ));
}

fn player_control(
    key: Res<Input<KeyCode>>,
    btn: Res<Input<MouseButton>>,
    mut motion_evr: EventReader<MouseMotion>,
    time: Res<Time>,
    mut blocks: ResMut<Blocks>,
    mut window: Query<&mut Window, With<PrimaryWindow>>,
    mut player: Query<&mut Transform, With<PlayerCamera>>,
    mut chunk_textures: Query<&mut Handle<StandardMaterial>, With<IsChunk>>,
) {
    const MOTION_SPEED: f32 = 3.0;
    const KEYBOARD_ROTATION_SPEED: f32 = PI / 4.0;
    const SENSITIVITY: f32 = 1.0;
    let mut window = window.get_single_mut().unwrap();

    let window_width = window.resolution.physical_width() as f32;
    let window_height = window.resolution.physical_height() as f32;
    let window_scale = window_width.min(window_height);

    if btn.just_pressed(MouseButton::Left) {
        window.cursor.grab_mode = CursorGrabMode::Locked;
        window.cursor.visible = false;
    }
    if key.just_pressed(KeyCode::Escape) {
        window.cursor.grab_mode = CursorGrabMode::None;
        window.cursor.visible = true;
    }
    if key.just_pressed(KeyCode::F11) {
        window.mode = match window.mode {
            WindowMode::Windowed => WindowMode::Fullscreen,
            _ => WindowMode::Windowed,
        };
    }

    let mut player = player.get_single_mut().unwrap();
    let delta = time.delta_seconds();
    let up = player.up();
    let forward = player.forward();
    let right = player.right();
    if key.pressed(KeyCode::Space) {
        player.translation += up * MOTION_SPEED * delta;
    }
    if key.pressed(KeyCode::ShiftLeft) {
        player.translation -= up * MOTION_SPEED * delta;
    }
    if key.pressed(KeyCode::W) {
        player.translation += forward * MOTION_SPEED * delta;
    }
    if key.pressed(KeyCode::A) {
        player.translation -= right * MOTION_SPEED * delta;
    }
    if key.pressed(KeyCode::S) {
        player.translation -= forward * MOTION_SPEED * delta;
    }
    if key.pressed(KeyCode::D) {
        player.translation += right * MOTION_SPEED * delta;
    }
    if key.pressed(KeyCode::Q) {
        player.rotate_local_z(-KEYBOARD_ROTATION_SPEED * delta)
    }
    if key.pressed(KeyCode::E) {
        player.rotate_local_z(KEYBOARD_ROTATION_SPEED * delta)
    }
    if key.just_pressed(KeyCode::T) {
        for mut texture in chunk_textures.iter_mut() {
            if blocks.swapped {
                *texture = blocks.white_ore.clone();
                blocks.swapped = false;
            } else {
                *texture = blocks.coords.clone();
                blocks.swapped = true;
            }
        }
    }

    for ev in motion_evr.iter() {
        if window.cursor.grab_mode == CursorGrabMode::Locked {
            let pitch = SENSITIVITY * ev.delta.x / window_scale;
            let yaw = SENSITIVITY * ev.delta.y / window_scale;
            player.rotate_local_y(-pitch);
            player.rotate_local_x(-yaw);
        }
    }
}

fn ui_system(
    mut contexts: EguiContexts,
    diagnostics: Res<DiagnosticsStore>,
    query: Query<&Transform, With<PlayerCamera>>,
) {
    let ctx = contexts.ctx_mut();
    egui::Window::new("Graphics")
        .default_pos((0.0, 0.0))
        .show(&*ctx, |ui: &mut egui::Ui| {
            let player = query.get_single().unwrap();
            ui.label(format!(
                "pos = ({:.1}, {:.1}, {:.1})",
                player.translation.x, player.translation.y, player.translation.z
            ));
            ui.label(format!(
                "rotation = ({:.1}, {:.1}, {:.1}, {:.1})",
                player.rotation.x, player.rotation.y, player.rotation.z, player.rotation.w
            ));
            if let Some(fps) = diagnostics.get_measurement(FrameTimeDiagnosticsPlugin::FPS) {
                ui.label(format!("FPS = {:.1}", fps.value));
            }
        });
}
