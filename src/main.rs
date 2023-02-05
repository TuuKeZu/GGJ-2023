use std::f32::consts::PI;

use bevy::{
    diagnostic::FrameTimeDiagnosticsPlugin, math::*, prelude::*, time::FixedTimestep,
    window::WindowResizeConstraints,
};

use bevy_common_assets::json::JsonAssetPlugin;

pub mod components;
pub mod interpolation;
pub mod systems;
pub mod ui;

use components::*;
use systems::*;
use ui::*;

const VERSION: &str = env!("CARGO_PKG_VERSION");

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
pub const TILE_COLOR: Color = Color::rgb(0.2, 0.2, 0.2);
pub const START_COLOR: Color = Color::rgb(0., 1., 0.);
pub const END_COLOR: Color = Color::rgb(1., 0., 0.);

pub const MAP_SIZE: i32 = 16; // Map width and height are 2 * MAP_SIZE
pub const TILE_SIZE: f32 = 64.;
pub const SPRITE_SIZE: f32 = 16.; // DO NOT TOUCH!!!!!

pub const PROJECTILE_LAYER: f32 = 6.;

pub const BACKGROUND_LAYER: f32 = 1.;
pub const PATH_LAYER: f32 = 2.;

pub const ENEMY_LAYER: f32 = 5.;
pub const CURSOR_LAYER: f32 = 10.;

pub const MAX_DEFLECTION_ANGLE: f32 = 10. * PI / 180.;

pub type Texture = bevy::prelude::Handle<bevy::prelude::Image>;

pub fn window_primary() -> WindowDescriptor {
    WindowDescriptor {
        width: 1280.0,
        height: 720.0,
        resizable: true,
        title: format!("rootsTD v{VERSION}"),
        present_mode: bevy::window::PresentMode::AutoNoVsync,
        resize_constraints: WindowResizeConstraints {
            min_width: 400.0,
            min_height: 300.0,
            max_width: 1920.0,
            max_height: 1080.0,
        },

        ..default()
    }
}

fn main() {
    App::new()
        .add_plugins(
            DefaultPlugins
                .set(ImagePlugin::default_nearest())
                .set(WindowPlugin {
                    window: window_primary(),
                    ..default()
                }),
        )
        .add_plugin(FrameTimeDiagnosticsPlugin::default())
        .add_plugin(JsonAssetPlugin::<Level>::new(&["json"]))
        .insert_resource(ClearColor(BACKGROUND_COLOR))
        .insert_resource(Menu {
            current_item: MenuItem::Turret2x2,
        })
        .insert_resource(Path::default())
        .add_state(AppState::Loading)
        .add_startup_system(setup)
        .add_system_set(SystemSet::on_update(AppState::Loading).with_system(spawn_level))
        .add_system_set(
            SystemSet::new()
                .with_run_criteria(FixedTimestep::step(TIME_STEP as f64))
                .with_system(move_cursor)
                .with_system(handle_collisions)
                .with_system(handle_gunners)
                .with_system(handle_place.after(handle_collisions))
                .with_system(handle_cursor_visibility)
                .with_system(handle_sell)
                .with_system(handle_shop)
                .with_system(handle_projectiles)
                .with_system(handle_enemies)
                .with_system(animate_sprite)
                .with_system(game_tick),
        )
        .add_system(update_scoreboard)
        .add_system(bevy::window::close_on_esc)
        .add_system(update_fps)
        .run();
}

#[derive(Clone, Eq, PartialEq, Debug, Hash)]
pub enum AppState {
    Loading,
    Level,
}
