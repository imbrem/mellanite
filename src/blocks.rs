use bevy::prelude::*;

#[derive(Resource, Default)]
pub struct Blocks {
    pub coords: Handle<StandardMaterial>,
    pub white_ore: Handle<StandardMaterial>,
    pub swapped: bool,
}