//! Game feel: screen shake + lightweight particle bursts.
//!
//! The juice that makes deliveries and crashes *land*: a trauma-based screen
//! shake (read by the camera in `player.rs`) and short-lived sprite particles
//! for coins, sparks and exhaust.

use bevy::prelude::*;
use rand::prelude::*;

use crate::game::assets::GameAssets;
use crate::game::delivery::{DeliveryCompletedEvent, DeliveryLateEvent, PackagePickedUpEvent};
use crate::game::gamestate::GameState;
use crate::game::player::{CollisionBumpEvent, NitroActivatedEvent, Player};

/// Trauma-based camera shake. `trauma` decays every frame; the camera squares
/// it for a punchy falloff.
#[derive(Resource, Default)]
pub struct ScreenShake {
    pub trauma: f32,
}

impl ScreenShake {
    pub fn add(&mut self, amount: f32) {
        self.trauma = (self.trauma + amount).min(1.0);
    }
}

#[derive(Component)]
pub struct Particle {
    velocity: Vec2,
    life: f32,
    max_life: f32,
    spin: f32,
    gravity: f32,
    start_scale: f32,
    end_scale: f32,
    base_color: Color,
}

/// Marker so all particles can be cleared on state transitions.
#[derive(Component)]
pub struct Fx;

pub struct FxPlugin;

impl Plugin for FxPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<ScreenShake>()
            .add_systems(OnEnter(GameState::Playing), clear_fx)
            .add_systems(OnEnter(GameState::GameOver), clear_fx)
            .add_systems(
                Update,
                (
                    decay_shake,
                    update_particles,
                    on_bump,
                    on_nitro,
                    on_delivery_complete,
                    on_pickup,
                    on_late,
                    exhaust_trail,
                )
                    .run_if(in_state(GameState::Playing)),
            );
    }
}

fn clear_fx(mut commands: Commands, q: Query<Entity, With<Particle>>) {
    for e in &q {
        commands.entity(e).despawn();
    }
}

fn decay_shake(time: Res<Time>, mut shake: ResMut<ScreenShake>) {
    shake.trauma = (shake.trauma - time.delta_seconds() * 1.7).max(0.0);
}

#[allow(clippy::too_many_arguments)]
fn spawn_particle(
    commands: &mut Commands,
    texture: Handle<Image>,
    pos: Vec2,
    velocity: Vec2,
    life: f32,
    color: Color,
    start_scale: f32,
    end_scale: f32,
    gravity: f32,
) {
    commands.spawn((
        SpriteBundle {
            texture,
            transform: Transform::from_xyz(pos.x, pos.y, 30.0)
                .with_scale(Vec3::splat(start_scale)),
            sprite: Sprite {
                custom_size: Some(Vec2::splat(8.0)),
                color,
                ..default()
            },
            ..default()
        },
        Particle {
            velocity,
            life,
            max_life: life,
            spin: thread_rng().gen_range(-6.0..6.0),
            gravity,
            start_scale,
            end_scale,
            base_color: color,
        },
        Fx,
    ));
}

fn update_particles(
    time: Res<Time>,
    mut commands: Commands,
    mut q: Query<(Entity, &mut Transform, &mut Sprite, &mut Particle)>,
) {
    let dt = time.delta_seconds().min(0.05);
    for (e, mut tr, mut sprite, mut p) in &mut q {
        p.life -= dt;
        if p.life <= 0.0 {
            commands.entity(e).despawn();
            continue;
        }
        p.velocity.y -= p.gravity * dt;
        p.velocity *= 1.0 - 2.0 * dt; // drag
        tr.translation.x += p.velocity.x * dt;
        tr.translation.y += p.velocity.y * dt;
        tr.rotation *= Quat::from_rotation_z(p.spin * dt);

        let t = 1.0 - (p.life / p.max_life);
        let scale = p.start_scale + (p.end_scale - p.start_scale) * t;
        tr.scale = Vec3::splat(scale);
        let alpha = (p.life / p.max_life).clamp(0.0, 1.0);
        let c = p.base_color.to_srgba();
        sprite.color = Color::srgba(c.red, c.green, c.blue, alpha);
    }
}

fn on_bump(
    mut commands: Commands,
    assets: Res<GameAssets>,
    mut shake: ResMut<ScreenShake>,
    mut events: EventReader<CollisionBumpEvent>,
) {
    let mut rng = thread_rng();
    for ev in events.read() {
        shake.add(0.35);
        for _ in 0..7 {
            let ang = rng.gen_range(0.0..std::f32::consts::TAU);
            let spd = rng.gen_range(40.0..130.0);
            let vel = Vec2::new(ang.cos(), ang.sin()) * spd;
            spawn_particle(
                &mut commands,
                assets.spark.clone(),
                ev.pos,
                vel,
                rng.gen_range(0.25..0.5),
                Color::srgb(1.0, 0.85, 0.45),
                1.2,
                0.2,
                160.0,
            );
        }
    }
}

fn on_nitro(
    mut commands: Commands,
    assets: Res<GameAssets>,
    mut shake: ResMut<ScreenShake>,
    mut events: EventReader<NitroActivatedEvent>,
    player_q: Query<&Transform, With<Player>>,
) {
    let mut rng = thread_rng();
    let Ok(tr) = player_q.get_single() else {
        events.clear();
        return;
    };
    let pos = tr.translation.truncate();
    for _ in events.read() {
        shake.add(0.3);
        for _ in 0..14 {
            let ang = rng.gen_range(0.0..std::f32::consts::TAU);
            let spd = rng.gen_range(60.0..170.0);
            let vel = Vec2::new(ang.cos(), ang.sin()) * spd;
            spawn_particle(
                &mut commands,
                assets.spark.clone(),
                pos,
                vel,
                rng.gen_range(0.3..0.6),
                Color::srgb(1.0, 0.7, 0.2),
                1.6,
                0.3,
                0.0,
            );
        }
    }
}

fn on_delivery_complete(
    mut commands: Commands,
    assets: Res<GameAssets>,
    mut shake: ResMut<ScreenShake>,
    mut events: EventReader<DeliveryCompletedEvent>,
) {
    let mut rng = thread_rng();
    for ev in events.read() {
        shake.add(0.28 + (ev.streak.min(8) as f32) * 0.02);
        // Coin fountain.
        for _ in 0..12 {
            let vel = Vec2::new(rng.gen_range(-70.0..70.0), rng.gen_range(120.0..220.0));
            spawn_particle(
                &mut commands,
                assets.coin_icon.clone(),
                ev.pos,
                vel,
                rng.gen_range(0.6..1.0),
                Color::WHITE,
                1.6,
                1.2,
                380.0,
            );
        }
        // Star sparkle.
        for _ in 0..8 {
            let ang = rng.gen_range(0.0..std::f32::consts::TAU);
            let spd = rng.gen_range(50.0..140.0);
            spawn_particle(
                &mut commands,
                assets.spark.clone(),
                ev.pos,
                Vec2::new(ang.cos(), ang.sin()) * spd,
                rng.gen_range(0.4..0.7),
                Color::srgb(1.0, 0.95, 0.5),
                1.8,
                0.2,
                40.0,
            );
        }
    }
}

fn on_pickup(
    mut commands: Commands,
    assets: Res<GameAssets>,
    mut events: EventReader<PackagePickedUpEvent>,
) {
    let mut rng = thread_rng();
    for ev in events.read() {
        for _ in 0..8 {
            let ang = rng.gen_range(0.0..std::f32::consts::TAU);
            let spd = rng.gen_range(40.0..110.0);
            spawn_particle(
                &mut commands,
                assets.spark.clone(),
                ev.pos,
                Vec2::new(ang.cos(), ang.sin()) * spd,
                rng.gen_range(0.3..0.55),
                Color::srgb(0.45, 0.7, 1.0),
                1.4,
                0.2,
                30.0,
            );
        }
    }
}

fn on_late(
    mut shake: ResMut<ScreenShake>,
    mut events: EventReader<DeliveryLateEvent>,
) {
    for ev in events.read() {
        shake.add(if ev.game_over { 0.8 } else { 0.55 });
    }
}

/// Puff of exhaust trailing the van while any boost is active.
fn exhaust_trail(
    time: Res<Time>,
    mut commands: Commands,
    assets: Res<GameAssets>,
    mut acc: Local<f32>,
    player_q: Query<(&Transform, &Player)>,
) {
    let Ok((tr, player)) = player_q.get_single() else {
        return;
    };
    if !player.is_boosted() || player.velocity.length() < 40.0 {
        return;
    }
    *acc += time.delta_seconds();
    if *acc < 0.045 {
        return;
    }
    *acc = 0.0;
    let mut rng = thread_rng();
    let back = -player.velocity.normalize_or_zero();
    let pos = tr.translation.truncate() + back * 12.0;
    spawn_particle(
        &mut commands,
        assets.spark.clone(),
        pos + Vec2::new(rng.gen_range(-3.0..3.0), rng.gen_range(-3.0..3.0)),
        back * rng.gen_range(20.0..50.0),
        rng.gen_range(0.25..0.45),
        Color::srgba(0.85, 0.85, 0.9, 0.8),
        1.2,
        2.2,
        0.0,
    );
}
