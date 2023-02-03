use std::ops::Div;

use bevy::{
    math::*,
    prelude::*,
    sprite::collide_aabb::{collide, Collision},
    time::FixedTimestep,
};

// Defines the amount of time that should elapse between each physics step.
const TIME_STEP: f32 = 1.0 / 360.;

// These constants are defined in `Transform` units.

const SCOREBOARD_FONT_SIZE: f32 = 40.0;
const SCOREBOARD_TEXT_PADDING: Val = Val::Px(5.0);

const BACKGROUND_COLOR: Color = Color::rgb(0.1, 0.1, 0.1);
const PADDLE_COLOR: Color = Color::rgb(0.3, 0.3, 0.7);
const WALL_COLOR: Color = Color::rgb(0.1, 0.5, 0.5);
const TEXT_COLOR: Color = Color::rgb(0.8, 0.8, 1.8);
const SCORE_COLOR: Color = Color::rgb(1.0, 0.5, 0.5);
const ERROR_COLOR: Color = Color::rgb(1.0, 0., 0.);

const TILE_SIZE: f32 = 16.;

/// todo
/// - [X] impl new for Cursor component
/// -[ ] create utils.rs

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
                .with_system(handle_place)
                .with_system(handle_cursor_visibility)
                .with_system(handle_collisions)
                .with_system(handle_sell),
        )
        .add_system(update_scoreboard)
        .add_system(bevy::window::close_on_esc)
        .run();
}

#[derive(Component)]
struct Collider;

#[derive(Bundle, Default)]
struct Cursor {
    sprite_bundle: SpriteBundle,
    grid_cursor: GridCursor,
}

impl Cursor {
    fn new() -> Self {
        Self {
            sprite_bundle: SpriteBundle {
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
            },
            ..default()
        }
    }
}

#[derive(Component, Default)]
struct GridCursor {
    can_place: bool,
}

#[derive(Component)]
enum Placeable {
    AssemblingMachine,
}

#[derive(Bundle)]
struct Turret {
    sprite_bundle: SpriteBundle,
}

impl Turret {
    fn new() -> (Self, Collider) {
        (
            Self {
                sprite_bundle: SpriteBundle {
                    transform: Transform::from_xyz(0., 0., 0.).with_scale(vec3(3., 3., 3.)),
                    sprite: Sprite {
                        color: WALL_COLOR,
                        ..default()
                    },
                    ..default()
                },
            },
            Collider,
        )
    }

    fn with_transform(transform: Transform) -> (Self, Collider) {
        (
            Self {
                sprite_bundle: SpriteBundle {
                    transform,
                    sprite: Sprite {
                        color: WALL_COLOR,
                        ..default()
                    },
                    ..default()
                },
            },
            Collider,
        )
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
    commands.spawn(Cursor::new()).with_children(|parent| {
        parent.spawn((Turret::new(), Placeable::AssemblingMachine));
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

fn ease(x: f32) -> f32 {
    0.5 - (x.max(0.).min(1.) * std::f32::consts::PI).cos() / 2.
}

fn move_cursor(mut query: Query<&mut Transform, With<GridCursor>>, windows: Res<Windows>) {
    let window = windows.get_primary().unwrap();
    let window_size = Vec2::new(window.width(), window.height());

    if let Some(cursor_position) = window.cursor_position() {
        let mut cursor_transform = query.get_single_mut().unwrap();
        let mut cursor_position = cursor_position - (window_size.div(2.));

        cursor_position =
            (cursor_position / TILE_SIZE).floor() * TILE_SIZE + Vec2::splat(TILE_SIZE / 2.);

        // info!("{}", cursor_position);

        let prevt = cursor_transform.translation;
        let delta = cursor_position.extend(0.) - prevt;
        let dist = delta.length();
        const EASE_DIST: f32 = 5.0;
        cursor_transform.translation = prevt + delta * ease(dist / EASE_DIST);
    } else {
        // Window is not active => game should be paused
    }
}

fn handle_cursor_visibility(
    cursor_q: Query<&GridCursor>,
    mut child_q: Query<&mut Sprite, With<Placeable>>,
) {
    let cursor = cursor_q.get_single().unwrap();
    let mut sprite = child_q.get_single_mut().unwrap();

    sprite.color = if cursor.can_place {
        WALL_COLOR
    } else {
        ERROR_COLOR
    };
}

fn handle_place(
    mut commands: Commands,
    cursor_q: Query<(&Transform, &GridCursor), Without<Placeable>>,
    child_q: Query<(&Transform, &Placeable)>,
    buttons: Res<Input<MouseButton>>,
) {
    let (cursor_transform, cursor) = cursor_q.get_single().unwrap();
    let (transform, placeable) = child_q.get_single().unwrap();

    if !cursor.can_place {
        return;
    }

    if buttons.just_pressed(MouseButton::Left) {
        //info!("yeet");
        match placeable {
            Placeable::AssemblingMachine => {
                let cursor_transform =
                    cursor_transform.with_scale(cursor_transform.scale * transform.scale);
                commands.spawn(Turret::with_transform(cursor_transform));
            }
        }
    }
}

fn handle_collisions(
    mut cursor_q: Query<(&Transform, &mut GridCursor)>,
    child_q: Query<&Transform, With<Placeable>>,
    collider_q: Query<(&Transform, &Collider), Without<GridCursor>>,
) {
    let (cursor_transform, mut cursor) = cursor_q.single_mut();
    let transform = child_q.single();
    let transform = transform.with_scale(transform.scale * cursor_transform.scale);

    for (collider_transform, _c) in &collider_q {
        let collision = collide(
            cursor_transform.translation,
            transform.scale.truncate(),
            collider_transform.translation,
            collider_transform.scale.truncate(),
        );

        cursor.can_place = collision.is_none();
    }
}

fn handle_sell(
    mut commands: Commands,
    mut cursor_q: Query<&Transform, With<GridCursor>>,
    collider_q: Query<(&Transform, Entity, &Collider), Without<GridCursor>>,
    buttons: Res<Input<MouseButton>>,
) {
    let cursor_transform = cursor_q.single_mut();

    if buttons.just_pressed(MouseButton::Right) {
        for (collider_transform, entity, _c) in &collider_q {
            if let Some(Collision::Inside) = collide(
                cursor_transform.translation,
                cursor_transform.scale.truncate(),
                collider_transform.translation,
                collider_transform.scale.truncate(),
            ) {
                commands.entity(entity).despawn();
            }
        }
    }
}

fn update_scoreboard(scoreboard: Res<Scoreboard>, mut query: Query<&mut Text>) {
    let mut text = query.single_mut();
    text.sections[1].value = scoreboard.score.to_string();
}
