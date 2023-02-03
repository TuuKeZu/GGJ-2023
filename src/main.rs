//! A simplified implementation of the classic game "Breakout".

use std::{any::Any, ops::Div, time::Duration};

use bevy::{
    prelude::*,
    sprite::collide_aabb::{collide, Collision},
    sprite::MaterialMesh2dBundle,
    time::FixedTimestep,
    transform,
    utils::Instant,
};

// Defines the amount of time that should elapse between each physics step.
const TIME_STEP: f32 = 1.0 / 240.;

// These constants are defined in `Transform` units.
// Using the default 2D camera they correspond 1:1 with screen pixels.
const PADDLE_SIZE: Vec3 = Vec3::new(20.0, 20.0, 0.0);
const GAP_BETWEEN_PADDLE_AND_FLOOR: f32 = 60.0;
const PADDLE_SPEED: f32 = 250.0;
// How close can the paddle get to the wall

const WALL_THICKNESS: f32 = 10.0;
// x coordinates
const LEFT_WALL: f32 = -450.;
const RIGHT_WALL: f32 = 450.;
// y coordinates
const BOTTOM_WALL: f32 = -300.;
const TOP_WALL: f32 = 300.;

const SCOREBOARD_FONT_SIZE: f32 = 40.0;
const SCOREBOARD_TEXT_PADDING: Val = Val::Px(5.0);

const BACKGROUND_COLOR: Color = Color::rgb(0.9, 0.9, 0.9);
const PADDLE_COLOR: Color = Color::rgb(0.3, 0.3, 0.7);
const WALL_COLOR: Color = Color::rgb(0.1, 0.5, 0.5);
const TEXT_COLOR: Color = Color::rgb(0.5, 0.5, 1.0);
const SCORE_COLOR: Color = Color::rgb(1.0, 0.5, 0.5);

const CENTER_OBJECT_SIZE: Vec2 = Vec2::new(50., 50.);

const TILE_SIZE: f32 = 32.;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .insert_resource(Scoreboard { score: 0 })
        .insert_resource(ClearColor(BACKGROUND_COLOR))
        .add_startup_system(setup)
        .add_system_set(
            SystemSet::new()
                .with_run_criteria(FixedTimestep::step(TIME_STEP as f64))
                .with_system(move_cursor)
                .with_system(handle_click),
        )
        .add_system(update_scoreboard)
        .add_system(bevy::window::close_on_esc)
        .run();
}

#[derive(Component, Default)]
struct GridCursor {
    is_locked: bool,
}

#[derive(Component)]
enum Placeable {
    AssemblingMachine,
}

#[derive(Bundle)]
struct AssemblingMachine {
    sprite_bundle: SpriteBundle,
}

impl AssemblingMachine {
    fn new() -> Self {
        Self {
            sprite_bundle: SpriteBundle {
                transform: Transform::from_xyz(0., 0., 0.).with_scale(Vec3::new(3., 3., 3.)),
                sprite: Sprite {
                    color: WALL_COLOR,
                    ..default()
                },
                ..default()
            },
        }
    }

    fn with_transform(transform: Transform) -> Self {
        Self {
            sprite_bundle: SpriteBundle {
                transform,
                sprite: Sprite {
                    color: WALL_COLOR,
                    ..default()
                },
                ..default()
            },
        }
    }
}

// This resource tracks the game's score
#[derive(Resource)]
struct Scoreboard {
    score: usize,
}

// Add the game's entities to our world
fn setup(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
    asset_server: Res<AssetServer>,
) {
    // Camera
    commands.spawn(Camera2dBundle::default());

    // Cursor
    commands
        .spawn((
            {
                SpriteBundle {
                    transform: Transform::from_xyz(0., 0., 0.)
                        .with_rotation(Quat::from_euler(EulerRot::XYZ, 0., 0., 0.))
                        .with_scale(Vec3 {
                            x: TILE_SIZE,
                            y: TILE_SIZE,
                            z: 0.,
                        }),
                    sprite: Sprite {
                        color: PADDLE_COLOR,
                        ..default()
                    },
                    ..default()
                }
            },
            GridCursor { is_locked: true },
        ))
        .with_children(|parent| {
            parent.spawn((AssemblingMachine::new(), Placeable::AssemblingMachine));
        });

    // Scoreboard
    commands.spawn(
        TextBundle::from_sections([
            TextSection::new(
                "Score: ",
                TextStyle {
                    font: asset_server.load("fonts/FiraSans-Bold.ttf"),
                    font_size: SCOREBOARD_FONT_SIZE,
                    color: TEXT_COLOR,
                },
            ),
            TextSection::from_style(TextStyle {
                font: asset_server.load("fonts/FiraMono-Medium.ttf"),
                font_size: SCOREBOARD_FONT_SIZE,
                color: SCORE_COLOR,
            }),
        ])
        .with_style(Style {
            position_type: PositionType::Absolute,
            position: UiRect {
                top: SCOREBOARD_TEXT_PADDING,
                left: SCOREBOARD_TEXT_PADDING,
                ..default()
            },
            ..default()
        }),
    );
}

fn move_cursor(mut query: Query<(&mut Transform, &mut GridCursor)>, windows: Res<Windows>) {
    let window = windows.get_primary().unwrap();
    let window_size = Vec2::new(window.width(), window.height());

    if let Some(cursor_position) = window.cursor_position() {
        let (mut cursor_transform, cursor) = query.get_single_mut().unwrap();
        let mut cursor_position = cursor_position - (window_size.div(2.));

        if cursor.is_locked {
            cursor_position = Vec2 {
                x: f32::floor(cursor_position.x / TILE_SIZE) * TILE_SIZE,
                y: f32::floor(cursor_position.y / TILE_SIZE) * TILE_SIZE,
            } + Vec2::new(TILE_SIZE / 2., TILE_SIZE / 2.);
        }

        // info!("{}", cursor_position);

        cursor_transform.translation = cursor_position.extend(0.);
    } else {
        // Window is not active => game should be paused
    }
}

fn handle_click(
    mut commands: Commands,
    mut cursor_q: Query<(&mut Transform, &mut GridCursor, &mut Children), Without<Placeable>>,
    child_q: Query<(&Transform, &Placeable)>,
    buttons: Res<Input<MouseButton>>,
) {
    if buttons.just_pressed(MouseButton::Left) {
        if let Ok((cursor_transform, cursor, mut children)) = cursor_q.get_single_mut() {
            //info!("yeet");
            let child = children.iter().next().unwrap();
            let (transform, placeable) = child_q.get(*child).unwrap();

            match placeable {
                Placeable::AssemblingMachine => {
                    let cursor_transform =
                        cursor_transform.with_scale(cursor_transform.scale * transform.scale);
                    commands.spawn(AssemblingMachine::with_transform(cursor_transform));
                }
            }
        } else {
            // no item in hand
        }
    }
}

fn update_scoreboard(scoreboard: Res<Scoreboard>, mut query: Query<&mut Text>) {
    let mut text = query.single_mut();
    text.sections[1].value = scoreboard.score.to_string();
}
