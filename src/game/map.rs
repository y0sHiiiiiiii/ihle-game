//! Germering-Tilemap: 128x128 Tiles, an der echten Stadt orientiert.

use bevy::prelude::*;
use bevy::render::render_asset::RenderAssetUsages;
use bevy::render::render_resource::{Extent3d, TextureDimension, TextureFormat};

use crate::game::assets::{fill_rect, set_px, Rgba};
use crate::game::GameState;

pub const TILE_SIZE: f32 = 16.0;
pub const MAP_WIDTH: i32 = 128;
pub const MAP_HEIGHT: i32 = 128;

pub const WORLD_HALF_W: f32 = MAP_WIDTH as f32 * TILE_SIZE * 0.5;
pub const WORLD_HALF_H: f32 = MAP_HEIGHT as f32 * TILE_SIZE * 0.5;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum TileType {
    Grass,
    Park,
    Sidewalk,
    Road,
    RoadH,
    RoadV,
    Crosswalk,
    Water,
    House,
    Roof,
    IhleStore,
    JannickStore,
    Rathaus,
    Bahnhof,
    Rails,
    Aldi,
    Rewe,
    Parking,
    Tree,
}

impl TileType {
    pub fn is_blocking(self) -> bool {
        matches!(
            self,
            TileType::House
                | TileType::Roof
                | TileType::IhleStore
                | TileType::JannickStore
                | TileType::Rathaus
                | TileType::Bahnhof
                | TileType::Rails
                | TileType::Aldi
                | TileType::Rewe
                | TileType::Tree
                | TileType::Water
        )
    }
}

#[derive(Clone, Debug)]
pub struct Landmark {
    pub name: String,
    #[allow(dead_code)]
    pub kind: LandmarkKind,
    pub tile: IVec2,
    pub interact_tile: IVec2,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum LandmarkKind {
    IhleStore,
    JannickStore,
    Rathaus,
    Bahnhof,
    Aldi,
    Rewe,
}

#[derive(Clone, Debug)]
pub struct AddressTarget {
    pub name: String,
    pub tile: IVec2,
}

#[derive(Resource)]
pub struct GameMap {
    pub tiles: Vec<TileType>,
    pub ihle_stores: Vec<Landmark>,
    pub jannick: Landmark,
    #[allow(dead_code)]
    pub landmarks: Vec<Landmark>,
    pub addresses: Vec<AddressTarget>,
    pub spawn_tile: IVec2,
}

impl GameMap {
    pub fn tile_at(&self, x: i32, y: i32) -> TileType {
        if x < 0 || y < 0 || x >= MAP_WIDTH || y >= MAP_HEIGHT {
            return TileType::Grass;
        }
        self.tiles[(y * MAP_WIDTH + x) as usize]
    }

    pub fn world_to_tile(world: Vec2) -> IVec2 {
        IVec2::new(
            ((world.x + WORLD_HALF_W) / TILE_SIZE).floor() as i32,
            ((world.y + WORLD_HALF_H) / TILE_SIZE).floor() as i32,
        )
    }

    pub fn tile_to_world(tile: IVec2) -> Vec2 {
        Vec2::new(
            tile.x as f32 * TILE_SIZE - WORLD_HALF_W + TILE_SIZE * 0.5,
            tile.y as f32 * TILE_SIZE - WORLD_HALF_H + TILE_SIZE * 0.5,
        )
    }
}

#[derive(Component)]
pub struct WorldTile;

pub struct MapPlugin;

impl Plugin for MapPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, generate_map)
            .add_systems(OnEnter(GameState::Playing), spawn_world);
    }
}

fn generate_map(mut commands: Commands) {
    let mut tiles = vec![TileType::Grass; (MAP_WIDTH * MAP_HEIGHT) as usize];

    let set_t = |tiles: &mut Vec<TileType>, x: i32, y: i32, t: TileType| {
        if x >= 0 && y >= 0 && x < MAP_WIDTH && y < MAP_HEIGHT {
            tiles[(y * MAP_WIDTH + x) as usize] = t;
        }
    };

    for y in 10..18 {
        for x in 8..30 {
            set_t(&mut tiles, x, y, TileType::Park);
            if (x + y * 3) % 17 == 0 {
                set_t(&mut tiles, x, y, TileType::Tree);
            }
        }
    }

    for y in 100..118 {
        for x in 100..125 {
            set_t(&mut tiles, x, y, TileType::Park);
            if (x * 5 + y) % 19 == 0 {
                set_t(&mut tiles, x, y, TileType::Tree);
            }
        }
    }

    for y in 60..70 {
        for x in 0..32 {
            set_t(&mut tiles, x, y, TileType::Water);
        }
    }
    for y in 62..68 {
        for x in 32..48 {
            set_t(&mut tiles, x, y, TileType::Water);
        }
    }

    let road_h_rows = [22i32, 42, 64, 86, 108];
    let road_v_cols = [20i32, 48, 76, 104];

    for &y in &road_h_rows {
        for x in 0..MAP_WIDTH {
            set_t(&mut tiles, x, y - 1, TileType::Sidewalk);
            set_t(&mut tiles, x, y, TileType::RoadH);
            set_t(&mut tiles, x, y + 1, TileType::RoadH);
            set_t(&mut tiles, x, y + 2, TileType::Sidewalk);
        }
    }
    for &x in &road_v_cols {
        for y in 0..MAP_HEIGHT {
            set_t(&mut tiles, x - 1, y, TileType::Sidewalk);
            set_t(&mut tiles, x, y, TileType::RoadV);
            set_t(&mut tiles, x + 1, y, TileType::RoadV);
            set_t(&mut tiles, x + 2, y, TileType::Sidewalk);
        }
    }

    for &y in &road_h_rows {
        for &x in &road_v_cols {
            for dx in -1..=2 {
                for dy in -1..=2 {
                    set_t(&mut tiles, x + dx, y + dy, TileType::Road);
                }
            }
            set_t(&mut tiles, x - 2, y, TileType::Crosswalk);
            set_t(&mut tiles, x - 2, y + 1, TileType::Crosswalk);
            set_t(&mut tiles, x + 3, y, TileType::Crosswalk);
            set_t(&mut tiles, x + 3, y + 1, TileType::Crosswalk);
            set_t(&mut tiles, x, y - 2, TileType::Crosswalk);
            set_t(&mut tiles, x + 1, y - 2, TileType::Crosswalk);
            set_t(&mut tiles, x, y + 3, TileType::Crosswalk);
            set_t(&mut tiles, x + 1, y + 3, TileType::Crosswalk);
        }
    }

    let house_blocks = [
        (3, 25, 16, 14),
        (25, 25, 22, 14),
        (51, 25, 22, 14),
        (79, 25, 22, 14),
        (107, 25, 18, 14),
        (3, 45, 16, 16),
        (51, 45, 22, 16),
        (79, 45, 22, 16),
        (107, 45, 18, 16),
        (3, 67, 16, 18),
        (25, 67, 22, 18),
        (51, 67, 22, 18),
        (79, 67, 22, 18),
        (107, 67, 18, 18),
        (3, 89, 16, 16),
        (51, 89, 22, 16),
        (79, 89, 22, 16),
        (107, 89, 18, 16),
        (25, 111, 22, 14),
        (51, 111, 22, 14),
        (79, 111, 22, 14),
        (107, 111, 18, 14),
    ];

    for &(bx, by, bw, bh) in &house_blocks {
        place_block(&mut tiles, bx, by, bw, bh);
    }

    let rathaus_rect = (54, 54, 8, 6);
    place_landmark(&mut tiles, rathaus_rect, TileType::Rathaus);

    let bahnhof_rect = (28, 89, 12, 6);
    place_landmark(&mut tiles, bahnhof_rect, TileType::Bahnhof);
    for x in 0..MAP_WIDTH {
        if !matches!(
            tiles[(95 * MAP_WIDTH + x) as usize],
            TileType::RoadV | TileType::Road | TileType::Sidewalk
        ) {
            set_t(&mut tiles, x, 95, TileType::Rails);
        }
        if !matches!(
            tiles[(96 * MAP_WIDTH + x) as usize],
            TileType::RoadV | TileType::Road | TileType::Sidewalk
        ) {
            set_t(&mut tiles, x, 96, TileType::Rails);
        }
    }

    let aldi_rect = (94, 16, 9, 5);
    place_landmark(&mut tiles, aldi_rect, TileType::Aldi);

    let rewe_rect = (60, 16, 9, 5);
    place_landmark(&mut tiles, rewe_rect, TileType::Rewe);

    let ihle_bahnhof = (40, 91, 5, 3);
    place_landmark(&mut tiles, ihle_bahnhof, TileType::IhleStore);

    let ihle_zentrum = (66, 56, 5, 3);
    place_landmark(&mut tiles, ihle_zentrum, TileType::IhleStore);

    let ihle_nord = (50, 16, 5, 3);
    place_landmark(&mut tiles, ihle_nord, TileType::IhleStore);

    let ihle_sued = (50, 112, 5, 3);
    place_landmark(&mut tiles, ihle_sued, TileType::IhleStore);

    let jannick_rect = (84, 60, 5, 3);
    place_landmark(&mut tiles, jannick_rect, TileType::JannickStore);

    let parking_rects = [(94, 22, 10, 2), (60, 22, 10, 2), (30, 95, 12, 2)];
    for &(x, y, w, h) in &parking_rects {
        for dy in 0..h {
            for dx in 0..w {
                set_t(&mut tiles, x + dx, y + dy, TileType::Parking);
            }
        }
    }

    let parking_landmark = (110, 64, 6, 4);
    for dy in 0..parking_landmark.3 {
        for dx in 0..parking_landmark.2 {
            set_t(
                &mut tiles,
                parking_landmark.0 + dx,
                parking_landmark.1 + dy,
                TileType::Parking,
            );
        }
    }

    let ihle_stores = vec![
        Landmark {
            name: "Ihle Bahnhof".to_string(),
            kind: LandmarkKind::IhleStore,
            tile: IVec2::new(ihle_bahnhof.0 + 2, ihle_bahnhof.1 + 1),
            interact_tile: IVec2::new(ihle_bahnhof.0 + 2, ihle_bahnhof.1 + 3),
        },
        Landmark {
            name: "Ihle Zentrum".to_string(),
            kind: LandmarkKind::IhleStore,
            tile: IVec2::new(ihle_zentrum.0 + 2, ihle_zentrum.1 + 1),
            interact_tile: IVec2::new(ihle_zentrum.0 + 2, ihle_zentrum.1 + 3),
        },
        Landmark {
            name: "Ihle Nord".to_string(),
            kind: LandmarkKind::IhleStore,
            tile: IVec2::new(ihle_nord.0 + 2, ihle_nord.1 + 1),
            interact_tile: IVec2::new(ihle_nord.0 + 2, ihle_nord.1 + 3),
        },
        Landmark {
            name: "Ihle Sued".to_string(),
            kind: LandmarkKind::IhleStore,
            tile: IVec2::new(ihle_sued.0 + 2, ihle_sued.1 + 1),
            interact_tile: IVec2::new(ihle_sued.0 + 2, ihle_sued.1 + 3),
        },
    ];

    let jannick = Landmark {
        name: "Jannicks Koelner Eck".to_string(),
        kind: LandmarkKind::JannickStore,
        tile: IVec2::new(jannick_rect.0 + 2, jannick_rect.1 + 1),
        interact_tile: IVec2::new(jannick_rect.0 + 2, jannick_rect.1 + 3),
    };

    let landmarks = vec![
        Landmark {
            name: "Rathaus".to_string(),
            kind: LandmarkKind::Rathaus,
            tile: IVec2::new(rathaus_rect.0 + 4, rathaus_rect.1 + 3),
            interact_tile: IVec2::new(rathaus_rect.0 + 4, rathaus_rect.1 + 6),
        },
        Landmark {
            name: "Bahnhof Germering".to_string(),
            kind: LandmarkKind::Bahnhof,
            tile: IVec2::new(bahnhof_rect.0 + 6, bahnhof_rect.1 + 3),
            interact_tile: IVec2::new(bahnhof_rect.0 + 6, bahnhof_rect.1 + 6),
        },
        Landmark {
            name: "Aldi Sued".to_string(),
            kind: LandmarkKind::Aldi,
            tile: IVec2::new(aldi_rect.0 + 4, aldi_rect.1 + 2),
            interact_tile: IVec2::new(aldi_rect.0 + 4, aldi_rect.1 + 5),
        },
        Landmark {
            name: "Rewe".to_string(),
            kind: LandmarkKind::Rewe,
            tile: IVec2::new(rewe_rect.0 + 4, rewe_rect.1 + 2),
            interact_tile: IVec2::new(rewe_rect.0 + 4, rewe_rect.1 + 5),
        },
    ];

    let addresses = vec![
        AddressTarget {
            name: "Hauptstr. 14".to_string(),
            tile: IVec2::new(56, 28),
        },
        AddressTarget {
            name: "Untere Bahnhofstr. 7".to_string(),
            tile: IVec2::new(78, 36),
        },
        AddressTarget {
            name: "Eichenauer Str. 33".to_string(),
            tile: IVec2::new(110, 38),
        },
        AddressTarget {
            name: "Wittelsbacher Str. 12".to_string(),
            tile: IVec2::new(30, 48),
        },
        AddressTarget {
            name: "Lerchenweg 5".to_string(),
            tile: IVec2::new(106, 56),
        },
        AddressTarget {
            name: "Goethestr. 21".to_string(),
            tile: IVec2::new(80, 78),
        },
        AddressTarget {
            name: "Schillerplatz 3".to_string(),
            tile: IVec2::new(108, 78),
        },
        AddressTarget {
            name: "Lindenallee 18".to_string(),
            tile: IVec2::new(58, 98),
        },
        AddressTarget {
            name: "Kirchenplatz 2".to_string(),
            tile: IVec2::new(62, 58),
        },
        AddressTarget {
            name: "Eschenrieder Str. 9".to_string(),
            tile: IVec2::new(106, 100),
        },
        AddressTarget {
            name: "Tulpenweg 4".to_string(),
            tile: IVec2::new(35, 78),
        },
        AddressTarget {
            name: "Roggensteiner Str. 8".to_string(),
            tile: IVec2::new(34, 38),
        },
        AddressTarget {
            name: "Augsburger Str. 25".to_string(),
            tile: IVec2::new(82, 120),
        },
        AddressTarget {
            name: "Muenchner Str. 16".to_string(),
            tile: IVec2::new(58, 120),
        },
        AddressTarget {
            name: "Karl-Sommer-Str. 6".to_string(),
            tile: IVec2::new(110, 120),
        },
        AddressTarget {
            name: "Stadtpark Nord".to_string(),
            tile: IVec2::new(18, 18),
        },
    ];

    let spawn_tile = IVec2::new(48, 64);

    commands.insert_resource(GameMap {
        tiles,
        ihle_stores,
        jannick,
        landmarks,
        addresses,
        spawn_tile,
    });
}

fn place_block(tiles: &mut [TileType], bx: i32, by: i32, bw: i32, bh: i32) {
    let inner_x = bx + 1;
    let inner_y = by + 1;
    let inner_w = bw - 2;
    let inner_h = bh - 2;

    for dy in 0..bh {
        for dx in 0..bw {
            let x = bx + dx;
            let y = by + dy;
            if x < 0 || y < 0 || x >= MAP_WIDTH || y >= MAP_HEIGHT {
                continue;
            }
            tiles[(y * MAP_WIDTH + x) as usize] = TileType::Grass;
        }
    }

    let strip = 3;
    if inner_h > strip * 2 {
        let house_y0 = inner_y;
        let house_y1 = inner_y + inner_h - strip;
        for dx in 0..inner_w {
            let x = inner_x + dx;
            if dx % 6 < 4 {
                for dy in 0..strip {
                    if !(0..MAP_WIDTH).contains(&x) {
                        continue;
                    }
                    tiles[((house_y0 + dy) * MAP_WIDTH + x) as usize] =
                        if dy == 0 { TileType::Roof } else { TileType::House };
                    tiles[((house_y1 + dy) * MAP_WIDTH + x) as usize] =
                        if dy == 0 { TileType::Roof } else { TileType::House };
                }
            } else {
                for dy in 0..strip {
                    if !(0..MAP_WIDTH).contains(&x) {
                        continue;
                    }
                    tiles[((house_y0 + dy) * MAP_WIDTH + x) as usize] = TileType::Grass;
                    tiles[((house_y1 + dy) * MAP_WIDTH + x) as usize] = TileType::Grass;
                }
            }
        }
    }
    if inner_w > strip * 2 {
        let house_x0 = inner_x;
        let house_x1 = inner_x + inner_w - strip;
        for dy in 0..inner_h {
            let y = inner_y + dy;
            if dy % 6 < 4 && y > inner_y + 3 && y < inner_y + inner_h - 3 {
                for dx in 0..strip {
                    if !(0..MAP_HEIGHT).contains(&y) {
                        continue;
                    }
                    tiles[(y * MAP_WIDTH + house_x0 + dx) as usize] =
                        if dx == 0 { TileType::Roof } else { TileType::House };
                    tiles[(y * MAP_WIDTH + house_x1 + dx) as usize] =
                        if dx == 0 { TileType::Roof } else { TileType::House };
                }
            }
        }
    }

    for dy in 0..inner_h {
        for dx in 0..inner_w {
            let x = inner_x + dx;
            let y = inner_y + dy;
            if x < 0 || y < 0 || x >= MAP_WIDTH || y >= MAP_HEIGHT {
                continue;
            }
            let idx = (y * MAP_WIDTH + x) as usize;
            if tiles[idx] == TileType::Grass && ((dx * 7 + dy * 11) % 31 == 0) {
                tiles[idx] = TileType::Tree;
            }
        }
    }
}

fn place_landmark(tiles: &mut [TileType], rect: (i32, i32, i32, i32), kind: TileType) {
    let (x, y, w, h) = rect;
    for dy in 0..h {
        for dx in 0..w {
            let tx = x + dx;
            let ty = y + dy;
            if tx < 0 || ty < 0 || tx >= MAP_WIDTH || ty >= MAP_HEIGHT {
                continue;
            }
            tiles[(ty * MAP_WIDTH + tx) as usize] = kind;
        }
    }
}

fn spawn_world(
    mut commands: Commands,
    map: Res<GameMap>,
    existing: Query<Entity, With<WorldTile>>,
    mut images: ResMut<Assets<Image>>,
) {
    for e in &existing {
        commands.entity(e).despawn();
    }

    let pixel_w = MAP_WIDTH as u32 * TILE_SIZE as u32;
    let pixel_h = MAP_HEIGHT as u32 * TILE_SIZE as u32;
    let mut buf = vec![0u8; (pixel_w * pixel_h * 4) as usize];

    for ty in 0..MAP_HEIGHT {
        for tx in 0..MAP_WIDTH {
            let kind = map.tile_at(tx, ty);
            let px0 = (tx as u32) * TILE_SIZE as u32;
            let py0 = (ty as u32) * TILE_SIZE as u32;
            draw_tile(&mut buf, pixel_w, px0 as i32, py0 as i32, tx, ty, kind);
        }
    }

    flip_y(&mut buf, pixel_w, pixel_h);

    let map_image = Image::new(
        Extent3d {
            width: pixel_w,
            height: pixel_h,
            depth_or_array_layers: 1,
        },
        TextureDimension::D2,
        buf,
        TextureFormat::Rgba8UnormSrgb,
        RenderAssetUsages::default(),
    );
    let handle = images.add(map_image);

    commands.spawn((
        SpriteBundle {
            texture: handle,
            transform: Transform::from_xyz(0.0, 0.0, 0.0),
            sprite: Sprite {
                custom_size: Some(Vec2::new(pixel_w as f32, pixel_h as f32)),
                ..default()
            },
            ..default()
        },
        WorldTile,
    ));
}

fn flip_y(buf: &mut [u8], w: u32, h: u32) {
    let stride = (w * 4) as usize;
    for y in 0..(h as usize / 2) {
        let top = y * stride;
        let bot = (h as usize - 1 - y) * stride;
        for x in 0..stride {
            buf.swap(top + x, bot + x);
        }
    }
}

fn draw_tile(
    buf: &mut [u8],
    w: u32,
    px: i32,
    py: i32,
    tx: i32,
    ty: i32,
    kind: TileType,
) {
    let size = TILE_SIZE as i32;
    match kind {
        TileType::Grass => {
            let g: Rgba = (90, 145, 70, 255);
            let g2: Rgba = (75, 130, 60, 255);
            fill_rect(buf, w, px, py, size, size, g);
            for dy in 0..size {
                for dx in 0..size {
                    if ((tx * 17 + ty * 23 + dx * 5 + dy * 7) & 31) == 0 {
                        set_px(buf, w, px + dx, py + dy, g2);
                    }
                }
            }
        }
        TileType::Park => {
            let g: Rgba = (60, 130, 55, 255);
            fill_rect(buf, w, px, py, size, size, g);
            for dy in 0..size {
                for dx in 0..size {
                    if ((tx * 19 + ty * 29 + dx * 11 + dy * 13) & 15) == 0 {
                        set_px(buf, w, px + dx, py + dy, (40, 100, 35, 255));
                    }
                }
            }
        }
        TileType::Sidewalk => {
            let c: Rgba = (175, 175, 180, 255);
            let dark: Rgba = (140, 140, 150, 255);
            fill_rect(buf, w, px, py, size, size, c);
            fill_rect(buf, w, px, py, size, 1, dark);
            fill_rect(buf, w, px, py + size / 2, size, 1, dark);
            fill_rect(buf, w, px, py, 1, size, dark);
            fill_rect(buf, w, px + size / 2, py, 1, size, dark);
        }
        TileType::Road => {
            let c: Rgba = (55, 55, 58, 255);
            fill_rect(buf, w, px, py, size, size, c);
        }
        TileType::RoadH => {
            let c: Rgba = (55, 55, 58, 255);
            let stripe: Rgba = (235, 200, 60, 255);
            fill_rect(buf, w, px, py, size, size, c);
            let mid = py + size / 2;
            if (tx & 1) == 0 {
                fill_rect(buf, w, px + 2, mid, size - 4, 1, stripe);
            }
        }
        TileType::RoadV => {
            let c: Rgba = (55, 55, 58, 255);
            let stripe: Rgba = (235, 200, 60, 255);
            fill_rect(buf, w, px, py, size, size, c);
            let mid = px + size / 2;
            if (ty & 1) == 0 {
                fill_rect(buf, w, mid, py + 2, 1, size - 4, stripe);
            }
        }
        TileType::Crosswalk => {
            let c: Rgba = (55, 55, 58, 255);
            let white: Rgba = (235, 235, 235, 255);
            fill_rect(buf, w, px, py, size, size, c);
            for i in 0..4 {
                fill_rect(buf, w, px + i * 4, py + 2, 2, size - 4, white);
            }
        }
        TileType::Water => {
            let c: Rgba = (60, 110, 200, 255);
            let highlight: Rgba = (110, 160, 230, 255);
            fill_rect(buf, w, px, py, size, size, c);
            for dy in (0..size).step_by(4) {
                let yo = (tx + dy) & 3;
                fill_rect(buf, w, px + 1 + yo, py + dy, 4, 1, highlight);
            }
        }
        TileType::House => {
            let body: Rgba = (200, 180, 150, 255);
            let outline: Rgba = (90, 60, 40, 255);
            fill_rect(buf, w, px, py, size, size, body);
            fill_rect(buf, w, px, py, size, 1, outline);
            fill_rect(buf, w, px, py + size - 1, size, 1, outline);
            fill_rect(buf, w, px, py, 1, size, outline);
            fill_rect(buf, w, px + size - 1, py, 1, size, outline);
            let win: Rgba = (130, 200, 235, 255);
            fill_rect(buf, w, px + 3, py + 4, 3, 3, win);
            fill_rect(buf, w, px + 10, py + 4, 3, 3, win);
            fill_rect(buf, w, px + 3, py + 10, 3, 3, win);
            fill_rect(buf, w, px + 10, py + 10, 3, 3, win);
        }
        TileType::Roof => {
            let body: Rgba = (160, 70, 50, 255);
            let outline: Rgba = (80, 30, 20, 255);
            fill_rect(buf, w, px, py, size, size, body);
            for dy in 0..size {
                for dx in 0..size {
                    if (dx + dy) % 3 == 0 {
                        set_px(buf, w, px + dx, py + dy, outline);
                    }
                }
            }
        }
        TileType::IhleStore => {
            let blue: Rgba = (30, 80, 180, 255);
            let white: Rgba = (240, 240, 245, 255);
            let outline: Rgba = (15, 30, 90, 255);
            fill_rect(buf, w, px, py, size, size, blue);
            fill_rect(buf, w, px, py, size, 1, outline);
            fill_rect(buf, w, px, py + size - 1, size, 1, outline);
            fill_rect(buf, w, px, py, 1, size, outline);
            fill_rect(buf, w, px + size - 1, py, 1, size, outline);
            fill_rect(buf, w, px + 2, py + 5, size - 4, 6, white);
            fill_rect(buf, w, px + 3, py + 6, 1, 4, blue);
            fill_rect(buf, w, px + 5, py + 6, 1, 4, blue);
            fill_rect(buf, w, px + 5, py + 6, 2, 1, blue);
            fill_rect(buf, w, px + 8, py + 6, 1, 4, blue);
            fill_rect(buf, w, px + 8, py + 6, 3, 1, blue);
            fill_rect(buf, w, px + 8, py + 9, 3, 1, blue);
            fill_rect(buf, w, px + 12, py + 6, 1, 4, blue);
            fill_rect(buf, w, px + 12, py + 6, 2, 1, blue);
            fill_rect(buf, w, px + 6, py + 2, 4, 2, white);
        }
        TileType::JannickStore => {
            let red: Rgba = (210, 35, 35, 255);
            let white: Rgba = (240, 240, 240, 255);
            let outline: Rgba = (90, 15, 15, 255);
            let yellow: Rgba = (250, 220, 70, 255);
            fill_rect(buf, w, px, py, size, size, (180, 140, 100, 255));
            fill_rect(buf, w, px, py, size, 1, outline);
            fill_rect(buf, w, px, py + size - 1, size, 1, outline);
            fill_rect(buf, w, px, py, 1, size, outline);
            fill_rect(buf, w, px + size - 1, py, 1, size, outline);
            for i in 0..8 {
                let col = if i % 2 == 0 { red } else { white };
                fill_rect(buf, w, px + i * 2, py + 2, 2, 4, col);
            }
            fill_rect(buf, w, px + 6, py + 8, 4, 6, (140, 90, 60, 255));
            fill_rect(buf, w, px + 6, py + 8, 4, 1, outline);
            fill_rect(buf, w, px + 6, py + 13, 4, 1, outline);
            fill_rect(buf, w, px + 2, py + 10, 3, 3, yellow);
            fill_rect(buf, w, px + 11, py + 10, 3, 3, yellow);
            fill_rect(buf, w, px + 4, py + 6, 1, 2, white);
            fill_rect(buf, w, px + 4, py + 5, 3, 1, white);
            fill_rect(buf, w, px + 5, py + 4, 1, 1, white);
        }
        TileType::Rathaus => {
            let body: Rgba = (220, 200, 180, 255);
            let roof: Rgba = (110, 50, 40, 255);
            let outline: Rgba = (60, 40, 30, 255);
            let win: Rgba = (130, 200, 235, 255);
            fill_rect(buf, w, px, py, size, size, body);
            fill_rect(buf, w, px, py, size, 4, roof);
            fill_rect(buf, w, px + 6, py + 1, 4, 6, roof);
            set_px(buf, w, px + 7, py - 1, (250, 220, 60, 255));
            set_px(buf, w, px + 8, py - 1, (250, 220, 60, 255));
            fill_rect(buf, w, px, py, size, 1, outline);
            fill_rect(buf, w, px, py + size - 1, size, 1, outline);
            fill_rect(buf, w, px, py, 1, size, outline);
            fill_rect(buf, w, px + size - 1, py, 1, size, outline);
            fill_rect(buf, w, px + 3, py + 7, 3, 4, win);
            fill_rect(buf, w, px + 10, py + 7, 3, 4, win);
        }
        TileType::Bahnhof => {
            let body: Rgba = (200, 170, 120, 255);
            let roof: Rgba = (100, 50, 35, 255);
            let outline: Rgba = (50, 30, 20, 255);
            let win: Rgba = (130, 200, 235, 255);
            let blue: Rgba = (30, 70, 160, 255);
            fill_rect(buf, w, px, py, size, size, body);
            fill_rect(buf, w, px, py, size, 3, roof);
            fill_rect(buf, w, px, py, size, 1, outline);
            fill_rect(buf, w, px, py + size - 1, size, 1, outline);
            fill_rect(buf, w, px, py, 1, size, outline);
            fill_rect(buf, w, px + size - 1, py, 1, size, outline);
            fill_rect(buf, w, px + 2, py + 5, 3, 3, win);
            fill_rect(buf, w, px + 11, py + 5, 3, 3, win);
            fill_rect(buf, w, px + 6, py + 5, 4, 5, blue);
            fill_rect(buf, w, px + 7, py + 6, 2, 3, (240, 240, 240, 255));
        }
        TileType::Rails => {
            let ground: Rgba = (90, 80, 65, 255);
            let rail: Rgba = (180, 180, 190, 255);
            let tie: Rgba = (60, 40, 30, 255);
            fill_rect(buf, w, px, py, size, size, ground);
            for i in (0..size).step_by(4) {
                fill_rect(buf, w, px + i, py + 2, 2, size - 4, tie);
            }
            fill_rect(buf, w, px, py + 4, size, 1, rail);
            fill_rect(buf, w, px, py + size - 5, size, 1, rail);
        }
        TileType::Aldi => {
            let body: Rgba = (240, 240, 240, 255);
            let outline: Rgba = (40, 60, 130, 255);
            let blue: Rgba = (30, 70, 170, 255);
            let red: Rgba = (220, 35, 35, 255);
            let orange: Rgba = (255, 160, 30, 255);
            fill_rect(buf, w, px, py, size, size, body);
            fill_rect(buf, w, px, py, size, 1, outline);
            fill_rect(buf, w, px, py + size - 1, size, 1, outline);
            fill_rect(buf, w, px, py, 1, size, outline);
            fill_rect(buf, w, px + size - 1, py, 1, size, outline);
            fill_rect(buf, w, px + 2, py + 3, size - 4, 4, blue);
            fill_rect(buf, w, px + 3, py + 4, 2, 2, orange);
            fill_rect(buf, w, px + 6, py + 4, 1, 2, red);
            fill_rect(buf, w, px + 8, py + 4, 1, 2, red);
            fill_rect(buf, w, px + 10, py + 4, 1, 2, red);
            fill_rect(buf, w, px + 2, py + 9, size - 4, 5, (200, 200, 215, 255));
            fill_rect(buf, w, px + 6, py + 11, 4, 3, (90, 60, 40, 255));
        }
        TileType::Rewe => {
            let body: Rgba = (240, 240, 240, 255);
            let outline: Rgba = (90, 15, 15, 255);
            let red: Rgba = (220, 35, 35, 255);
            let white: Rgba = (245, 245, 245, 255);
            fill_rect(buf, w, px, py, size, size, body);
            fill_rect(buf, w, px, py, size, 1, outline);
            fill_rect(buf, w, px, py + size - 1, size, 1, outline);
            fill_rect(buf, w, px, py, 1, size, outline);
            fill_rect(buf, w, px + size - 1, py, 1, size, outline);
            fill_rect(buf, w, px + 2, py + 3, size - 4, 4, red);
            fill_rect(buf, w, px + 4, py + 4, 1, 2, white);
            fill_rect(buf, w, px + 4, py + 4, 2, 1, white);
            fill_rect(buf, w, px + 7, py + 4, 1, 2, white);
            fill_rect(buf, w, px + 7, py + 4, 2, 1, white);
            fill_rect(buf, w, px + 10, py + 4, 2, 1, white);
            fill_rect(buf, w, px + 10, py + 4, 1, 2, white);
            fill_rect(buf, w, px + 10, py + 5, 2, 1, white);
            fill_rect(buf, w, px + 2, py + 9, size - 4, 5, (200, 200, 215, 255));
            fill_rect(buf, w, px + 6, py + 11, 4, 3, (90, 60, 40, 255));
        }
        TileType::Parking => {
            let c: Rgba = (75, 75, 80, 255);
            let stripe: Rgba = (240, 240, 240, 255);
            fill_rect(buf, w, px, py, size, size, c);
            for i in (0..size).step_by(4) {
                fill_rect(buf, w, px + i, py + 2, 1, size - 4, stripe);
            }
        }
        TileType::Tree => {
            fill_rect(buf, w, px, py, size, size, (90, 145, 70, 255));
            fill_rect(buf, w, px + 4, py + 4, 8, 8, (30, 90, 30, 255));
            fill_rect(buf, w, px + 3, py + 5, 10, 6, (40, 110, 35, 255));
            fill_rect(buf, w, px + 4, py + 4, 8, 1, (20, 70, 20, 255));
            fill_rect(buf, w, px + 7, py + 6, 2, 4, (90, 50, 25, 255));
        }
    }
}
