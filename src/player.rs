use std::f32::consts::PI;

use bevy::{
    input::mouse::MouseMotion,
    prelude::{shape::Cube, *},
    window::{CursorGrabMode, PrimaryWindow, WindowMode},
};
use bevy_rapier3d::prelude::*;

#[derive(Component)]
pub struct PlayerCamera;

#[derive(Component)]
pub struct HighlightedBlock;

pub fn setup_player(
    mut commands: Commands,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut meshes: ResMut<Assets<Mesh>>,
) {
    commands.spawn((
        Camera3dBundle::default(),
        KinematicCharacterController::default(),
        Collider::capsule_y(1.0, 0.5),
        PlayerCamera,
    ));

    commands.spawn((
        PbrBundle {
            mesh: meshes.add(Cube { size: 0.3 }.into()),
            material: materials.add(StandardMaterial {
                unlit: true,
                fog_enabled: false,
                alpha_mode: AlphaMode::Blend,
                ..default()
            }),
            visibility: Visibility::Hidden,
            ..default()
        },
        HighlightedBlock,
    ));
}

pub fn player_control(
    key: Res<Input<KeyCode>>,
    btn: Res<Input<MouseButton>>,
    mut motion_evr: EventReader<MouseMotion>,
    time: Res<Time>,
    rapier_context: Res<RapierContext>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut window: Query<&mut Window, With<PrimaryWindow>>,
    mut player: Query<(&mut Transform, &mut KinematicCharacterController), With<PlayerCamera>>,
    mut highlight: Query<
        (&mut Transform, &Handle<StandardMaterial>, &mut Visibility),
        (With<HighlightedBlock>, Without<PlayerCamera>),
    >,
) {
    const MOTION_SPEED: f32 = 6.0;
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

    let mut highlight = highlight.get_single_mut().unwrap();
    let ray_origin = player_position.translation + 0.5 * player_position.forward();
    match rapier_context.cast_ray(ray_origin, player_position.forward(), 8.0, false, default()) {
        Some((_entity, toi)) if toi > 0.5 => {
            highlight.0.translation = ray_origin + toi * player_position.forward();
            if btn.just_pressed(MouseButton::Left) {
                materials.get_mut(highlight.1).unwrap().base_color = Color::RED;
            } else if btn.just_pressed(MouseButton::Right) {
                materials.get_mut(highlight.1).unwrap().base_color = Color::BLUE;
            } else if btn.just_pressed(MouseButton::Middle) {
                materials.get_mut(highlight.1).unwrap().base_color = Color::GREEN;
            } else {
                materials.get_mut(highlight.1).unwrap().base_color =
                    Color::rgba(1.0 - toi / 7.0, 1.0, 1.0, 1.0 - toi / 7.0);
            }
            *highlight.2 = Visibility::Visible
        }
        _ => *highlight.2 = Visibility::Hidden,
    }

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
