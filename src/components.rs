use std::collections::VecDeque;
use std::time::Duration;

use bevy::reflect::TypeUuid;
use bevy::{math::*, prelude::*};

use rand::{thread_rng, Rng};

use crate::{
    Texture, CURSOR_COLOR, ENEMY_LAYER, PATH_LAYER, SPRITE_SIZE, TILE_SIZE, TIME_STEP, WALL_COLOR,
};

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
pub struct Selected;

#[derive(Component, Debug, Default)]
pub struct GridCursor {
    pub can_place: bool,
    pub last_target_pos: Vec2,
    pub last_sample: f32,
    pub selection_size: Vec2,
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
        asset_server.load(match self {
            Turret::Turret1x1 => "resources/turret-2.png",
            Turret::Turret2x2 => "resources/turret-1.png",
        })
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

#[derive(Component, Debug, Clone, Copy)]
pub enum Gun {
    Gun1,
    Gun2,
}

#[derive(Component, Debug, Clone, Copy, Default)]
pub struct GunState {
    pub last_shot: Duration,
}

impl Gun {
    pub fn scale(&self) -> Vec2 {
        match self {
            Gun::Gun1 => Vec2::splat(1. / SPRITE_SIZE),
            Gun::Gun2 => Vec2::splat(2. / SPRITE_SIZE),
        }
    }

    pub fn sprite(&self, asset_server: &Res<AssetServer>) -> Handle<Image> {
        asset_server.load(match self {
            Gun::Gun1 => "resources/gun-2.png",
            Gun::Gun2 => "resources/gun-1-alt.png",
        })
    }

    pub fn range(&self) -> f32 {
        TILE_SIZE
            * match self {
                Self::Gun1 => 5.0,
                Self::Gun2 => 8.0,
            }
    }

    pub fn rate(&self) -> f32 {
        match self {
            Self::Gun1 => 1.8,
            Self::Gun2 => 2.5,
        }
    }
}
#[derive(Bundle)]
pub struct GunBundle {
    pub sprite_bundle: SpriteBundle,
    pub gun: Gun,
    pub gun_state: GunState,
}

impl GunBundle {
    pub fn new(gun: Gun, asset_server: &Res<AssetServer>) -> Self {
        Self {
            sprite_bundle: SpriteBundle {
                transform: Transform::from_xyz(0., 0., 10.)
                    .with_scale(Vec2::splat(1. / SPRITE_SIZE).extend(0.0)), // TODO z layer
                texture: gun.sprite(asset_server),
                ..default()
            },
            gun,
            gun_state: default(),
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

#[derive(Component, Debug, Clone, Copy)]
pub enum Projectile {
    Knife,
}

impl Projectile {
    pub fn scale(&self) -> Vec2 {
        match self {
            Projectile::Knife => Vec2::splat(1. / SPRITE_SIZE),
        }
    }

    pub fn sprite(&self, asset_server: &Res<AssetServer>) -> Handle<Image> {
        asset_server.load(match self {
            Projectile::Knife => "resources/knife.png",
        })
    }

    pub fn velocity(&self) -> f32 {
        TIME_STEP
            * TILE_SIZE
            * match self {
                Self::Knife => 3.5,
            }
    }
}
#[derive(Bundle)]
pub struct ProjectileBundle {
    pub sprite_bundle: SpriteBundle,
    pub projectile: Projectile,
}

impl ProjectileBundle {
    pub fn new(projectile: Projectile, asset_server: &Res<AssetServer>) -> Self {
        Self {
            sprite_bundle: SpriteBundle {
                transform:
                    Transform::from_xyz(0., 0., 10.) // TODO z layer?
                        .with_scale(Vec2::splat(1. / SPRITE_SIZE).extend(0.0)),
                texture: projectile.sprite(asset_server),
                ..default()
            },
            projectile,
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

// Background tile
#[derive(Bundle)]
pub struct Tile {
    pub sprite_bundle: SpriteBundle,
}

impl Tile {
    pub fn new(asset_server: &Res<AssetServer>) -> Self {
        let mut rng = thread_rng();
        Self {
            sprite_bundle: SpriteBundle {
                transform: Transform::from_scale(Vec2::splat(TILE_SIZE / SPRITE_SIZE).extend(0.)),
                texture: asset_server.load(if rng.gen_range(0..10) == 0 {
                    "resources/grass.png"
                } else {
                    "resources/dirt.png"
                }),
                ..default()
            },
        }
    }

    pub fn with_position(mut self, translation: Vec3) -> Self {
        self.sprite_bundle.transform.translation = translation;
        self
    }

    pub fn with_texture(mut self, texture: Texture) -> Self {
        self.sprite_bundle.texture = texture;
        self
    }
}

// Root path
#[derive(Bundle)]
pub struct PathTile {
    pub sprite_bundle: SpriteBundle,
    pub collider: Collider,
}

impl PathTile {
    pub fn new(asset_server: &Res<AssetServer>) -> Self {
        let mut rng = thread_rng();
        Self {
            sprite_bundle: SpriteBundle {
                transform: Transform::from_scale(
                    Vec2::splat(TILE_SIZE / SPRITE_SIZE).extend(PATH_LAYER),
                ),
                texture: asset_server.load(if rng.gen_range(0..10) == 0 {
                    "resources/overgrown_path.png"
                } else {
                    "resources/path.png"
                }),
                ..default()
            },
            collider: Collider,
        }
    }

    pub fn with_position(mut self, translation: Vec3) -> Self {
        self.sprite_bundle.transform.translation = translation;
        self
    }

    pub fn with_texture(mut self, texture: Texture) -> Self {
        self.sprite_bundle.texture = texture;
        self
    }
}

#[derive(Bundle)]
pub struct EnemyBundle {
    pub sprite_sheet_bundle: SpriteSheetBundle,
    pub enemy: Enemy,
}

impl EnemyBundle {
    pub fn new(
        enemy: Enemy,
        asset_server: &Res<AssetServer>,
        texture_atlases: &mut ResMut<Assets<TextureAtlas>>,
    ) -> Self {
        let atlas = enemy.atlas(asset_server);
        let tah = texture_atlases.add(atlas); // FIXME adding the texture every time is probably wrong (branch atlas-refactoring)

        Self {
            sprite_sheet_bundle: SpriteSheetBundle {
                transform: Transform::from_scale(
                    Vec2::splat(TILE_SIZE / SPRITE_SIZE).extend(ENEMY_LAYER),
                ),
                texture_atlas: tah,
                ..default()
            },
            enemy,
        }
    }
    pub fn with_position(mut self, translation: Vec3) -> Self {
        self.sprite_sheet_bundle.transform.translation = translation;
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
        TIME_STEP
            * TILE_SIZE
            * match self.kind {
                EnemyKind::Potato => 2.5,
                EnemyKind::Carrot => 4.,
                EnemyKind::Pepper => 1.,
            }
    }

    pub fn atlas(&self, asset_server: &Res<AssetServer>) -> TextureAtlas {
        let texture_handle = asset_server.load(match self.kind {
            EnemyKind::Potato => "resources/potato.png",
            EnemyKind::Carrot => "resources/carrot.png",
            EnemyKind::Pepper => "resources/pepper.png",
        });

        TextureAtlas::from_grid(texture_handle, Vec2::new(32.0, 32.0), 4, 1, None, None)
    }
}

#[derive(Debug, Clone)]
pub enum EnemyKind {
    Potato,
    Carrot,
    Pepper,
}

#[derive(Resource, Default, Debug)]
pub struct Path {
    pub start_position: Vec2,
    pub end_position: Vec2,
    pub positions: VecDeque<PathNode>,
}

#[derive(Default, Debug)]
pub struct PathNode {
    pub position: Vec2,
}

impl PathNode {
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
