//! NPC-System: 4 Typen Passanten + Jannick (Pizzeria) mit Koelsch-Dialog.

use bevy::prelude::*;
use rand::prelude::*;

use crate::game::assets::{ts_body, GameAssets, UiFonts};
use crate::game::gamestate::GameState;
use crate::game::map::{GameMap, TileType, MAP_HEIGHT, MAP_WIDTH, ROAD_H_ROWS, ROAD_V_COLS};
use crate::game::player::{CollisionBumpEvent, Player};
use crate::game::shop::ShopRequest;
use crate::game::speech::spawn_speech_bubble;

/// Seconds each direction stays green before the lights flip.
pub const LIGHT_GREEN_TIME: f32 = 6.0;

/// Global, synchronised traffic-light phase. When `horizontal_green` is true,
/// east–west traffic flows and north–south waits (and vice-versa).
#[derive(Resource)]
pub struct TrafficLights {
    pub timer: f32,
    pub horizontal_green: bool,
}

impl Default for TrafficLights {
    fn default() -> Self {
        Self {
            timer: LIGHT_GREEN_TIME,
            horizontal_green: true,
        }
    }
}

/// A traffic-light lamp at an intersection. `controls_horizontal` lamps go green
/// for east–west traffic; the others for north–south.
#[derive(Component)]
pub struct TrafficLight {
    pub controls_horizontal: bool,
}

/// Is movement in `dir` allowed by the lights right now? A pedestrian crossing a
/// road on a zebra obeys the same rule as a car travelling that way.
fn light_go(lights: &TrafficLights, dir: IVec2) -> bool {
    if dir.x != 0 {
        lights.horizontal_green
    } else {
        !lights.horizontal_green
    }
}

/// What a pedestrian should do next.
enum PedStep {
    Go(IVec2),
    Wait,
}

/// Angry Bavarian shouts when the Sprinter slams into someone or another car.
pub const ANGRY_LINES: [&str; 6] = [
    "Sauhund, pass auf!",
    "Spinnst, oda was?!",
    "Mei Auto! Bist deppert?!",
    "Geh weida, du Depp!",
    "Ja sauba, fahr doch g'scheit!",
    "Himmi Herrgott, schau hi!",
];

pub const BASE_NPC_COUNT: usize = 16;
pub const EXTRA_NPC_COUNT: usize = 22;
pub const TRAFFIC_COUNT: usize = 16;
pub const JANNICK_INTERACT_RADIUS: f32 = 40.0;
/// Jannick shouts a Kölsch line at anyone driving within this range.
pub const JANNICK_CHATTER_RADIUS: f32 = 95.0;

pub const JANNICK_DIALOGS: [&str; 6] = [
    "Wat willste? Ne Pizza odder nen Kaffee? Hae?",
    "Ich sach dir, Germering is nix. Koelle! DAS is ne Stadt!",
    "Lecker Koelsch haet ich och, aevver dat verstehs du nit.",
    "Kauf dir wat, ich han Hunger un muss schliesse!",
    "Em Koelle am Rhing, da simmer doheem. Hier nit, aevver jut.",
    "Ey, dae Sprinter kuett vill ze schnell, pass op Minsch!",
];

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum NpcKind {
    Man,
    Woman,
    Child,
    Elder,
}

#[derive(Component)]
pub struct Npc {
    #[allow(dead_code)]
    pub kind: NpcKind,
    pub state_timer: f32,
    pub state: NpcState,
    /// The walkable tile the pedestrian is currently stepping toward.
    pub target_tile: IVec2,
    /// Last heading, so they prefer to keep going straight along the pavement.
    pub dir: IVec2,
    pub speed: f32,
    /// Stops the angry crash-line from re-triggering every frame.
    pub speech_cooldown: f32,
}

/// A car that drives the road grid on its own — pure rolling obstacle.
#[derive(Component)]
pub struct TrafficCar {
    pub dir: IVec2,
    pub speed: f32,
    pub target_tile: IVec2,
    /// True while halted at a red light (so it doesn't re-roll its heading).
    pub waiting: bool,
    pub speech_cooldown: f32,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum NpcState {
    Walking,
    Idle,
}

#[derive(Component)]
pub struct Jannick;

#[derive(Component)]
pub struct JannickHint;

#[derive(Resource, Default)]
pub struct JannickPromptVisible(pub bool);

/// Tracks Jannick's spontaneous chatter so he doesn't repeat every frame.
#[derive(Resource, Default)]
pub struct JannickChatter {
    pub cooldown: f32,
    pub was_near: bool,
}

pub struct NpcPlugin;

impl Plugin for NpcPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<JannickPromptVisible>()
            .init_resource::<JannickChatter>()
            .init_resource::<TrafficLights>()
            .add_systems(OnEnter(GameState::Playing), spawn_npcs)
            .add_systems(OnExit(GameState::GameOver), despawn_npcs)
            .add_systems(
                Update,
                (
                    cycle_traffic_lights.run_if(in_state(GameState::Playing)),
                    update_light_visuals.run_if(in_state(GameState::Playing)),
                    update_npcs.run_if(in_state(GameState::Playing)),
                    update_traffic.run_if(in_state(GameState::Playing)),
                    player_crash_reactions.run_if(in_state(GameState::Playing)),
                    handle_jannick_interaction.run_if(in_state(GameState::Playing)),
                    jannick_chatter.run_if(in_state(GameState::Playing)),
                    update_jannick_hint.run_if(in_state(GameState::Playing)),
                ),
            )
            .add_systems(
                Update,
                (update_jannick_hint,).run_if(in_state(GameState::Paused)),
            );
    }
}

fn spawn_npcs(
    mut commands: Commands,
    assets: Res<GameAssets>,
    fonts: Res<UiFonts>,
    map: Res<GameMap>,
    existing: Query<
        Entity,
        Or<(
            With<Npc>,
            With<TrafficCar>,
            With<TrafficLight>,
            With<Jannick>,
            With<JannickHint>,
        )>,
    >,
) {
    for e in &existing {
        commands.entity(e).despawn_recursive();
    }

    let mut rng = thread_rng();

    let total = BASE_NPC_COUNT + EXTRA_NPC_COUNT;
    let mut spawned = 0;
    let mut attempts = 0;
    while spawned < total && attempts < total * 30 {
        attempts += 1;
        let tx = rng.gen_range(2..MAP_WIDTH - 2);
        let ty = rng.gen_range(2..MAP_HEIGHT - 2);
        if map.tile_at(tx, ty) != TileType::Sidewalk {
            continue;
        }
        let kind_roll = rng.gen_range(0..4);
        let kind = match kind_roll {
            0 => NpcKind::Man,
            1 => NpcKind::Woman,
            2 => NpcKind::Child,
            _ => NpcKind::Elder,
        };
        let (handle, size) = match kind {
            NpcKind::Man => (assets.npc_man.clone(), Vec2::new(16.0, 22.0)),
            NpcKind::Woman => (assets.npc_woman.clone(), Vec2::new(16.0, 22.0)),
            NpcKind::Child => (assets.npc_child.clone(), Vec2::new(13.0, 18.0)),
            NpcKind::Elder => (assets.npc_elder.clone(), Vec2::new(16.0, 22.0)),
        };
        let speed = match kind {
            NpcKind::Child => 38.0,
            NpcKind::Elder => 18.0,
            _ => 26.0,
        };
        let world = GameMap::tile_to_world(IVec2::new(tx, ty));
        commands.spawn((
            SpriteBundle {
                texture: handle,
                transform: Transform::from_xyz(world.x, world.y, 6.0),
                sprite: Sprite {
                    custom_size: Some(size),
                    ..default()
                },
                ..default()
            },
            Npc {
                kind,
                state_timer: rng.gen_range(0.3..1.5),
                state: NpcState::Walking,
                target_tile: IVec2::new(tx, ty),
                dir: IVec2::ZERO,
                speed,
                speech_cooldown: 0.0,
            },
        ));
        spawned += 1;
    }

    // Rolling traffic on the road grid.
    let mut cars = 0;
    let mut car_attempts = 0;
    while cars < TRAFFIC_COUNT && car_attempts < TRAFFIC_COUNT * 80 {
        car_attempts += 1;
        let tx = rng.gen_range(3..MAP_WIDTH - 3);
        let ty = rng.gen_range(3..MAP_HEIGHT - 3);
        let dir = match map.tile_at(tx, ty) {
            TileType::RoadH => {
                if rng.gen_bool(0.5) {
                    IVec2::new(1, 0)
                } else {
                    IVec2::new(-1, 0)
                }
            }
            TileType::RoadV => {
                if rng.gen_bool(0.5) {
                    IVec2::new(0, 1)
                } else {
                    IVec2::new(0, -1)
                }
            }
            TileType::Road => *[
                IVec2::new(1, 0),
                IVec2::new(-1, 0),
                IVec2::new(0, 1),
                IVec2::new(0, -1),
            ]
            .choose(&mut rng)
            .unwrap(),
            _ => continue,
        };
        if !map.is_road(tx + dir.x, ty + dir.y) {
            continue;
        }
        let Some(texture) = assets.traffic_cars.choose(&mut rng).cloned() else {
            break;
        };
        let world = GameMap::tile_to_world(IVec2::new(tx, ty));
        let angle = dir.as_vec2().to_angle() - std::f32::consts::FRAC_PI_2;
        commands.spawn((
            SpriteBundle {
                texture,
                transform: Transform::from_xyz(world.x, world.y, 6.0)
                    .with_rotation(Quat::from_rotation_z(angle)),
                sprite: Sprite {
                    custom_size: Some(Vec2::new(15.0, 21.0)),
                    ..default()
                },
                ..default()
            },
            TrafficCar {
                dir,
                speed: rng.gen_range(48.0..72.0),
                target_tile: IVec2::new(tx, ty) + dir,
                waiting: false,
                speech_cooldown: 0.0,
            },
        ));
        cars += 1;
    }

    // Traffic lights at every road intersection (two lamps per crossing).
    for &row in &ROAD_H_ROWS {
        for &col in &ROAD_V_COLS {
            let center = GameMap::tile_to_world(IVec2::new(col, row));
            for (offset, controls_horizontal) in
                [(Vec2::new(-22.0, 22.0), true), (Vec2::new(22.0, 22.0), false)]
            {
                commands.spawn((
                    SpriteBundle {
                        texture: assets.light_dot.clone(),
                        transform: Transform::from_xyz(
                            center.x + offset.x,
                            center.y + offset.y,
                            9.0,
                        ),
                        sprite: Sprite {
                            custom_size: Some(Vec2::new(9.0, 9.0)),
                            ..default()
                        },
                        ..default()
                    },
                    TrafficLight {
                        controls_horizontal,
                    },
                ));
            }
        }
    }

    let jannick_pos = GameMap::tile_to_world(map.jannick.interact_tile);
    commands.spawn((
        SpriteBundle {
            texture: assets.jannick.clone(),
            transform: Transform::from_xyz(jannick_pos.x, jannick_pos.y, 6.5),
            sprite: Sprite {
                custom_size: Some(Vec2::new(20.0, 26.0)),
                ..default()
            },
            ..default()
        },
        Jannick,
    ));

    commands.spawn((
        NodeBundle {
            style: Style {
                position_type: PositionType::Absolute,
                bottom: Val::Px(120.0),
                left: Val::Percent(50.0),
                margin: UiRect::left(Val::Px(-160.0)),
                width: Val::Px(320.0),
                padding: UiRect::all(Val::Px(10.0)),
                display: Display::None,
                justify_content: JustifyContent::Center,
                align_items: AlignItems::Center,
                ..default()
            },
            background_color: Color::srgba(0.05, 0.06, 0.1, 0.92).into(),
            border_color: Color::srgb(0.95, 0.85, 0.2).into(),
            ..default()
        },
        JannickHint,
    ))
    .with_children(|p| {
        p.spawn(TextBundle::from_section(
            "Jannick: [E] - Shop Kölner Eck",
            ts_body(&fonts, 20.0, Color::srgb(0.95, 0.85, 0.2)),
        ));
    });
}

fn despawn_npcs(
    mut commands: Commands,
    q: Query<
        Entity,
        Or<(
            With<Npc>,
            With<TrafficCar>,
            With<TrafficLight>,
            With<Jannick>,
            With<JannickHint>,
        )>,
    >,
) {
    for e in &q {
        commands.entity(e).despawn_recursive();
    }
}

fn cycle_traffic_lights(time: Res<Time>, mut lights: ResMut<TrafficLights>) {
    lights.timer -= time.delta_seconds();
    if lights.timer <= 0.0 {
        lights.horizontal_green = !lights.horizontal_green;
        lights.timer = LIGHT_GREEN_TIME;
    }
}

fn update_light_visuals(
    lights: Res<TrafficLights>,
    mut q: Query<(&TrafficLight, &mut Sprite)>,
) {
    for (light, mut sprite) in &mut q {
        let go = if light.controls_horizontal {
            lights.horizontal_green
        } else {
            !lights.horizontal_green
        };
        sprite.color = if go {
            Color::srgb(0.2, 0.95, 0.3)
        } else {
            Color::srgb(0.95, 0.2, 0.2)
        };
    }
}

/// Pick a pedestrian's next pavement tile. Prefers carrying straight on, never
/// immediately doubles back, and only steps onto a zebra crossing when the light
/// allows it — otherwise it waits at the kerb.
fn choose_ped_dir(
    map: &GameMap,
    cur: IVec2,
    prev_dir: IVec2,
    lights: &TrafficLights,
    rng: &mut ThreadRng,
) -> PedStep {
    let reverse = -prev_dir;
    let on_crosswalk = map.tile_at(cur.x, cur.y) == TileType::Crosswalk;
    let mut opts: Vec<IVec2> = Vec::new();
    let mut blocked_by_light = false;

    for &d in &[
        IVec2::new(1, 0),
        IVec2::new(-1, 0),
        IVec2::new(0, 1),
        IVec2::new(0, -1),
    ] {
        if d == reverse && prev_dir != IVec2::ZERO {
            continue;
        }
        let nt = cur + d;
        if !map.is_walkable(nt.x, nt.y) {
            continue;
        }
        // Stepping off the kerb onto a crossing obeys the pedestrian light.
        let entering_crossing = map.tile_at(nt.x, nt.y) == TileType::Crosswalk && !on_crosswalk;
        if entering_crossing && !light_go(lights, d) {
            blocked_by_light = true;
            continue;
        }
        if d == prev_dir {
            opts.push(d);
            opts.push(d);
        }
        opts.push(d);
    }

    if let Some(&d) = opts.choose(rng) {
        return PedStep::Go(d);
    }
    // Nothing legal ahead: wait for a red crossing, otherwise turn around.
    if blocked_by_light {
        return PedStep::Wait;
    }
    if map.is_walkable((cur + reverse).x, (cur + reverse).y) {
        return PedStep::Go(reverse);
    }
    PedStep::Wait
}

fn update_npcs(
    time: Res<Time>,
    map: Res<GameMap>,
    lights: Res<TrafficLights>,
    mut npcs: Query<(&mut Transform, &mut Npc, &mut Sprite)>,
) {
    let mut rng = thread_rng();
    let dt = time.delta_seconds().min(0.05);

    for (mut tr, mut npc, mut sprite) in &mut npcs {
        npc.state_timer -= dt;

        match npc.state {
            NpcState::Idle => {
                if npc.state_timer <= 0.0 {
                    let cur = GameMap::world_to_tile(tr.translation.truncate());
                    match choose_ped_dir(&map, cur, npc.dir, &lights, &mut rng) {
                        PedStep::Go(d) => {
                            npc.dir = d;
                            npc.target_tile = cur + d;
                            npc.state = NpcState::Walking;
                            npc.state_timer = rng.gen_range(3.0..7.0);
                        }
                        // Still red / blocked — check again shortly.
                        PedStep::Wait => npc.state_timer = rng.gen_range(0.3..0.8),
                    }
                }
            }
            NpcState::Walking => {
                let pos = tr.translation.truncate();
                let target_world = GameMap::tile_to_world(npc.target_tile);
                let to = target_world - pos;
                let dist = to.length();
                let move_amt = npc.speed * dt;

                if dist <= move_amt.max(0.5) {
                    // Arrived at the tile centre — choose the next step.
                    tr.translation.x = target_world.x;
                    tr.translation.y = target_world.y;
                    let cur = npc.target_tile;
                    if rng.gen_bool(0.12) {
                        npc.state = NpcState::Idle;
                        npc.state_timer = rng.gen_range(0.6..2.5);
                        continue;
                    }
                    match choose_ped_dir(&map, cur, npc.dir, &lights, &mut rng) {
                        PedStep::Go(d) => {
                            npc.dir = d;
                            npc.target_tile = cur + d;
                        }
                        PedStep::Wait => {
                            npc.state = NpcState::Idle;
                            npc.state_timer = rng.gen_range(0.3..0.8);
                        }
                    }
                } else {
                    let stepv = to / dist * move_amt;
                    tr.translation.x += stepv.x;
                    tr.translation.y += stepv.y;
                    if stepv.x < -0.1 {
                        sprite.flip_x = true;
                    } else if stepv.x > 0.1 {
                        sprite.flip_x = false;
                    }
                }
            }
        }
    }
}

fn update_traffic(
    time: Res<Time>,
    map: Res<GameMap>,
    lights: Res<TrafficLights>,
    mut cars: Query<(&mut Transform, &mut TrafficCar)>,
) {
    let mut rng = thread_rng();
    let dt = time.delta_seconds().min(0.05);

    for (mut tr, mut car) in &mut cars {
        let pos = tr.translation.truncate();
        let target_world = GameMap::tile_to_world(car.target_tile);
        let to = target_world - pos;
        let dist = to.length();
        let move_amt = car.speed * dt;

        if dist <= move_amt.max(1.0) {
            // Reached the tile centre — snap and decide the next move.
            tr.translation.x = target_world.x;
            tr.translation.y = target_world.y;
            let cur = car.target_tile;
            // While stopped at a light, hold the heading instead of re-rolling it.
            let next_dir = if car.waiting {
                car.dir
            } else {
                choose_traffic_dir(&map, cur, car.dir, &mut rng)
            };
            let next_tile = cur + next_dir;
            // Stop at the line before entering an intersection / zebra on red.
            let entering = !is_junction_zone(&map, cur) && is_junction_zone(&map, next_tile);
            if entering && !light_go(&lights, next_dir) {
                car.dir = next_dir;
                car.target_tile = cur;
                car.waiting = true;
            } else {
                car.dir = next_dir;
                car.target_tile = next_tile;
                car.waiting = false;
            }
        } else {
            let stepv = to / dist * move_amt;
            tr.translation.x += stepv.x;
            tr.translation.y += stepv.y;
        }

        let angle = car.dir.as_vec2().to_angle() - std::f32::consts::FRAC_PI_2;
        tr.rotation = Quat::from_rotation_z(angle);
    }
}

/// A tile that lies inside a junction (the crossing core or a zebra strip).
fn is_junction_zone(map: &GameMap, tile: IVec2) -> bool {
    matches!(
        map.tile_at(tile.x, tile.y),
        TileType::Road | TileType::Crosswalk
    )
}

/// Pick the next tile heading for a car. Cars keep going straight along a road
/// and only consider turning at an intersection (a `Road` tile). Straight is
/// favoured so traffic flows naturally; dead ends force a turn or U-turn.
fn choose_traffic_dir(map: &GameMap, cur: IVec2, dir: IVec2, rng: &mut ThreadRng) -> IVec2 {
    let straight = dir;
    let at_intersection = matches!(map.tile_at(cur.x, cur.y), TileType::Road);

    if !at_intersection && map.is_road(cur.x + straight.x, cur.y + straight.y) {
        return straight;
    }

    let left = IVec2::new(-dir.y, dir.x);
    let right = IVec2::new(dir.y, -dir.x);
    let mut opts: Vec<IVec2> = Vec::new();
    if map.is_road(cur.x + straight.x, cur.y + straight.y) {
        opts.push(straight);
        opts.push(straight); // bias toward continuing straight
    }
    if map.is_road(cur.x + left.x, cur.y + left.y) {
        opts.push(left);
    }
    if map.is_road(cur.x + right.x, cur.y + right.y) {
        opts.push(right);
    }
    if opts.is_empty() {
        let reverse = -dir;
        if map.is_road(cur.x + reverse.x, cur.y + reverse.y) {
            return reverse;
        }
        return dir;
    }
    *opts.choose(rng).unwrap()
}

const CRASH_SPEED: f32 = 85.0;
const CRASH_RADIUS_NPC: f32 = 16.0;
const CRASH_RADIUS_CAR: f32 = 22.0;

/// When the Sprinter rams a pedestrian or another car at speed, they bounce off
/// each other and the victim yells something rude in Bavarian. Pure feedback —
/// no time or coin penalty.
#[allow(clippy::type_complexity)]
fn player_crash_reactions(
    mut commands: Commands,
    time: Res<Time>,
    fonts: Res<UiFonts>,
    mut player_q: Query<(&Transform, &mut Player)>,
    mut npcs: Query<(Entity, &Transform, &mut Npc), (Without<Player>, Without<TrafficCar>)>,
    mut cars: Query<(Entity, &Transform, &mut TrafficCar), (Without<Player>, Without<Npc>)>,
    mut bump_ev: EventWriter<CollisionBumpEvent>,
) {
    let dt = time.delta_seconds();
    let Ok((ptr, mut player)) = player_q.get_single_mut() else {
        return;
    };
    let ppos = ptr.translation.truncate();
    let speed = player.velocity.length();
    let mut rng = thread_rng();
    let mut hit = false;

    for (e, tr, mut npc) in &mut npcs {
        if npc.speech_cooldown > 0.0 {
            npc.speech_cooldown -= dt;
        }
        let d = tr.translation.truncate().distance(ppos);
        if speed > CRASH_SPEED && d < CRASH_RADIUS_NPC && npc.speech_cooldown <= 0.0 {
            let line = ANGRY_LINES.choose(&mut rng).copied().unwrap_or("Hey!");
            spawn_speech_bubble(
                &mut commands,
                &fonts,
                Some(e),
                tr.translation.truncate(),
                line,
                Color::srgb(1.0, 0.55, 0.4),
                2.2,
            );
            npc.speech_cooldown = 3.0;
            // A fright: scuttle off somewhere new.
            npc.state = NpcState::Walking;
            npc.state_timer = 2.0;
            hit = true;
        }
    }

    for (e, tr, mut car) in &mut cars {
        if car.speech_cooldown > 0.0 {
            car.speech_cooldown -= dt;
        }
        let d = tr.translation.truncate().distance(ppos);
        if speed > CRASH_SPEED && d < CRASH_RADIUS_CAR && car.speech_cooldown <= 0.0 {
            let line = ANGRY_LINES.choose(&mut rng).copied().unwrap_or("Hey!");
            spawn_speech_bubble(
                &mut commands,
                &fonts,
                Some(e),
                tr.translation.truncate(),
                line,
                Color::srgb(1.0, 0.55, 0.4),
                2.2,
            );
            car.speech_cooldown = 3.0;
            hit = true;
        }
    }

    if hit {
        bump_ev.send(CollisionBumpEvent { pos: ppos });
        player.velocity *= -0.35;
    }
}

fn update_jannick_hint(
    map: Res<GameMap>,
    player_q: Query<&Transform, With<Player>>,
    mut hint_q: Query<&mut Style, With<JannickHint>>,
    mut visible: ResMut<JannickPromptVisible>,
) {
    let Ok(player_tr) = player_q.get_single() else {
        return;
    };
    let pos = player_tr.translation.truncate();
    let jpos = GameMap::tile_to_world(map.jannick.interact_tile);
    let is_close = pos.distance(jpos) < JANNICK_INTERACT_RADIUS;
    visible.0 = is_close;
    for mut style in &mut hint_q {
        style.display = if is_close {
            Display::Flex
        } else {
            Display::None
        };
    }
}

fn jannick_chatter(
    time: Res<Time>,
    map: Res<GameMap>,
    mut commands: Commands,
    fonts: Res<UiFonts>,
    player_q: Query<&Transform, With<Player>>,
    jannick_q: Query<Entity, With<Jannick>>,
    mut chatter: ResMut<JannickChatter>,
) {
    if chatter.cooldown > 0.0 {
        chatter.cooldown -= time.delta_seconds();
    }
    let Ok(player_tr) = player_q.get_single() else {
        return;
    };
    let jpos = GameMap::tile_to_world(map.jannick.interact_tile);
    let near = player_tr.translation.truncate().distance(jpos) < JANNICK_CHATTER_RADIUS;

    // Fire once when first entering range (with a cooldown so he isn't a parrot).
    if near && !chatter.was_near && chatter.cooldown <= 0.0 {
        let mut rng = thread_rng();
        let line = JANNICK_DIALOGS
            .choose(&mut rng)
            .copied()
            .unwrap_or("Ey, Minsch!");
        let target = jannick_q.get_single().ok();
        spawn_speech_bubble(
            &mut commands,
            &fonts,
            target,
            jpos,
            line,
            Color::srgb(0.7, 0.9, 1.0),
            3.0,
        );
        chatter.cooldown = 6.0;
    }
    chatter.was_near = near;
}

fn handle_jannick_interaction(
    keys: Res<ButtonInput<KeyCode>>,
    map: Res<GameMap>,
    player_q: Query<&Transform, With<Player>>,
    mut shop_request: EventWriter<ShopRequest>,
) {
    if !keys.just_pressed(KeyCode::KeyE) {
        return;
    }
    let Ok(player_tr) = player_q.get_single() else {
        return;
    };
    let pos = player_tr.translation.truncate();
    let jpos = GameMap::tile_to_world(map.jannick.interact_tile);
    if pos.distance(jpos) < JANNICK_INTERACT_RADIUS {
        let mut rng = thread_rng();
        let dialog = JANNICK_DIALOGS
            .choose(&mut rng)
            .copied()
            .unwrap_or("Wat willste?");
        shop_request.send(ShopRequest {
            dialog: dialog.to_string(),
        });
    }
}
