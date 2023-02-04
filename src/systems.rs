use std::process::id;

use bevy::{
    core_pipeline::bloom::BloomSettings,
    math::*,
    prelude::*,
    render::render_resource::{Extent3d, TextureDimension, TextureFormat},
    sprite::collide_aabb::{collide, Collision},
};

use crate::components::*;
use crate::interpolation::ease;
use crate::*;

// Add the game's entities to our world
pub fn setup(
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

    let turret = commands
        .spawn((
            TurretBundle::new(Turret::Turret2x2, &asset_server),
            Selected {},
        ))
        .id();

    // Cursor
    commands.spawn(Cursor::new()).add_child(turret);

    // Scoreboard
    let font = asset_server.load("fonts/ComicMono.ttf");
    commands.spawn(GUIBundle::new(font));
}

pub fn spawn_level(
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

pub fn move_cursor(
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

pub fn handle_cursor_visibility(
    cursor_q: Query<&GridCursor>,
    mut child_q: Query<&mut Sprite, With<Selected>>,
) {
    let cursor = cursor_q.get_single().unwrap();
    if let Ok(mut sprite) = child_q.get_single_mut() {
        sprite.color = if cursor.can_place {
            WALL_COLOR
        } else {
            ERROR_COLOR
        };
    }
}

pub fn handle_place(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    cursor_q: Query<(&Transform, &GridCursor), Without<Turret>>,
    child_q: Query<(&Transform, &Turret), With<Selected>>,
    buttons: Res<Input<MouseButton>>,
) {
    let (cursor_transform, cursor) = cursor_q.get_single().unwrap();
    if !cursor.can_place {
        return;
    }

    if let Ok((transform, placeable)) = child_q.get_single() {
        if buttons.just_pressed(MouseButton::Left) {
            //info!("yeet");
            let cursor_transform =
                Transform::from_xyz(cursor.last_target_pos.x, cursor.last_target_pos.y, 0.)
                    .with_scale(cursor_transform.scale * transform.scale);

            commands.spawn(
                TurretBundle::new(*placeable, &asset_server).with_transform(cursor_transform),
            );
        }
    }
}

pub fn handle_collisions(
    mut cursor_q: Query<&mut GridCursor>,
    child_q: Query<&Transform, With<Selected>>,
    collider_q: Query<
        (&Transform, &Collider),
        (Without<GridCursor>, Without<Selected>, Without<Turret>),
    >,
    turret_q: Query<(&Transform, &Collider), (With<Turret>, Without<Selected>)>,
) {
    let mut cursor = cursor_q.single_mut();
    if let Ok(c_transform) = child_q.get_single() {
        let cursor_transform = cursor.last_target_pos.extend(0.);
        let c_transform = c_transform.with_scale((c_transform.scale * SPRITE_SIZE) * TILE_SIZE);

        let mut colliding = false;

        for (collider_transform, _c) in turret_q.iter() {
            let collider_transform =
                collider_transform.with_scale((collider_transform.scale / 2.) * TILE_SIZE);

            let collision = collide(
                cursor_transform,
                c_transform.scale.truncate(),
                collider_transform.translation,
                collider_transform.scale.truncate(),
            );

            if collision.is_some() {
                colliding = true;
                break;
            }
        }

        for (collider_transform, _c) in collider_q.iter() {
            let collision = collide(
                cursor_transform,
                c_transform.scale.truncate(),
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
}

pub fn handle_sell(
    mut commands: Commands,
    cursor_q: Query<&GridCursor>,
    collider_q: Query<(&Transform, Entity, &Collider, &Turret), Without<GridCursor>>,
    buttons: Res<Input<MouseButton>>,
) {
    let grid_cursor = cursor_q.single();

    if buttons.just_pressed(MouseButton::Right) {
        for (collider_transform, entity, _c, _p) in &collider_q {
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

pub fn game_tick(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    mut path: ResMut<Path>, //
    mut enemy_q: Query<(&mut Transform, &mut Enemy)>,
) {
    if enemy_q.iter().count() < 1 {
        let texture = asset_server.load("resources/potato.png");

        commands.spawn(
            EnemyBundle::new(Enemy {
                kind: EnemyKind::Potato,
                idx: 0,
            })
            .with_texture(texture)
            .with_position(path.start_position.extend(0.)),
        );
    }

    for (mut enemy_transform, mut enemy) in enemy_q.iter_mut() {
        if let Some(next_tile) = path.positions.get_mut(enemy.idx + 1) {
            let next_pos = next_tile.position;
            let prev_pos = enemy_transform.translation.xy();
            let newp = (next_pos - prev_pos).normalize_or_zero() * enemy.speed() + prev_pos;
            enemy_transform.translation = newp.extend(0.);

            if (enemy_transform.translation).abs_diff_eq(next_pos.extend(0.), TILE_SIZE / 20.) {
                enemy.idx += 1;
            }
        };
    }
}

pub fn handle_shop(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    cursor_q: Query<Entity, With<GridCursor>>,
    selected_q: Query<(Entity, &Transform), With<Selected>>,
    mut menu: ResMut<Menu>,
    keys: Res<Input<KeyCode>>,
) {
    let item_map = MenuItem::all();

    if keys.any_just_pressed([KeyCode::Key1, KeyCode::Key2]) {
        let idx: usize = if keys.just_pressed(KeyCode::Key1) {
            0
        } else if keys.just_pressed(KeyCode::Key2) {
            1
        } else {
            return;
        };

        menu.current_item = item_map[idx];
        if let Ok((child, transform)) = selected_q.get_single() {
            let cursor = cursor_q.single();
            let new_child = match item_map[idx] {
                MenuItem::Turret1x1 => commands
                    .spawn((
                        TurretBundle::new(Turret::Turret1x1, &asset_server),
                        Selected {},
                    ))
                    .id(),
                MenuItem::Turret2x2 => commands
                    .spawn((
                        TurretBundle::new(Turret::Turret2x2, &asset_server),
                        Selected {},
                    ))
                    .id(),
            };

            commands.entity(cursor).remove_children(&[child]);
            commands.entity(child).despawn();
            commands.entity(cursor).add_child(new_child);
        }

        //commands.entity(cursor).remove_children(children)
    }
}

pub fn update_scoreboard(menu: Res<Menu>, mut query: Query<&mut Text>) {
    let mut text = query.single_mut();
    text.sections[1].value = format!("{:?}", menu.current_item);
}
