//! Sauce https://github.com/bevyengine/bevy/blob/latest/examples/2d/texture_atlas.rs

use bevy::{asset::LoadState, prelude::*};

use crate::AppState;

#[derive(Resource, Default)]
pub struct SpriteHandles {
    pub handles: Vec<HandleUntyped>,
}

pub fn load_textures(mut sprite_handles: ResMut<SpriteHandles>, asset_server: Res<AssetServer>) {
    sprite_handles.handles = asset_server.load_folder("resources/").unwrap();
}

pub fn check_textures(
    mut state: ResMut<State<AppState>>,
    sprite_handles: ResMut<SpriteHandles>,
    asset_server: Res<AssetServer>,
) {
    if let LoadState::Loaded =
        asset_server.get_group_load_state(sprite_handles.handles.iter().map(|handle| handle.id))
    {
        state.set(AppState::Setup).unwrap();
    }
}
