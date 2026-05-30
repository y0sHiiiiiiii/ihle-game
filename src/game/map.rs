//! Germering-Tilemap: 128x128 Tiles, an der echten Stadt orientiert.
//!
//! Aufbau: ein Haupt-Arterien-Raster (mit Ampeln, siehe `npc.rs`) plus
//! engmaschige Nebenstraßen, dazwischen dicht bebaute Wohnblöcke mit Vorgärten
//! und Hecken. Eingestreut: vier Ihle-Bäckereien, Kirchplatz mit St. Martin,
//! Rathaus, Bahnhof mit Bahnsteig & S8-Gleisen, Aldi/Rewe mit Parkplatz,
//! Jannicks Kölner Eck, Stadtpark und Germeringer See.

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

/// Centre rows/cols of the *arterial* road grid — also the intersection
/// coordinates where traffic lights sit (consumed by `npc.rs`).
pub const ROAD_H_ROWS: [i32; 5] = [22, 42, 64, 86, 108];
pub const ROAD_V_COLS: [i32; 4] = [20, 48, 76, 104];

/// Narrower residential side streets woven between the arterials. They carry
/// cars too (they are normal road tiles) but have no traffic lights.
const SIDE_H_ROWS: [i32; 4] = [32, 53, 75, 97];
const SIDE_V_COLS: [i32; 3] = [34, 62, 90];

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum TileType {
    Grass,
    Park,
    Garden,
    Sidewalk,
    Cobble,
    Road,
    RoadH,
    RoadV,
    Crosswalk,
    Water,
    House,
    Roof,
    Hedge,
    IhleStore,
    JannickStore,
    Rathaus,
    Church,
    Bahnhof,
    Platform,
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
                | TileType::Hedge
                | TileType::IhleStore
                | TileType::JannickStore
                | TileType::Rathaus
                | TileType::Church
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

    /// True if a tile can be driven on by traffic (carriageway, not parking bay).
    pub fn is_road(&self, x: i32, y: i32) -> bool {
        matches!(
            self.tile_at(x, y),
            TileType::Road | TileType::RoadH | TileType::RoadV | TileType::Crosswalk
        )
    }

    /// True if a pedestrian may stand here — pavements, squares, platforms and
    /// zebra crossings.
    pub fn is_walkable(&self, x: i32, y: i32) -> bool {
        matches!(
            self.tile_at(x, y),
            TileType::Sidewalk | TileType::Crosswalk | TileType::Cobble | TileType::Platform
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

#[inline]
fn set_t(tiles: &mut [TileType], x: i32, y: i32, t: TileType) {
    if x >= 0 && y >= 0 && x < MAP_WIDTH && y < MAP_HEIGHT {
        tiles[(y * MAP_WIDTH + x) as usize] = t;
    }
}

#[inline]
fn get_t(tiles: &[TileType], x: i32, y: i32) -> TileType {
    if x < 0 || y < 0 || x >= MAP_WIDTH || y >= MAP_HEIGHT {
        return TileType::Grass;
    }
    tiles[(y * MAP_WIDTH + x) as usize]
}

fn fill_t(tiles: &mut [TileType], x0: i32, y0: i32, x1: i32, y1: i32, t: TileType) {
    for y in y0..=y1 {
        for x in x0..=x1 {
            set_t(tiles, x, y, t);
        }
    }
}

fn generate_map(mut commands: Commands) {
    let mut tiles = vec![TileType::Grass; (MAP_WIDTH * MAP_HEIGHT) as usize];

    // --- Road network (arterials + side streets) ------------------------------
    let all_h: Vec<i32> = ROAD_H_ROWS.iter().chain(SIDE_H_ROWS.iter()).copied().collect();
    let all_v: Vec<i32> = ROAD_V_COLS.iter().chain(SIDE_V_COLS.iter()).copied().collect();

    for &y in &all_h {
        for x in 0..MAP_WIDTH {
            set_t(&mut tiles, x, y - 1, TileType::Sidewalk);
            set_t(&mut tiles, x, y, TileType::RoadH);
            set_t(&mut tiles, x, y + 1, TileType::RoadH);
            set_t(&mut tiles, x, y + 2, TileType::Sidewalk);
        }
    }
    for &x in &all_v {
        for y in 0..MAP_HEIGHT {
            set_t(&mut tiles, x - 1, y, TileType::Sidewalk);
            set_t(&mut tiles, x, y, TileType::RoadV);
            set_t(&mut tiles, x + 1, y, TileType::RoadV);
            set_t(&mut tiles, x + 2, y, TileType::Sidewalk);
        }
    }

    // Every road crossing gets a small asphalt core so cars can turn cleanly.
    for &y in &all_h {
        for &x in &all_v {
            fill_t(&mut tiles, x, y, x + 1, y + 1, TileType::Road);
        }
    }

    // Zebra crossings only on the big arterial intersections (matches lights).
    for &y in &ROAD_H_ROWS {
        for &x in &ROAD_V_COLS {
            fill_t(&mut tiles, x - 1, y, x + 2, y + 1, TileType::Road);
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

    // --- Residential blocks in the gaps between the roads ----------------------
    // Free spans (inclusive) computed from the road bands above.
    let row_spans: [(i32, i32); 10] = [
        (0, 20),
        (25, 30),
        (35, 40),
        (45, 51),
        (56, 62),
        (67, 73),
        (78, 84),
        (89, 95),
        (100, 106),
        (111, 127),
    ];
    let col_spans: [(i32, i32); 8] = [
        (0, 18),
        (23, 32),
        (37, 46),
        (51, 60),
        (65, 74),
        (79, 88),
        (93, 102),
        (107, 127),
    ];

    for &(ry0, ry1) in &row_spans {
        for &(rx0, rx1) in &col_spans {
            place_residential(&mut tiles, rx0, ry0, rx1, ry1);
        }
    }

    // --- Green spaces ----------------------------------------------------------
    // Stadtpark Nord (NW corner) with a pond and tree clusters.
    place_park(&mut tiles, 1, 1, 18, 20);
    fill_t(&mut tiles, 5, 4, 14, 10, TileType::Water);
    fill_t(&mut tiles, 8, 13, 11, 13, TileType::Cobble); // a path stub
    // Germeringer See (small lake on the west flank).
    place_park(&mut tiles, 1, 56, 17, 62);
    fill_t(&mut tiles, 2, 57, 15, 61, TileType::Water);
    // Wittelsbacherpark (SE corner).
    place_park(&mut tiles, 108, 111, 126, 126);

    // --- Kirchplatz: cobbled square with St. Martin + fountain -----------------
    fill_t(&mut tiles, 51, 56, 60, 62, TileType::Cobble);
    place_landmark(&mut tiles, (53, 56, 5, 5), TileType::Church);
    fill_t(&mut tiles, 58, 60, 59, 61, TileType::Water); // little fountain
    set_t(&mut tiles, 51, 62, TileType::Tree);
    set_t(&mut tiles, 60, 56, TileType::Tree);

    // Rathaus on its own apron just west of the square.
    fill_t(&mut tiles, 37, 56, 46, 62, TileType::Cobble);
    let rathaus_rect = (38, 57, 8, 4);
    place_landmark(&mut tiles, rathaus_rect, TileType::Rathaus);

    // --- Bahnhof Germering + Bahnsteig + S8-Gleise -----------------------------
    let bahnhof_rect = (24, 89, 9, 3);
    fill_t(&mut tiles, 23, 89, 46, 93, TileType::Cobble); // forecourt + platform base
    place_landmark(&mut tiles, bahnhof_rect, TileType::Bahnhof);
    fill_t(&mut tiles, 24, 92, 46, 93, TileType::Platform);
    for x in 0..MAP_WIDTH {
        if !matches!(
            get_t(&tiles, x, 94),
            TileType::RoadV | TileType::Road | TileType::Sidewalk
        ) {
            set_t(&mut tiles, x, 94, TileType::Rails);
        }
        if !matches!(
            get_t(&tiles, x, 95),
            TileType::RoadV | TileType::Road | TileType::Sidewalk
        ) {
            set_t(&mut tiles, x, 95, TileType::Rails);
        }
    }

    // --- Supermärkte mit Parkplatz (Nord-Achse) --------------------------------
    place_store_with_lot(&mut tiles, 94, 25, 9, 4, TileType::Aldi);
    place_store_with_lot(&mut tiles, 66, 25, 9, 4, TileType::Rewe);

    // --- Jannicks Kölner Eck an der Hauptstraße --------------------------------
    fill_t(&mut tiles, 83, 56, 88, 62, TileType::Cobble);
    place_landmark(&mut tiles, (84, 56, 5, 3), TileType::JannickStore);
    fill_t(&mut tiles, 84, 59, 88, 60, TileType::Parking);

    // --- Vier Ihle-Bäckereien (mit Parkbucht zur Straße hin) -------------------
    let ihle_nord = place_ihle(&mut tiles, 38, 16);
    let ihle_zentrum = place_ihle(&mut tiles, 52, 36);
    let ihle_bahnhof = place_ihle(&mut tiles, 66, 80);
    let ihle_sued = place_ihle(&mut tiles, 52, 102);

    let ihle_stores = vec![
        ihle_landmark("Ihle Bahnhof", ihle_bahnhof),
        ihle_landmark("Ihle Zentrum", ihle_zentrum),
        ihle_landmark("Ihle Nord", ihle_nord),
        ihle_landmark("Ihle Sued", ihle_sued),
    ];

    let jannick = Landmark {
        name: "Jannicks Koelner Eck".to_string(),
        kind: LandmarkKind::JannickStore,
        tile: IVec2::new(86, 57),
        interact_tile: IVec2::new(86, 59),
    };

    let landmarks = vec![
        Landmark {
            name: "Rathaus".to_string(),
            kind: LandmarkKind::Rathaus,
            tile: IVec2::new(42, 58),
            interact_tile: IVec2::new(42, 62),
        },
        Landmark {
            name: "Bahnhof Germering".to_string(),
            kind: LandmarkKind::Bahnhof,
            tile: IVec2::new(28, 90),
            interact_tile: IVec2::new(28, 88),
        },
        Landmark {
            name: "Aldi Sued".to_string(),
            kind: LandmarkKind::Aldi,
            tile: IVec2::new(98, 26),
            interact_tile: IVec2::new(98, 31),
        },
        Landmark {
            name: "Rewe".to_string(),
            kind: LandmarkKind::Rewe,
            tile: IVec2::new(70, 26),
            interact_tile: IVec2::new(70, 31),
        },
    ];

    // Kundenadressen — alle auf Bürgersteig-Tiles, also zu Fuß erreichbar.
    let addresses = vec![
        addr("Augsburger Str. 12", 26, 24),
        addr("Kirchenstr. 5", 58, 55),
        addr("Untere Bahnhofstr. 7", 44, 88),
        addr("Hauptstr. 14", 80, 66),
        addr("Eichenauer Str. 33", 110, 41),
        addr("Wittelsbacher Str. 12", 30, 41),
        addr("Lerchenweg 5", 98, 55),
        addr("Goethestr. 21", 82, 77),
        addr("Schillerplatz 3", 110, 74),
        addr("Lindenallee 18", 58, 96),
        addr("Eschenrieder Str. 9", 108, 99),
        addr("Tulpenweg 4", 40, 77),
        addr("Roggensteiner Str. 8", 28, 44),
        addr("Augsburger Str. 25", 108, 24),
        addr("Muenchner Str. 16", 58, 110),
        addr("Karl-Sommer-Str. 6", 110, 110),
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

fn addr(name: &str, x: i32, y: i32) -> AddressTarget {
    AddressTarget {
        name: name.to_string(),
        tile: IVec2::new(x, y),
    }
}

/// Build the `Landmark` for an Ihle store from its top-left store corner.
fn ihle_landmark(name: &str, sx_sy: (i32, i32)) -> Landmark {
    let (sx, sy) = sx_sy;
    Landmark {
        name: name.to_string(),
        kind: LandmarkKind::IhleStore,
        tile: IVec2::new(sx + 3, sy + 1),
        interact_tile: IVec2::new(sx + 3, sy + 3),
    }
}

/// Place a 6×3 Ihle bakery at `(sx, sy)` with a 6×2 parking bay directly below
/// (at the interact tile). The surrounding lot is cleared so the van always has
/// a clean approach from the street below. Returns the store's top-left corner.
fn place_ihle(tiles: &mut [TileType], sx: i32, sy: i32) -> (i32, i32) {
    fill_t(tiles, sx - 1, sy - 1, sx + 6, sy + 4, TileType::Garden);
    fill_t(tiles, sx, sy, sx + 5, sy + 2, TileType::IhleStore);
    fill_t(tiles, sx, sy + 3, sx + 5, sy + 4, TileType::Parking);
    (sx, sy)
}

/// A supermarket building with a striped parking lot in front (below it).
fn place_store_with_lot(tiles: &mut [TileType], bx: i32, by: i32, bw: i32, bh: i32, kind: TileType) {
    fill_t(tiles, bx - 1, by - 1, bx + bw, by + bh + 2, TileType::Grass);
    place_landmark(tiles, (bx, by, bw, bh), kind);
    fill_t(tiles, bx, by + bh, bx + bw - 1, by + bh + 1, TileType::Parking);
}

/// Fill a block with a tidy German residential subdivision: garden ground, rows
/// of small houses separated by garden lanes, a few hedges and trees.
fn place_residential(tiles: &mut [TileType], bx0: i32, by0: i32, bx1: i32, by1: i32) {
    let bw = bx1 - bx0 + 1;
    let bh = by1 - by0 + 1;
    if bw < 2 || bh < 2 {
        return;
    }
    fill_t(tiles, bx0, by0, bx1, by1, TileType::Garden);
    if bw < 4 || bh < 3 {
        return; // narrow strips stay as gardens
    }

    let lot_w = 4;
    let lot_h = 3;
    let mut ly = by0 + 1;
    while ly < by1 - 1 {
        let hy1 = (ly + lot_h - 1).min(by1 - 1);
        if hy1 - ly + 1 >= 2 {
            let mut lx = bx0 + 1;
            while lx + 2 < bx1 {
                let hx1 = (lx + lot_w - 1).min(bx1 - 1);
                if hx1 - lx + 1 >= 3 {
                    for x in lx..=hx1 {
                        set_t(tiles, x, ly, TileType::Roof);
                    }
                    for y in (ly + 1)..=hy1 {
                        for x in lx..=hx1 {
                            set_t(tiles, x, y, TileType::House);
                        }
                    }
                }
                lx += lot_w + 1;
            }
        }
        ly += lot_h + 1;
    }

    // Scatter trees and the odd hedge into the leftover garden tiles.
    for y in by0..=by1 {
        for x in bx0..=bx1 {
            if get_t(tiles, x, y) == TileType::Garden {
                if (x * 7 + y * 13).rem_euclid(23) == 0 {
                    set_t(tiles, x, y, TileType::Tree);
                } else if (x * 5 + y * 11).rem_euclid(31) == 0 {
                    set_t(tiles, x, y, TileType::Hedge);
                }
            }
        }
    }
}

/// A park area: green ground with dense tree clusters.
fn place_park(tiles: &mut [TileType], x0: i32, y0: i32, x1: i32, y1: i32) {
    for y in y0..=y1 {
        for x in x0..=x1 {
            set_t(tiles, x, y, TileType::Park);
            if (x * 5 + y * 3).rem_euclid(11) == 0 {
                set_t(tiles, x, y, TileType::Tree);
            }
        }
    }
}

fn place_landmark(tiles: &mut [TileType], rect: (i32, i32, i32, i32), kind: TileType) {
    let (x, y, w, h) = rect;
    for dy in 0..h {
        for dx in 0..w {
            set_t(tiles, x + dx, y + dy, kind);
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

/// Pick a deterministic palette entry for a tile so neighbouring buildings vary.
fn variant<const N: usize>(tx: i32, ty: i32, salt: i32, palette: [Rgba; N]) -> Rgba {
    let i = ((tx / 4) * 13 + (ty / 3) * 7 + salt).rem_euclid(N as i32) as usize;
    palette[i]
}

fn draw_tile(buf: &mut [u8], w: u32, px: i32, py: i32, tx: i32, ty: i32, kind: TileType) {
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
        TileType::Garden => {
            // Slightly lush front-yard green, dotted with little flowers.
            let g: Rgba = (95, 160, 80, 255);
            let g2: Rgba = (80, 145, 68, 255);
            fill_rect(buf, w, px, py, size, size, g);
            for dy in 0..size {
                for dx in 0..size {
                    let h = tx * 13 + ty * 7 + dx * 3 + dy * 5;
                    if h & 7 == 0 {
                        set_px(buf, w, px + dx, py + dy, g2);
                    } else if h.rem_euclid(37) == 0 {
                        let flower: Rgba = match h.rem_euclid(3) {
                            0 => (235, 90, 90, 255),
                            1 => (245, 220, 80, 255),
                            _ => (240, 240, 245, 255),
                        };
                        set_px(buf, w, px + dx, py + dy, flower);
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
        TileType::Cobble => {
            // Warm grey pavement of small set stones (Kirchplatz / forecourts).
            let base: Rgba = (158, 150, 140, 255);
            let joint: Rgba = (120, 112, 104, 255);
            let hi: Rgba = (178, 170, 160, 255);
            fill_rect(buf, w, px, py, size, size, base);
            for dy in 0..size {
                for dx in 0..size {
                    if dx % 4 == 0 || dy % 4 == 0 {
                        set_px(buf, w, px + dx, py + dy, joint);
                    } else if (dx + dy) % 4 == 1 {
                        set_px(buf, w, px + dx, py + dy, hi);
                    }
                }
            }
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
            let body = variant(
                tx,
                ty,
                0,
                [
                    (200, 180, 150, 255),
                    (212, 196, 166, 255),
                    (193, 170, 142, 255),
                    (206, 188, 158, 255),
                    (186, 200, 206, 255),
                ],
            );
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
            let body = variant(
                tx,
                ty,
                4,
                [
                    (160, 70, 50, 255),
                    (122, 62, 56, 255),
                    (150, 92, 56, 255),
                    (96, 84, 96, 255),
                    (172, 112, 62, 255),
                ],
            );
            let outline: Rgba = (
                (body.0 as f32 * 0.55) as u8,
                (body.1 as f32 * 0.55) as u8,
                (body.2 as f32 * 0.55) as u8,
                255,
            );
            fill_rect(buf, w, px, py, size, size, body);
            for dy in 0..size {
                for dx in 0..size {
                    if (dx + dy) % 3 == 0 {
                        set_px(buf, w, px + dx, py + dy, outline);
                    }
                }
            }
        }
        TileType::Hedge => {
            let g: Rgba = (52, 110, 48, 255);
            let dark: Rgba = (36, 84, 34, 255);
            let hi: Rgba = (74, 132, 64, 255);
            fill_rect(buf, w, px, py, size, size, g);
            for dy in 0..size {
                for dx in 0..size {
                    match (dx * 3 + dy * 5) % 5 {
                        0 => set_px(buf, w, px + dx, py + dy, dark),
                        2 => set_px(buf, w, px + dx, py + dy, hi),
                        _ => {}
                    }
                }
            }
            fill_rect(buf, w, px, py + size - 1, size, 1, (28, 64, 26, 255));
        }
        TileType::IhleStore => draw_ihle_bakery(buf, w, px, py, size),
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
        TileType::Church => {
            // Pale stone wall with a tall arched window; the cobbled square and
            // the building mass read as the parish church St. Martin.
            let stone: Rgba = (196, 190, 176, 255);
            let shade: Rgba = (168, 160, 146, 255);
            let outline: Rgba = (96, 88, 74, 255);
            let roof: Rgba = (84, 64, 96, 255);
            let win: Rgba = (90, 140, 190, 255);
            fill_rect(buf, w, px, py, size, size, stone);
            fill_rect(buf, w, px, py, size, 3, roof);
            fill_rect(buf, w, px, py, size, 1, outline);
            fill_rect(buf, w, px, py + size - 1, size, 1, outline);
            fill_rect(buf, w, px, py, 1, size, outline);
            fill_rect(buf, w, px + size - 1, py, 1, size, outline);
            fill_rect(buf, w, px + 2, py + size / 2, size - 4, 1, shade);
            fill_rect(buf, w, px + 6, py + 6, 4, 8, win);
            fill_rect(buf, w, px + 7, py + 5, 2, 1, win);
            fill_rect(buf, w, px + 7, py + 7, 2, 1, (230, 240, 250, 255));
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
        TileType::Platform => {
            // Light concrete platform with a yellow safety line at the rail edge.
            let slab: Rgba = (198, 198, 202, 255);
            let joint: Rgba = (150, 150, 156, 255);
            let warn: Rgba = (240, 200, 50, 255);
            fill_rect(buf, w, px, py, size, size, slab);
            for dx in (0..size).step_by(4) {
                fill_rect(buf, w, px + dx, py, 1, size, joint);
            }
            fill_rect(buf, w, px, py + size - 2, size, 2, warn);
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

/// A warm, recognisable Ihle bakery storefront in one 16×16 tile: red/white
/// awning, cream "IHLE" sign, a golden Brezn and a shop window with bread.
fn draw_ihle_bakery(buf: &mut [u8], w: u32, px: i32, py: i32, size: i32) {
    let facade: Rgba = (212, 180, 138, 255);
    let outline: Rgba = (96, 62, 36, 255);
    let cream: Rgba = (240, 230, 205, 255);
    let letter: Rgba = (84, 52, 30, 255);
    let red: Rgba = (205, 45, 45, 255);
    let white: Rgba = (245, 245, 245, 255);
    let brez: Rgba = (180, 116, 48, 255);
    let brez_hi: Rgba = (214, 156, 86, 255);
    let glass: Rgba = (150, 200, 222, 255);
    let bread: Rgba = (196, 142, 70, 255);
    let door: Rgba = (120, 80, 48, 255);

    fill_rect(buf, w, px, py, size, size, facade);
    fill_rect(buf, w, px, py, size, 1, outline);
    fill_rect(buf, w, px, py + size - 1, size, 1, outline);
    fill_rect(buf, w, px, py, 1, size, outline);
    fill_rect(buf, w, px + size - 1, py, 1, size, outline);

    // Striped awning across the top.
    for i in 0..8 {
        let col = if i % 2 == 0 { red } else { white };
        fill_rect(buf, w, px + i * 2, py + 1, 2, 3, col);
    }

    // Cream sign band with a tiny "IHLE".
    fill_rect(buf, w, px + 1, py + 4, size - 2, 4, cream);
    // I
    fill_rect(buf, w, px + 2, py + 5, 1, 3, letter);
    // H
    fill_rect(buf, w, px + 4, py + 5, 1, 3, letter);
    fill_rect(buf, w, px + 6, py + 5, 1, 3, letter);
    set_px(buf, w, px + 5, py + 6, letter);
    // L
    fill_rect(buf, w, px + 8, py + 5, 1, 3, letter);
    set_px(buf, w, px + 9, py + 7, letter);
    // E
    fill_rect(buf, w, px + 11, py + 5, 1, 3, letter);
    fill_rect(buf, w, px + 11, py + 5, 2, 1, letter);
    fill_rect(buf, w, px + 11, py + 6, 2, 1, letter);
    fill_rect(buf, w, px + 11, py + 7, 2, 1, letter);

    // Golden Brezn emblem.
    set_px(buf, w, px + 5, py + 9, brez);
    set_px(buf, w, px + 6, py + 9, brez);
    set_px(buf, w, px + 9, py + 9, brez);
    set_px(buf, w, px + 10, py + 9, brez);
    set_px(buf, w, px + 4, py + 10, brez);
    set_px(buf, w, px + 7, py + 10, brez_hi);
    set_px(buf, w, px + 8, py + 10, brez_hi);
    set_px(buf, w, px + 11, py + 10, brez);
    for dx in 5..=10 {
        set_px(buf, w, px + dx, py + 11, brez);
    }
    set_px(buf, w, px + 6, py + 12, brez);
    set_px(buf, w, px + 9, py + 12, brez);

    // Shop window with loaves + a door on the right.
    fill_rect(buf, w, px + 1, py + 13, 9, 2, glass);
    fill_rect(buf, w, px + 2, py + 13, 2, 1, bread);
    fill_rect(buf, w, px + 5, py + 13, 2, 1, bread);
    fill_rect(buf, w, px + 8, py + 13, 1, 1, bread);
    fill_rect(buf, w, px + 11, py + 11, 3, 4, door);
    set_px(buf, w, px + 12, py + 13, (230, 220, 180, 255));
}
