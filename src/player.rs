use std::f32::consts::PI;

use bevy::{
    input::mouse::MouseMotion,
    prelude::*,
    window::{CursorGrabMode, PrimaryWindow, WindowMode},
};
use bevy_rapier3d::prelude::*;

#[derive(Component)]
pub struct PlayerCamera;

pub fn setup_player(mut commands: Commands) {
    commands.spawn((
        Camera3dBundle::default(),
        KinematicCharacterController::default(),
        Collider::capsule_y(1.0, 0.5),
        PlayerCamera,
    ));
}

pub fn player_control(
    key: Res<Input<KeyCode>>,
    btn: Res<Input<MouseButton>>,
    mut motion_evr: EventReader<MouseMotion>,
    time: Res<Time>,
    mut window: Query<&mut Window, With<PrimaryWindow>>,
    mut player: Query<(&mut Transform, &mut KinematicCharacterController), With<PlayerCamera>>,
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

    let (mut player_position, mut player_controller) = player.get_single_mut().unwrap();
    let delta = time.delta_seconds();
    let up = player_position.up();
    let forward = player_position.forward();
    let right = player_position.right();
    if key.pressed(KeyCode::Space) {
        player_controller.translation = Some(up * MOTION_SPEED * delta);
    }
    if key.pressed(KeyCode::ShiftLeft) {
        player_controller.translation = Some(-up * MOTION_SPEED * delta);
    }
    if key.pressed(KeyCode::W) {
        player_controller.translation = Some(forward * MOTION_SPEED * delta);
    }
    if key.pressed(KeyCode::A) {
        player_controller.translation = Some(-right * MOTION_SPEED * delta);
    }
    if key.pressed(KeyCode::S) {
        player_controller.translation = Some(-forward * MOTION_SPEED * delta);
    }
    if key.pressed(KeyCode::D) {
        player_controller.translation = Some(right * MOTION_SPEED * delta);
    }
    if key.pressed(KeyCode::Q) {
        player_position.rotate_local_z(-KEYBOARD_ROTATION_SPEED * delta)
    }
    if key.pressed(KeyCode::E) {
        player_position.rotate_local_z(KEYBOARD_ROTATION_SPEED * delta)
    }

    for ev in motion_evr.iter() {
        if window.cursor.grab_mode == CursorGrabMode::Locked {
            let pitch = SENSITIVITY * ev.delta.x / window_scale;
            let yaw = SENSITIVITY * ev.delta.y / window_scale;
            player_position.rotate_local_y(-pitch);
            player_position.rotate_local_x(-yaw);
        }
    }
}
