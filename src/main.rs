use bevy::{math::*, prelude::*, time::FixedTimestep};

use bevy_common_assets::json::JsonAssetPlugin;

pub mod components;
pub mod interpolation;
pub mod systems;
pub mod textures;
pub mod ui;

use components::*;
use systems::*;
use textures::*;
use ui::*;

// Defines the amount of time that should elapse between each physics step.
pub const TIME_STEP: f32 = 1.0 / 360.;

// These constants are defined in `Transform` units.

pub const SCOREBOARD_FONT_SIZE: f32 = 40.0;
pub const SCOREBOARD_TEXT_PADDING: Val = Val::Px(5.0);

pub const BACKGROUND_COLOR: Color = Color::rgb(0.1, 0.1, 0.1);
pub const CURSOR_COLOR: Color = Color::rgb_linear(0.3, 0.3, 2.7);
pub const WALL_COLOR: Color = Color::rgb(1., 1., 1.);
pub const TEXT_COLOR: Color = Color::rgb(0.8, 0.8, 1.8);
pub const SCORE_COLOR: Color = Color::rgb(1.0, 0.5, 0.5);
pub const ERROR_COLOR: Color = Color::rgb(1.0, 0., 0.);
pub const START_COLOR: Color = Color::rgb(0., 1., 0.);
pub const END_COLOR: Color = Color::rgb(1., 0., 0.);

pub const TILE_SIZE: f32 = 64.;
pub const SPRITE_SIZE: f32 = 16.; // DO NOT TOUCH!!!!!

pub const ENEMY_LAYER: f32 = 1.;

pub type Texture = bevy::prelude::Handle<bevy::prelude::Image>;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins.set(ImagePlugin::default_nearest()))
        .add_plugin(JsonAssetPlugin::<Level>::new(&["json"]))
        .init_resource::<SpriteHandles>()
        .insert_resource(ClearColor(BACKGROUND_COLOR))
        .insert_resource(Menu {
            current_item: MenuItem::Turret2x2,
        })
        .insert_resource(Path::default())
        .add_state(AppState::Loading)
        .add_system_set(SystemSet::on_enter(AppState::Loading).with_system(load_textures))
        .add_system_set(SystemSet::on_update(AppState::Loading).with_system(check_textures)) // sets state = Setup
        .add_system_set(
            SystemSet::on_enter(AppState::Setup)
                .with_system(setup)
                .label("setup"),
        )
        .add_system_set(SystemSet::on_update(AppState::Setup).with_system(spawn_level)) // sets state = Level
        .add_system_set(
            SystemSet::on_update(AppState::Level)
                .with_run_criteria(FixedTimestep::step(TIME_STEP as f64))
                .with_system(move_cursor.after("setup")),
            // .with_system(handle_collisions)
            // .with_system(handle_place.after(handle_collisions)),
            // .with_system(handle_cursor_visibility)
            // .with_system(handle_sell)
            // .with_system(handle_shop)
            // .with_system(animate_sprite)
            // .with_system(game_tick.after(animate_sprite))
            // .with_system(update_scoreboard),
        )
        .add_system(bevy::window::close_on_esc)
        .run();
}

#[derive(Clone, Eq, PartialEq, Debug, Hash)]
pub enum AppState {
    Loading,
    Setup,
    Level,
}
