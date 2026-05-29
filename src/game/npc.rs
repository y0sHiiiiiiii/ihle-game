//! NPC-System: 4 Typen Passanten + Jannick (Pizzeria) mit Koelsch-Dialog.

use bevy::prelude::*;
use rand::prelude::*;

use crate::game::assets::{ts_body, GameAssets, UiFonts};
use crate::game::gamestate::GameState;
use crate::game::map::{GameMap, TileType, MAP_HEIGHT, MAP_WIDTH};
use crate::game::player::Player;
use crate::game::shop::ShopRequest;

pub const BASE_NPC_COUNT: usize = 12;
pub const EXTRA_NPC_COUNT: usize = 18;
pub const JANNICK_INTERACT_RADIUS: f32 = 40.0;

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
    pub target_tile: IVec2,
    pub speed: f32,
    pub home_tile: IVec2,
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

pub struct NpcPlugin;

impl Plugin for NpcPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<JannickPromptVisible>()
            .add_systems(OnEnter(GameState::Playing), spawn_npcs)
            .add_systems(OnExit(GameState::GameOver), despawn_npcs)
            .add_systems(
                Update,
                (
                    update_npcs.run_if(in_state(GameState::Playing)),
                    handle_jannick_interaction.run_if(in_state(GameState::Playing)),
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
    existing: Query<Entity, Or<(With<Npc>, With<Jannick>, With<JannickHint>)>>,
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
        let home_tile = IVec2::new(tx, ty);
        let target = find_random_sidewalk_target(&map, home_tile, &mut rng);
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
                state_timer: rng.gen_range(0.5..2.0),
                state: NpcState::Walking,
                target_tile: target,
                speed,
                home_tile,
            },
        ));
        spawned += 1;
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
    q: Query<Entity, Or<(With<Npc>, With<Jannick>, With<JannickHint>)>>,
) {
    for e in &q {
        commands.entity(e).despawn_recursive();
    }
}

fn find_random_sidewalk_target(
    map: &GameMap,
    near: IVec2,
    rng: &mut ThreadRng,
) -> IVec2 {
    for _ in 0..40 {
        let dx = rng.gen_range(-12..=12);
        let dy = rng.gen_range(-12..=12);
        let t = near + IVec2::new(dx, dy);
        if t.x < 2 || t.y < 2 || t.x >= MAP_WIDTH - 2 || t.y >= MAP_HEIGHT - 2 {
            continue;
        }
        if map.tile_at(t.x, t.y) == TileType::Sidewalk {
            return t;
        }
    }
    near
}

fn update_npcs(
    time: Res<Time>,
    map: Res<GameMap>,
    player_q: Query<&Transform, (With<Player>, Without<Npc>)>,
    mut npcs: Query<(&mut Transform, &mut Npc, &mut Sprite)>,
) {
    let mut rng = thread_rng();
    let player_pos = player_q
        .get_single()
        .map(|t| t.translation.truncate())
        .unwrap_or(Vec2::ZERO);
    let dt = time.delta_seconds().min(0.05);

    for (mut tr, mut npc, mut sprite) in &mut npcs {
        npc.state_timer -= dt;

        match npc.state {
            NpcState::Idle => {
                if npc.state_timer <= 0.0 {
                    npc.state = NpcState::Walking;
                    npc.state_timer = rng.gen_range(2.0..6.0);
                    npc.target_tile = find_random_sidewalk_target(&map, npc.home_tile, &mut rng);
                }
            }
            NpcState::Walking => {
                let pos = tr.translation.truncate();
                let target_world = GameMap::tile_to_world(npc.target_tile);
                let to_target = target_world - pos;
                let dist = to_target.length();
                if dist < 4.0 || npc.state_timer <= 0.0 {
                    npc.state = NpcState::Idle;
                    npc.state_timer = rng.gen_range(1.0..3.0);
                    continue;
                }
                let mut dir = to_target / dist;
                let to_player = player_pos - pos;
                let player_dist = to_player.length();
                if player_dist < 45.0 && player_dist > 1.0 {
                    let avoid = -to_player / player_dist;
                    dir = (dir + avoid * 1.8).normalize_or_zero();
                }

                let mut step = dir * npc.speed * dt;
                let new_pos = pos + step;
                let new_tile = GameMap::world_to_tile(new_pos);
                let kind = map.tile_at(new_tile.x, new_tile.y);
                if kind.is_blocking() {
                    npc.target_tile =
                        find_random_sidewalk_target(&map, npc.home_tile, &mut rng);
                    step = Vec2::ZERO;
                }
                tr.translation.x += step.x;
                tr.translation.y += step.y;

                let face_left = step.x < -0.1;
                sprite.flip_x = face_left;
            }
        }
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
