use std::{f32::consts::PI, time::Duration};

use bevy::{
    core_pipeline::bloom::BloomSettings,
    math::*,
    prelude::*,
    sprite::collide_aabb::{collide, Collision},
};

use crate::components::*;
use crate::interpolation::ease;
use crate::*;

// Add the game's entities to our world
pub fn setup(mut commands: Commands, asset_server: Res<AssetServer>) {
    // level
    let level = LevelHandle(asset_server.load("map.json"));
    commands.insert_resource(level);

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
    commands.spawn(Cursor::new());

    // Scoreboard
    let font = asset_server.load("fonts/ComicMono.ttf");
    commands.spawn(GUIBundle::new(font));

    commands.spawn(
        GunBundle::new(Gun::Gun1, &asset_server).with_transform(
            Transform::from_xyz(-2.5 * TILE_SIZE, -1.5 * TILE_SIZE, 10.)
                .with_scale(Vec3::splat(TILE_SIZE / SPRITE_SIZE)),
        ),
    );

    // Textures
    // let texture_handle = asset_server.load("resources/potato.png");
    // let texture_atlas =
    //     TextureAtlas::from_grid(texture_handle, Vec2::new(32.0, 32.0), 1, 1, None, None);
    // texture_atlases.add(texture_atlas);

    // let texture_handle = asset_server.load("resources/carrot.png");
    // let texture_atlas =
    //     TextureAtlas::from_grid(texture_handle, Vec2::new(32.0, 32.0), 1, 1, None, None);
    // texture_atlases.add(texture_atlas);

    // let texture_handle = asset_server.load("resources/pepper.png");
    // let texture_atlas =
    //     TextureAtlas::from_grid(texture_handle, Vec2::new(32.0, 32.0), 4, 1, None, None);
    // let th = texture_atlases.add(texture_atlas);

    // let animation_indices = AnimationIndices { first: 0, last: 3 };
    // commands.spawn((
    //     EnemyBundle::new(
    //         Enemy {
    //             kind: EnemyKind::Pepper,
    //             idx: 0,
    //         },
    //         &asset_server,
    //         &mut texture_atlases,
    //     )
    //     .with_position(Vec3::splat(1.)),
    //     animation_indices,
    //     AnimationTimer(Timer::from_seconds(0.1, TimerMode::Repeating)),
    // ));
}

#[derive(Component)]
pub struct AnimationIndices {
    first: usize,
    last: usize,
}

#[derive(Component, Deref, DerefMut)]
pub struct AnimationTimer(Timer);

pub fn animate_sprite(
    time: Res<Time>,
    mut query: Query<(
        &AnimationIndices,
        &mut AnimationTimer,
        &mut TextureAtlasSprite,
    )>,
) {
    for (indices, mut timer, mut sprite) in &mut query {
        timer.tick(time.delta());
        if timer.just_finished() {
            sprite.index = if sprite.index == indices.last {
                indices.first
            } else {
                sprite.index + 1
            };
        }
    }
}

pub fn spawn_level(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    level: Res<LevelHandle>,
    mut levels: ResMut<Assets<Level>>,
    mut state: ResMut<State<AppState>>,
    mut query: Query<&mut Transform, With<Camera>>,
    mut path: ResMut<Path>,
) {
    if let Some(level) = levels.remove(level.0.id()) {
        let mut camera_transform = query.get_single_mut().unwrap();
        let positions: Vec<Vec2> = level
            .path
            .iter()
            .map(|pos| vec2(pos[0], pos[1]) * Vec2::splat(TILE_SIZE) + Vec2::splat(TILE_SIZE / 2.))
            .collect();

        let mut tiles = positions.iter();

        let start = tiles.next().unwrap();
        path.start_position = *start;
        path.positions.push_back(PathNode::new(*start));

        commands.spawn(
            Tile::new(&asset_server)
                .with_texture(asset_server.load("resources/hole.png"))
                .with_position(start.extend(1.)),
        );

        let end = tiles.next_back().unwrap();
        path.end_position = *end;
        path.positions.push_front(PathNode::new(*end));

        commands.spawn(
            PathTile::new(&asset_server)
                .with_texture(asset_server.load("resources/hole.png"))
                .with_position(end.extend(1.)),
        );

        // Move camera to middle of the map based on naive assumptions
        camera_transform.translation += (*end - *start).extend(0.) / 2.;

        for x in -MAP_SIZE..MAP_SIZE {
            for y in -MAP_SIZE..MAP_SIZE {
                commands.spawn(Tile::new(&asset_server).with_position(Vec3 {
                    x: x as f32 * TILE_SIZE - TILE_SIZE / 2.,
                    y: y as f32 * TILE_SIZE - TILE_SIZE / 2.,
                    z: 0.,
                }));
            }
        }

        for pos in tiles {
            path.positions.push_back(PathNode::new(*pos));
            commands.spawn(PathTile::new(&asset_server).with_position(pos.extend(PATH_LAYER)));
        }

        state.set(AppState::Level).unwrap();
    }
}

pub fn move_cursor(
    mut query: Query<(&mut Transform, &mut GridCursor), (Without<Camera>, Without<Selected>)>,
    child_q: Query<&Transform, With<Selected>>,
    camera_q: Query<&Transform, With<Camera>>,
    windows: Res<Windows>,
    time: Res<Time>,
) {
    let window = windows.get_primary().unwrap();
    let window_size = Vec2::new(window.width(), window.height());
    let camera_transform = camera_q.get_single().unwrap();

    let sprite = if let Ok(sprite) = child_q.get_single() {
        sprite.with_scale(sprite.scale * SPRITE_SIZE)
    } else {
        return;
    };

    // TODO clean this ugly mess
    if let Some(cursor_position) = window.cursor_position() {
        let elapsed = time.elapsed().as_secs_f32();
        let (mut cursor_transform, mut grid_cursor) = query.get_single_mut().unwrap();
        let mut target_pos_grid =
            cursor_position - (window_size / 2.) + camera_transform.translation.truncate();

        // 1 round of magic
        if sprite.scale.z as i32 % 2 == 0 {
            target_pos_grid -= Vec2::splat(TILE_SIZE / 2.);
        }

        target_pos_grid =
            (target_pos_grid / TILE_SIZE).floor() * TILE_SIZE + Vec2::splat(TILE_SIZE / 2.);

        // 2 rounds of magic
        if sprite.scale.z as i32 % 2 == 0 {
            target_pos_grid += Vec2::splat(TILE_SIZE / 2.);
        }

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
    child_q: Query<(&Transform, &Turret), With<Selected>>,
    collider_q: Query<
        (&Transform, &Collider),
        (Without<GridCursor>, Without<Selected>, Without<Turret>),
    >,
    turret_q: Query<(&Transform, &Collider, &Turret), Without<Selected>>,
) {
    let mut cursor = cursor_q.single_mut();
    if let Ok((c_transform, turret)) = child_q.get_single() {
        let cursor_transform = cursor.last_target_pos.extend(0.);
        let c_transform = c_transform.with_scale((c_transform.scale * SPRITE_SIZE) * TILE_SIZE);

        let mut colliding = false;

        for (collider_transform, _c, turret) in turret_q.iter() {
            // check turret yes yes
            let collider_transform =
                collider_transform.with_scale(collider_transform.scale / SPRITE_SIZE);

            let collision = collide(
                cursor_transform,
                turret.scale(),
                collider_transform.translation,
                collider_transform.scale.truncate(),
            );

            if collision.is_some() {
                colliding = true;
                break;
            }
        }

        for (collider_transform, _c) in collider_q.iter() {
            // check turret in cursor <=> something colliding
            let collision = collide(
                cursor_transform,
                turret.scale(),
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
    mut texture_atlases: ResMut<Assets<TextureAtlas>>,
) {
    if enemy_q.iter().count() < 1 {
        commands.spawn((
            EnemyBundle::new(
                Enemy {
                    kind: EnemyKind::Carrot,
                    idx: 0,
                },
                &asset_server,
                &mut texture_atlases,
            )
            .with_position(path.start_position.extend(ENEMY_LAYER)),
            Collider,
        ));

        let animation_indices = AnimationIndices { first: 0, last: 3 };
        commands.spawn((
            EnemyBundle::new(
                Enemy {
                    kind: EnemyKind::Pepper,
                    idx: 0,
                },
                &asset_server,
                &mut texture_atlases,
            )
            .with_position(path.start_position.extend(0.)),
            animation_indices,
            AnimationTimer(Timer::from_seconds(0.1, TimerMode::Repeating)),
            Collider,
        ));
    }

    for (mut enemy_transform, mut enemy) in enemy_q.iter_mut() {
        if let Some(next_tile) = path.positions.get_mut(enemy.idx + 1) {
            let next_pos = next_tile.position;
            let prev_pos = enemy_transform.translation.xy();
            let newp = (next_pos - prev_pos).normalize_or_zero() * enemy.speed() + prev_pos;
            enemy_transform.translation = newp.extend(ENEMY_LAYER);

            if (enemy_transform.translation)
                .abs_diff_eq(next_pos.extend(ENEMY_LAYER), TILE_SIZE / 20.)
            {
                enemy.idx += 1;
            }
        };
    }
}

pub fn handle_shop(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    mut cursor_q: Query<(Entity, &mut GridCursor)>,
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
        let (cursor_ent, mut cursor) = cursor_q.single_mut();
        if let Ok((child, _)) = selected_q.get_single() {
            commands.entity(cursor_ent).remove_children(&[child]);
            commands.entity(child).despawn();
        }
        let new_turret = match item_map[idx] {
            MenuItem::Turret1x1 => TurretBundle::new(Turret::Turret1x1, &asset_server),
            MenuItem::Turret2x2 => TurretBundle::new(Turret::Turret2x2, &asset_server),
        };

        cursor.selection_size = new_turret.turret.scale();
        let child = commands.spawn((new_turret, Selected)).id();
        commands.entity(cursor_ent).add_child(child);
    }
}

pub fn update_scoreboard(menu: Res<Menu>, mut query: Query<&mut Text>) {
    let mut text = query.single_mut();
    text.sections[1].value = format!("{:?}", menu.current_item);
}

pub fn handle_gunners(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    mut gun_q: Query<(&mut Transform, &Gun, &mut GunState), Without<Selected>>,
    enemies: Query<(&Transform, &Enemy), Without<Gun>>,
    time: Res<Time>,
) {
    for (mut gun_t, gun, mut gun_state) in gun_q.iter_mut() {
        if let Some(nearest_enemy) = enemies.iter().min_by(|(enemy_t_a, _), (enemy_t_b, _)| {
            gun_t
                .translation
                .distance(enemy_t_a.translation)
                .partial_cmp(&gun_t.translation.distance(enemy_t_b.translation))
                .unwrap()
        }) {
            let delta = nearest_enemy.0.translation - gun_t.translation;
            if delta.length() > gun.range() {
                // TODO scaling?
                continue;
            }
            let angle = delta.y.atan2(delta.x);
            gun_t.rotation =
                Quat::from_euler(EulerRot::XYZ, 0., 0., angle - std::f32::consts::PI / 2.);
            if gun_state.last_shot + Duration::from_secs_f32(1. / gun.rate()) < time.elapsed() {
                commands.spawn((
                    ProjectileBundle::new(Projectile::new(ProjectileType::Knife, 1), &asset_server)
                        .with_transform(
                            Transform::from_translation(
                                gun_t.translation.truncate().extend(PROJECTILE_LAYER),
                            )
                            .with_rotation(
                                gun_t.rotation * Quat::from_euler(EulerRot::XYZ, 0., 0., -PI / 2.),
                            )
                            .with_scale(Vec2::splat(2.0).extend(0.)),
                        ),
                    Collider,
                ));

                gun_state.last_shot = time.elapsed();
            }
        }
    }
}

pub fn handle_projectiles(
    mut commands: Commands,
    mut projectile_q: Query<(Entity, &mut Transform, &Collider, &mut Projectile)>,
    enemies: Query<(Entity, &Transform, &Collider), (With<Enemy>, Without<Projectile>)>,
) {
    for (projectile_ent, mut projectile_t, _, mut projectile) in projectile_q.iter_mut() {
        if projectile_t.translation.x > MAP_SIZE as f32 * TILE_SIZE
            || projectile_t.translation.y > MAP_SIZE as f32 * TILE_SIZE
            || projectile_t.translation.x < -MAP_SIZE as f32 * TILE_SIZE
            || projectile_t.translation.y < -MAP_SIZE as f32 * TILE_SIZE
        {
            commands.entity(projectile_ent).despawn();
        }
        let dir = projectile_t.rotation * Vec3::X;
        projectile_t.translation -= projectile.velocity() * dir;

        for (enemy_ent, enemy_t, _) in enemies.iter() {
            let enemy_scale = enemy_t.scale.truncate() * 16.0; // TODO fix relative scale of enemies

            let collision = collide(
                projectile_t.translation,
                Vec2::ZERO,
                enemy_t.translation,
                enemy_scale,
            );

            if collision.is_some() {
                commands.entity(enemy_ent).despawn();
                projectile.health -= 1;
            }
            if projectile.health < 0 {
                commands.entity(projectile_ent).despawn();
            }
        }
    }
}
