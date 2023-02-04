use std::collections::VecDeque;

use bevy::reflect::TypeUuid;
use bevy::{math::*, prelude::*};

use crate::{Texture, CURSOR_COLOR, SPRITE_SIZE, TILE_COLOR, TILE_SIZE, WALL_COLOR};

#[derive(serde::Deserialize, TypeUuid, Debug)]
#[uuid = "413be529-bfeb-41b3-9db0-4b8b380a2c46"]
pub struct Level {
    pub path: Vec<[f32; 2]>,
}

#[derive(Resource, Debug)]
pub struct LevelHandle(pub Handle<Level>);

#[derive(Component)]
pub struct Collider;

#[derive(Bundle, Default)]
pub struct Cursor {
    sprite_bundle: SpriteBundle,
    grid_cursor: GridCursor,
}

impl Cursor {
    pub fn new() -> Self {
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

#[derive(Component, Debug)]
pub struct Selected {}

#[derive(Component, Debug, Default)]
pub struct GridCursor {
    pub can_place: bool,
    pub last_target_pos: Vec2,
    pub last_sample: f32,
}

#[derive(Component, Debug, Clone, Copy)]
pub enum Turret {
    Turret1x1,
    Turret2x2,
}

impl Turret {
    pub fn scale(&self) -> Vec2 {
        match self {
            Turret::Turret1x1 => Vec2::splat(1. / SPRITE_SIZE),
            Turret::Turret2x2 => Vec2::splat(2. / SPRITE_SIZE),
        }
    }

    pub fn sprite(&self, asset_server: &Res<AssetServer>) -> Handle<Image> {
        match self {
            Turret::Turret1x1 => asset_server.load("resources/turret-2.png"),
            Turret::Turret2x2 => asset_server.load("resources/turret-1.png"),
        }
    }
}

#[derive(Bundle)]
pub struct TurretBundle {
    pub sprite_bundle: SpriteBundle,
    pub collider: Collider,
    pub turret: Turret,
}

impl TurretBundle {
    pub fn new(turret: Turret, asset_server: &Res<AssetServer>) -> Self {
        Self {
            sprite_bundle: SpriteBundle {
                transform: Transform::from_xyz(0., 0., 0.)
                    .with_scale(Vec2::splat(1. / SPRITE_SIZE).extend(turret.scale().x)),
                sprite: Sprite {
                    color: WALL_COLOR,
                    ..default()
                },
                texture: turret.sprite(asset_server),
                ..default()
            },
            turret,
            collider: Collider,
        }
    }

    pub fn with_transform(mut self, transform: Transform) -> Self {
        self.sprite_bundle.transform = transform;
        self
    }

    pub fn with_texture(mut self, texture: Texture) -> Self {
        self.sprite_bundle.texture = texture;
        self
    }
}

#[derive(Bundle)]
pub struct Tile {
    pub sprite_bundle: SpriteBundle,
    pub collider: Collider,
}

impl Tile {
    pub fn new() -> Self {
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

    pub fn with_position(mut self, translation: Vec3) -> Self {
        self.sprite_bundle.transform.translation = translation;
        self
    }

    pub fn with_sprite(mut self, sprite: Sprite) -> Self {
        self.sprite_bundle.sprite = sprite;
        self
    }
}

#[derive(Bundle)]
pub struct EnemyBundle {
    pub sprite_bundle: SpriteBundle,
    pub enemy: Enemy,
}

impl EnemyBundle {
    pub fn new(enemy: Enemy) -> Self {
        Self {
            sprite_bundle: SpriteBundle {
                transform: Transform::from_xyz(0., 0., 0.)
                    .with_scale(Vec3::splat(TILE_SIZE / SPRITE_SIZE)),
                ..default()
            },
            enemy,
        }
    }

    pub fn with_texture(mut self, texture: Texture) -> Self {
        self.sprite_bundle.texture = texture;
        self
    }

    pub fn with_position(mut self, translation: Vec3) -> Self {
        self.sprite_bundle.transform.translation = translation;
        self
    }
}

#[derive(Component)]
pub struct Enemy {
    pub kind: EnemyKind,
    pub idx: usize,
}

impl Enemy {
    pub fn speed(&self) -> f32 {
        match self.kind {
            EnemyKind::Potato => 0.2,
            EnemyKind::Carrot => 0.4,
        }
    }
}

pub enum EnemyKind {
    Potato,
    Carrot,
}

#[derive(Resource, Default, Debug)]
pub struct Path {
    pub start_position: Vec2,
    pub end_position: Vec2,
    pub positions: VecDeque<PathTile>,
}

#[derive(Default, Debug)]
pub struct PathTile {
    pub position: Vec2,
}

impl PathTile {
    pub fn new(position: Vec2) -> Self {
        Self {
            position,
            ..default()
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, PartialOrd, Ord, Eq)]
pub enum MenuItem {
    Turret1x1,
    Turret2x2,
}

impl MenuItem {
    pub fn all() -> [Self; 2] {
        [Self::Turret1x1, Self::Turret2x2]
    }
}

#[derive(Resource)]
pub struct Menu {
    pub current_item: MenuItem,
}
