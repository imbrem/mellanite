use std::f32::consts::PI;

use bevy::{
    input::mouse::MouseMotion,
    prelude::*,
    window::{CursorGrabMode, PrimaryWindow, WindowMode},
};

#[derive(Component)]
pub struct PlayerCamera;

pub fn setup_player(mut commands: Commands) {
    commands.spawn((Camera3dBundle::default(), PlayerCamera));
}

pub fn player_control(
    key: Res<Input<KeyCode>>,
    btn: Res<Input<MouseButton>>,
    mut motion_evr: EventReader<MouseMotion>,
    time: Res<Time>,
    mut window: Query<&mut Window, With<PrimaryWindow>>,
    mut player: Query<&mut Transform, With<PlayerCamera>>,
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

    for ev in motion_evr.iter() {
        if window.cursor.grab_mode == CursorGrabMode::Locked {
            let pitch = SENSITIVITY * ev.delta.x / window_scale;
            let yaw = SENSITIVITY * ev.delta.y / window_scale;
            player.rotate_local_y(-pitch);
            player.rotate_local_x(-yaw);
        }
    }
}
