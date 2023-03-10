use core::panic;
use std::{f32::consts::PI, time::Duration};

use bevy::{
    core_pipeline::bloom::BloomSettings,
    diagnostic::{Diagnostics, FrameTimeDiagnosticsPlugin},
    math::*,
    prelude::*,
    sprite::collide_aabb::{collide, Collision},
};
use rand::{thread_rng, Rng};

use crate::components::*;
use crate::interpolation::ease;
use crate::*;

// Add the game's entities to our world
pub fn setup(mut commands: Commands, asset_server: Res<AssetServer>) {
    // level
    let level = LevelHandle(asset_server.load("map.json"));
    commands.insert_resource(level);

    let enemylist = RoundList(vec![
        vec![(5, EnemyKind::Potato), (3, EnemyKind::Carrot)],
        vec![(8, EnemyKind::Potato), (5, EnemyKind::Carrot)],
        vec![(10, EnemyKind::Carrot), (1, EnemyKind::Pepper)],
    ]);
    commands.insert_resource(enemylist);

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
            threshold: 0.15,
            ..default()
        },
    ));

    // Cursor
    commands.spawn((Cursor::new(), Collider(ColliderType::Cursor)));

    // Scoreboard
    let font = asset_server.load("fonts/ComicMono.ttf");
    commands.spawn(FPSBundle::new(font.clone()));
    commands.spawn(GUIBundle::new(font));

    // commands.spawn(
    //     GunBundle::new(Gun::Gun2, &asset_server).with_transform(
    //         Transform::from_xyz(-2.5 * TILE_SIZE, -1.5 * TILE_SIZE, 10.)
    //             .with_scale(Vec3::splat(TILE_SIZE / SPRITE_SIZE)),
    //     ),
    // );

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
                .with_position(start.extend(PATH_LAYER)),
        );

        let end = tiles.next_back().unwrap();
        path.end_position = *end;
        path.positions.push_front(PathNode::new(*end));

        commands.spawn(
            PathTile::new(&asset_server)
                .with_texture(asset_server.load("resources/hole.png"))
                .with_position(end.extend(PATH_LAYER)),
        );

        // Move camera to middle of the map based on strong assumptions
        camera_transform.translation += Vec2::from_array(level.center_pos).extend(0.) * TILE_SIZE;
        let mut rng = thread_rng();

        for x in -MAP_SIZE..MAP_SIZE {
            for y in -MAP_SIZE..MAP_SIZE {
                commands.spawn(Tile::new(&asset_server).with_position(Vec3 {
                    x: x as f32 * TILE_SIZE,
                    y: y as f32 * TILE_SIZE,
                    z: BACKGROUND_LAYER,
                }));

                let r = rng.gen_range(0..50);
                if r < 5 {
                    commands.spawn(
                        Tile::new(&asset_server)
                            .with_texture(asset_server.load("resources/grass.png"))
                            .with_position(Vec3 {
                                x: x as f32 * TILE_SIZE - TILE_SIZE / 2.,
                                y: y as f32 * TILE_SIZE - TILE_SIZE / 2.,
                                z: (BACKGROUND_LAYER + PATH_LAYER) / 2., // TODO name layer
                            }),
                    );
                } else if r < 7 {
                    commands.spawn(
                        Tile::new(&asset_server)
                            .with_texture(asset_server.load("resources/stone.png"))
                            .with_position(Vec3 {
                                x: x as f32 * TILE_SIZE - TILE_SIZE / 2.,
                                y: y as f32 * TILE_SIZE - TILE_SIZE / 2.,
                                z: (BACKGROUND_LAYER + PATH_LAYER) / 2.,
                            }),
                    );
                }
            }
        }

        for pos in tiles {
            path.positions.push_back(PathNode::new(*pos));
            commands.spawn(PathTile::new(&asset_server).with_position(pos.extend(PATH_LAYER)));
        }

        for (image_path, pos, blocks) in level.decor.into_iter() {
            let decor_asset = asset_server.load::<Image, _>(image_path);
            let decor_pos = vec2(pos[0], pos[1]) * TILE_SIZE + Vec2::splat(TILE_SIZE / 2.);
            let tile =
                Tile::new_decor(decor_asset).with_position(decor_pos.extend(PATH_LAYER + 0.1));
            if blocks {
                commands.spawn((tile, Collider(ColliderType::Decor)));
            } else {
                commands.spawn(tile);
            }
        }

        commands.insert_resource(SpawnTimer::default());
        commands.insert_resource(RoundCounter::default());
        commands.insert_resource(Round(vec![]));

        state.set(AppState::Level).unwrap();
    }
}

pub fn move_cursor(
    mut query: Query<(&mut Transform, &mut GridCursor), (Without<Camera>, Without<Selected>)>,
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

        // 1 round of magic
        if grid_cursor.selection_size.x as i32 % 2 == 0 {
            target_pos_grid -= Vec2::splat(TILE_SIZE / 2.);
        }

        target_pos_grid =
            (target_pos_grid / TILE_SIZE).floor() * TILE_SIZE + Vec2::splat(TILE_SIZE / 2.);

        // 2 rounds of magic
        if grid_cursor.selection_size.x as i32 % 2 == 0 {
            target_pos_grid += Vec2::splat(TILE_SIZE / 2.);
        }

        let prev_pos = cursor_transform.translation.xy();
        let delta = target_pos_grid - prev_pos;

        if prev_pos == target_pos_grid {
            grid_cursor.last_sample = elapsed;
        }
        if grid_cursor.last_target_pos != target_pos_grid {
            // if the target grid has changed
            grid_cursor.last_target_pos = target_pos_grid;
            // makes continuous target switching feel smooth
            grid_cursor.last_sample = (grid_cursor.last_sample + elapsed) / 2. - 0.05;
        }

        const EASE_TIME: f32 = 1.0;
        let dt = elapsed - grid_cursor.last_sample;

        let newp = prev_pos + delta * ease(dt / EASE_TIME);
        cursor_transform.translation = newp.extend(CURSOR_LAYER);
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
            let target_transform = Transform::from_xyz(
                cursor.last_target_pos.x,
                cursor.last_target_pos.y,
                CURSOR_LAYER,
            )
            .with_scale(transform.scale * TILE_SIZE);

            let turret =
                TurretBundle::new(*placeable, &asset_server).with_transform(target_transform);

            let gun = GunBundle::new(turret.turret.gun(), &asset_server).with_transform(
                target_transform.with_translation(target_transform.translation + vec3(0., 0., 1.)),
            );

            commands.spawn(turret);
            commands.spawn(gun);
        }
    }
}

pub fn handle_collisions(
    mut cursor_q: Query<(&Transform, &mut GridCursor)>,
    collider_q: Query<
        (&Transform, &Collider),
        (Without<GridCursor>, Without<Turret>, Without<Projectile>),
    >,
    turret_q: Query<(&Transform, &Turret), (With<Collider>, Without<Selected>)>,
) {
    let (_cursor_transform, mut cursor) = cursor_q.single_mut();

    let mut colliding = false;

    for (collision_transform, _collider) in &collider_q {
        let collision = collide(
            cursor.last_target_pos.extend(0.),
            Vec2::splat(TILE_SIZE * cursor.selection_size.x),
            collision_transform.translation,
            collision_transform.scale.xy(),
        );

        if collision.is_some() {
            colliding = true;
            break;
        }
    }

    for (collision_transform, _turret) in &turret_q {
        let collision = collide(
            cursor.last_target_pos.extend(0.),
            Vec2::splat(TILE_SIZE * cursor.selection_size.x),
            collision_transform.translation,
            collision_transform.scale.xy(),
        );

        if collision.is_some() {
            colliding = true;
            break;
        }
    }

    cursor.can_place = !colliding;
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
    mut round_counter: ResMut<RoundCounter>,
    roundlist: Res<RoundList>,
    mut round: ResMut<Round>,
    time: Res<Time>,
    mut spawn_timer: ResMut<SpawnTimer>,
) {
    spawn_timer.0.tick(time.delta());
    if spawn_timer.0.finished() {
        if let Some(kind) = round.0.pop() {
            if kind == EnemyKind::Pepper {
                let animation_indices = AnimationIndices { first: 0, last: 3 };
                commands.spawn((
                    EnemyBundle::new(
                        Enemy::new(EnemyKind::Pepper, 0),
                        &asset_server,
                        &mut texture_atlases,
                    )
                    .with_position(path.start_position.extend(0.)),
                    animation_indices,
                    AnimationTimer(Timer::from_seconds(0.1, TimerMode::Repeating)),
                    Collider(ColliderType::Enemy),
                ));
            }
            commands.spawn((
                EnemyBundle::new(Enemy::new(kind, 0), &asset_server, &mut texture_atlases)
                    .with_position(path.start_position.extend(0.)),
                Collider(ColliderType::Enemy),
            ));
        }
    }

    if enemy_q.iter().count() < 1 && round.0.is_empty() {
        round_counter.next();

        let mut enemies: Vec<EnemyKind> = vec![];
        if let Some(enemylist) = (*roundlist).0.get(round_counter.0 - 1) {
            for (amount, kind) in enemylist {
                for _ in 0..*amount {
                    enemies.push(kind.clone());
                }
            }
        } else {
            for _ in 0..round_counter.0 {
                enemies.push(EnemyKind::Pepper);
            }
        }
        commands.insert_resource(Round(enemies));

        // commands.spawn((
        //     EnemyBundle::new(
        //         Enemy::new(EnemyKind::Carrot, 0),
        //         &asset_server,
        //         &mut texture_atlases,
        //     )
        //     .with_position(path.start_position.extend(ENEMY_LAYER)),
        //     Collider(ColliderType::Enemy),
        // ));

        // let animation_indices = AnimationIndices { first: 0, last: 3 };
        // commands.spawn((
        //     EnemyBundle::new(
        //         Enemy::new(EnemyKind::Pepper, 0),
        //         &asset_server,
        //         &mut texture_atlases,
        //     )
        //     .with_position(path.start_position.extend(0.)),
        //     animation_indices,
        //     AnimationTimer(Timer::from_seconds(0.1, TimerMode::Repeating)),
        //     Collider(ColliderType::Enemy),
        // ));
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

        cursor.selection_size = new_turret.turret.scale() * SPRITE_SIZE;
        let child = commands.spawn((new_turret, Selected)).id();

        commands.entity(cursor_ent).add_child(child);
    }
}

pub fn handle_gunners(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    mut texture_atlases: ResMut<Assets<TextureAtlas>>,
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
                continue;
            }
            let angle = delta.y.atan2(delta.x);
            gun_t.rotation = Quat::from_euler(EulerRot::XYZ, 0., 0., angle - PI / 2.);
            if gun_state.last_shot + Duration::from_secs_f32(1. / gun.rate()) < time.elapsed() {
                commands.spawn((
                    ProjectileBundle::new(
                        Projectile::new(ProjectileType::ChefsKnife),
                        &asset_server,
                        &mut texture_atlases,
                    )
                    .with_transform(
                        Transform::from_translation(
                            gun_t.translation.truncate().extend(PROJECTILE_LAYER),
                        )
                        .with_rotation(
                            gun_t.rotation * Quat::from_euler(EulerRot::XYZ, 0., 0., -PI / 2.),
                        )
                        .with_scale(Vec2::splat(TILE_SIZE / SPRITE_SIZE / 2.).extend(0.)),
                    ),
                    Collider(ColliderType::Projectile),
                    AnimationIndices { first: 0, last: 2 },
                    AnimationTimer(Timer::from_seconds(0.6, TimerMode::Repeating)),
                ));

                gun_state.last_shot = time.elapsed();
            }
        }
    }
}

pub fn handle_projectiles(
    mut commands: Commands,
    mut projectile_q: Query<(Entity, &mut Transform, &Projectile), With<Collider>>,
) {
    for (projectile_ent, mut projectile_t, projectile) in projectile_q.iter_mut() {
        if projectile_t.translation.x > MAP_SIZE as f32 * TILE_SIZE
            || projectile_t.translation.y > MAP_SIZE as f32 * TILE_SIZE
            || projectile_t.translation.x < -MAP_SIZE as f32 * TILE_SIZE
            || projectile_t.translation.y < -MAP_SIZE as f32 * TILE_SIZE
        {
            commands.entity(projectile_ent).despawn();
        }

        let dir = projectile_t.rotation * Vec3::X;

        projectile_t.translation -= projectile.velocity() * dir;

        if projectile.health <= 0 {
            commands.entity(projectile_ent).despawn();
        }
    }
}

pub fn handle_projectile_collisions(
    mut commands: Commands,
    mut projectile_q: Query<(&mut Transform, &mut Projectile), With<Collider>>,
    mut enemies: Query<(Entity, &mut Enemy, &Transform), (Without<Projectile>, With<Collider>)>,
) {
    let mut rng = thread_rng();

    for (enemy_ent, mut enemy, enemy_t) in enemies.iter_mut() {
        for (mut projectile_t, mut projectile) in projectile_q.iter_mut() {
            let enemy_scale = enemy_t.scale.truncate() * SPRITE_SIZE; // TODO fix relative scale of enemies

            let collision = collide(
                projectile_t.translation,
                projectile.scale(),
                enemy_t.translation,
                enemy_scale,
            );

            let enemy_entid = commands.entity(enemy_ent).id();
            if collision.is_some() && !projectile.hit_enemies.contains(&enemy_entid) {
                projectile.health -= 1;
                enemy.health -= 1;
                projectile.hit_enemies.push(enemy_entid);
                projectile_t.rotation *= Quat::from_euler(
                    EulerRot::XYZ,
                    0.,
                    0.,
                    rng.gen_range(-MAX_DEFLECTION_ANGLE..MAX_DEFLECTION_ANGLE),
                );
            }
        }
    }
}

pub fn handle_enemies(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    mut texture_atlases: ResMut<Assets<TextureAtlas>>,
    mut enemies: Query<(Entity, &mut Enemy, &Transform)>,
) {
    let mut rng = thread_rng();

    for (enemy_ent, enemy, enemy_t) in enemies.iter_mut() {
        if enemy.health <= 0 {
            commands.entity(enemy_ent).despawn();

            if let Some((amount, kind)) = enemy.split() {
                for i in 1..=amount {
                    let j = i as f32;
                    commands.spawn((
                        EnemyBundle::new(
                            Enemy::new(kind.clone(), enemy.idx),
                            &asset_server,
                            &mut texture_atlases,
                        )
                        .with_position(
                            enemy_t.translation
                                + Vec2::new(
                                    rng.gen_range(0..TILE_SIZE as i32 / amount) as f32 * (j - 1.),
                                    rng.gen_range(0..TILE_SIZE as i32 / amount) as f32 * (j - 1.),
                                )
                                .extend(0.),
                        ),
                        Collider(ColliderType::Enemy),
                    ));
                }
            }
        }
    }
}

pub fn update_scoreboard(
    menu: Res<Menu>,
    mut query: Query<&mut Text, With<GUIText>>,
    windows: Res<Windows>,
    camera_q: Query<&Transform, With<Camera>>,
) {
    let window = windows.get_primary().unwrap();
    let window_size = Vec2::new(window.width(), window.height());
    let camera_transform = camera_q.get_single().unwrap();
    if let Ok(mut text) = query.get_single_mut() {
        if let Some(cursor_position) = window.cursor_position() {
            let mut cursor_grid =
                cursor_position - (window_size / 2.) + camera_transform.translation.truncate();

            cursor_grid = (cursor_grid / TILE_SIZE).floor();
            text.sections[1].value = format!(
                "{:?} {:>3} {:>3}",
                menu.current_item, cursor_grid.x, cursor_grid.y
            );
        } else {
            text.sections[1].value = format!("{:?} --- ---", menu.current_item);
        }
    }
}

pub fn update_fps(diagnostics: Res<Diagnostics>, mut query: Query<&mut Text, With<FPSText>>) {
    if let Ok(mut text) = query.get_single_mut() {
        if let Some(fps) = diagnostics.get(FrameTimeDiagnosticsPlugin::FPS) {
            if let Some(value) = fps.smoothed() {
                // Update the value of the second section
                text.sections[1].value = format!("{value:.2}");
            }
        }
    }
}
