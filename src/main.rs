use std::ops::Div;
use std::time::Duration;
use std::{borrow::Cow, collections::VecDeque};

use bevy::{
    core_pipeline::bloom::BloomSettings,
    math::*,
    prelude::*,
    render::{
        render_resource::{
            BindGroupLayoutDescriptor, BindGroupLayoutEntry, BindingType, BufferBindingType,
            ComputePipelineDescriptor, Extent3d, PipelineCache, ShaderStages, TextureDimension,
            TextureFormat,
        },
        renderer::RenderDevice,
    },
    sprite::collide_aabb::{collide, Collision},
    time::FixedTimestep,
};

use bevy::reflect::TypeUuid;
use bevy_common_assets::json::JsonAssetPlugin;

// Defines the amount of time that should elapse between each physics step.
const TIME_STEP: f32 = 1.0 / 360.;
const TICK_STEP: f32 = 1.0 / 5.;

// These constants are defined in `Transform` units.

const SCOREBOARD_FONT_SIZE: f32 = 40.0;
const SCOREBOARD_TEXT_PADDING: Val = Val::Px(5.0);

const BACKGROUND_COLOR: Color = Color::rgb(0.1, 0.1, 0.1);
const CURSOR_COLOR: Color = Color::rgb_linear(0.3, 0.3, 2.7);
const WALL_COLOR: Color = Color::rgb(1., 1., 1.);
const TEXT_COLOR: Color = Color::rgb(0.8, 0.8, 1.8);
const SCORE_COLOR: Color = Color::rgb(1.0, 0.5, 0.5);
const ERROR_COLOR: Color = Color::rgb(1.0, 0., 0.);
const TILE_COLOR: Color = Color::rgb(0.2, 0.2, 0.2);
const START_COLOR: Color = Color::rgb(0., 1., 0.);
const END_COLOR: Color = Color::rgb(1., 0., 0.);

const TILE_SIZE: f32 = 32.;
const SPRITE_SIZE: f32 = 16.;

type Texture = bevy::prelude::Handle<bevy::prelude::Image>;

/// todo
/// - [X] impl new for Cursor component
/// -[ ] create utils.rs

fn main() {
    App::new()
        .add_plugins(DefaultPlugins.set(ImagePlugin::default_nearest()))
        .add_plugin(JsonAssetPlugin::<Level>::new(&["json"]))
        .insert_resource(Scoreboard { score: 0 })
        .insert_resource(ClearColor(BACKGROUND_COLOR))
        .insert_resource(Path::default())
        .add_state(AppState::Loading)
        .add_startup_system(setup)
        .add_system_set(SystemSet::on_update(AppState::Loading).with_system(spawn_level))
        .add_system_set(
            SystemSet::new()
                .with_run_criteria(FixedTimestep::step(TIME_STEP as f64))
                .with_system(move_cursor)
                .with_system(handle_collisions)
                .with_system(handle_place.after(handle_collisions))
                .with_system(handle_cursor_visibility)
                .with_system(handle_sell),
        )
        .add_system_set(
            SystemSet::new()
                .with_run_criteria(FixedTimestep::step(TICK_STEP as f64))
                .with_system(game_tick),
        )
        .add_system(update_scoreboard)
        .add_system(bevy::window::close_on_esc)
        .run();
}

#[derive(Clone, Eq, PartialEq, Debug, Hash)]
enum AppState {
    Loading,
    Level,
}

#[derive(serde::Deserialize, TypeUuid, Debug)]
#[uuid = "413be529-bfeb-41b3-9db0-4b8b380a2c46"]
struct Level {
    path: Vec<[f32; 2]>,
}

#[derive(Resource, Debug)]
struct LevelHandle(Handle<Level>);

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
                    color: CURSOR_COLOR,
                    ..default()
                },
                ..default()
            },
            ..default()
        }
    }
}

#[derive(Component, Debug, Default)]
struct GridCursor {
    can_place: bool,
    last_target_pos: Vec2,
    last_sample: f32,
}

#[derive(Component)]
enum Placeable {
    Turret,
}

#[derive(Bundle)]
struct Turret {
    sprite_bundle: SpriteBundle,
    collider: Collider,
}

impl Turret {
    fn new() -> Self {
        Self {
            sprite_bundle: SpriteBundle {
                transform: Transform::from_xyz(0., 0., 0.)
                    .with_scale(Vec3::splat(2. / SPRITE_SIZE)),
                sprite: Sprite {
                    color: WALL_COLOR,
                    ..default()
                },
                ..default()
            },
            collider: Collider,
        }
    }

    fn with_transform(mut self, transform: Transform) -> Self {
        self.sprite_bundle.transform = transform;
        self
    }

    fn with_texture(mut self, texture: Texture) -> Self {
        self.sprite_bundle.texture = texture;
        self
    }
}

#[derive(Bundle)]
struct Tile {
    sprite_bundle: SpriteBundle,
    collider: Collider,
}

impl Tile {
    fn new() -> Self {
        Self {
            sprite_bundle: SpriteBundle {
                transform: Transform::from_xyz(0., 0., 0.).with_scale(Vec3::splat(TILE_SIZE)),
                sprite: Sprite {
                    color: TILE_COLOR,
                    ..default()
                },
                ..default()
            },
            collider: Collider,
        }
    }

    fn with_position(mut self, translation: Vec3) -> Self {
        self.sprite_bundle.transform.translation = translation;
        self
    }

    fn with_sprite(mut self, sprite: Sprite) -> Self {
        self.sprite_bundle.sprite = sprite;
        self
    }
}

#[derive(Bundle)]
struct PotatoBundle {
    sprite_bundle: SpriteBundle,
    potato: Potato,
}

impl PotatoBundle {
    fn new(potato: Potato) -> Self {
        Self {
            sprite_bundle: SpriteBundle {
                transform: Transform::from_xyz(0., 0., 0.).with_scale(Vec3::splat(1.)),
                ..default()
            },
            potato,
        }
    }

    fn with_texture(mut self, texture: Texture) -> Self {
        self.sprite_bundle.texture = texture;
        self
    }

    fn with_position(mut self, translation: Vec3) -> Self {
        self.sprite_bundle.transform.translation = translation;
        self
    }
}

#[derive(Component)]
struct Potato {
    idx: usize,
}

#[derive(Resource, Default, Debug)]
struct Path {
    start_position: Vec2,
    end_position: Vec2,
    positions: VecDeque<PathTile>,
}

#[derive(Default, Debug)]
struct PathTile {
    state: TileState,
    position: Vec2,
}

impl PathTile {
    fn new(position: Vec2) -> Self {
        Self {
            position,
            ..default()
        }
    }
}

#[derive(Debug, PartialEq)]
enum TileState {
    Free,
    Occupied,
}

impl Default for TileState {
    fn default() -> Self {
        Self::Free
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
    windows: Res<Windows>,
    asset_server: Res<AssetServer>,
    mut images: ResMut<Assets<Image>>,
) {
    // level
    let level = LevelHandle(asset_server.load("map.json"));
    commands.insert_resource(level);

    // grid
    let window = windows.get_primary().unwrap();
    let size = Extent3d {
        width: window.width() as u32,
        height: window.height() as u32,
        ..default()
    };
    let mut image = Image::new_fill(
        size,
        TextureDimension::D2,
        &[0, 0, 0, 0],
        TextureFormat::Rgba8Unorm,
    );
    info!("{size:?}");
    for (y, row) in image
        .data
        .chunks_mut(window.width() as usize * 4)
        .enumerate()
    {
        for (x, pixel) in row.chunks_mut(4).enumerate() {
            match (y % TILE_SIZE as usize, (x + 1) % TILE_SIZE as usize) {
                (7..=8, _) | (_, 0..=1) => {
                    pixel[0] = 15;
                    pixel[1] = 15;
                    pixel[2] = 15;
                    pixel[3] = 255;
                }
                _ => {}
            };
        }
    }

    let image_handle = images.add(image);

    commands.spawn(SpriteBundle {
        // transform: Transform::from_xyz(0., 0., -1.).with_scale(vec3(1., 1., 1.)),
        sprite: Sprite {
            custom_size: Some(Vec2::new(window.width(), window.height())),
            ..default()
        },
        texture: image_handle,
        ..default()
    });

    // Camera
    commands.spawn((
        Camera2dBundle {
            camera: Camera {
                hdr: true, // 1. HDR must be enabled on the camera
                ..default()
            },
            ..default()
        },
        BloomSettings {
            threshold: 0.2,
            ..default()
        },
    ));

    // Cursor
    commands.spawn(Cursor::new()).with_children(|parent| {
        parent.spawn((
            Turret::new().with_texture(asset_server.load("resources/turret-1.png")),
            Placeable::Turret,
        ));
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

fn spawn_level(
    mut commands: Commands,
    level: Res<LevelHandle>,
    mut levels: ResMut<Assets<Level>>,
    mut state: ResMut<State<AppState>>,
    mut query: Query<&mut Transform, With<Camera>>,
    mut path: ResMut<Path>,
) {
    let mut camera_transform = query.get_single_mut().unwrap();
    if let Some(level) = levels.remove(level.0.id()) {
        let positions: Vec<Vec2> = level
            .path
            .iter()
            .map(|pos| vec2(pos[0], pos[1]) * Vec2::splat(TILE_SIZE) + Vec2::splat(TILE_SIZE / 2.))
            .collect();

        let mut tiles = positions.iter();

        let start = tiles.next().unwrap();
        path.start_position = *start;
        path.positions.push_back(PathTile::new(*start));

        commands.spawn(
            Tile::new()
                .with_sprite(Sprite {
                    color: START_COLOR,
                    ..default()
                })
                .with_position(start.extend(0.)),
        );

        let end = tiles.next_back().unwrap();
        path.end_position = *end;
        path.positions.push_front(PathTile::new(*end));

        commands.spawn(
            Tile::new()
                .with_sprite(Sprite {
                    color: END_COLOR,
                    ..default()
                })
                .with_position(end.extend(0.)),
        );

        // Move camera to middle of the map based on naive assumptions
        camera_transform.translation += (*end - *start).extend(0.) / 2.;

        for pos in tiles {
            path.positions.push_back(PathTile::new(*pos));
            commands.spawn(Tile::new().with_position(pos.extend(0.)));
        }

        info!("{:#?}", path);
        state.set(AppState::Level).unwrap();
    }
}

fn ease_old(x: f32) -> f32 {
    0.5 - (x.max(0.).min(1.) * std::f32::consts::PI).cos() / 2.
}

fn ease(mut x: f32) -> f32 {
    x = x.max(0.).min(1.);
    x.powi(2) * (x - 2.).powi(2)
}

fn move_cursor(
    mut query: Query<(&mut Transform, &mut GridCursor), Without<Camera>>,
    camera_q: Query<&Transform, With<Camera>>,
    windows: Res<Windows>,
    time: Res<Time>,
) {
    let window = windows.get_primary().unwrap();
    let window_size = Vec2::new(window.width(), window.height());
    let camera_transform = camera_q.get_single().unwrap();

    // TODO clean this ugly mess
    if let Some(cursor_position) = window.cursor_position() {
        let elapsed = time.elapsed().as_secs_f32();
        let (mut cursor_transform, mut grid_cursor) = query.get_single_mut().unwrap();
        let mut target_pos_grid =
            cursor_position - (window_size / 2.) + camera_transform.translation.truncate();

        target_pos_grid =
            (target_pos_grid / TILE_SIZE).floor() * TILE_SIZE + Vec2::splat(TILE_SIZE / 2.);

        let prev_pos = cursor_transform.translation.xy();
        let delta = target_pos_grid - prev_pos;

        const EASE_TIME: f32 = 1.0;
        let dt = elapsed - grid_cursor.last_sample;

        info!("{}", grid_cursor.can_place);
        let newp = prev_pos + delta * ease(dt / EASE_TIME);
        cursor_transform.translation = newp.extend(0.);
        if grid_cursor.last_target_pos == newp {
            // if the cursor hasn't moved
        }
        if grid_cursor.last_target_pos != target_pos_grid {
            // if the target grid has changed
            grid_cursor.last_target_pos = target_pos_grid;
            grid_cursor.last_sample = elapsed - 0.1;
        }
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
    asset_server: Res<AssetServer>,
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
            Placeable::Turret => {
                let texture = asset_server.load("resources/potato.png");
                let cursor_transform =
                    Transform::from_xyz(cursor.last_target_pos.x, cursor.last_target_pos.y, 0.)
                        .with_scale(cursor_transform.scale * transform.scale);
                commands.spawn(
                    Turret::new()
                        .with_transform(cursor_transform)
                        .with_texture(texture),
                );
            }
        }
    }
}

fn handle_collisions(
    mut cursor_q: Query<&mut GridCursor>,
    child_q: Query<&Transform, With<Placeable>>,
    collider_q: Query<(&Transform, &Collider), Without<GridCursor>>,
) {
    let mut cursor = cursor_q.single_mut();
    let transform = child_q.single();

    let cursor_transform = cursor.last_target_pos.extend(0.);
    let transform = transform.with_scale((transform.scale * SPRITE_SIZE) * TILE_SIZE);

    let mut colliding = false;

    for (collider_transform, _c) in &collider_q {
        let collision = collide(
            cursor_transform,
            transform.scale.truncate(),
            collider_transform.translation,
            collider_transform.scale.truncate(),
        );

        if collision.is_some() {
            colliding = true;
            break;
        }
    }

    cursor.can_place = !colliding;
}

fn handle_sell(
    mut commands: Commands,
    cursor_q: Query<&GridCursor>,
    collider_q: Query<(&Transform, Entity, &Collider), Without<GridCursor>>,
    buttons: Res<Input<MouseButton>>,
) {
    let grid_cursor = cursor_q.single();

    if buttons.just_pressed(MouseButton::Right) {
        for (collider_transform, entity, _c) in &collider_q {
            if let Some(Collision::Inside) = collide(
                grid_cursor.last_target_pos.extend(0.),
                vec2(0., 0.),
                collider_transform.translation,
                collider_transform.scale.truncate(),
            ) {
                commands.entity(entity).despawn();
            }
        }
    }
}

fn game_tick(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    mut path: ResMut<Path>, //
    mut potato_q: Query<(&mut Transform, &mut Potato)>,
) {
    let mut moved = false;
    if potato_q.iter().count() < 7 {
        let texture = asset_server.load("resources/potato.png");

        commands.spawn(
            PotatoBundle::new(Potato { idx: 0 })
                .with_texture(texture)
                .with_position(path.start_position.extend(0.)),
        );

        path.positions[0].state = TileState::Occupied;
        info!("Spawned potato");
    }

    for (mut potato_transform, mut potato) in potato_q.iter_mut() {
        if let Some(next_position) = path.positions.get_mut(potato.idx + 1) {
            if next_position.state == TileState::Free {
                // move the potato
                potato_transform.translation = next_position.position.extend(0.);

                // occupy the next tile
                next_position.state = TileState::Occupied;

                // freet he current tile

                // set current tile to new
                potato.idx += 1;
                moved = true;
            }
        } else {
            potato_transform.translation = path.end_position.extend(0.);
            potato.idx = 0;
            moved = true;
        };

        if moved {
            let current_position = &mut path.positions[potato.idx];
            current_position.state = TileState::Free;
        }
    }
}

fn update_scoreboard(scoreboard: Res<Scoreboard>, mut query: Query<&mut Text>) {
    let mut text = query.single_mut();
    text.sections[1].value = scoreboard.score.to_string();
}
