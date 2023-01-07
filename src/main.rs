

//! A simplified implementation of the classic game "Breakout".


use std::time::Duration;

use bevy::{
    prelude::*,
    sprite::collide_aabb::{collide, Collision},
    sprite::MaterialMesh2dBundle,
    time::FixedTimestep, utils::Instant,
};

// Defines the amount of time that should elapse between each physics step.
const TIME_STEP: f32 = 1.0 / 60.0;

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
const WALL_COLOR: Color = Color::rgb(0.8, 0.8, 0.8);
const TEXT_COLOR: Color = Color::rgb(0.5, 0.5, 1.0);
const SCORE_COLOR: Color = Color::rgb(1.0, 0.5, 0.5);

const CENTER_OBJECT_SIZE: Vec2 = Vec2::new(50., 50.);

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .insert_resource(Scoreboard { score: 0 })
        .insert_resource(ClearColor(BACKGROUND_COLOR))
        .add_startup_system(setup)
        .add_event::<CollisionEvent>()
        .add_system_set(
            SystemSet::new()
                .with_run_criteria(FixedTimestep::step(TIME_STEP as f64))
                .with_system(check_for_collisions)
                .with_system(move_paddle.after(check_for_collisions))
                .with_system(apply_velocity.after(check_for_collisions))
                .with_system(play_collision_sound.after(check_for_collisions)),
        )
        .add_system(update_scoreboard)
        .add_system(bevy::window::close_on_esc)
        .run();
}

#[derive(Component)]
struct Paddle;

#[derive(Component, Deref, DerefMut, Debug)]
struct Velocity(Vec2);

#[derive(Component)]
struct Collider;

#[derive(Default)]
struct CollisionEvent;

#[derive(Component)]
struct Brick;

#[derive(Resource)]
struct CollisionSound(Handle<AudioSource>);

// This bundle is a collection of the components that define a "wall" in our game
#[derive(Bundle)]
struct WallBundle {
    // You can nest bundles inside of other bundles like this
    // Allowing you to compose their functionality
    sprite_bundle: SpriteBundle,
    collider: Collider,
}

/// Which side of the arena is this wall located on?
enum WallLocation {
    Left,
    Right,
    Bottom,
    Top,
    Center
}

impl WallLocation {
    fn position(&self) -> Vec2 {
        match self {
            WallLocation::Left => Vec2::new(LEFT_WALL, 0.),
            WallLocation::Right => Vec2::new(RIGHT_WALL, 0.),
            WallLocation::Bottom => Vec2::new(0., BOTTOM_WALL),
            WallLocation::Top => Vec2::new(0., TOP_WALL),
            WallLocation::Center => Vec2::new(0., 0.),
        }
    }

    fn size(&self) -> Vec2 {
        let arena_height = TOP_WALL - BOTTOM_WALL;
        let arena_width = RIGHT_WALL - LEFT_WALL;
        // Make sure we haven't messed up our constants
        assert!(arena_height > 0.0);
        assert!(arena_width > 0.0);

        match self {
            WallLocation::Left | WallLocation::Right => {
                Vec2::new(WALL_THICKNESS, arena_height + WALL_THICKNESS)
            }
            WallLocation::Bottom | WallLocation::Top => {
                Vec2::new(arena_width + WALL_THICKNESS, WALL_THICKNESS)
            }
            WallLocation::Center => {
                CENTER_OBJECT_SIZE
            },
        }
    }
}

impl WallBundle {
    // This "builder method" allows us to reuse logic across our wall entities,
    // making our code easier to read and less prone to bugs when we change the logic
    fn new(location: WallLocation) -> WallBundle {
        WallBundle {
            sprite_bundle: SpriteBundle {
                transform: Transform {
                    // We need to convert our Vec2 into a Vec3, by giving it a z-coordinate
                    // This is used to determine the order of our sprites
                    translation: location.position().extend(0.0),
                    // The z-scale of 2D objects must always be 1.0,
                    // or their ordering will be affected in surprising ways.
                    // See https://github.com/bevyengine/bevy/issues/4149
                    scale: location.size().extend(1.0),
                    ..default()
                },
                sprite: Sprite {
                    color: WALL_COLOR,
                    ..default()
                },
                ..default()
            },
            collider: Collider,
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

    // Sound
    let ball_collision_sound = asset_server.load("sounds/breakout_collision.ogg");
    commands.insert_resource(CollisionSound(ball_collision_sound));

    // Paddle
    let paddle_y = BOTTOM_WALL + GAP_BETWEEN_PADDLE_AND_FLOOR;

    commands.spawn((
        SpriteBundle {
            transform: Transform {
                translation: Vec3::new(0.0, paddle_y, 0.0),
                scale: PADDLE_SIZE,
                ..default()
            },
            sprite: Sprite {
                color: PADDLE_COLOR,
                ..default()
            },
            ..default()
        },
        Paddle,
        Velocity(Vec2::new(0., 0.)),
    ))
    .with_children(|parent| {
        parent.spawn(
            SpriteBundle {
                transform: Transform {
                    translation: Vec3::new(0.0, 0.0, 0.0),
                    scale: Vec3::new(5., 10., 0.),
                    ..default()
                },
                sprite: Sprite {
                    color: Color::RED,
                    ..default()
                },
                ..default()
            }
        );
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

    // Walls
    commands.spawn(WallBundle::new(WallLocation::Left));
    commands.spawn(WallBundle::new(WallLocation::Right));
    commands.spawn(WallBundle::new(WallLocation::Bottom));
    commands.spawn(WallBundle::new(WallLocation::Top));
    commands.spawn(WallBundle::new(WallLocation::Center));
}

fn move_paddle(
    mut keyboard_input: ResMut<Input<KeyCode>>,
    mut query: Query<&mut Velocity, With<Paddle>>,
) {
    let mut paddle_velocity = query.single_mut();
    let mut new_velocity = paddle_velocity.clone().normalize_or_zero();
    
    
    if keyboard_input.any_pressed([KeyCode::D, KeyCode::A, KeyCode::W, KeyCode::S]) {

        if keyboard_input.pressed(KeyCode::D) {
            new_velocity.x = f32::clamp(new_velocity.x + 1., -1., 1.);
            paddle_velocity.x = new_velocity.x * PADDLE_SPEED;
        }
    
        if keyboard_input.pressed(KeyCode::A) {
            new_velocity.x = f32::clamp(new_velocity.x - 1., -1., 1.);
            paddle_velocity.x = new_velocity.x * PADDLE_SPEED;
        }
    
        if keyboard_input.pressed(KeyCode::W) {
            new_velocity.y = f32::clamp(new_velocity.y + 1., -1., 1.);
            paddle_velocity.y = new_velocity.y * PADDLE_SPEED;
        }
    
        if keyboard_input.pressed(KeyCode::S) {
            new_velocity.y = f32::clamp(new_velocity.y - 1., -1., 1.);
            paddle_velocity.y = new_velocity.y * PADDLE_SPEED;
        }
    }
    
}

fn apply_velocity(mut query: Query<(&mut Transform, &mut Velocity)>) {
    for (mut transform, mut velocity) in &mut query {
        transform.translation.x += velocity.x * TIME_STEP;
        transform.translation.y += velocity.y * TIME_STEP;

        **velocity *= Vec2::new(0.8, 0.8); // drag
    }
}

fn update_scoreboard(scoreboard: Res<Scoreboard>, mut query: Query<&mut Text>) {
    let mut text = query.single_mut();
    text.sections[1].value = scoreboard.score.to_string();
}

fn check_for_collisions(
    mut commands: Commands,
    mut scoreboard: ResMut<Scoreboard>,
    mut paddle_query: Query<(&mut Velocity, &Transform), With<Paddle>>,
    collider_query: Query<(Entity, &Transform, Option<&Collider>), With<Collider>>,
    mut collision_events: EventWriter<CollisionEvent>,
) {

    let (mut paddle_velocity, paddle_transform) = paddle_query.single_mut();
    let size = paddle_transform.scale.truncate() + Vec2::new(5., 5.);

    // check collision with walls
    for (_, transform, _) in &collider_query {

        let collision_paddle = collide(
            transform.translation,
            transform.scale.truncate(),
            paddle_transform.translation,
            size,
        );

        if let Some(collision) = collision_paddle {
            let normal = paddle_velocity.normalize_or_zero();

            match collision {
                Collision::Left => paddle_velocity.x = f32::clamp(normal.x, 0., 1.) * PADDLE_SPEED,
                Collision::Right => paddle_velocity.x = f32::clamp(normal.x, -1., 0.) * PADDLE_SPEED,
                Collision::Top  => paddle_velocity.y = f32::clamp(normal.y, -1., 0.) * PADDLE_SPEED,
                Collision::Bottom => paddle_velocity.y = f32::clamp(normal.y, 0., 1.) * PADDLE_SPEED,
                Collision::Inside => { /* do nothing */ }
            }
        }
    }
}

fn play_collision_sound(
    collision_events: EventReader<CollisionEvent>,
    audio: Res<Audio>,
    sound: Res<CollisionSound>,
) {
    // Play a sound once per frame if a collision occurred.
    if !collision_events.is_empty() {
        // This prevents events staying active on the next frame.
        collision_events.clear();
        audio.play(sound.0.clone());
    }
}