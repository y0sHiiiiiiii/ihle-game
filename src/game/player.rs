//! Ihle-Sprinter — Steuerung, Traegheit, Kollision, Speed-Boost.

use bevy::prelude::*;

use crate::game::assets::GameAssets;
use crate::game::map::{GameMap, TileType, MAP_HEIGHT, MAP_WIDTH, TILE_SIZE};
use crate::game::GameState;

pub const PLAYER_BASE_MAX_SPEED: f32 = 175.0;
pub const PLAYER_ACCEL: f32 = 620.0;
pub const PLAYER_FRICTION: f32 = 520.0;
pub const PLAYER_HITBOX: Vec2 = Vec2::new(18.0, 12.0);
pub const PLAYER_OFFROAD_FACTOR: f32 = 0.6;
pub const ROTATION_SPEED: f32 = 9.0;
pub const CAMERA_LERP: f32 = 6.5;

/// Extra top speed while a Nitro burst is active.
pub const NITRO_FACTOR: f32 = 0.6;
/// How long one full Nitro burst lasts.
pub const NITRO_DURATION: f32 = 3.0;

#[derive(Component, Default)]
pub struct Player {
    pub velocity: Vec2,
    pub facing: f32,
    pub last_input_dir: Vec2,
    pub has_package: bool,
    pub speed_boost_timer: f32,
    pub speed_boost_factor: f32,
    /// Accumulated wheel rotation phase, used to animate the rolling frames.
    pub wheel_phase: f32,
    /// Filled by driving; spent on the active Nitro burst (0..=1).
    pub nitro: f32,
    /// Seconds of Nitro burst still active.
    pub nitro_timer: f32,
}

impl Player {
    pub fn current_max_speed(&self) -> f32 {
        let mut mult = 1.0;
        if self.speed_boost_timer > 0.0 {
            mult += self.speed_boost_factor;
        }
        if self.nitro_timer > 0.0 {
            mult += NITRO_FACTOR;
        }
        PLAYER_BASE_MAX_SPEED * mult
    }

    /// True while any speed bonus (shop boost or Nitro) is active.
    pub fn is_boosted(&self) -> bool {
        self.speed_boost_timer > 0.0 || self.nitro_timer > 0.0
    }
}

#[derive(Component)]
pub struct PlayerCamera;

#[derive(Component)]
pub struct PackageVisual;

#[derive(Component)]
pub struct ShadowVisual;

#[derive(Event)]
pub struct CollisionBumpEvent {
    /// World position where the bump happened (for sparks).
    pub pos: Vec2,
}

#[derive(Event)]
pub struct NitroActivatedEvent;

pub struct PlayerPlugin;

impl Plugin for PlayerPlugin {
    fn build(&self, app: &mut App) {
        app.add_event::<CollisionBumpEvent>()
            .add_event::<NitroActivatedEvent>()
            .add_systems(OnEnter(GameState::Playing), spawn_player)
            .add_systems(OnExit(GameState::GameOver), despawn_player_on_reset)
            .add_systems(
                Update,
                (
                    player_input,
                    player_movement,
                    update_player_sprite,
                    camera_follow,
                    update_package_visual,
                    tick_boost_timer,
                )
                    .chain()
                    .run_if(in_state(GameState::Playing)),
            )
            .add_systems(
                Update,
                (camera_follow, update_player_sprite).run_if(in_state(GameState::Paused)),
            )
            .add_systems(
                Update,
                (camera_follow, update_player_sprite).run_if(in_state(GameState::Shopping)),
            );
    }
}

fn spawn_player(
    mut commands: Commands,
    assets: Res<GameAssets>,
    map: Res<GameMap>,
    existing: Query<Entity, With<Player>>,
    cams: Query<Entity, With<PlayerCamera>>,
    menu_cams: Query<Entity, (With<Camera>, Without<PlayerCamera>)>,
) {
    for e in &existing {
        commands.entity(e).despawn_recursive();
    }
    for e in &cams {
        commands.entity(e).despawn();
    }
    for e in &menu_cams {
        commands.entity(e).despawn();
    }

    let spawn_world = GameMap::tile_to_world(map.spawn_tile);

    commands
        .spawn((
            SpriteBundle {
                texture: assets.sprinter.clone(),
                transform: Transform::from_xyz(spawn_world.x, spawn_world.y, 10.0)
                    .with_scale(Vec3::splat(1.0)),
                sprite: Sprite {
                    custom_size: Some(Vec2::new(28.0, 18.0)),
                    ..default()
                },
                ..default()
            },
            Player {
                last_input_dir: Vec2::X,
                ..default()
            },
        ))
        .with_children(|parent| {
            // Soft drop shadow under the van (renders below the body).
            parent.spawn((
                SpriteBundle {
                    texture: assets.shadow.clone(),
                    transform: Transform::from_xyz(1.5, -5.0, -0.6),
                    sprite: Sprite {
                        custom_size: Some(Vec2::new(30.0, 16.0)),
                        ..default()
                    },
                    ..default()
                },
                ShadowVisual,
            ));
            parent.spawn((
                SpriteBundle {
                    texture: assets.package.clone(),
                    transform: Transform::from_xyz(0.0, 4.0, 0.4),
                    sprite: Sprite {
                        custom_size: Some(Vec2::new(10.0, 10.0)),
                        color: Color::srgba(1.0, 1.0, 1.0, 0.0),
                        ..default()
                    },
                    ..default()
                },
                PackageVisual,
            ));
        });

    let mut cam_bundle = Camera2dBundle::default();
    cam_bundle.transform = Transform::from_xyz(spawn_world.x, spawn_world.y, 999.9);
    cam_bundle.projection.scale = 0.55;
    cam_bundle.projection.near = -1000.0;
    cam_bundle.projection.far = 1000.0;
    commands.spawn((cam_bundle, PlayerCamera));
}

fn despawn_player_on_reset(
    mut commands: Commands,
    players: Query<Entity, With<Player>>,
    cams: Query<Entity, With<PlayerCamera>>,
) {
    for e in &players {
        commands.entity(e).despawn_recursive();
    }
    for e in &cams {
        commands.entity(e).despawn();
    }
}

fn player_input(
    keys: Res<ButtonInput<KeyCode>>,
    mut q: Query<&mut Player>,
    mut nitro_ev: EventWriter<NitroActivatedEvent>,
) {
    let Ok(mut player) = q.get_single_mut() else {
        return;
    };
    let mut dir = Vec2::ZERO;
    if keys.pressed(KeyCode::KeyW) || keys.pressed(KeyCode::ArrowUp) {
        dir.y += 1.0;
    }
    if keys.pressed(KeyCode::KeyS) || keys.pressed(KeyCode::ArrowDown) {
        dir.y -= 1.0;
    }
    if keys.pressed(KeyCode::KeyD) || keys.pressed(KeyCode::ArrowRight) {
        dir.x += 1.0;
    }
    if keys.pressed(KeyCode::KeyA) || keys.pressed(KeyCode::ArrowLeft) {
        dir.x -= 1.0;
    }
    let dir = dir.normalize_or_zero();
    player.last_input_dir = if dir != Vec2::ZERO {
        dir
    } else {
        player.last_input_dir
    };

    // Nitro: a full meter can be spent for a strong burst.
    if (keys.just_pressed(KeyCode::Space) || keys.just_pressed(KeyCode::ShiftLeft))
        && player.nitro >= 1.0
        && player.nitro_timer <= 0.0
    {
        player.nitro = 0.0;
        player.nitro_timer = NITRO_DURATION;
        nitro_ev.send(NitroActivatedEvent);
    }
}

fn player_movement(
    time: Res<Time>,
    keys: Res<ButtonInput<KeyCode>>,
    map: Res<GameMap>,
    mut q: Query<(&mut Transform, &mut Player)>,
    mut bump_events: EventWriter<CollisionBumpEvent>,
) {
    let Ok((mut transform, mut player)) = q.get_single_mut() else {
        return;
    };

    let mut dir = Vec2::ZERO;
    if keys.pressed(KeyCode::KeyW) || keys.pressed(KeyCode::ArrowUp) {
        dir.y += 1.0;
    }
    if keys.pressed(KeyCode::KeyS) || keys.pressed(KeyCode::ArrowDown) {
        dir.y -= 1.0;
    }
    if keys.pressed(KeyCode::KeyD) || keys.pressed(KeyCode::ArrowRight) {
        dir.x += 1.0;
    }
    if keys.pressed(KeyCode::KeyA) || keys.pressed(KeyCode::ArrowLeft) {
        dir.x -= 1.0;
    }
    let dir = dir.normalize_or_zero();
    let dt = time.delta_seconds().min(0.05);

    let pos = transform.translation.truncate();
    let center_tile = GameMap::world_to_tile(pos);
    let on_road = matches!(
        map.tile_at(center_tile.x, center_tile.y),
        TileType::Road
            | TileType::RoadH
            | TileType::RoadV
            | TileType::Crosswalk
            | TileType::Parking
    );

    let surface_factor = if on_road { 1.0 } else { PLAYER_OFFROAD_FACTOR };
    let max_speed = player.current_max_speed() * surface_factor;

    let target = dir * max_speed;
    let delta = target - player.velocity;
    let max_step = if dir != Vec2::ZERO {
        PLAYER_ACCEL * dt
    } else {
        PLAYER_FRICTION * dt
    };
    if delta.length() < max_step {
        player.velocity = target;
    } else {
        player.velocity += delta.normalize_or_zero() * max_step;
    }

    let velocity = player.velocity;
    let step = velocity * dt;
    let pos = transform.translation;

    let mut new_x = pos.x + step.x;
    if step.x.abs() > 0.0 && collides_at(&map, Vec2::new(new_x, pos.y)) {
        new_x = pos.x;
        if player.velocity.x.abs() > 60.0 {
            bump_events.send(CollisionBumpEvent {
                pos: pos.truncate(),
            });
        }
        player.velocity.x = -player.velocity.x * 0.2;
    }

    let mut new_y = pos.y + step.y;
    if step.y.abs() > 0.0 && collides_at(&map, Vec2::new(new_x, new_y)) {
        new_y = pos.y;
        if player.velocity.y.abs() > 60.0 {
            bump_events.send(CollisionBumpEvent {
                pos: pos.truncate(),
            });
        }
        player.velocity.y = -player.velocity.y * 0.2;
    }

    let clamp_w = (MAP_WIDTH as f32 * TILE_SIZE) * 0.5 - PLAYER_HITBOX.x;
    let clamp_h = (MAP_HEIGHT as f32 * TILE_SIZE) * 0.5 - PLAYER_HITBOX.y;
    transform.translation.x = new_x.clamp(-clamp_w, clamp_w);
    transform.translation.y = new_y.clamp(-clamp_h, clamp_h);

    if player.velocity.length_squared() > 25.0 {
        let target_facing = player.velocity.y.atan2(player.velocity.x);
        let mut current = player.facing;
        let mut delta = target_facing - current;
        while delta > std::f32::consts::PI {
            delta -= std::f32::consts::TAU;
        }
        while delta < -std::f32::consts::PI {
            delta += std::f32::consts::TAU;
        }
        let step = ROTATION_SPEED * dt;
        let clamped = delta.clamp(-step, step);
        current += clamped;
        player.facing = current;
    }

    // Charge the Nitro meter by actually driving — faster on tarmac.
    if player.nitro_timer <= 0.0 && player.nitro < 1.0 {
        let speed_frac = (player.velocity.length() / PLAYER_BASE_MAX_SPEED).clamp(0.0, 1.0);
        let rate = if on_road { 0.115 } else { 0.05 };
        player.nitro = (player.nitro + rate * speed_frac * dt).min(1.0);
    }
}

fn collides_at(map: &GameMap, pos: Vec2) -> bool {
    let half_w = PLAYER_HITBOX.x * 0.5;
    let half_h = PLAYER_HITBOX.y * 0.5;
    let corners = [
        Vec2::new(pos.x - half_w, pos.y - half_h),
        Vec2::new(pos.x + half_w, pos.y - half_h),
        Vec2::new(pos.x - half_w, pos.y + half_h),
        Vec2::new(pos.x + half_w, pos.y + half_h),
        Vec2::new(pos.x, pos.y),
    ];
    for c in corners {
        let t = GameMap::world_to_tile(c);
        if map.tile_at(t.x, t.y).is_blocking() {
            return true;
        }
    }
    false
}

fn update_player_sprite(
    time: Res<Time>,
    assets: Res<GameAssets>,
    mut q: Query<(&mut Player, &mut Transform, &mut Handle<Image>, &mut Sprite)>,
) {
    let Ok((mut player, mut transform, mut texture, mut sprite)) = q.get_single_mut() else {
        return;
    };
    // No bitmap rotation — pick a clean directional sprite instead.
    transform.rotation = Quat::IDENTITY;

    let speed = player.velocity.length();
    player.wheel_phase += speed * time.delta_seconds() * 0.18;
    let frame = (player.wheel_phase as i32 % 2).unsigned_abs() as usize;

    // Choose orientation from the facing vector (cardinal snap).
    let dir = Vec2::from_angle(player.facing);
    let horizontal = dir.x.abs() >= dir.y.abs();
    let (handle, size, flip_x, flip_y) = if horizontal {
        (
            assets.van_side[frame].clone(),
            Vec2::new(30.0, 20.0),
            dir.x < 0.0,
            false,
        )
    } else {
        (
            assets.van_top[frame].clone(),
            Vec2::new(20.0, 30.0),
            false,
            dir.y < 0.0,
        )
    };

    if texture.id() != handle.id() {
        *texture = handle;
    }
    sprite.custom_size = Some(size);
    sprite.flip_x = flip_x;
    sprite.flip_y = flip_y;

    // Boost / nitro flashes the body warm; otherwise full white.
    if player.is_boosted() || player.nitro_timer > 0.0 {
        let t = (time.elapsed_seconds() * 14.0).sin() * 0.5 + 0.5;
        sprite.color = Color::srgb(1.0, 0.92 + t * 0.08, 0.55 + t * 0.45);
    } else {
        sprite.color = Color::WHITE;
    }
}

fn update_package_visual(
    player_q: Query<&Player>,
    mut visual_q: Query<&mut Sprite, With<PackageVisual>>,
) {
    let Ok(player) = player_q.get_single() else {
        return;
    };
    for mut sprite in &mut visual_q {
        let target_alpha = if player.has_package { 1.0 } else { 0.0 };
        sprite.color = Color::srgba(1.0, 1.0, 1.0, target_alpha);
    }
}

fn camera_follow(
    time: Res<Time>,
    shake: Res<crate::game::fx::ScreenShake>,
    player_q: Query<(&Transform, &Player), Without<PlayerCamera>>,
    mut cam_q: Query<&mut Transform, With<PlayerCamera>>,
) {
    let Ok((player_tr, player)) = player_q.get_single() else {
        return;
    };
    let Ok(mut cam) = cam_q.get_single_mut() else {
        return;
    };
    // Look slightly ahead in the direction of travel for a more dynamic feel.
    let lookahead = player.velocity * 0.18;
    let target = player_tr.translation.truncate() + lookahead;
    let current = cam.translation.truncate();
    let t = (CAMERA_LERP * time.delta_seconds()).clamp(0.0, 1.0);
    let mut new = current.lerp(target, t);

    // Trauma-based shake (quadratic falloff), driven by FX events.
    let amount = shake.trauma * shake.trauma;
    if amount > 0.0001 {
        let et = time.elapsed_seconds();
        new.x += (et * 47.0).sin() * amount * 11.0;
        new.y += (et * 59.0).cos() * amount * 11.0;
    }
    cam.translation.x = new.x;
    cam.translation.y = new.y;
}

fn tick_boost_timer(time: Res<Time>, mut q: Query<&mut Player>) {
    if let Ok(mut player) = q.get_single_mut() {
        let dt = time.delta_seconds();
        if player.speed_boost_timer > 0.0 {
            player.speed_boost_timer -= dt;
            if player.speed_boost_timer < 0.0 {
                player.speed_boost_timer = 0.0;
                player.speed_boost_factor = 0.0;
            }
        }
        if player.nitro_timer > 0.0 {
            player.nitro_timer -= dt;
            if player.nitro_timer < 0.0 {
                player.nitro_timer = 0.0;
            }
        }
    }
}
