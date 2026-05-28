//! Programmatisch generierte Pixel-Sprites — kein externes Asset noetig.

use bevy::prelude::*;
use bevy::render::render_asset::RenderAssetUsages;
use bevy::render::render_resource::{Extent3d, TextureDimension, TextureFormat};

pub struct AssetsPlugin;

impl Plugin for AssetsPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<GameAssets>()
            .add_systems(PreStartup, generate_all_sprites);
    }
}

#[derive(Resource, Default)]
pub struct GameAssets {
    /// Side-view van (facing east), 2 wheel frames. Flip_x for west.
    pub van_side: [Handle<Image>; 2],
    /// Top-view van (facing north/up), 2 wheel frames. Flip_y for south.
    pub van_top: [Handle<Image>; 2],
    pub shadow: Handle<Image>,
    /// Kept as a default spawn texture (== van_side[0]).
    pub sprinter: Handle<Image>,
    pub sprinter_icon: Handle<Image>,
    pub package: Handle<Image>,
    pub npc_man: Handle<Image>,
    pub npc_woman: Handle<Image>,
    pub npc_child: Handle<Image>,
    pub npc_elder: Handle<Image>,
    pub jannick: Handle<Image>,
    pub target_x: Handle<Image>,
    pub pickup_marker: Handle<Image>,
    pub coin_icon: Handle<Image>,
    pub arrow: Handle<Image>,
    pub spark: Handle<Image>,
}

pub type Rgba = (u8, u8, u8, u8);

pub fn make_image(width: u32, height: u32, pixels: Vec<u8>) -> Image {
    Image::new(
        Extent3d {
            width,
            height,
            depth_or_array_layers: 1,
        },
        TextureDimension::D2,
        pixels,
        TextureFormat::Rgba8UnormSrgb,
        RenderAssetUsages::default(),
    )
}

pub fn set_px(buf: &mut [u8], w: u32, x: i32, y: i32, c: Rgba) {
    if x < 0 || y < 0 || (x as u32) >= w {
        return;
    }
    let idx = ((y as u32) * w + (x as u32)) as usize * 4;
    if idx + 3 >= buf.len() {
        return;
    }
    buf[idx] = c.0;
    buf[idx + 1] = c.1;
    buf[idx + 2] = c.2;
    buf[idx + 3] = c.3;
}

pub fn fill_rect(buf: &mut [u8], w: u32, x: i32, y: i32, rw: i32, rh: i32, c: Rgba) {
    for dy in 0..rh {
        for dx in 0..rw {
            set_px(buf, w, x + dx, y + dy, c);
        }
    }
}

fn generate_all_sprites(mut images: ResMut<Assets<Image>>, mut assets: ResMut<GameAssets>) {
    assets.van_side = [
        images.add(make_image(24, 16, draw_van_side(0))),
        images.add(make_image(24, 16, draw_van_side(1))),
    ];
    assets.van_top = [
        images.add(make_image(16, 24, draw_van_top(0))),
        images.add(make_image(16, 24, draw_van_top(1))),
    ];
    assets.shadow = images.add(make_image(26, 14, draw_shadow()));
    assets.sprinter = assets.van_side[0].clone();
    assets.spark = images.add(make_image(8, 8, draw_spark()));
    assets.sprinter_icon = images.add(make_image(16, 10, draw_sprinter_icon()));
    assets.package = images.add(make_image(8, 8, draw_package()));
    assets.npc_man = images.add(make_image(12, 16, draw_npc((40, 90, 180), (60, 40, 20))));
    assets.npc_woman = images.add(make_image(12, 16, draw_npc((180, 60, 110), (120, 80, 30))));
    assets.npc_child = images.add(make_image(10, 12, draw_npc_child()));
    assets.npc_elder = images.add(make_image(12, 16, draw_npc((90, 90, 100), (220, 220, 220))));
    assets.jannick = images.add(make_image(14, 18, draw_jannick()));
    assets.target_x = images.add(make_image(12, 12, draw_target_x()));
    assets.pickup_marker = images.add(make_image(12, 12, draw_pickup_marker()));
    assets.coin_icon = images.add(make_image(8, 8, draw_coin()));
    assets.arrow = images.add(make_image(16, 16, draw_arrow()));
}

// Shared van palette.
const VAN_BODY: Rgba = (244, 245, 250, 255);
const VAN_SHADE: Rgba = (205, 208, 220, 255);
const VAN_OUTLINE: Rgba = (22, 22, 30, 255);
const VAN_WINDOW: Rgba = (120, 195, 235, 255);
const VAN_WINDOW_HI: Rgba = (180, 225, 245, 255);
const VAN_BLUE: Rgba = (30, 80, 180, 255);
const VAN_DBLUE: Rgba = (18, 50, 130, 255);
const VAN_WHEEL: Rgba = (26, 26, 32, 255);
const VAN_HUB: Rgba = (90, 92, 105, 255);
const VAN_LIGHT: Rgba = (255, 240, 180, 255);
const VAN_TAIL: Rgba = (220, 60, 50, 255);

/// Side-view van facing east (+x). `frame` toggles the wheel hub for a rolling
/// feel. Flip horizontally to face west.
fn draw_van_side(frame: u8) -> Vec<u8> {
    let w = 24u32;
    let h = 16u32;
    let mut buf = vec![0u8; (w * h * 4) as usize];

    // Wheels poke out top & bottom (top-down side view).
    let hub_off = if frame == 0 { 0 } else { 1 };
    for &wx in &[2i32, 18] {
        fill_rect(&mut buf, w, wx, 0, 4, 2, VAN_WHEEL);
        fill_rect(&mut buf, w, wx, 14, 4, 2, VAN_WHEEL);
        set_px(&mut buf, w, wx + 1 + hub_off, 0, VAN_HUB);
        set_px(&mut buf, w, wx + 1 + hub_off, 15, VAN_HUB);
    }

    // Body.
    fill_rect(&mut buf, w, 1, 1, 22, 14, VAN_OUTLINE);
    fill_rect(&mut buf, w, 2, 2, 20, 12, VAN_BODY);
    fill_rect(&mut buf, w, 2, 12, 20, 2, VAN_SHADE); // bottom shading

    // Windshield (front = right) + cab pillar.
    fill_rect(&mut buf, w, 17, 3, 4, 10, VAN_WINDOW);
    fill_rect(&mut buf, w, 17, 3, 4, 2, VAN_WINDOW_HI);
    fill_rect(&mut buf, w, 21, 3, 1, 10, VAN_OUTLINE);

    // Headlight (front) + tail light (rear).
    fill_rect(&mut buf, w, 22, 2, 1, 3, VAN_LIGHT);
    fill_rect(&mut buf, w, 22, 11, 1, 3, VAN_LIGHT);
    fill_rect(&mut buf, w, 1, 2, 1, 3, VAN_TAIL);
    fill_rect(&mut buf, w, 1, 11, 1, 3, VAN_TAIL);

    // Ihle logo panel on the side.
    fill_rect(&mut buf, w, 5, 5, 9, 6, VAN_BLUE);
    fill_rect(&mut buf, w, 5, 6, 1, 4, VAN_BODY);
    fill_rect(&mut buf, w, 7, 5, 1, 6, VAN_BODY);
    fill_rect(&mut buf, w, 9, 5, 1, 6, VAN_BODY);
    fill_rect(&mut buf, w, 11, 5, 1, 6, VAN_BODY);
    fill_rect(&mut buf, w, 13, 5, 1, 6, VAN_BODY);
    fill_rect(&mut buf, w, 5, 7, 9, 1, VAN_DBLUE);
    fill_rect(&mut buf, w, 5, 9, 9, 1, VAN_DBLUE);

    buf
}

/// Top-view van facing north (+y, up). Flip vertically to face south.
fn draw_van_top(frame: u8) -> Vec<u8> {
    let w = 16u32;
    let h = 24u32;
    let mut buf = vec![0u8; (w * h * 4) as usize];

    // Wheels on left & right (front near top, rear near bottom).
    let hub_off = if frame == 0 { 0 } else { 1 };
    for &wy in &[3i32, 17] {
        fill_rect(&mut buf, w, 0, wy, 2, 4, VAN_WHEEL);
        fill_rect(&mut buf, w, 14, wy, 2, 4, VAN_WHEEL);
        set_px(&mut buf, w, 0, wy + 1 + hub_off, VAN_HUB);
        set_px(&mut buf, w, 15, wy + 1 + hub_off, VAN_HUB);
    }

    // Body.
    fill_rect(&mut buf, w, 1, 1, 14, 22, VAN_OUTLINE);
    fill_rect(&mut buf, w, 2, 2, 12, 20, VAN_BODY);
    fill_rect(&mut buf, w, 2, 2, 2, 20, VAN_SHADE); // left edge shading

    // Windshield at the front (top) + headlights.
    fill_rect(&mut buf, w, 3, 2, 10, 4, VAN_WINDOW);
    fill_rect(&mut buf, w, 3, 2, 10, 1, VAN_WINDOW_HI);
    fill_rect(&mut buf, w, 2, 2, 2, 1, VAN_LIGHT);
    fill_rect(&mut buf, w, 12, 2, 2, 1, VAN_LIGHT);

    // Rear doors + tail lights.
    fill_rect(&mut buf, w, 7, 6, 1, 16, VAN_SHADE);
    fill_rect(&mut buf, w, 2, 21, 3, 1, VAN_TAIL);
    fill_rect(&mut buf, w, 11, 21, 3, 1, VAN_TAIL);

    // Ihle roof badge.
    fill_rect(&mut buf, w, 4, 9, 8, 8, VAN_BLUE);
    fill_rect(&mut buf, w, 5, 10, 6, 6, VAN_BODY);
    fill_rect(&mut buf, w, 6, 11, 4, 1, VAN_BLUE);
    fill_rect(&mut buf, w, 6, 13, 4, 1, VAN_BLUE);
    fill_rect(&mut buf, w, 7, 11, 2, 4, VAN_BLUE);

    buf
}

fn draw_shadow() -> Vec<u8> {
    let w = 26u32;
    let h = 14u32;
    let mut buf = vec![0u8; (w * h * 4) as usize];
    let cx = w as f32 / 2.0;
    let cy = h as f32 / 2.0;
    for y in 0..h as i32 {
        for x in 0..w as i32 {
            let dx = (x as f32 - cx) / cx;
            let dy = (y as f32 - cy) / cy;
            if dx * dx + dy * dy <= 1.0 {
                set_px(&mut buf, w, x, y, (0, 0, 0, 90));
            }
        }
    }
    buf
}

fn draw_spark() -> Vec<u8> {
    let w = 8u32;
    let h = 8u32;
    let mut buf = vec![0u8; (w * h * 4) as usize];
    let c: Rgba = (255, 255, 255, 255);
    fill_rect(&mut buf, w, 2, 1, 4, 6, c);
    fill_rect(&mut buf, w, 1, 2, 6, 4, c);
    buf
}

fn draw_sprinter_icon() -> Vec<u8> {
    let w = 16u32;
    let h = 10u32;
    let mut buf = vec![0u8; (w * h * 4) as usize];
    let body: Rgba = (242, 242, 248, 255);
    let outline: Rgba = (20, 20, 25, 255);
    let blue: Rgba = (30, 80, 180, 255);
    let wheel: Rgba = (28, 28, 32, 255);
    let window: Rgba = (130, 200, 235, 255);
    fill_rect(&mut buf, w, 0, 1, 16, 8, outline);
    fill_rect(&mut buf, w, 1, 2, 14, 6, body);
    fill_rect(&mut buf, w, 1, 0, 2, 2, wheel);
    fill_rect(&mut buf, w, 13, 0, 2, 2, wheel);
    fill_rect(&mut buf, w, 1, 8, 2, 2, wheel);
    fill_rect(&mut buf, w, 13, 8, 2, 2, wheel);
    fill_rect(&mut buf, w, 11, 3, 3, 4, window);
    fill_rect(&mut buf, w, 3, 3, 6, 4, blue);
    buf
}

fn draw_package() -> Vec<u8> {
    let w = 8u32;
    let h = 8u32;
    let mut buf = vec![0u8; (w * h * 4) as usize];
    let outline: Rgba = (60, 40, 20, 255);
    let body: Rgba = (190, 140, 70, 255);
    let band: Rgba = (240, 60, 60, 255);
    fill_rect(&mut buf, w, 0, 0, 8, 8, outline);
    fill_rect(&mut buf, w, 1, 1, 6, 6, body);
    fill_rect(&mut buf, w, 3, 0, 2, 8, band);
    fill_rect(&mut buf, w, 0, 3, 8, 2, band);
    buf
}

fn draw_npc(shirt: (u8, u8, u8), hair: (u8, u8, u8)) -> Vec<u8> {
    let w = 12u32;
    let h = 16u32;
    let mut buf = vec![0u8; (w * h * 4) as usize];
    let skin: Rgba = (245, 200, 165, 255);
    let pants: Rgba = (40, 40, 60, 255);
    let shoe: Rgba = (20, 20, 25, 255);
    let outline: Rgba = (20, 20, 25, 255);

    fill_rect(&mut buf, w, 3, 1, 6, 5, outline);
    fill_rect(&mut buf, w, 4, 2, 4, 4, skin);
    fill_rect(&mut buf, w, 3, 1, 6, 2, (hair.0, hair.1, hair.2, 255));
    fill_rect(&mut buf, w, 5, 3, 1, 1, outline);
    fill_rect(&mut buf, w, 7, 3, 1, 1, outline);

    fill_rect(&mut buf, w, 2, 6, 8, 5, outline);
    fill_rect(&mut buf, w, 3, 7, 6, 4, (shirt.0, shirt.1, shirt.2, 255));
    fill_rect(&mut buf, w, 1, 7, 2, 3, (shirt.0, shirt.1, shirt.2, 255));
    fill_rect(&mut buf, w, 9, 7, 2, 3, (shirt.0, shirt.1, shirt.2, 255));
    fill_rect(&mut buf, w, 1, 10, 2, 1, skin);
    fill_rect(&mut buf, w, 9, 10, 2, 1, skin);

    fill_rect(&mut buf, w, 3, 11, 6, 4, pants);
    fill_rect(&mut buf, w, 5, 11, 1, 4, outline);
    fill_rect(&mut buf, w, 6, 11, 1, 4, outline);
    fill_rect(&mut buf, w, 3, 15, 2, 1, shoe);
    fill_rect(&mut buf, w, 7, 15, 2, 1, shoe);
    buf
}

fn draw_npc_child() -> Vec<u8> {
    let w = 10u32;
    let h = 12u32;
    let mut buf = vec![0u8; (w * h * 4) as usize];
    let skin: Rgba = (250, 210, 175, 255);
    let shirt: Rgba = (250, 200, 60, 255);
    let pants: Rgba = (40, 90, 200, 255);
    let hair: Rgba = (90, 60, 30, 255);
    let outline: Rgba = (20, 20, 25, 255);
    fill_rect(&mut buf, w, 2, 0, 6, 5, outline);
    fill_rect(&mut buf, w, 3, 1, 4, 4, skin);
    fill_rect(&mut buf, w, 2, 0, 6, 2, hair);
    fill_rect(&mut buf, w, 4, 3, 1, 1, outline);
    fill_rect(&mut buf, w, 6, 3, 1, 1, outline);

    fill_rect(&mut buf, w, 2, 5, 6, 4, outline);
    fill_rect(&mut buf, w, 3, 6, 4, 3, shirt);
    fill_rect(&mut buf, w, 2, 9, 6, 3, pants);
    fill_rect(&mut buf, w, 4, 9, 1, 3, outline);
    fill_rect(&mut buf, w, 5, 9, 1, 3, outline);
    buf
}

fn draw_jannick() -> Vec<u8> {
    let w = 14u32;
    let h = 18u32;
    let mut buf = vec![0u8; (w * h * 4) as usize];
    let skin: Rgba = (240, 195, 160, 255);
    let red: Rgba = (220, 30, 30, 255);
    let white: Rgba = (245, 245, 245, 255);
    let apron: Rgba = (245, 245, 245, 255);
    let pants: Rgba = (40, 30, 30, 255);
    let outline: Rgba = (20, 20, 25, 255);
    let hat: Rgba = (255, 255, 255, 255);

    fill_rect(&mut buf, w, 3, 0, 8, 3, hat);
    fill_rect(&mut buf, w, 3, 0, 8, 1, outline);
    fill_rect(&mut buf, w, 3, 3, 8, 1, outline);

    fill_rect(&mut buf, w, 4, 4, 6, 5, outline);
    fill_rect(&mut buf, w, 5, 5, 4, 4, skin);
    fill_rect(&mut buf, w, 6, 6, 1, 1, outline);
    fill_rect(&mut buf, w, 8, 6, 1, 1, outline);
    fill_rect(&mut buf, w, 6, 8, 3, 1, outline);

    fill_rect(&mut buf, w, 3, 9, 8, 5, outline);
    fill_rect(&mut buf, w, 4, 10, 1, 3, red);
    fill_rect(&mut buf, w, 5, 10, 1, 3, white);
    fill_rect(&mut buf, w, 6, 10, 1, 3, red);
    fill_rect(&mut buf, w, 7, 10, 1, 3, white);
    fill_rect(&mut buf, w, 8, 10, 1, 3, red);
    fill_rect(&mut buf, w, 9, 10, 1, 3, white);
    fill_rect(&mut buf, w, 4, 13, 6, 1, apron);

    fill_rect(&mut buf, w, 4, 14, 6, 4, pants);
    fill_rect(&mut buf, w, 6, 14, 1, 4, outline);
    fill_rect(&mut buf, w, 7, 14, 1, 4, outline);
    buf
}

fn draw_target_x() -> Vec<u8> {
    let w = 12u32;
    let h = 12u32;
    let mut buf = vec![0u8; (w * h * 4) as usize];
    let red: Rgba = (235, 35, 35, 255);
    let dark: Rgba = (90, 10, 10, 255);
    for i in 0..12 {
        set_px(&mut buf, w, i, i, dark);
        set_px(&mut buf, w, 11 - i, i, dark);
    }
    for i in 1..11 {
        set_px(&mut buf, w, i, i, red);
        set_px(&mut buf, w, 11 - i, i, red);
        set_px(&mut buf, w, i - 1, i, red);
        set_px(&mut buf, w, i + 1, i, red);
        set_px(&mut buf, w, 11 - i - 1, i, red);
        set_px(&mut buf, w, 11 - i + 1, i, red);
    }
    buf
}

fn draw_pickup_marker() -> Vec<u8> {
    let w = 12u32;
    let h = 12u32;
    let mut buf = vec![0u8; (w * h * 4) as usize];
    let blue: Rgba = (40, 90, 220, 255);
    let dark: Rgba = (15, 30, 100, 255);
    let white: Rgba = (240, 240, 240, 255);
    fill_rect(&mut buf, w, 2, 1, 8, 8, dark);
    fill_rect(&mut buf, w, 3, 2, 6, 6, blue);
    fill_rect(&mut buf, w, 5, 3, 2, 4, white);
    fill_rect(&mut buf, w, 4, 4, 4, 2, white);
    fill_rect(&mut buf, w, 5, 9, 2, 3, dark);
    buf
}

fn draw_coin() -> Vec<u8> {
    let w = 8u32;
    let h = 8u32;
    let mut buf = vec![0u8; (w * h * 4) as usize];
    let gold: Rgba = (245, 200, 30, 255);
    let dark: Rgba = (140, 90, 10, 255);
    fill_rect(&mut buf, w, 1, 0, 6, 8, dark);
    fill_rect(&mut buf, w, 0, 1, 8, 6, dark);
    fill_rect(&mut buf, w, 2, 1, 4, 6, gold);
    fill_rect(&mut buf, w, 1, 2, 6, 4, gold);
    fill_rect(&mut buf, w, 3, 3, 2, 2, (255, 240, 150, 255));
    buf
}

fn draw_arrow() -> Vec<u8> {
    let w = 16u32;
    let h = 16u32;
    let mut buf = vec![0u8; (w * h * 4) as usize];
    let green: Rgba = (60, 230, 80, 255);
    let dark: Rgba = (15, 100, 25, 255);

    for y in 6..10 {
        for x in 1..10 {
            set_px(&mut buf, w, x, y, green);
        }
    }
    for dy in 0..7 {
        let mid = 7;
        for dx in 0..=dy {
            set_px(&mut buf, w, 9 + dx, mid - dy + 3, green);
            set_px(&mut buf, w, 9 + dx, mid + dy - 3, green);
        }
    }
    for y in 5..11 {
        set_px(&mut buf, w, 1, y, dark);
    }
    for dy in 0..7 {
        set_px(&mut buf, w, 9 + dy, 7 - dy + 3, dark);
        set_px(&mut buf, w, 9 + dy, 7 + dy - 3, dark);
    }
    buf
}
