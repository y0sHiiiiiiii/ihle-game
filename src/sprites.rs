//! Alle Pixel-Art-Sprites als statische Char-Grids.
//!
//! Jeder Charakter entspricht einer Palette-Farbe.
//! Bei Spielstart werden die Grids in `Texture2D` umgewandelt.
//!
//! Palette siehe `palette()` unten.

use macroquad::prelude::*;

// ------------------------------------------------------------------------
//  Palette
// ------------------------------------------------------------------------

pub fn palette(c: char) -> Color {
    match c {
        '.' => Color::new(0.0, 0.0, 0.0, 0.0),
        '0' => Color::new(0.05, 0.04, 0.07, 1.0),
        '1' => Color::new(1.00, 0.83, 0.65, 1.0),
        '2' => Color::new(0.40, 0.24, 0.10, 1.0),
        '3' => Color::new(0.85, 0.18, 0.18, 1.0),
        '4' => Color::new(0.98, 0.97, 0.94, 1.0),
        '5' => Color::new(0.20, 0.45, 0.85, 1.0),
        '6' => Color::new(0.13, 0.20, 0.45, 1.0),
        '7' => Color::new(1.00, 0.85, 0.15, 1.0),
        '8' => Color::new(0.30, 0.70, 0.30, 1.0),
        '9' => Color::new(0.90, 0.55, 0.20, 1.0),
        'a' => Color::new(0.16, 0.40, 0.18, 1.0),
        'b' => Color::new(0.70, 0.72, 0.74, 1.0),
        'c' => Color::new(0.35, 0.38, 0.42, 1.0),
        'd' => Color::new(0.30, 0.65, 0.90, 1.0),
        'e' => Color::new(0.75, 0.92, 0.98, 1.0),
        'f' => Color::new(0.65, 0.30, 0.85, 1.0),
        'g' => Color::new(0.55, 0.35, 0.20, 1.0),
        'h' => Color::new(0.92, 0.85, 0.55, 1.0),
        'i' => Color::new(0.50, 0.08, 0.08, 1.0),
        'j' => Color::new(0.85, 0.65, 0.20, 1.0),
        'k' => Color::new(0.78, 0.80, 0.82, 1.0),
        'l' => Color::new(1.00, 1.00, 1.00, 1.0),
        'm' => Color::new(0.95, 0.55, 0.65, 1.0),
        'n' => Color::new(0.45, 0.80, 0.40, 1.0),
        'o' => Color::new(0.65, 0.45, 0.25, 1.0),
        'p' => Color::new(0.75, 0.25, 0.20, 1.0),
        'q' => Color::new(0.30, 0.18, 0.08, 1.0),
        'r' => Color::new(1.00, 0.95, 0.55, 1.0),
        's' => Color::new(0.55, 0.40, 0.18, 1.0),
        _ => Color::new(1.0, 0.0, 1.0, 1.0),
    }
}

// ------------------------------------------------------------------------
//  Sprite-Builder
// ------------------------------------------------------------------------

pub fn make_texture(grid: &[&str]) -> Texture2D {
    let h = grid.len() as u16;
    let w = grid[0].chars().count() as u16;
    let mut img = Image::gen_image_color(w, h, Color::new(0.0, 0.0, 0.0, 0.0));
    for (y, row) in grid.iter().enumerate() {
        for (x, c) in row.chars().enumerate() {
            if (x as u16) < w && (y as u16) < h {
                img.set_pixel(x as u32, y as u32, palette(c));
            }
        }
    }
    let t = Texture2D::from_image(&img);
    t.set_filter(FilterMode::Nearest);
    t
}

/// Einfaches 1×1-Pixel-Texture in beliebiger Farbe — nützlich für gefärbte Rechtecke.
pub fn make_pixel(color: Color) -> Texture2D {
    let mut img = Image::gen_image_color(1, 1, color);
    let _ = &mut img;
    let mut img2 = Image::gen_image_color(1, 1, color);
    img2.set_pixel(0, 0, color);
    let t = Texture2D::from_image(&img2);
    t.set_filter(FilterMode::Nearest);
    t
}

// ------------------------------------------------------------------------
//  PLAYER — Max Huber, Bäckerlehrling (16x16, alle Zeilen exakt 16 chars)
// ------------------------------------------------------------------------

pub const PLAYER_DOWN_A: [&str; 16] = [
    "................",
    ".....022220.....",
    "....02222220....",
    "....02333320....",
    "....01111110....",
    "....01010110....",
    "....01111110....",
    "....01122110....",
    ".....044440.....",
    "....04444440....",
    "....04434440....",
    "....04443440....",
    "....04444440....",
    "....06606600....",
    "....06606600....",
    "....02..02......",
];

pub const PLAYER_DOWN_B: [&str; 16] = [
    "................",
    ".....022220.....",
    "....02222220....",
    "....02333320....",
    "....01111110....",
    "....01010110....",
    "....01111110....",
    "....01122110....",
    ".....044440.....",
    "....04444440....",
    "....04434440....",
    "....04443440....",
    "....04444440....",
    "....06606600....",
    "...066..06600...",
    "..02......02....",
];

pub const PLAYER_UP_A: [&str; 16] = [
    "................",
    ".....022220.....",
    "....02222220....",
    "....02222220....",
    "....02222220....",
    "....02222220....",
    "....02222220....",
    "....02222220....",
    ".....044440.....",
    "....04444440....",
    "....04444440....",
    "....04444440....",
    "....04444440....",
    "....06606600....",
    "....06606600....",
    "....02..02......",
];

pub const PLAYER_UP_B: [&str; 16] = [
    "................",
    ".....022220.....",
    "....02222220....",
    "....02222220....",
    "....02222220....",
    "....02222220....",
    "....02222220....",
    "....02222220....",
    ".....044440.....",
    "....04444440....",
    "....04444440....",
    "....04444440....",
    "....04444440....",
    "....06606600....",
    "...066..06600...",
    "..02......02....",
];

pub const PLAYER_LEFT_A: [&str; 16] = [
    "................",
    ".....022220.....",
    "....02222220....",
    "....02233220....",
    "....01101110....",
    "....01001110....",
    "....01111110....",
    "....01122110....",
    ".....044440.....",
    "....04444440....",
    "....04444440....",
    "....04443340....",
    "....04444440....",
    "....06606600....",
    "....06606600....",
    "....02..02......",
];

/// Schritt-Frame nach links: vorderer Fuß weit vorne (linke Bildseite).
pub const PLAYER_LEFT_B: [&str; 16] = [
    "................",
    ".....022220.....",
    "....02222220....",
    "....02233220....",
    "....01101110....",
    "....01001110....",
    "....01111110....",
    "....01122110....",
    ".....044440.....",
    "....04444440....",
    "....04444440....",
    "....04443340....",
    "....04444440....",
    "....06606600....",
    "..066....66000..",
    ".02........02...",
];

pub const PLAYER_RIGHT_A: [&str; 16] = [
    "................",
    ".....022220.....",
    "....02222220....",
    "....02233220....",
    "....01110110....",
    "....01110010....",
    "....01111110....",
    "....01122110....",
    ".....044440.....",
    "....04444440....",
    "....04444440....",
    "....04334440....",
    "....04444440....",
    "....06606600....",
    "....06606600....",
    "....02..02......",
];

/// Schritt-Frame nach rechts: vorderer Fuß weit vorne (rechte Bildseite).
pub const PLAYER_RIGHT_B: [&str; 16] = [
    "................",
    ".....022220.....",
    "....02222220....",
    "....02233220....",
    "....01110110....",
    "....01110010....",
    "....01111110....",
    "....01122110....",
    ".....044440.....",
    "....04444440....",
    "....04444440....",
    "....04334440....",
    "....04444440....",
    "....06606600....",
    "..00066....660..",
    "...02........02.",
];

pub const PLAYER_SWIM: [&str; 16] = [
    "................",
    "................",
    "................",
    "................",
    "................",
    ".....022220.....",
    "....02222220....",
    "....02333320....",
    "....01111110....",
    "....01010110....",
    "....01111110....",
    ".dddd022220dddd.",
    "ddddddddddddddd.",
    "ddddddddddddddd.",
    ".ddd.dddd.ddddd.",
    "................",
];

// ------------------------------------------------------------------------
//  GEGNER (5 Typen) — 16x16
// ------------------------------------------------------------------------

pub const ENEMY_MOLD: [&str; 16] = [
    "................",
    "................",
    "....088888880...",
    "...08aaaaaa80...",
    "..08aanaanaa80..",
    "..08aaaaaaaa80..",
    "..08aanaanaa80..",
    "...08aaaaaa80...",
    "....088888880...",
    "....0bbbbbb0....",
    "....0b4004b0....",
    "....0b0000b0....",
    "....0bbbbbb0....",
    "....0b0..0b0....",
    "....02....02....",
    "................",
];

pub const ENEMY_BLOB: [&str; 16] = [
    "................",
    "................",
    "................",
    ".....00000......",
    "....0ddddd0.....",
    "...0deeeeed0....",
    "..0deeeeeeed0...",
    "..0deeleeleed0..",
    "..0deeeeeeeed0..",
    "..0deelaaleed0..",
    "..0dee0aa0eed0..",
    "...0deeeeeed0...",
    "....0ddddddd0...",
    "....0d0dd0d0....",
    ".....000000.....",
    "................",
];

pub const ENEMY_RAT: [&str; 16] = [
    "................",
    "................",
    "................",
    "................",
    "...0........0...",
    "..0g0......0g0..",
    "..0gg00000gg0...",
    ".0ggggggggggg0..",
    ".0g4g0gggg0g4g0.",
    ".0gggggggggggg0.",
    ".0gg0gggggg0gg0.",
    "..0ggggggggg0...",
    "..0g0gg0gg0gg0..",
    "..0..0...0...0..",
    "................",
    "................",
];

pub const ENEMY_BEAT: [&str; 16] = [
    "................",
    "................",
    "...0ffffff0.....",
    "..0ffllfflff0...",
    ".0fflllllllff0..",
    ".0flllllllllf0..",
    ".0ffllfflllff0..",
    "..0ffffffffff0..",
    "...0ffffff0.....",
    "....0bbbb0......",
    "....0b40b0......",
    "....0b04b0......",
    "....0bbbb0......",
    "....0b..b0......",
    "....02..02......",
    "................",
];

pub const ENEMY_ICE: [&str; 16] = [
    "................",
    "................",
    "....0eeeeee0....",
    "...0eeleelee0...",
    "..0eeleeeelee0..",
    "..0eeeeeeeeee0..",
    "..0eel00ll00le0.",
    "..0eel0ldd0lee0.",
    "...0eeeeeeee0...",
    "....0bbbbbb0....",
    "....0b4ee4b0....",
    "....0beeeebe....",
    "....0bbbbbb0....",
    "....0b0..0b0....",
    "....02....02....",
    "................",
];

// ------------------------------------------------------------------------
//  BOSS — Schimmelmeister Modrý (48×48, alle Zeilen exakt 48 chars)
// ------------------------------------------------------------------------

pub const BOSS_MODRY: [&str; 48] = [
    "................................................",
    "................................................",
    "................................................",
    "..............000888888888000...................",
    ".............08888aaaaaa88880...................",
    "............08aaaaiiiiiiaaaa80..................",
    "...........08aaaiii3333iiiaaa80.................",
    "..........08aaiii333nn333iiiaa80................",
    ".........08aii333nnllllnn333iia80...............",
    "........08aii33nnlll44llnn33iiaa0...............",
    ".......08aii3nnll4400044llnn3iia0...............",
    ".......08aii3nll4400ll004ll4ln3i0...............",
    ".......08aii3nll40llll04ll4lln3i0...............",
    ".......08aii3nllll4ll4llll4lln30...............",
    ".......08aii3nllllll4llllllln3i0...............",
    "........08aii3nll4lllllll4lln3i0...............",
    "........08aii33nll44004ll4n33ii0...............",
    "........08aaii33nnnnnnnn33iia80.................",
    "........08aaaiii33333333iiia80..................",
    "........08aaaaiiiiiiiiiiia80....................",
    ".........08aaaaaaaaaaaaaa80.....................",
    ".........08888888888888880......................",
    "........0bbbbb444444444bbbbb0...................",
    ".......0b44443333444433444440b..................",
    "......0b4443333344333344334440b.................",
    "......0b4433333344334433334430b.................",
    ".......0b4433333344443333440b...................",
    "........0bbb44333344433333bbb0..................",
    ".........0bbb443333344bbb0......................",
    "..........0bbb44444bbb0.........................",
    "...........0bbbbbbbb0...........................",
    "...........0b000000b0...........................",
    "...........0b0....0b0...........................",
    "...........0b0....0b0...........................",
    "..........088......880..........................",
    "..........088......880..........................",
    "................................................",
    "................................................",
    "................................................",
    "................................................",
    "................................................",
    "................................................",
    "................................................",
    "................................................",
    "................................................",
    "................................................",
    "................................................",
    "................................................",
];

// ------------------------------------------------------------------------
//  COINS / HEARTS — 16x16
// ------------------------------------------------------------------------

pub const COIN_COPPER: [&str; 16] = [
    "................",
    "................",
    "................",
    ".....09990......",
    "....099990......",
    "...09909990.....",
    "...09909990.....",
    "...09099090.....",
    "...09099090.....",
    "...09909990.....",
    "...09909990.....",
    "....099990......",
    ".....09990......",
    "................",
    "................",
    "................",
];

pub const COIN_SILVER: [&str; 16] = [
    "................",
    "................",
    "................",
    ".....0kkk0......",
    "....0kkkkk0.....",
    "...0kkk5kkk0....",
    "...0kk5l5kk0....",
    "...0kk5l5kk0....",
    "...0kk5l5kk0....",
    "...0kkk5kkk0....",
    "....0kkkkk0.....",
    ".....0kkk0......",
    "................",
    "................",
    "................",
    "................",
];

pub const COIN_GOLD: [&str; 16] = [
    "................",
    "................",
    "................",
    ".....07770......",
    "....0777770.....",
    "...07734770.....",
    "...07343470.....",
    "...07333470.....",
    "...07343470.....",
    "...07734770.....",
    "....0777770.....",
    ".....07770......",
    "................",
    "................",
    "................",
    "................",
];

pub const COIN_BREZEL: [&str; 16] = [
    "................",
    "................",
    "...0jjjjjjjj0...",
    "..0jj0000000jj..",
    ".0jj0jjjjjj0jj0.",
    ".0j0jjj00jjjj0j.",
    ".0j0jj0jj0jjj0j.",
    ".0j0j0jjjj0jj0j.",
    ".0j0j0jjjj0jj0j.",
    ".0j0jj0jj0jjj0j.",
    ".0j0jjj00jjjj0j.",
    ".0jj0jjjjjj0jj0.",
    "..0jj0000000jj..",
    "...0jjjjjjjj0...",
    "................",
    "................",
];

pub const HEART_FULL: [&str; 16] = [
    "................",
    "................",
    "................",
    "................",
    "....0330330.....",
    "...033333330....",
    "...033333330....",
    "...033333330....",
    "....0333330.....",
    ".....03330......",
    "......030.......",
    "................",
    "................",
    "................",
    "................",
    "................",
];

pub const HEART_EMPTY: [&str; 16] = [
    "................",
    "................",
    "................",
    "................",
    "....0bb0bb0.....",
    "...0bbbbbbb0....",
    "...0b00000b0....",
    "...0b00000b0....",
    "....0b000b0.....",
    ".....0b0b0......",
    "......0b0.......",
    "................",
    "................",
    "................",
    "................",
    "................",
];

// ------------------------------------------------------------------------
//  TILES — 16×16 — verschiedene Bodentypen
// ------------------------------------------------------------------------

pub const TILE_GRASS: [&str; 16] = [
    "8a888a8a8888a888",
    "8888a8888a88a888",
    "a888888a888a8a88",
    "8a88n8888a888888",
    "888a888888a8a88a",
    "88a888a888888a88",
    "888888a888a88a88",
    "a888a888888a8888",
    "888a88a88a8a8888",
    "88a888a8888888a8",
    "888a8888a8a8888a",
    "a888a888888a888a",
    "888888a8a888a888",
    "8a8a888888888a88",
    "888888a888a8a888",
    "a8a888a888a888a8",
];

pub const TILE_DIRT: [&str; 16] = [
    "ggggogggggoggggo",
    "goggogoogggogggg",
    "ggggogggogoggggo",
    "gogogogggggogogg",
    "ggoggogoggoggggo",
    "ggggoggggogoggoo",
    "ogoggogggoggogog",
    "gogogogogogoggog",
    "ggoggoggogggogog",
    "goggoggogoggogog",
    "ggogoggogggoggog",
    "ggogogogogoggogo",
    "ogggoggoggoggogg",
    "ggoggoggogggoggo",
    "ggggoggoggoggoog",
    "goggggogggoggogo",
];

pub const TILE_ROAD: [&str; 16] = [
    "cccccccccccccccc",
    "cbccccccbcccccbc",
    "ccccbccccccccccb",
    "cccccccccccccccc",
    "ccccccccbccccccc",
    "cccccccccccccccc",
    "4444444444444444",
    "cccccccccccccccc",
    "ccccbccccccccbcc",
    "cccccccccccccccc",
    "ccbcccccccccccbc",
    "cccccccccccccccc",
    "4444444444444444",
    "ccccccbcccccbccc",
    "cccccccccccccccc",
    "cccccccbcccccccc",
];

pub const TILE_SIDEWALK: [&str; 16] = [
    "bbbbbbbbbbbbbbbb",
    "bbbbbbbbbbbbbbbc",
    "bbbbbbbbbbbbbbbb",
    "bbbbbbbbbbbbbbbb",
    "bbbbbbbbbbbbbbbb",
    "bbbbbbbbbbbbbbbb",
    "bbbbbbbbbbbbbbbb",
    "cccccccccccccccc",
    "bbbbbbbbbbbbbbbb",
    "bbbbbbbbbbbbbbbb",
    "bbbbbbbbbbbbbbbb",
    "bbbbbbbbbbbbbbbb",
    "bbbbbbbbbbbbbbbb",
    "bbbbbbbbbbbbbbbb",
    "bbbbbbbbbbbbbbbb",
    "cccccccccccccccc",
];

pub const TILE_REDTHREAD: [&str; 16] = [
    "bbbbbbbbbbbbbbbb",
    "bbbbbbbbbbbbbbbb",
    "bbbbbbbbbbbbbbbb",
    "b333bbbbbbb333bb",
    "b333bbbbbbb333bb",
    "bbbbbbbbbbbbbbbb",
    "bbbbbbb333bbbbbb",
    "bbbbbbb333bbbbbb",
    "bbbb333bbbbbb333",
    "bbbb333bbbbbb333",
    "bbbbbbbbb333bbbb",
    "bbbbbbbbb333bbbb",
    "b333bbbbbbbbbbbb",
    "b333bbbbbbbbbbbb",
    "bbbbbbbb333bbbbb",
    "bbbbbbbb333bbbbb",
];

pub const TILE_WATER: [&str; 16] = [
    "dddddddddddddddd",
    "dddeeddddddeeddd",
    "ddddddddddddeddd",
    "deddddeedddddddd",
    "dddddddddeddddee",
    "ddddeddddddddddd",
    "dddddddeedddddee",
    "deeddddddeddddee",
    "dddddddddddeeddd",
    "ddeedddddddddddd",
    "dddddddeeddddedd",
    "dddedddddddeeedd",
    "ddddddddeeddeedd",
    "dddeedddddddddee",
    "dddddddddddddddd",
    "dddddeeeddddddee",
];

pub const TILE_SAND: [&str; 16] = [
    "hhhhhhhhhhhhhhhh",
    "hhh7hhhhhhhh7hhh",
    "hhhhhhhh7hhhhhhh",
    "hhhhhh7hhhhhhhhh",
    "hh7hhhhhhhhhh7hh",
    "hhhhhhhhh7hhhhhh",
    "hhhh7hhhhhhhhhhh",
    "hhhhhhhhhh7hhhhh",
    "hhhhh7hhhhhhh7hh",
    "hhhhhhhhh7hhhhhh",
    "hh7hhhhhhhhhhhhh",
    "hhhhhhh7hhhh7hhh",
    "hhhhh7hhhhhhhhhh",
    "hhhhhhhhhh7hhhhh",
    "hh7hhhhh7hhhhhhh",
    "hhhhhhhhhhhhhhhh",
];

pub const TILE_ICE: [&str; 16] = [
    "eeeeeeeeeeeeeeee",
    "elllleeeeeeeeeee",
    "eelllllleeeeeeee",
    "eeeellllleeeeeee",
    "eeeeeeeeeeeeleee",
    "eeeeeeeeeellleee",
    "eeeeeeeeeeeeeeee",
    "eeeellllleeeeeee",
    "eelleeeeelllleee",
    "ellleeeeeeelleee",
    "eeeeeeeeeeeellll",
    "eeeeeellleeeeeee",
    "eeeleellleeeeeee",
    "eeeeeeeeeeeellll",
    "eeeeeeeeeeeeeeee",
    "eeeeeeeeeeeeeeee",
];

pub const TILE_FOREST: [&str; 16] = [
    "8a8888aa8aa888aa",
    "a888aa8aaaa888aa",
    "aaa8aaaaaaaaaa8a",
    "aaaaqqqqaaaaaaaa",
    "aaaqqqqqqaaqqqqa",
    "aaqqqqqqqqqqqqqa",
    "aaqqqqqqqqqqqqqa",
    "aaqqqqqqqqqqqqqa",
    "aaqqqqqqqqqqqqqa",
    "aaaqqqqqqaaqqqqa",
    "aaaaqqqqaaaaaaaa",
    "aaaaqqqqaaaaqqqa",
    "aaaaqqqqaaaaqqqa",
    "aaaaqqqqaaaaqqqa",
    "8a8aaaa8aaa888aa",
    "888aa8aaaa888aaa",
];

pub const TILE_BUILDING: [&str; 16] = [
    "pppppppppppppppp",
    "pp00ppppp00ppppp",
    "pp44ppppp44ppppp",
    "pp44ppppp44ppppp",
    "pp00ppppp00ppppp",
    "pppppppppppppppp",
    "pppppppppppppppp",
    "pppp00ppppp00ppp",
    "pppp44ppppp44ppp",
    "pppp44ppppp44ppp",
    "pppp00ppppp00ppp",
    "pppppppppppppppp",
    "pppppppppppppppp",
    "pp00ppppp00ppppp",
    "pp44ppppp44ppppp",
    "pppppppppppppppp",
];

pub const TILE_IHLE_WALL: [&str; 16] = [
    "pppppppppppppppp",
    "p44444444444444p",
    "p47700pppp00774p",
    "p47000pppp00074p",
    "p47000pppp00074p",
    "p47700pppp00774p",
    "p44444444444444p",
    "p43333333333334p",
    "p44444444444444p",
    "p44444444444444p",
    "pppppppppppppppp",
    "pp00ppppppp00ppp",
    "pp44ppppppp44ppp",
    "pp44ppppppp44ppp",
    "pp00ppppppp00ppp",
    "pppppppppppppppp",
];

pub const TILE_BOSSFLOOR: [&str; 16] = [
    "iiiiiiiiiiiiiiii",
    "i00ii00ii00ii00i",
    "iiiiiiiiiiiiiiii",
    "iiaaiiaaiiaaiiii",
    "iaaaiaaaiaaaaiii",
    "iiaaiiaaiiaaiiii",
    "iiiiiiiiiiiiiiii",
    "i00ii00ii00ii00i",
    "iiiiiiiiiiiiiiii",
    "iiaaiiaaiiaaiiii",
    "iaaaiaaaiaaaaiii",
    "iiaaiiaaiiaaiiii",
    "iiiiiiiiiiiiiiii",
    "i00ii00ii00ii00i",
    "iiiiiiiiiiiiiiii",
    "iiiiiiiiiiiiiiii",
];

pub const TILE_MOLD_SLOW: [&str; 16] = [
    "8a8a8a8a8a8a8a8a",
    "aaaaaaaaaaaaaaaa",
    "8a8a8a8a8a8a8a8a",
    "aaaaaaaaaaaaaaaa",
    "8a8a8a8a8a8a8a8a",
    "aaaaaaaaaaaaaaaa",
    "8a8a8a8a8a8a8a8a",
    "aaaaaaaaaaaaaaaa",
    "8a8a8a8a8a8a8a8a",
    "aaaaaaaaaaaaaaaa",
    "8a8a8a8a8a8a8a8a",
    "aaaaaaaaaaaaaaaa",
    "8a8a8a8a8a8a8a8a",
    "aaaaaaaaaaaaaaaa",
    "8a8a8a8a8a8a8a8a",
    "aaaaaaaaaaaaaaaa",
];

pub const TILE_BRUNNEN: [&str; 16] = [
    "................",
    ".....0ssss0.....",
    "....0sllll0s....",
    "...0sldddls0....",
    "...0sld4dls0....",
    "...0slddddls0...",
    "...0sllllls0....",
    "....0ssss0s.....",
    "....0sssssss....",
    "...0ssssssss0...",
    "...0sssssssss0..",
    "..0ssssssssss0..",
    "..0ss0000000s0..",
    "..0sssssssss0...",
    "...0sssssss0....",
    "....0000000.....",
];

// Projektil-Sporenball (8×8 in 16x16)
pub const PROJECTILE_SPORE: [&str; 16] = [
    "................",
    "................",
    "................",
    "................",
    "................",
    ".......888......",
    "......8aaa8.....",
    "......8aaa8.....",
    "......8aaa8.....",
    ".......888......",
    "................",
    "................",
    "................",
    "................",
    "................",
    "................",
];

// NPC-Generisch (Bürger im Hemd) — Stadtführer Klaus
pub const NPC_KLAUS: [&str; 16] = [
    "................",
    ".....022220.....",
    "....02hhhh20....",
    "....02hhhh20....",
    "....01111110....",
    "....01010110....",
    "....01111110....",
    "....01ggg110....",
    ".....033330.....",
    "....03333330....",
    "....03333330....",
    "....03433330....",
    "....03333330....",
    "....02202200....",
    "....02202200....",
    "....02..02......",
];

// NPC — Meister Ihle (Bäckermütze)
pub const NPC_IHLE: [&str; 16] = [
    "................",
    "....044444440...",
    "...04444444440..",
    "....044444440...",
    "....02222220....",
    "....01111110....",
    "....01010110....",
    "....01122110....",
    ".....044440.....",
    "....04444440....",
    "....04434440....",
    "....04443440....",
    "....04444440....",
    "....06606600....",
    "....06606600....",
    "....02..02......",
];

// NPC — Oma Liesl
pub const NPC_OMA: [&str; 16] = [
    "................",
    "....0kkkkk0.....",
    "...0kbbbbbk0....",
    "...0kbbkkbbk....",
    "....02222220....",
    "....01010110....",
    "....01111110....",
    "....01122110....",
    ".....0iiii0.....",
    "....0iiiiii0....",
    "....0iiiiiii....",
    "....0iiiiii0....",
    "....0iiiiii0....",
    "....02000200....",
    "....02000200....",
    "....02..02......",
];

// NPC — Gerhard S-Bahn
pub const NPC_GERHARD: [&str; 16] = [
    "................",
    "....055555550...",
    "...05555555550..",
    "....055555550...",
    "....01111110....",
    "....01010110....",
    "....01111110....",
    "....01122110....",
    ".....055550.....",
    "....05555550....",
    "....05545540....",
    "....05445550....",
    "....05555550....",
    "....06606600....",
    "....06606600....",
    "....02..02......",
];

// NPC — Franz (Sportfreunde-Stiller-Fan)
pub const NPC_FRANZ: [&str; 16] = [
    "................",
    ".....022220.....",
    "....02222220....",
    "....02000020....",
    "....01111110....",
    "....01010110....",
    "....01111110....",
    "....01122110....",
    ".....033330.....",
    "....03334330....",
    "....03343330....",
    "....03333330....",
    "....03333330....",
    "....02202200....",
    "....02202200....",
    "....02..02......",
];

// ------------------------------------------------------------------------
//  DUNGEON-KRISTALL — 16×16, leuchtender Edelstein
// ------------------------------------------------------------------------

pub const CRYSTAL: [&str; 16] = [
    "................",
    ".......00.......",
    "......0ff0......",
    ".....0flff0.....",
    "....0fllff0.....",
    "....0fllff0.....",
    "...0flllff0.....",
    "...0flllff0.....",
    "...0fflfff0.....",
    "....0fffff0.....",
    "....0fffff0.....",
    ".....0fff0......",
    "......0f0.......",
    ".......0........",
    "................",
    "................",
];

