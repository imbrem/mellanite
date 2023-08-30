use bevy::{
    diagnostic::{DiagnosticsStore, FrameTimeDiagnosticsPlugin},
    prelude::*,
};
use bevy_egui::{
    egui::{self},
    EguiContexts,
};

use crate::player::PlayerCamera;

pub fn ui_system(
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
