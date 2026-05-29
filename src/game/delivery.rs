//! Auftrags-System: Pickup, Dropoff, Timer, Schwierigkeit, Leben, Score.

use bevy::prelude::*;
use rand::prelude::*;

use crate::game::assets::GameAssets;
use crate::game::gamestate::GameState;
use crate::game::map::{AddressTarget, GameMap, Landmark};
use crate::game::player::Player;

pub const INTERACT_RADIUS: f32 = 36.0;
pub const STARTING_LIVES: u32 = 3;
pub const STARTING_COINS: u32 = 8;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum DeliveryPhase {
    GoToPickup,
    GoToDropoff,
}

#[derive(Resource, Clone, Debug)]
pub struct ActiveDelivery {
    pub phase: DeliveryPhase,
    pub pickup: Landmark,
    pub dropoff: AddressTarget,
    pub time_remaining: f32,
    #[allow(dead_code)]
    pub time_limit: f32,
    pub used_boost: bool,
}

#[derive(Resource, Default)]
pub struct DeliveryStats {
    pub score: u64,
    pub coins: u32,
    pub lives: u32,
    pub delivery_count: u32,
    pub streak: u32,
}

impl DeliveryStats {
    pub fn fresh_start() -> Self {
        Self {
            score: 0,
            coins: STARTING_COINS,
            lives: STARTING_LIVES,
            delivery_count: 0,
            streak: 0,
        }
    }
}

#[derive(Resource, Default)]
pub struct LateNotification {
    pub timer: f32,
    pub address: String,
}

#[derive(Resource, Default)]
pub struct DeliveryFeedback {
    pub timer: f32,
    pub text: String,
    pub color: Color,
}

#[derive(Component)]
pub struct PickupMarker;

#[derive(Component)]
pub struct DropoffMarker;

/// Fired when a delivery is dropped off successfully.
#[derive(Event)]
pub struct DeliveryCompletedEvent {
    pub pos: Vec2,
    pub points: u64,
    pub streak: u32,
}

/// Fired when a package is picked up at an Ihle store.
#[derive(Event)]
pub struct PackagePickedUpEvent {
    pub pos: Vec2,
}

/// Fired when the timer runs out and a life is lost.
#[derive(Event)]
pub struct DeliveryLateEvent {
    pub game_over: bool,
}

pub struct DeliveryPlugin;

impl Plugin for DeliveryPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<LateNotification>()
            .init_resource::<DeliveryFeedback>()
            .add_event::<DeliveryCompletedEvent>()
            .add_event::<PackagePickedUpEvent>()
            .add_event::<DeliveryLateEvent>()
            .add_systems(OnEnter(GameState::Playing), start_run)
            .add_systems(OnExit(GameState::Playing), clear_markers_on_exit)
            .add_systems(
                Update,
                (
                    tick_delivery_timer,
                    sync_markers,
                    handle_interaction,
                    bob_markers,
                    tick_feedback_timers,
                )
                    .run_if(in_state(GameState::Playing)),
            );
    }
}

fn start_run(mut commands: Commands, map: Res<GameMap>) {
    commands.insert_resource(DeliveryStats::fresh_start());
    let delivery = generate_delivery(&map, 0);
    commands.insert_resource(delivery);
    commands.insert_resource(LateNotification::default());
    commands.insert_resource(DeliveryFeedback::default());
}

pub fn generate_delivery(map: &GameMap, delivery_count: u32) -> ActiveDelivery {
    let mut rng = thread_rng();
    let pickup = map.ihle_stores.choose(&mut rng).cloned().unwrap();
    let min_dist = if delivery_count >= 20 { 40 } else { 14 };
    let candidates: Vec<&AddressTarget> = map
        .addresses
        .iter()
        .filter(|a| (a.tile - pickup.tile).as_vec2().length() > min_dist as f32)
        .collect();
    let dropoff = if candidates.is_empty() {
        map.addresses.choose(&mut rng).cloned().unwrap()
    } else {
        (*candidates.choose(&mut rng).unwrap()).clone()
    };

    let limit = time_limit_for(delivery_count);

    ActiveDelivery {
        phase: DeliveryPhase::GoToPickup,
        pickup,
        dropoff,
        time_remaining: limit,
        time_limit: limit,
        used_boost: false,
    }
}

pub fn time_limit_for(count: u32) -> f32 {
    match count {
        0..=4 => 60.0,
        5..=9 => 50.0,
        10..=19 => 40.0,
        20..=34 => 30.0,
        _ => 22.0,
    }
}

fn tick_delivery_timer(
    time: Res<Time>,
    mut commands: Commands,
    map: Res<GameMap>,
    mut active: ResMut<ActiveDelivery>,
    mut stats: ResMut<DeliveryStats>,
    mut late: ResMut<LateNotification>,
    mut feedback: ResMut<DeliveryFeedback>,
    mut next_state: ResMut<NextState<GameState>>,
    mut player_q: Query<&mut Player>,
    mut late_ev: EventWriter<DeliveryLateEvent>,
) {
    active.time_remaining -= time.delta_seconds();
    if active.time_remaining > 0.0 {
        return;
    }

    if stats.lives > 0 {
        stats.lives -= 1;
    }
    stats.streak = 0;
    late.timer = 2.0;
    late.address = active.dropoff.name.clone();
    feedback.timer = 2.0;
    feedback.text = "ZU SPÄT! Kunde wartet nicht mehr!".to_string();
    feedback.color = Color::srgb(1.0, 0.25, 0.25);

    if let Ok(mut p) = player_q.get_single_mut() {
        p.has_package = false;
    }

    late_ev.send(DeliveryLateEvent {
        game_over: stats.lives == 0,
    });

    if stats.lives == 0 {
        next_state.set(GameState::GameOver);
        return;
    }

    let next = generate_delivery(&map, stats.delivery_count);
    commands.insert_resource(next);
}

fn sync_markers(
    mut commands: Commands,
    assets: Res<GameAssets>,
    active: Res<ActiveDelivery>,
    pickup_q: Query<Entity, With<PickupMarker>>,
    dropoff_q: Query<Entity, With<DropoffMarker>>,
) {
    let want_pickup = active.phase == DeliveryPhase::GoToPickup;
    let want_dropoff = active.phase == DeliveryPhase::GoToDropoff;

    let pickup_world = GameMap::tile_to_world(active.pickup.interact_tile);
    let dropoff_world = GameMap::tile_to_world(active.dropoff.tile);

    if want_pickup && pickup_q.is_empty() {
        commands.spawn((
            SpriteBundle {
                texture: assets.pickup_marker.clone(),
                transform: Transform::from_xyz(pickup_world.x, pickup_world.y + 22.0, 8.0),
                sprite: Sprite {
                    custom_size: Some(Vec2::new(20.0, 20.0)),
                    ..default()
                },
                ..default()
            },
            PickupMarker,
        ));
    }
    if !want_pickup {
        for e in &pickup_q {
            commands.entity(e).despawn();
        }
    }

    if want_dropoff && dropoff_q.is_empty() {
        commands.spawn((
            SpriteBundle {
                texture: assets.target_x.clone(),
                transform: Transform::from_xyz(dropoff_world.x, dropoff_world.y + 22.0, 8.0),
                sprite: Sprite {
                    custom_size: Some(Vec2::new(20.0, 20.0)),
                    ..default()
                },
                ..default()
            },
            DropoffMarker,
        ));
    }
    if !want_dropoff {
        for e in &dropoff_q {
            commands.entity(e).despawn();
        }
    }
}

fn bob_markers(
    time: Res<Time>,
    mut pickup_q: Query<&mut Transform, (With<PickupMarker>, Without<DropoffMarker>)>,
    mut dropoff_q: Query<
        (&mut Transform, &mut Sprite),
        (With<DropoffMarker>, Without<PickupMarker>),
    >,
) {
    let t = time.elapsed_seconds();
    for mut tr in &mut pickup_q {
        tr.translation.z = 8.0;
        tr.scale = Vec3::splat(1.0 + 0.08 * (t * 3.5).sin());
    }
    for (mut tr, mut sprite) in &mut dropoff_q {
        tr.translation.z = 8.0;
        tr.scale = Vec3::splat(1.0 + 0.12 * (t * 4.5).sin());
        let blink = ((t * 3.0).sin() * 0.5 + 0.5) * 0.4 + 0.6;
        sprite.color = Color::srgba(1.0, blink, blink, 1.0);
    }
}

fn handle_interaction(
    keys: Res<ButtonInput<KeyCode>>,
    mut commands: Commands,
    map: Res<GameMap>,
    mut active: ResMut<ActiveDelivery>,
    mut stats: ResMut<DeliveryStats>,
    mut feedback: ResMut<DeliveryFeedback>,
    mut player_q: Query<(&Transform, &mut Player)>,
    mut completed_ev: EventWriter<DeliveryCompletedEvent>,
    mut pickup_ev: EventWriter<PackagePickedUpEvent>,
) {
    if !keys.just_pressed(KeyCode::KeyE) {
        return;
    }
    let Ok((tr, mut player)) = player_q.get_single_mut() else {
        return;
    };
    let pos = tr.translation.truncate();

    match active.phase {
        DeliveryPhase::GoToPickup => {
            let target = GameMap::tile_to_world(active.pickup.interact_tile);
            if pos.distance(target) <= INTERACT_RADIUS {
                player.has_package = true;
                active.phase = DeliveryPhase::GoToDropoff;
                if player.is_boosted() {
                    active.used_boost = true;
                }
                feedback.timer = 1.5;
                feedback.text =
                    format!("Paket geladen - Ziel: {}", active.dropoff.name.clone());
                feedback.color = Color::srgb(0.45, 0.95, 0.45);
                pickup_ev.send(PackagePickedUpEvent { pos: target });
            }
        }
        DeliveryPhase::GoToDropoff => {
            let target = GameMap::tile_to_world(active.dropoff.tile);
            if pos.distance(target) <= INTERACT_RADIUS {
                player.has_package = false;
                let remaining = active.time_remaining.max(0.0);
                let time_frac = remaining / active.time_limit.max(1.0);
                let mut points = 100u64 + (remaining * 5.0) as u64;
                // Perfect-time bonus rewards fast, clean runs.
                if time_frac > 0.6 {
                    points += 50;
                }
                if player.is_boosted() || active.used_boost {
                    points += 20;
                }
                stats.streak += 1;
                // Escalating streak multiplier, capped, so a hot run snowballs.
                let mult = match stats.streak {
                    0..=2 => 1.0,
                    3..=4 => 1.5,
                    5..=7 => 2.0,
                    _ => 2.5,
                };
                points = (points as f32 * mult) as u64;
                let coins_earned = 2 + (remaining as u32 / 8) + (stats.streak / 3);
                stats.coins = stats.coins.saturating_add(coins_earned);
                stats.score = stats.score.saturating_add(points);
                stats.delivery_count += 1;
                // A clean delivery tops up the Nitro meter as a reward.
                player.nitro = (player.nitro + 0.34).min(1.0);

                feedback.timer = 2.0;
                let streak_tag = if stats.streak >= 3 {
                    format!("  STREAK x{:.1}", mult)
                } else {
                    String::new()
                };
                feedback.text = format!(
                    "Geliefert!  +{} Punkte  (+{} Münzen){}",
                    points, coins_earned, streak_tag
                );
                feedback.color = Color::srgb(0.95, 0.85, 0.25);

                completed_ev.send(DeliveryCompletedEvent {
                    pos: target,
                    points,
                    streak: stats.streak,
                });

                let next = generate_delivery(&map, stats.delivery_count);
                commands.insert_resource(next);
            }
        }
    }
}

fn tick_feedback_timers(
    time: Res<Time>,
    mut late: ResMut<LateNotification>,
    mut feedback: ResMut<DeliveryFeedback>,
) {
    if late.timer > 0.0 {
        late.timer -= time.delta_seconds();
        if late.timer < 0.0 {
            late.timer = 0.0;
        }
    }
    if feedback.timer > 0.0 {
        feedback.timer -= time.delta_seconds();
        if feedback.timer < 0.0 {
            feedback.timer = 0.0;
        }
    }
}

fn clear_markers_on_exit(
    mut commands: Commands,
    pickup_q: Query<Entity, With<PickupMarker>>,
    dropoff_q: Query<Entity, With<DropoffMarker>>,
) {
    for e in &pickup_q {
        commands.entity(e).despawn();
    }
    for e in &dropoff_q {
        commands.entity(e).despawn();
    }
}
