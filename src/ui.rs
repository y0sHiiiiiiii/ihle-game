//! UI: HUD, Minimap, Dialog-Box, Shop-Fenster, Area-Banner.
//!
//! Wichtig: Alle UI-Funktionen werden im **Bildschirm-Space** gerendert
//! (nach dem Upscale der virtuellen Pixel-Welt). Das Layout bleibt aber
//! in virtuellen 480×270-Koordinaten — `UiCtx` skaliert die Aufrufe auf
//! die echte Bildschirmauflösung. So bleibt der Text **scharf**, während
//! die Pixel-Art-Welt weiterhin mit Nearest-Filter hochskaliert wird.

use crate::consts::*;
use crate::npc::{DialogState, Npc, NpcKind};
use crate::player::Player;
use crate::shop::{header_line, ShopState};
use crate::world::{clock_string, Tile, World};
use macroquad::prelude::*;

// ------------------------------------------------------------------------
//  Texturen-Container (gehalten vom Hauptspiel, hier durchgereicht)
// ------------------------------------------------------------------------

pub struct Textures {
    pub heart_full: Texture2D,
    pub heart_empty: Texture2D,
    pub coin_copper: Texture2D,
    pub coin_silver: Texture2D,
    pub coin_gold: Texture2D,
    pub coin_brezel: Texture2D,
    pub player_down_a: Texture2D,
    pub player_down_b: Texture2D,
    pub player_up: Texture2D,
    pub player_up_b: Texture2D,
    pub player_left: Texture2D,
    pub player_left_b: Texture2D,
    pub player_right: Texture2D,
    pub player_right_b: Texture2D,
    pub player_swim: Texture2D,
    pub enemy_mold: Texture2D,
    pub enemy_blob: Texture2D,
    pub enemy_rat: Texture2D,
    pub enemy_beat: Texture2D,
    pub enemy_ice: Texture2D,
    pub boss: Texture2D,
    pub projectile: Texture2D,
    pub npc_ihle: Texture2D,
    pub npc_klaus: Texture2D,
    pub npc_gerhard: Texture2D,
    pub npc_oma: Texture2D,
    pub npc_franz: Texture2D,
    pub tile_grass: Texture2D,
    pub tile_dirt: Texture2D,
    pub tile_road: Texture2D,
    pub tile_sidewalk: Texture2D,
    pub tile_redthread: Texture2D,
    pub tile_water: Texture2D,
    pub tile_sand: Texture2D,
    pub tile_ice: Texture2D,
    pub tile_forest: Texture2D,
    pub tile_building: Texture2D,
    pub tile_ihle: Texture2D,
    pub tile_bossfloor: Texture2D,
    pub tile_moldslow: Texture2D,
    pub tile_brunnen: Texture2D,
    pub tile_zebra: Texture2D,
    pub crystal: Texture2D,
}

impl Textures {
    pub fn tile_texture(&self, t: Tile) -> &Texture2D {
        match t {
            Tile::Grass => &self.tile_grass,
            Tile::Dirt => &self.tile_dirt,
            Tile::Road => &self.tile_road,
            Tile::Sidewalk => &self.tile_sidewalk,
            Tile::RedThread => &self.tile_redthread,
            Tile::Water => &self.tile_water,
            Tile::Sand => &self.tile_sand,
            Tile::Ice => &self.tile_ice,
            Tile::Forest => &self.tile_forest,
            Tile::Building => &self.tile_building,
            Tile::IhleWall => &self.tile_ihle,
            Tile::BossFloor => &self.tile_bossfloor,
            Tile::MoldSlow => &self.tile_moldslow,
            Tile::Brunnen => &self.tile_brunnen,
            Tile::Zebra => &self.tile_zebra,
        }
    }
}

// ------------------------------------------------------------------------
//  UiCtx — Virtuelle UI-Koordinaten → Bildschirm-Space
// ------------------------------------------------------------------------

/// Mappt das virtuelle 480×270-Layout auf den echten Bildschirm (letterboxed).
/// Wird einmal pro Frame nach `screen_width/height` aus der Hauptschleife gebaut.
pub struct UiCtx {
    pub scale: f32,
    pub offset_x: f32,
    pub offset_y: f32,
}

impl UiCtx {
    pub fn new() -> Self {
        let aspect = VIRTUAL_W as f32 / VIRTUAL_H as f32;
        let sw = screen_width();
        let sh = screen_height();
        let (w, h) = if sw / sh > aspect {
            (sh * aspect, sh)
        } else {
            (sw, sw / aspect)
        };
        let offset_x = (sw - w) / 2.0;
        let offset_y = (sh - h) / 2.0;
        let scale = h / VIRTUAL_H as f32;
        UiCtx { scale, offset_x, offset_y }
    }

    #[inline]
    pub fn x(&self, vx: f32) -> f32 { self.offset_x + vx * self.scale }
    #[inline]
    pub fn y(&self, vy: f32) -> f32 { self.offset_y + vy * self.scale }
    #[inline]
    pub fn s(&self, vs: f32) -> f32 { vs * self.scale }

    pub fn text(&self, t: &str, vx: f32, vy: f32, vsize: f32, col: Color) {
        draw_text(t, self.x(vx), self.y(vy), self.s(vsize), col);
    }

    /// Breite eines Textstücks in **virtuellen** Pixel-Einheiten.
    pub fn text_w(&self, t: &str, vsize: f32) -> f32 {
        let px = self.s(vsize).max(1.0) as u16;
        measure_text(t, None, px, 1.0).width / self.scale.max(0.0001)
    }

    pub fn rect(&self, vx: f32, vy: f32, vw: f32, vh: f32, col: Color) {
        draw_rectangle(self.x(vx), self.y(vy), self.s(vw), self.s(vh), col);
    }

    pub fn rect_lines(&self, vx: f32, vy: f32, vw: f32, vh: f32, thick: f32, col: Color) {
        draw_rectangle_lines(
            self.x(vx),
            self.y(vy),
            self.s(vw),
            self.s(vh),
            (thick * self.scale).max(1.0),
            col,
        );
    }

    pub fn circle(&self, vx: f32, vy: f32, vr: f32, col: Color) {
        draw_circle(self.x(vx), self.y(vy), self.s(vr), col);
    }

    pub fn texture(&self, t: &Texture2D, vx: f32, vy: f32, vw: f32, vh: f32, col: Color) {
        draw_texture_ex(
            t,
            self.x(vx),
            self.y(vy),
            col,
            DrawTextureParams {
                dest_size: Some(vec2(self.s(vw), self.s(vh))),
                ..Default::default()
            },
        );
    }

    pub fn texture_src(
        &self,
        t: &Texture2D,
        vx: f32,
        vy: f32,
        vw: f32,
        vh: f32,
        src: Rect,
        col: Color,
    ) {
        draw_texture_ex(
            t,
            self.x(vx),
            self.y(vy),
            col,
            DrawTextureParams {
                dest_size: Some(vec2(self.s(vw), self.s(vh))),
                source: Some(src),
                ..Default::default()
            },
        );
    }

    /// Voll-Bildschirm-Rechteck (zB. Dunkel-Overlay).
    pub fn fullscreen_rect(&self, col: Color) {
        self.rect(0.0, 0.0, VIRTUAL_W as f32, VIRTUAL_H as f32, col);
    }
}

// ------------------------------------------------------------------------
//  HUD
// ------------------------------------------------------------------------

pub fn draw_hud(
    ui: &UiCtx,
    tex: &Textures,
    player: &Player,
    game_secs: f32,
    area_name: &str,
    area_fade: f32,
    _cordobar_unlocked: bool,
    purchases: u32,
    crystals: u8,
    quest_hint: &str,
    quest_hint_fade: f32,
) {
    // Herzen oben links
    let max = player.max_hearts;
    let cur = player.hearts;
    for i in 0..max {
        let t = if i < cur { &tex.heart_full } else { &tex.heart_empty };
        ui.texture_src(
            t,
            2.0 + (i as f32) * 10.0,
            2.0,
            12.0,
            12.0,
            Rect::new(2.0, 2.0, 12.0, 12.0),
            WHITE,
        );
    }

    // Münzen oben rechts (rechtsbündig)
    let coin_text = format!("{}", player.coins);
    let coin_w = ui.text_w(&coin_text, 13.0);
    let right_edge = VIRTUAL_W as f32 - 4.0;
    let coin_text_x = right_edge - coin_w;
    let coin_icon_x = coin_text_x - 18.0;
    ui.texture(&tex.coin_gold, coin_icon_x, 0.0, 16.0, 16.0, WHITE);
    ui.text(&coin_text, coin_text_x, 13.0, 13.0, WHITE);
    if purchases >= 10 {
        ui.text("GP", right_edge - 10.0, 25.0, 8.0, YELLOW);
    }

    // Kristall-Counter direkt darunter (rechtsbündig in derselben Spalte)
    let n = crystals.count_ones();
    let cry_text = format!("{}/4", n);
    let cry_w = ui.text_w(&cry_text, 11.0);
    let cry_text_x = right_edge - cry_w;
    let cry_icon_x = cry_text_x - 16.0;
    ui.texture(&tex.crystal, cry_icon_x, 17.0, 14.0, 14.0, WHITE);
    ui.text(
        &cry_text,
        cry_text_x,
        28.0,
        11.0,
        Color::new(0.85, 0.45, 1.0, 1.0),
    );

    // Quest-Hinweis (Fade-Banner oben mittig)
    if quest_hint_fade > 0.0 && !quest_hint.is_empty() {
        let alpha = quest_hint_fade.min(1.0).powf(0.7);
        let tw = ui.text_w(quest_hint, 11.0);
        ui.rect(
            VIRTUAL_W as f32 / 2.0 - tw / 2.0 - 6.0,
            34.0,
            tw + 12.0,
            14.0,
            Color::new(0.0, 0.0, 0.0, alpha * 0.8),
        );
        ui.text(
            quest_hint,
            VIRTUAL_W as f32 / 2.0 - tw / 2.0,
            44.0,
            11.0,
            Color::new(1.0, 0.9, 0.4, alpha),
        );
    }

    // Powerup-Icons unten links
    let mut px = 4.0;
    let py = VIRTUAL_H as f32 - 14.0;
    if player.powerups.speed > 0.0 {
        ui.rect(px, py, 28.0, 10.0, Color::new(0.9, 0.55, 0.20, 0.8));
        ui.text(&format!("S{:.0}", player.powerups.speed), px + 2.0, py + 8.0, 8.0, BLACK);
        px += 32.0;
    }
    if player.powerups.invuln > 0.0 {
        ui.rect(px, py, 28.0, 10.0, Color::new(1.0, 0.85, 0.15, 0.8));
        ui.text(&format!("I{:.0}", player.powerups.invuln), px + 2.0, py + 8.0, 8.0, BLACK);
        px += 32.0;
    }
    if player.powerups.fly > 0.0 {
        ui.rect(px, py, 28.0, 10.0, Color::new(0.7, 0.9, 1.0, 0.8));
        ui.text(&format!("F{:.0}", player.powerups.fly), px + 2.0, py + 8.0, 8.0, BLACK);
        px += 32.0;
    }
    if player.powerups.double_coin > 0.0 {
        ui.rect(px, py, 28.0, 10.0, Color::new(1.0, 0.85, 0.15, 0.9));
        ui.text(&format!("2x{:.0}", player.powerups.double_coin), px + 2.0, py + 8.0, 8.0, BLACK);
        px += 32.0;
    }
    if player.powerups.shield {
        ui.rect(px, py, 28.0, 10.0, Color::new(0.85, 0.65, 0.20, 0.9));
        ui.text("Brzl", px + 2.0, py + 8.0, 8.0, BLACK);
    }

    // Uhrzeit unten rechts
    let t = clock_string(game_secs);
    let tw = ui.text_w(&t, 10.0);
    ui.rect(
        VIRTUAL_W as f32 - tw - 8.0,
        VIRTUAL_H as f32 - 14.0,
        tw + 6.0,
        12.0,
        Color::new(0.0, 0.0, 0.0, 0.55),
    );
    ui.text(&t, VIRTUAL_W as f32 - tw - 4.0, VIRTUAL_H as f32 - 4.0, 10.0, WHITE);

    // Area-Banner (Fade in/out)
    if area_fade > 0.0 {
        let alpha = (area_fade.min(1.0).max(0.0)).powf(0.7);
        let s = format!("~ {} ~", area_name);
        let tw = ui.text_w(&s, 12.0);
        ui.rect(
            VIRTUAL_W as f32 / 2.0 - tw / 2.0 - 6.0,
            VIRTUAL_H as f32 - 32.0,
            tw + 12.0,
            14.0,
            Color::new(0.0, 0.0, 0.0, alpha * 0.7),
        );
        ui.text(
            &s,
            VIRTUAL_W as f32 / 2.0 - tw / 2.0,
            VIRTUAL_H as f32 - 22.0,
            12.0,
            Color::new(1.0, 1.0, 1.0, alpha),
        );
    }

    // Schwimm-Atem-Anzeige
    if player.on_water && player.powerups.fly <= 0.0 {
        let pct = (player.swim_air / 10.0).clamp(0.0, 1.0);
        ui.rect(
            VIRTUAL_W as f32 / 2.0 - 30.0,
            30.0,
            60.0,
            6.0,
            Color::new(0.0, 0.0, 0.0, 0.6),
        );
        ui.rect(
            VIRTUAL_W as f32 / 2.0 - 30.0,
            30.0,
            60.0 * pct,
            6.0,
            Color::new(0.3, 0.65, 0.90, 1.0),
        );
    }
}

// ------------------------------------------------------------------------
//  Minimap (TAB)
// ------------------------------------------------------------------------

pub fn draw_minimap(ui: &UiCtx, world: &World, player: &Player) {
    let mw = 360.0;
    let mh = 216.0;
    let mx = (VIRTUAL_W as f32 - mw) / 2.0;
    let my = (VIRTUAL_H as f32 - mh) / 2.0;

    ui.fullscreen_rect(Color::new(0.0, 0.0, 0.0, 0.6));
    ui.rect(mx - 2.0, my - 2.0, mw + 4.0, mh + 4.0, Color::new(0.1, 0.1, 0.15, 1.0));
    ui.rect(mx, my, mw, mh, Color::new(0.18, 0.20, 0.22, 1.0));

    let sx = mw / world.w as f32;
    let sy = mh / world.h as f32;

    for y in 0..world.h {
        for x in 0..world.w {
            let t = world.get(x, y);
            let col = match t {
                Tile::Grass => Color::new(0.30, 0.70, 0.30, 1.0),
                Tile::Forest => Color::new(0.16, 0.40, 0.18, 1.0),
                Tile::Water => Color::new(0.30, 0.65, 0.90, 1.0),
                Tile::Sand => Color::new(0.92, 0.85, 0.55, 1.0),
                Tile::Ice => Color::new(0.75, 0.92, 0.98, 1.0),
                Tile::Road => Color::new(0.35, 0.38, 0.42, 1.0),
                Tile::Sidewalk => Color::new(0.70, 0.72, 0.74, 1.0),
                Tile::RedThread => Color::new(0.85, 0.18, 0.18, 1.0),
                Tile::Building => Color::new(0.75, 0.25, 0.20, 1.0),
                Tile::IhleWall => Color::new(1.0, 0.85, 0.15, 1.0),
                Tile::BossFloor => Color::new(0.50, 0.08, 0.08, 1.0),
                Tile::Brunnen => Color::new(0.55, 0.40, 0.18, 1.0),
                Tile::Dirt => Color::new(0.55, 0.35, 0.20, 1.0),
                Tile::MoldSlow => Color::new(0.16, 0.40, 0.18, 1.0),
                Tile::Zebra => Color::new(0.90, 0.90, 0.90, 1.0),
            };
            ui.rect(mx + x as f32 * sx, my + y as f32 * sy, sx.ceil(), sy.ceil(), col);
        }
    }

    let (pcx, pcy) = player.center();
    let ptx = pcx / TILE_SIZE;
    let pty = pcy / TILE_SIZE;
    ui.circle(mx + ptx * sx, my + pty * sy, 3.0, YELLOW);

    for f in &FILIALEN {
        let fx = mx + f.tile_x as f32 * sx;
        let fy = my + f.tile_y as f32 * sy;
        ui.rect(fx - 3.0, fy - 3.0, 6.0, 6.0, Color::new(1.0, 0.85, 0.15, 1.0));
        ui.text(&format!("{}", f.nummer), fx - 2.0, fy + 3.0, 8.0, BLACK);
    }
    let bx = mx + 100.0 * sx;
    let by = my + 95.0 * sy;
    ui.circle(bx, by, 4.0, Color::new(0.5, 0.08, 0.08, 1.0));
    ui.text("BOSS", bx - 10.0, by + 12.0, 8.0, Color::new(1.0, 0.5, 0.5, 1.0));

    ui.text("GERMERING - MINIMAP", mx + 4.0, my - 4.0, 11.0, WHITE);
    ui.text(
        "[TAB] schließen     gelb = Ihle-Filialen     roter Faden",
        mx + 4.0,
        my + mh + 12.0,
        9.0,
        Color::new(0.85, 0.85, 0.85, 1.0),
    );
}

// ------------------------------------------------------------------------
//  Dialog
// ------------------------------------------------------------------------

/// Liefert das Portrait-Sprite passend zum NPC.
fn portrait_for<'a>(tex: &'a Textures, kind: NpcKind) -> &'a Texture2D {
    match kind {
        NpcKind::MeisterIhle => &tex.npc_ihle,
        NpcKind::Klaus => &tex.npc_klaus,
        NpcKind::Gerhard => &tex.npc_gerhard,
        NpcKind::OmaLiesl => &tex.npc_oma,
        NpcKind::Franz => &tex.npc_franz,
    }
}

pub fn draw_dialog(ui: &UiCtx, tex: &Textures, npc: &Npc, dialog: &DialogState) {
    let bw = 460.0;
    let bh = 56.0;
    let bx = (VIRTUAL_W as f32 - bw) / 2.0;
    let by = VIRTUAL_H as f32 - bh - 8.0;

    ui.rect(bx, by, bw, bh, Color::new(0.05, 0.04, 0.10, 0.95));
    ui.rect_lines(bx, by, bw, bh, 1.0, Color::new(0.85, 0.85, 0.85, 1.0));

    // Portrait-Frame mit Hintergrund + Border + animiertem NPC-Sprite
    let pf_x = bx + 4.0;
    let pf_y = by + 4.0;
    let pf_w = 28.0;
    let pf_h = 28.0;
    ui.rect(pf_x, pf_y, pf_w, pf_h, Color::new(0.18, 0.18, 0.24, 1.0));
    ui.rect_lines(pf_x, pf_y, pf_w, pf_h, 1.0, Color::new(0.55, 0.55, 0.60, 1.0));

    // Subtiles Bobbing — der NPC „lebt" während er spricht.
    let t = get_time() as f32;
    let bob = (t * 4.0).sin() * 0.8;
    // Beim Tippen extra zappelig (sieht aus als würde er reden).
    let speak = if dialog.typing { (t * 22.0).sin() * 0.4 } else { 0.0 };
    let sprite_x = pf_x + 3.0;
    let sprite_y = pf_y + 2.0 + bob + speak;
    ui.texture(portrait_for(tex, npc.kind), sprite_x, sprite_y, 22.0, 22.0, WHITE);

    // Name rechts neben dem Portrait
    ui.text(npc.name, bx + 38.0, by + 14.0, 11.0, Color::new(1.0, 0.85, 0.15, 1.0));

    // Text mit Typewriter
    let line = npc.dialog.get(dialog.line).copied().unwrap_or("...");
    let visible = dialog.char_count as usize;
    let shown: String = line.chars().take(visible).collect();
    ui.text(&shown, bx + 38.0, by + 32.0, 11.0, WHITE);

    if !dialog.typing {
        // Blinkender "weiter"-Indikator
        if (t * 3.0).sin() > 0.0 {
            ui.text(
                "[E] weiter",
                bx + bw - 56.0,
                by + bh - 4.0,
                8.0,
                Color::new(1.0, 0.85, 0.15, 1.0),
            );
        }
    }
}

// ------------------------------------------------------------------------
//  Shop
// ------------------------------------------------------------------------

pub fn draw_shop(
    ui: &UiCtx,
    tex: &Textures,
    shop: &ShopState,
    player: &Player,
    game_secs: f32,
    purchases: u32,
) {
    let bw = 380.0;
    let bh = 220.0;
    let bx = (VIRTUAL_W as f32 - bw) / 2.0;
    let by = (VIRTUAL_H as f32 - bh) / 2.0;

    ui.fullscreen_rect(Color::new(0.0, 0.0, 0.0, 0.6));
    ui.rect(bx, by, bw, bh, Color::new(0.08, 0.05, 0.05, 0.97));
    ui.rect_lines(bx, by, bw, bh, 1.0, Color::new(1.0, 0.85, 0.15, 1.0));

    // Ihle-Logo prominent links oben im Shop-Fenster
    let logo_x = bx + 6.0;
    let logo_y = by + 4.0;
    let logo_w = 56.0;
    let logo_h = 22.0;
    // Roter Rahmen
    ui.rect(logo_x, logo_y, logo_w, logo_h, Color::new(0.55, 0.12, 0.12, 1.0));
    // Gelber Innenraum
    ui.rect(logo_x + 1.5, logo_y + 1.5, logo_w - 3.0, logo_h - 3.0,
        Color::new(1.0, 0.85, 0.15, 1.0));
    // Glanz oben
    ui.rect(logo_x + 1.5, logo_y + 1.5, logo_w - 3.0, 3.0,
        Color::new(1.0, 0.95, 0.55, 1.0));
    // "IHLE" Schriftzug
    ui.text("IHLE", logo_x + 8.0, logo_y + 16.0, 16.0, Color::new(0.55, 0.12, 0.12, 1.0));
    // Untertitel "Landbäckerei"
    ui.text("Landbäckerei", logo_x + 1.0, logo_y + logo_h + 7.0, 7.0,
        Color::new(1.0, 0.85, 0.15, 0.8));

    // Header (rechts vom Logo)
    ui.text(&header_line(shop, game_secs), bx + 70.0, by + 14.0, 11.0,
        Color::new(1.0, 0.85, 0.15, 1.0));
    ui.text(
        FILIALEN[shop.filiale_idx].besonderheit,
        bx + 70.0,
        by + 28.0,
        9.0,
        Color::new(0.85, 0.85, 0.85, 1.0),
    );

    // Items
    let items = shop.visible_items();
    let mut y = by + 44.0;
    for (i, &idx) in items.iter().enumerate() {
        let it = &SHOP_ITEMS[idx];
        let price = shop.effective_price(idx, player, game_secs, purchases);
        let row_y = y;
        if i == shop.cursor {
            ui.rect(bx + 4.0, row_y - 2.0, bw - 8.0, 16.0, Color::new(0.4, 0.20, 0.10, 0.8));
            ui.text(">", bx + 6.0, row_y + 11.0, 12.0, Color::new(1.0, 0.85, 0.15, 1.0));
        }
        ui.text(it.name, bx + 18.0, row_y + 11.0, 11.0, WHITE);
        ui.text(it.effekt, bx + 130.0, row_y + 11.0, 9.0, Color::new(0.85, 0.85, 0.85, 1.0));
        let price_str = format!("{} M", price);
        let pw = ui.text_w(&price_str, 11.0);
        let coin_col = if player.coins >= price {
            Color::new(1.0, 0.85, 0.15, 1.0)
        } else {
            Color::new(0.6, 0.6, 0.6, 1.0)
        };
        ui.text(&price_str, bx + bw - pw - 8.0, row_y + 11.0, 11.0, coin_col);
        y += 16.0;
    }

    // Lore zum aktuellen Item
    if let Some(&idx) = items.get(shop.cursor) {
        let it = &SHOP_ITEMS[idx];
        ui.text(
            &format!("\"{}\"", it.lore),
            bx + 8.0,
            by + bh - 38.0,
            9.0,
            Color::new(0.85, 0.78, 0.65, 1.0),
        );
    }

    // Statuszeile
    let footer = "[W/S] wählen   [E] kaufen   [ESC] schließen";
    ui.text(footer, bx + 8.0, by + bh - 8.0, 9.0, Color::new(0.85, 0.85, 0.85, 1.0));

    // Münzen-Anzeige
    ui.texture(&tex.coin_gold, bx + bw - 78.0, by + 4.0, 14.0, 14.0, WHITE);
    ui.text(&format!("{}", player.coins), bx + bw - 62.0, by + 14.0, 11.0, Color::new(1.0, 0.85, 0.15, 1.0));

    // Pop-Message
    if !shop.message.is_empty() {
        let tw = ui.text_w(&shop.message, 11.0);
        ui.rect(
            VIRTUAL_W as f32 / 2.0 - tw / 2.0 - 6.0,
            by - 22.0,
            tw + 12.0,
            16.0,
            Color::new(0.0, 0.0, 0.0, 0.85),
        );
        ui.text(
            &shop.message,
            VIRTUAL_W as f32 / 2.0 - tw / 2.0,
            by - 10.0,
            11.0,
            Color::new(1.0, 0.85, 0.15, 1.0),
        );
    }
}

// ------------------------------------------------------------------------
//  Title / Menü / Game-Over / Victory
// ------------------------------------------------------------------------

pub fn draw_title_menu(ui: &UiCtx, cursor: usize, has_save: bool) {
    clear_background(Color::new(0.05, 0.04, 0.10, 1.0));
    let cx = VIRTUAL_W as f32 / 2.0;
    ui.rect(0.0, 60.0, VIRTUAL_W as f32, 60.0, Color::new(0.4, 0.08, 0.08, 1.0));

    let title = "GERMERING QUEST";
    let tw = ui.text_w(title, 30.0);
    ui.text(title, cx - tw / 2.0, 90.0, 30.0, Color::new(1.0, 0.85, 0.15, 1.0));

    let sub = "Die Schimmel-Invasion";
    let sw = ui.text_w(sub, 13.0);
    ui.text(sub, cx - sw / 2.0, 110.0, 13.0, WHITE);

    let items = ["Neues Spiel", "Spiel laden", "Credits", "Beenden"];
    let mut y = 150.0;
    for (i, it) in items.iter().enumerate() {
        let mut col = WHITE;
        let mut prefix = "  ";
        if i == 1 && !has_save {
            col = Color::new(0.5, 0.5, 0.5, 1.0);
        }
        if i == cursor {
            col = Color::new(1.0, 0.85, 0.15, 1.0);
            prefix = "> ";
        }
        let label = format!("{}{}", prefix, it);
        let lw = ui.text_w(&label, 14.0);
        ui.text(&label, cx - lw / 2.0, y, 14.0, col);
        y += 20.0;
    }
    let foot1 = "Servus, Max! Bereit, Germering zu retten?";
    let f1w = ui.text_w(foot1, 10.0);
    ui.text(foot1, cx - f1w / 2.0, 248.0, 10.0, Color::new(0.85, 0.85, 0.85, 1.0));

    let foot2 = "[W/S] wählen  [ENTER] OK  [ESC] Beenden";
    let f2w = ui.text_w(foot2, 9.0);
    ui.text(foot2, cx - f2w / 2.0, 262.0, 9.0, Color::new(0.7, 0.7, 0.7, 1.0));
}

pub fn draw_paused(ui: &UiCtx, cursor: usize) {
    ui.fullscreen_rect(Color::new(0.0, 0.0, 0.0, 0.75));
    let s = "PAUSE";
    let tw = ui.text_w(s, 28.0);
    ui.text(
        s,
        VIRTUAL_W as f32 / 2.0 - tw / 2.0,
        VIRTUAL_H as f32 / 2.0 - 50.0,
        28.0,
        Color::new(1.0, 0.85, 0.15, 1.0),
    );
    let items = ["Weiter", "Respawn am Checkpoint", "Speichern", "Hauptmenü"];
    let cx = VIRTUAL_W as f32 / 2.0;
    let mut y = VIRTUAL_H as f32 / 2.0 - 14.0;
    for (i, it) in items.iter().enumerate() {
        let (prefix, col) = if i == cursor {
            ("> ", Color::new(1.0, 0.85, 0.15, 1.0))
        } else {
            ("  ", WHITE)
        };
        let label = format!("{}{}", prefix, it);
        let lw = ui.text_w(&label, 13.0);
        ui.text(&label, cx - lw / 2.0, y, 13.0, col);
        y += 18.0;
    }
    let foot = "[W/S] wählen  [ENTER] OK  [P] schnell weiter";
    let fw = ui.text_w(foot, 9.0);
    ui.text(foot, cx - fw / 2.0, VIRTUAL_H as f32 - 18.0, 9.0,
        Color::new(0.7, 0.7, 0.7, 1.0));
}

pub fn draw_gameover(ui: &UiCtx) {
    ui.fullscreen_rect(Color::new(0.0, 0.0, 0.0, 0.85));
    let s = "I mog nimma...";
    let tw = ui.text_w(s, 22.0);
    ui.text(
        s,
        VIRTUAL_W as f32 / 2.0 - tw / 2.0,
        VIRTUAL_H as f32 / 2.0,
        22.0,
        Color::new(0.9, 0.3, 0.3, 1.0),
    );
    let s2 = "[ENTER] Respawn an letzter Filiale";
    let tw2 = ui.text_w(s2, 11.0);
    ui.text(
        s2,
        VIRTUAL_W as f32 / 2.0 - tw2 / 2.0,
        VIRTUAL_H as f32 / 2.0 + 22.0,
        11.0,
        WHITE,
    );
}

pub fn draw_victory(ui: &UiCtx, scroll: f32) {
    clear_background(Color::new(0.05, 0.04, 0.10, 1.0));
    let mut y = VIRTUAL_H as f32 - scroll;
    for line in VICTORY_TEXT.iter() {
        ui.text(line, 50.0, y, 12.0, Color::new(1.0, 0.85, 0.15, 1.0));
        y += 20.0;
    }
    y += 18.0;
    for line in CREDITS.iter() {
        ui.text(line, 50.0, y, 11.0, WHITE);
        y += 18.0;
    }
    ui.text(
        "[ENTER] Hauptmenü",
        150.0,
        VIRTUAL_H as f32 - 10.0,
        10.0,
        Color::new(0.7, 0.7, 0.7, 1.0),
    );
}

pub fn draw_intro(ui: &UiCtx, scroll: f32) {
    clear_background(Color::new(0.02, 0.02, 0.05, 1.0));
    let mut y = VIRTUAL_H as f32 - scroll;
    for line in INTRO_TEXT.iter() {
        ui.text(line, 30.0, y, 11.0, Color::new(0.9, 0.85, 0.7, 1.0));
        y += 20.0;
    }
    ui.text(
        "[ENTER] überspringen",
        150.0,
        VIRTUAL_H as f32 - 10.0,
        9.0,
        Color::new(0.6, 0.6, 0.6, 1.0),
    );
}

// ------------------------------------------------------------------------
//  Hint-Box (E zum Interagieren etc.)
// ------------------------------------------------------------------------

pub fn draw_hint(ui: &UiCtx, text: &str) {
    let tw = ui.text_w(text, 11.0);
    let bx = VIRTUAL_W as f32 / 2.0 - tw / 2.0 - 6.0;
    let by = VIRTUAL_H as f32 / 2.0 - 50.0;
    ui.rect(bx, by, tw + 12.0, 14.0, Color::new(0.0, 0.0, 0.0, 0.75));
    ui.text(text, bx + 6.0, by + 11.0, 11.0, Color::new(1.0, 0.85, 0.15, 1.0));
}

// ------------------------------------------------------------------------
//  Save-Hinweis (kurzes Toast oben/unten)
// ------------------------------------------------------------------------

pub fn draw_toast(ui: &UiCtx, text: &str, col: Color) {
    let tw = ui.text_w(text, 10.0);
    let bx = VIRTUAL_W as f32 / 2.0 - tw / 2.0 - 6.0;
    let by = VIRTUAL_H as f32 - 56.0;
    ui.rect(bx, by, tw + 12.0, 13.0, Color::new(0.0, 0.0, 0.0, 0.8));
    ui.text(text, bx + 6.0, by + 10.0, 10.0, col);
}

// ------------------------------------------------------------------------
//  Boss-HP-Balken
// ------------------------------------------------------------------------

pub fn draw_boss_bar(ui: &UiCtx, hp: i32, max: i32, taunt: &str, taunt_t: f32) {
    let bw = 240.0;
    let bx = (VIRTUAL_W as f32 - bw) / 2.0;
    let by = 16.0;
    let pct = (hp as f32 / max as f32).clamp(0.0, 1.0);
    ui.rect(bx - 2.0, by - 2.0, bw + 4.0, 12.0, Color::new(0.0, 0.0, 0.0, 0.8));
    ui.rect(bx, by, bw * pct, 8.0, Color::new(0.5, 0.08, 0.08, 1.0));
    let tw = ui.text_w(BOSS_NAME, 11.0);
    ui.text(BOSS_NAME, VIRTUAL_W as f32 / 2.0 - tw / 2.0, by - 4.0, 11.0, Color::new(1.0, 0.5, 0.5, 1.0));

    if taunt_t > 0.0 {
        let tw = ui.text_w(taunt, 12.0);
        ui.rect(
            VIRTUAL_W as f32 / 2.0 - tw / 2.0 - 6.0,
            36.0,
            tw + 12.0,
            16.0,
            Color::new(0.5, 0.08, 0.08, 0.9),
        );
        ui.text(taunt, VIRTUAL_W as f32 / 2.0 - tw / 2.0, 48.0, 12.0, WHITE);
    }
}
