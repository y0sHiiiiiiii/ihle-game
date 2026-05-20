//! Germering Quest — Die Schimmel-Invasion
//!
//! Hauptmodul: Fullscreen-Konfiguration, Viewport-Skalierung (480×270
//! virtuell, letterboxed), Game-State-Machine, Hauptschleife.

mod audio;
mod boss;
mod collision;
mod consts;
mod entities;
mod npc;
mod player;
mod save;
mod shop;
mod sprites;
mod ui;
mod world;

use audio::Audio;
use boss::{Boss, BossPhase};
use collision::Aabb;
use consts::*;
use entities::{
    spawn_world_entities, Coin, CoinKind, Enemy, EnemyKind, EnemyState, FloatingText, IcePatch,
    Projectile,
};
use macroquad::prelude::*;
use npc::{spawn_all_npcs, DialogState, Npc};
use player::{Facing, Player};
use save::SaveData;
use shop::{filiale_at, filiale_is_open, ShopState};
use ui::Textures;
use world::{time_of_day, GameCamera, TimeOfDay, World};

// ------------------------------------------------------------------------
//  Window-Konfiguration — Fullscreen 1920×1080
// ------------------------------------------------------------------------

fn window_conf() -> Conf {
    Conf {
        window_title: "Germering Quest".to_owned(),
        fullscreen: true,
        window_width: 1920,
        window_height: 1080,
        window_resizable: true,
        sample_count: 0,
        ..Default::default()
    }
}

// ------------------------------------------------------------------------
//  Game-State
// ------------------------------------------------------------------------

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum GameState {
    TitleMenu,
    Intro,
    Playing,
    Paused,
    Shopping,
    Dialog,
    GameOver,
    Victory,
    Credits,
}

struct Game {
    state: GameState,
    world: World,
    player: Player,
    coins: Vec<Coin>,
    enemies: Vec<Enemy>,
    projectiles: Vec<Projectile>,
    ice_patches: Vec<IcePatch>,
    slow_tiles: Vec<IcePatch>, // wir nutzen IcePatch-Struct generisch für Slow-Spuren
    npcs: Vec<Npc>,
    boss: Option<Boss>,
    cam: GameCamera,
    audio: Audio,
    tex: Textures,
    game_seconds: f32,
    coins_collected: u32,
    purchases: u32,
    has_genusspass: bool,
    cordobar_unlocked: bool,
    roman_artifact: bool,
    boss_defeated: bool,
    checkpoint_filiale: usize,
    minimap_open: bool,
    area_name: String,
    area_fade: f32,
    last_area: String,
    shop: Option<ShopState>,
    dialog: Option<DialogState>,
    menu_cursor: usize,
    intro_scroll: f32,
    victory_scroll: f32,
    attack_anim_t: f32,
    interact_landmark_cooldown: f32,
    boss_intro_seen: bool,
    germarbrunnen_active: f32, // Restzeit Münze/Sek
    germarbrunnen_acc: f32,
    save_message_t: f32,
    save_message: String,
    // --- Dungeon-Kristalle (Bitmask: bit0..3 für die 4 Kristalle) ---
    crystals: u8,
    // --- S-Bahn ---
    train_t: f32,       // Sekunden bis zur nächsten Zugdurchfahrt
    train_anim: f32,    // Animations-Fortschritt 0..TRAIN_DURATION (>0 = Zug fährt)
    train_dropped: bool,
    // --- WWK-Sprung ---
    jump_t: f32,        // Restzeit aktiver Sprung
    // --- See-Schwimmstrecke ---
    lake_visited: u8,   // Bitmask der besuchten Checkpoints
    lake_swim_done: bool,
    // --- Quest-Stage ---
    quest_stage: u8,
    quest_hint_t: f32,
    quest_hint: String,
    // --- Klaus-Tour ---
    klaus_tour_done: bool,
    klaus_tour_t: f32,
    // --- Floating Damage- / Coin-Texte ---
    floating_texts: Vec<FloatingText>,
}

// ------------------------------------------------------------------------
//  Build helpers
// ------------------------------------------------------------------------

async fn build_textures() -> Textures {
    Textures {
        heart_full: sprites::make_texture(&sprites::HEART_FULL),
        heart_empty: sprites::make_texture(&sprites::HEART_EMPTY),
        coin_copper: sprites::make_texture(&sprites::COIN_COPPER),
        coin_silver: sprites::make_texture(&sprites::COIN_SILVER),
        coin_gold: sprites::make_texture(&sprites::COIN_GOLD),
        coin_brezel: sprites::make_texture(&sprites::COIN_BREZEL),
        player_down_a: sprites::make_texture(&sprites::PLAYER_DOWN_A),
        player_down_b: sprites::make_texture(&sprites::PLAYER_DOWN_B),
        player_up: sprites::make_texture(&sprites::PLAYER_UP_A),
        player_up_b: sprites::make_texture(&sprites::PLAYER_UP_B),
        player_left: sprites::make_texture(&sprites::PLAYER_LEFT_A),
        player_left_b: sprites::make_texture(&sprites::PLAYER_LEFT_B),
        player_right: sprites::make_texture(&sprites::PLAYER_RIGHT_A),
        player_right_b: sprites::make_texture(&sprites::PLAYER_RIGHT_B),
        player_swim: sprites::make_texture(&sprites::PLAYER_SWIM),
        enemy_mold: sprites::make_texture(&sprites::ENEMY_MOLD),
        enemy_blob: sprites::make_texture(&sprites::ENEMY_BLOB),
        enemy_rat: sprites::make_texture(&sprites::ENEMY_RAT),
        enemy_beat: sprites::make_texture(&sprites::ENEMY_BEAT),
        enemy_ice: sprites::make_texture(&sprites::ENEMY_ICE),
        boss: sprites::make_texture(&sprites::BOSS_MODRY),
        projectile: sprites::make_texture(&sprites::PROJECTILE_SPORE),
        npc_ihle: sprites::make_texture(&sprites::NPC_IHLE),
        npc_klaus: sprites::make_texture(&sprites::NPC_KLAUS),
        npc_gerhard: sprites::make_texture(&sprites::NPC_GERHARD),
        npc_oma: sprites::make_texture(&sprites::NPC_OMA),
        npc_franz: sprites::make_texture(&sprites::NPC_FRANZ),
        tile_grass: sprites::make_texture(&sprites::TILE_GRASS),
        tile_dirt: sprites::make_texture(&sprites::TILE_DIRT),
        tile_road: sprites::make_texture(&sprites::TILE_ROAD),
        tile_sidewalk: sprites::make_texture(&sprites::TILE_SIDEWALK),
        tile_redthread: sprites::make_texture(&sprites::TILE_REDTHREAD),
        tile_water: sprites::make_texture(&sprites::TILE_WATER),
        tile_sand: sprites::make_texture(&sprites::TILE_SAND),
        tile_ice: sprites::make_texture(&sprites::TILE_ICE),
        tile_forest: sprites::make_texture(&sprites::TILE_FOREST),
        tile_building: sprites::make_texture(&sprites::TILE_BUILDING),
        tile_ihle: sprites::make_texture(&sprites::TILE_IHLE_WALL),
        tile_bossfloor: sprites::make_texture(&sprites::TILE_BOSSFLOOR),
        tile_moldslow: sprites::make_texture(&sprites::TILE_MOLD_SLOW),
        tile_brunnen: sprites::make_texture(&sprites::TILE_BRUNNEN),
        crystal: sprites::make_texture(&sprites::CRYSTAL),
    }
}

impl Game {
    fn new(tex: Textures, audio: Audio) -> Self {
        let world = World::new();
        let (coins, enemies) = spawn_world_entities(&world);
        let player = Player::new(100.0 * TILE_SIZE, 72.0 * TILE_SIZE);
        Self {
            state: GameState::TitleMenu,
            world,
            player,
            coins,
            enemies,
            projectiles: Vec::new(),
            ice_patches: Vec::new(),
            slow_tiles: Vec::new(),
            npcs: spawn_all_npcs(),
            boss: None,
            cam: GameCamera::new(),
            audio,
            tex,
            game_seconds: 0.0,
            coins_collected: 0,
            purchases: 0,
            has_genusspass: false,
            cordobar_unlocked: false,
            roman_artifact: false,
            boss_defeated: false,
            checkpoint_filiale: 0,
            minimap_open: false,
            area_name: String::new(),
            area_fade: 0.0,
            last_area: String::new(),
            shop: None,
            dialog: None,
            menu_cursor: 0,
            intro_scroll: 0.0,
            victory_scroll: 0.0,
            attack_anim_t: 0.0,
            interact_landmark_cooldown: 0.0,
            boss_intro_seen: false,
            germarbrunnen_active: 0.0,
            germarbrunnen_acc: 0.0,
            save_message_t: 0.0,
            save_message: String::new(),
            crystals: 0,
            train_t: TRAIN_INTERVAL_SECONDS,
            train_anim: 0.0,
            train_dropped: false,
            jump_t: 0.0,
            lake_visited: 0,
            lake_swim_done: false,
            quest_stage: 0,
            quest_hint_t: 0.0,
            quest_hint: String::new(),
            klaus_tour_done: false,
            klaus_tour_t: 0.0,
            floating_texts: Vec::new(),
        }
    }

    fn reset_for_new_game(&mut self) {
        let (coins, enemies) = spawn_world_entities(&self.world);
        self.coins = coins;
        self.enemies = enemies;
        self.projectiles.clear();
        self.ice_patches.clear();
        self.slow_tiles.clear();
        self.npcs = spawn_all_npcs();
        self.boss = None;
        self.player = Player::new(100.0 * TILE_SIZE, 72.0 * TILE_SIZE);
        self.game_seconds = 0.0;
        self.purchases = 0;
        self.cordobar_unlocked = false;
        self.roman_artifact = false;
        self.boss_defeated = false;
        self.checkpoint_filiale = 0;
        self.intro_scroll = 0.0;
        self.victory_scroll = 0.0;
        self.boss_intro_seen = false;
        self.crystals = 0;
        self.train_t = TRAIN_INTERVAL_SECONDS;
        self.train_anim = 0.0;
        self.train_dropped = false;
        self.jump_t = 0.0;
        self.lake_visited = 0;
        self.lake_swim_done = false;
        self.quest_stage = 0;
        self.quest_hint_t = 0.0;
        self.quest_hint.clear();
        self.klaus_tour_done = false;
        self.klaus_tour_t = 0.0;
        self.floating_texts.clear();
    }

    fn load_from_save(&mut self, d: SaveData) {
        self.player.coins = d.coins;
        self.player.hearts = d.hearts.max(1);
        self.player.max_hearts = d.max_hearts;
        self.player.aabb.x = d.player_x;
        self.player.aabb.y = d.player_y;
        self.purchases = d.purchases;
        self.checkpoint_filiale = d.checkpoint_filiale.min(FILIALEN.len() - 1);
        self.roman_artifact = d.roman_artifact;
        self.cordobar_unlocked = d.cordobar_unlocked;
        self.boss_defeated = d.boss_defeated;
        self.game_seconds = d.game_seconds;
        self.crystals = d.crystals;
        self.lake_swim_done = d.lake_swim_done;
        self.quest_stage = d.quest_stage;
        self.klaus_tour_done = d.klaus_tour_done;
        if self.roman_artifact {
            self.player.powerups.attack_bonus = 1;
        }
    }

    fn to_save(&self) -> SaveData {
        SaveData {
            coins: self.player.coins,
            hearts: self.player.hearts,
            max_hearts: self.player.max_hearts,
            player_x: self.player.aabb.x,
            player_y: self.player.aabb.y,
            purchases: self.purchases,
            checkpoint_filiale: self.checkpoint_filiale,
            roman_artifact: self.roman_artifact,
            cordobar_unlocked: self.cordobar_unlocked,
            boss_defeated: self.boss_defeated,
            game_seconds: self.game_seconds,
            crystals: self.crystals,
            lake_swim_done: self.lake_swim_done,
            quest_stage: self.quest_stage,
            klaus_tour_done: self.klaus_tour_done,
        }
    }

    fn respawn_at_checkpoint(&mut self) {
        let f = &FILIALEN[self.checkpoint_filiale];
        self.player.aabb.x = f.tile_x as f32 * TILE_SIZE;
        self.player.aabb.y = (f.tile_y + 3) as f32 * TILE_SIZE;
        self.player.hearts = self.player.max_hearts;
        self.player.powerups = Default::default();
        self.player.swim_air = 10.0;
        self.player.vx = 0.0;
        self.player.vy = 0.0;
        self.state = GameState::Playing;
    }

    // --------------------------------------------------------------
    //  Update
    // --------------------------------------------------------------

    fn update_playing(&mut self, dt: f32) {
        self.game_seconds += dt;

        // Pause
        if is_key_pressed(KeyCode::P) {
            self.state = GameState::Paused;
            return;
        }
        // Tab → Minimap
        if is_key_pressed(KeyCode::Tab) {
            self.minimap_open = !self.minimap_open;
        }
        // Manual save
        if is_key_pressed(KeyCode::F5) {
            save::save(&self.to_save());
            self.save_message = "Gespeichert (save.dat)".to_string();
            self.save_message_t = 2.0;
        }

        self.save_message_t = (self.save_message_t - dt).max(0.0);
        self.interact_landmark_cooldown = (self.interact_landmark_cooldown - dt).max(0.0);
        self.attack_anim_t = (self.attack_anim_t - dt).max(0.0);
        self.quest_hint_t = (self.quest_hint_t - dt).max(0.0);
        self.klaus_tour_t += dt;

        // S-Bahn-Zyklus
        self.tick_train(dt);

        // Klaus läuft den Roten Faden ab, bis ihn jemand anspricht.
        if !self.klaus_tour_done {
            let (kx, ky) = self.klaus_position();
            for n in self.npcs.iter_mut() {
                if n.kind == npc::NpcKind::Klaus {
                    n.aabb.x = kx;
                    n.aabb.y = ky;
                }
            }
        }

        // Germarbrunnen-Bonus
        if self.germarbrunnen_active > 0.0 {
            self.germarbrunnen_active -= dt;
            self.germarbrunnen_acc += dt;
            while self.germarbrunnen_acc >= 1.0 {
                self.germarbrunnen_acc -= 1.0;
                self.player.add_coins(1);
                self.audio.play_sfx(&self.audio.coin);
            }
        }

        // Music nach Tageszeit
        let tod = time_of_day(self.game_seconds);
        let in_boss = self.boss.as_ref().map(|b| b.alive()).unwrap_or(false);
        let in_cordobar = world::areas()
            .iter()
            .find(|a| a.name == AREA_CORDOBAR)
            .map(|a| a.contains_world(self.player.aabb.x, self.player.aabb.y))
            .unwrap_or(false);
        if in_boss {
            self.audio.play_music("boss", 0.45);
        } else if in_cordobar && self.cordobar_unlocked {
            self.audio.play_music("cordobar", 0.4);
        } else if tod == TimeOfDay::Nacht {
            self.audio.play_music("night", 0.3);
        } else {
            self.audio.play_music("day", 0.35);
        }

        // Player update
        self.player.update(&self.world, dt);
        let (pcx, pcy) = self.player.center();
        self.cam.follow(pcx, pcy);
        self.cam.update_shake(dt, self.game_seconds);

        // Floating-Texts tick
        self.floating_texts.retain_mut(|f| f.update(dt));

        // Area-Erkennung
        let tx = (pcx / TILE_SIZE) as i32;
        let ty = (pcy / TILE_SIZE) as i32;
        if let Some(a) = self.world.area_at(tx, ty) {
            if a.name != self.last_area {
                self.area_name = a.name.to_string();
                self.area_fade = 2.5;
                self.last_area = a.name.to_string();

                // Boss spawnen — nur wenn alle 4 Kristalle gesammelt!
                if a.name == AREA_KAUFHOF && self.boss.is_none() && !self.boss_defeated {
                    let collected = self.crystals.count_ones();
                    if collected >= 4 {
                        self.boss = Some(Boss::new(
                            100.0 * TILE_SIZE - 22.0,
                            95.0 * TILE_SIZE - 22.0,
                        ));
                    } else {
                        self.area_name = format!(
                            "Verriegelt! Du brauchst 4 Kristalle ({}/4)",
                            collected
                        );
                        self.area_fade = 3.5;
                    }
                }
            }
        }
        self.area_fade = (self.area_fade - dt * 0.5).max(0.0);

        // Coins (mit Magnet + Floating-Pickup-Text)
        for c in self.coins.iter_mut() {
            c.update(dt);
            // Magnet: Münzen in 32px-Umkreis fliegen zum Spieler
            if c.alive {
                let dx = pcx - (c.x + 8.0);
                let dy = pcy - (c.y + 8.0);
                let d2 = dx * dx + dy * dy;
                if d2 < 32.0 * 32.0 && d2 > 8.0 * 8.0 {
                    let d = d2.sqrt().max(0.001);
                    let pull = 220.0;
                    c.x += (dx / d) * pull * dt;
                    c.y += (dy / d) * pull * dt;
                }
            }
            if c.alive && self.player.aabb.intersects(&c.aabb()) {
                let v = c.kind.value();
                let final_v = if self.player.powerups.double_coin > 0.0 { v * 2 } else { v };
                self.player.add_coins(v);
                self.coins_collected += v;
                c.alive = false;
                c.respawn_t = COIN_RESPAWN_SECONDS;
                self.audio.play_sfx(&self.audio.coin);
                self.floating_texts.push(FloatingText::coins(c.x + 4.0, c.y, final_v));
            }
        }

        // Enemies
        let mut new_ice_spawns: Vec<(f32, f32, f32)> = Vec::new();
        let mut enemy_drops: Vec<(f32, f32, CoinKind, i32)> = Vec::new();
        for e in self.enemies.iter_mut() {
            e.update(&self.world, &self.player, dt, &mut new_ice_spawns);
            if e.alive() && self.player.aabb.intersects(&e.aabb) {
                if self.player.powerups.invuln <= 0.0 && self.player.damage_flash <= 0.0 {
                    let dmg = e.damage();
                    self.player.take_damage(dmg);
                    self.audio.play_sfx(&self.audio.damage);
                    self.cam.add_shake(3.5);
                    self.floating_texts.push(FloatingText::damage(
                        self.player.aabb.x + 4.0,
                        self.player.aabb.y,
                        dmg,
                    ));
                }
            }
        }
        for (x, y, l) in new_ice_spawns {
            self.ice_patches.push(IcePatch::new(x, y, l));
        }

        // WWK-Sprung: in der WWK-Hochhaussiedlung wird Leertaste zum Sprung.
        let in_wwk = world::areas()
            .iter()
            .find(|a| a.name == AREA_WWK)
            .map(|a| a.contains_world(self.player.aabb.x, self.player.aabb.y))
            .unwrap_or(false);
        self.jump_t = (self.jump_t - dt).max(0.0);
        if in_wwk && is_key_pressed(KeyCode::Space) && self.jump_t <= 0.0 {
            self.jump_t = 0.35;
            // Kurzer Flugmodus überspringt Gebäude-Kollision während des Sprungs.
            self.player.powerups.fly = self.player.powerups.fly.max(0.35);
            let (vx, vy) = match self.player.facing {
                Facing::Down => (0.0, 240.0),
                Facing::Up => (0.0, -240.0),
                Facing::Left => (-240.0, 0.0),
                Facing::Right => (240.0, 0.0),
            };
            self.player.vx = vx;
            self.player.vy = vy;
            self.audio.play_sfx(&self.audio.menu_select);
        }

        // Player-Angriff (Leertaste, außer in WWK = Sprung)
        if !in_wwk && is_key_pressed(KeyCode::Space) && self.player.attack_cooldown <= 0.0 {
            self.player.attack_cooldown = 0.32;
            self.attack_anim_t = 0.22;
            // Forward-Lunge — kurzer Schub in Blickrichtung
            let lunge = 180.0;
            let (lx, ly) = match self.player.facing {
                Facing::Down => (0.0, lunge),
                Facing::Up => (0.0, -lunge),
                Facing::Left => (-lunge, 0.0),
                Facing::Right => (lunge, 0.0),
            };
            self.player.vx = lx;
            self.player.vy = ly;
            self.audio.play_sfx(&self.audio.menu_select);
            // Hitbox vor dem Spieler
            let (cx, cy) = self.player.center();
            let (hx, hy) = match self.player.facing {
                Facing::Down => (cx - 12.0, cy + 6.0),
                Facing::Up => (cx - 12.0, cy - 22.0),
                Facing::Left => (cx - 22.0, cy - 12.0),
                Facing::Right => (cx + 6.0, cy - 12.0),
            };
            let hit = Aabb::new(hx, hy, 24.0, 24.0);
            let damage = 1 + self.player.powerups.attack_bonus;
            // Treffer auf Gegner
            for e in self.enemies.iter_mut() {
                if e.alive() && e.aabb.intersects(&hit) {
                    e.hurt(damage);
                    e.apply_knockback(cx, cy, 220.0);
                    self.audio.play_sfx(&self.audio.enemy_kill);
                    self.floating_texts.push(FloatingText::damage(
                        e.aabb.x + 4.0,
                        e.aabb.y,
                        damage,
                    ));
                    self.cam.add_shake(2.0);
                    if !e.alive() {
                        // Drops queuen
                        for (k, n) in e.drop() {
                            for i in 0..n {
                                enemy_drops.push((e.aabb.x + i as f32 * 4.0, e.aabb.y, k, i));
                            }
                        }
                    }
                }
            }
            // Treffer auf Boss
            if let Some(b) = self.boss.as_mut() {
                if b.hurt(damage, hit) {
                    self.audio.play_sfx(&self.audio.boss_hit);
                    self.floating_texts.push(FloatingText::damage(
                        b.aabb.x + 22.0,
                        b.aabb.y,
                        damage,
                    ));
                    self.cam.add_shake(3.5);
                }
            }
            // Treffer auf Projektile (Sporen können zerstört werden)
            self.projectiles.retain(|p| !hit.intersects(&p.aabb()));
        }

        // Drops zu Münzen
        for (x, y, k, _) in enemy_drops {
            let mut c = Coin::new(x, y, k);
            c.respawn_t = 99999.0; // verschwinden nach Aufnahme — kein Respawn
            c.spawn_x = x;
            c.spawn_y = y;
            self.coins.push(c);
        }

        // Tote Gegner entfernen
        self.enemies.retain(|e| {
            // Wir entfernen sie verzögert nicht — sie respawnen einfach nicht.
            // Aus der Liste werfen, sobald Dead.
            e.state != EnemyState::Dead
        });

        // Projektile
        let mut player_hit = false;
        self.projectiles.retain_mut(|p| {
            let alive = p.update(dt);
            if alive && self.player.aabb.intersects(&p.aabb()) {
                player_hit = true;
                return false;
            }
            alive
        });
        if player_hit {
            self.player.take_damage(2);
            self.audio.play_sfx(&self.audio.damage);
            self.cam.add_shake(3.0);
            self.floating_texts.push(FloatingText::damage(
                self.player.aabb.x + 4.0,
                self.player.aabb.y,
                2,
            ));
        }

        // Boss
        let mut new_slow_spawns: Vec<(f32, f32, f32)> = Vec::new();
        let mut boss_died = false;
        if let Some(b) = self.boss.as_mut() {
            b.update(pcx, pcy, dt, &mut self.projectiles, &mut new_slow_spawns);
            // Berührungs-Schaden vom Boss
            if b.alive() && b.aabb.intersects(&self.player.aabb)
                && self.player.damage_flash <= 0.0
                && self.player.powerups.invuln <= 0.0
            {
                self.player.take_damage(3);
                self.audio.play_sfx(&self.audio.damage);
                self.cam.add_shake(5.0);
                self.floating_texts.push(FloatingText::damage(
                    self.player.aabb.x + 4.0,
                    self.player.aabb.y,
                    3,
                ));
            }
            if !b.alive() && b.phase == BossPhase::Dead {
                boss_died = true;
            }
        }
        for (x, y, l) in new_slow_spawns {
            self.slow_tiles.push(IcePatch::new(x, y, l));
        }

        if boss_died {
            self.boss_defeated = true;
            self.advance_quest_if(3, 4);
            self.state = GameState::Victory;
            self.victory_scroll = 0.0;
            save::save(&self.to_save());
        }

        // Eisflecken & Slow-Tiles
        self.ice_patches.retain_mut(|p| {
            let alive = p.update(dt);
            // Effekt auf Spieler: setze ihn auf Ice
            if alive && self.player.aabb.intersects(&p.aabb()) {
                self.player.on_ice = true;
            }
            alive
        });
        self.slow_tiles.retain_mut(|p| {
            let alive = p.update(dt);
            if alive && self.player.aabb.intersects(&p.aabb()) {
                self.player.on_slow = true;
            }
            alive
        });

        // NPCs — Interaktion
        let mut start_dialog: Option<usize> = None;
        for (i, npc) in self.npcs.iter().enumerate() {
            if self.player.aabb.intersects(&npc.interact_zone()) && is_key_pressed(KeyCode::E) {
                start_dialog = Some(i);
                break;
            }
        }
        if let Some(i) = start_dialog {
            // Meister-Ihle-Dialog je nach Quest-Stage dynamisch setzen.
            if self.npcs[i].kind == npc::NpcKind::MeisterIhle {
                self.npcs[i].dialog = Self::meister_dialog_for_stage(self.quest_stage);
            }
            // Klaus zu sprechen bringt Quest 0 → 1.
            if self.npcs[i].kind == npc::NpcKind::Klaus {
                self.advance_quest_if(0, 1);
                self.klaus_tour_done = true; // Tour endet, sobald man mit ihm spricht
            }
            self.dialog = Some(DialogState::new(i));
            self.state = GameState::Dialog;
            return;
        }

        // Ihle-Filiale betreten (E) — Öffnungszeiten beachten
        if let Some(idx) = filiale_at(self.player.aabb.x, self.player.aabb.y) {
            if is_key_pressed(KeyCode::E) {
                if filiale_is_open(idx, self.game_seconds) {
                    self.checkpoint_filiale = idx;
                    self.shop = Some(ShopState::new(idx));
                    self.audio.play_sfx(&self.audio.jingle);
                    self.state = GameState::Shopping;
                    return;
                } else {
                    let (open, close) = FILIALE_OPEN_HOURS[idx];
                    self.area_name = format!(
                        "Geschlossen! Öffnet {:02}:00 - {:02}:00 Uhr",
                        open, close
                    );
                    self.area_fade = 2.5;
                }
            }
        }

        // Landmark-Interaktion (E auf Brunnen-Tile in Nähe)
        if self.interact_landmark_cooldown <= 0.0 && is_key_pressed(KeyCode::E) {
            let landmarks: &[((i32, i32), &str)] = &[
                ((100, 60), "germar"),
                ((115, 35), "jakobus"),
                ((85, 65), "marien"),
                ((8, 50), "ziegel"),
                ((20, 105), "cordobar"),
            ];
            for &((lx, ly), id) in landmarks {
                let dx = pcx - lx as f32 * TILE_SIZE - 8.0;
                let dy = pcy - ly as f32 * TILE_SIZE - 8.0;
                if dx * dx + dy * dy < 28.0 * 28.0 {
                    self.interact_landmark(id);
                    self.interact_landmark_cooldown = 1.0;
                    break;
                }
            }
        }

        // Museum betreten (Punktposition)
        let museum_dx = pcx - 115.0 * TILE_SIZE;
        let museum_dy = pcy - 58.0 * TILE_SIZE;
        if !self.roman_artifact
            && museum_dx * museum_dx + museum_dy * museum_dy < 24.0 * 24.0
            && is_key_pressed(KeyCode::E)
        {
            self.roman_artifact = true;
            self.player.powerups.attack_bonus = 1;
            self.area_name = "Römisches Artefakt gefunden! +1 Angriff dauerhaft".to_string();
            self.area_fade = 3.0;
            self.audio.play_sfx(&self.audio.jingle);
            self.advance_quest_if(1, 2);
        }

        // Dungeon-Kristalle einsammeln (Berührung — kein E nötig)
        for (i, &(cx_t, cy_t, name)) in CRYSTALS.iter().enumerate() {
            let bit = 1u8 << i;
            if (self.crystals & bit) != 0 {
                continue;
            }
            let cx_w = cx_t as f32 * TILE_SIZE + 8.0;
            let cy_w = cy_t as f32 * TILE_SIZE + 8.0;
            let dx = pcx - cx_w;
            let dy = pcy - cy_w;
            if dx * dx + dy * dy < 16.0 * 16.0 {
                self.crystals |= bit;
                self.player.add_coins(COIN_BREZEL); // jeder Kristall gibt eine Brezel-Münze
                self.area_name = format!("{} eingesammelt! ({}/4)", name, self.crystals.count_ones());
                self.area_fade = 3.0;
                self.audio.play_sfx(&self.audio.jingle);
                if self.crystals.count_ones() >= 4 {
                    self.advance_quest_if(2, 3);
                }
            }
        }

        // See-Schwimmstrecke: Checkpoints im Wasser
        if self.player.on_water {
            for (i, &(cx_t, cy_t)) in LAKE_CHECKPOINTS.iter().enumerate() {
                let bit = 1u8 << i;
                if (self.lake_visited & bit) != 0 {
                    continue;
                }
                let cx_w = cx_t as f32 * TILE_SIZE + 8.0;
                let cy_w = cy_t as f32 * TILE_SIZE + 8.0;
                let dx = pcx - cx_w;
                let dy = pcy - cy_w;
                if dx * dx + dy * dy < 24.0 * 24.0 {
                    self.lake_visited |= bit;
                    self.area_name = format!(
                        "See-Checkpoint {}/4!",
                        self.lake_visited.count_ones()
                    );
                    self.area_fade = 2.0;
                    self.audio.play_sfx(&self.audio.coin);
                }
            }
            if !self.lake_swim_done && self.lake_visited == 0b1111 {
                self.lake_swim_done = true;
                self.player.add_coins(COIN_GOLD * 3);
                self.area_name = "See umrundet! +60 Goldmünzen!".to_string();
                self.area_fade = 3.5;
                self.audio.play_sfx(&self.audio.jingle);
            }
        }

        // Game Over
        if !self.player.alive() {
            self.state = GameState::GameOver;
            self.audio.play_sfx(&self.audio.damage);
        }
    }

    /// S-Bahn-Timer + Münzregen am Bahnhof.
    fn tick_train(&mut self, dt: f32) {
        if self.train_anim > 0.0 {
            self.train_anim += dt;
            // Münzen droppen, sobald der Zug die Mitte erreicht.
            if !self.train_dropped && self.train_anim >= TRAIN_DURATION * 0.45 {
                self.train_dropped = true;
                let platform_y = TRAIN_PLATFORM_TILE_Y as f32 * TILE_SIZE;
                // 10 Silbermünzen entlang des Bahnsteigs verteilen.
                for i in 0..10 {
                    let x_tile = 96 + i * 3;
                    let x = x_tile as f32 * TILE_SIZE;
                    let mut c = Coin::new(x, platform_y, CoinKind::Silver);
                    c.respawn_t = 99999.0; // Einmal-Münzen
                    self.coins.push(c);
                }
                self.audio.play_sfx(&self.audio.coin);
            }
            if self.train_anim >= TRAIN_DURATION {
                self.train_anim = 0.0;
                self.train_dropped = false;
                self.train_t = TRAIN_INTERVAL_SECONDS;
            }
        } else {
            self.train_t -= dt;
            if self.train_t <= 0.0 {
                self.train_anim = 0.001; // Start
                self.train_dropped = false;
            }
        }
    }

    /// Klaus läuft beim ersten Mal entlang des Roten Fadens, bis er angesprochen wurde.
    fn klaus_position(&self) -> (f32, f32) {
        // Statische Pose nach abgeschlossener Tour.
        if self.klaus_tour_done {
            return (101.0 * TILE_SIZE, 62.0 * TILE_SIZE);
        }
        // Pfad-Wegpunkte (Tile-Koord), Klaus läuft sie langsam ab.
        let path: &[(f32, f32)] = &[
            (101.0, 62.0),
            (95.0, 60.0),
            (80.0, 60.0),
            (65.0, 60.0),
            (55.0, 60.0),
        ];
        let speed_seg = 12.0; // Sekunden pro Segment
        let t = self.klaus_tour_t / speed_seg;
        let seg = (t as usize).min(path.len() - 2);
        let local = (t - seg as f32).clamp(0.0, 1.0);
        let (x0, y0) = path[seg];
        let (x1, y1) = path[seg + 1];
        let x = x0 + (x1 - x0) * local;
        let y = y0 + (y1 - y0) * local;
        (x * TILE_SIZE, y * TILE_SIZE)
    }

    /// Bringt die Quest weiter, wenn aktuell `from` aktiv ist.
    fn advance_quest_if(&mut self, from: u8, to: u8) {
        if self.quest_stage == from {
            self.quest_stage = to;
            self.quest_hint = match to {
                1 => "Quest: Hol das römische Artefakt aus dem Museum.".to_string(),
                2 => "Quest: Sammle die 4 Dungeon-Kristalle.".to_string(),
                3 => "Quest: Bezwinge Schimmelmeister Modrý im alten Kaufhof!".to_string(),
                4 => "Quest abgeschlossen - Germering ist gerettet!".to_string(),
                _ => String::new(),
            };
            self.quest_hint_t = 4.0;
        }
    }

    /// Liefert Meister Ihles Dialog je nach Quest-Stage.
    fn meister_dialog_for_stage(stage: u8) -> Vec<&'static str> {
        match stage {
            0 => vec![
                "Servus Max! Schee dass du da bist.",
                "Der Goldene Brezel-Schlüssel - weg!",
                "Schimmelmeister Modrý hat ihn geklaut.",
                "Rede zuerst mit Stadtführer Klaus.",
                "Er steht am Germarbrunnen, gleich nördlich.",
            ],
            1 => vec![
                "Klaus hat dir alles erklärt? Sehr gut.",
                "Geh ins Stadtmuseum ZEIT+RAUM.",
                "Dort liegt ein römisches Artefakt.",
                "Es macht dich stärker im Kampf!",
            ],
            2 => vec![
                "Mit dem Artefakt: +1 Angriff für immer.",
                "Jetzt brauchst du die 4 Kristalle.",
                "Polariom, Forst, Cordobar, Parsberg.",
                "Erst dann öffnet sich der alte Kaufhof.",
                "Kauf dir vorher Brot - du wirst Kraft brauchen!",
            ],
            3 => vec![
                "Alle 4 Kristalle! Du bist bereit.",
                "Der alte Kaufhof am Stadtplatz wartet.",
                "Modrý sitzt im Keller. Pass auf seine Phasen auf.",
                "Hol uns die Brezel zurück. Pfiati!",
            ],
            _ => vec![
                "Servus Max - du hast Germering gerettet!",
                "Die Brezel ist wieder bei uns.",
                "Komm immer wieder vorbei, ja?",
            ],
        }
    }

    fn interact_landmark(&mut self, id: &str) {
        match id {
            "germar" => {
                self.germarbrunnen_active = 30.0;
                self.area_name = "Germarbrunnen aktiv: +1 Münze/Sek (30s)".to_string();
                self.area_fade = 3.0;
                self.audio.play_sfx(&self.audio.coin);
            }
            "jakobus" => {
                self.player.heal_full();
                self.area_name = "Jakobusbrunnen heilt dich vollständig!".to_string();
                self.area_fade = 3.0;
                self.audio.play_sfx(&self.audio.jingle);
            }
            "marien" => {
                self.player.powerups.invuln = self.player.powerups.invuln.max(20.0);
                self.area_name = "Mariensäule: 20s Unverwundbarkeit".to_string();
                self.area_fade = 3.0;
                self.audio.play_sfx(&self.audio.jingle);
            }
            "ziegel" => {
                self.player.add_coins(50);
                self.area_name = "Geheimgang offen: +50 Münzen!".to_string();
                self.area_fade = 3.0;
                self.audio.play_sfx(&self.audio.coin);
            }
            "cordobar" => {
                self.cordobar_unlocked = true;
                self.area_name = "Cordobar: 'Wellenreiten'-Chiptune freigeschaltet!".to_string();
                self.area_fade = 3.0;
                self.audio.play_sfx(&self.audio.jingle);
            }
            _ => {}
        }
    }

    fn update_shopping(&mut self, dt: f32) {
        if let Some(shop) = self.shop.as_mut() {
            shop.tick(dt);
            let items = shop.visible_items();
            if is_key_pressed(KeyCode::W) || is_key_pressed(KeyCode::Up) {
                if shop.cursor > 0 {
                    shop.cursor -= 1;
                }
                self.audio.play_sfx(&self.audio.menu_select);
            }
            if is_key_pressed(KeyCode::S) || is_key_pressed(KeyCode::Down) {
                if shop.cursor + 1 < items.len() {
                    shop.cursor += 1;
                }
                self.audio.play_sfx(&self.audio.menu_select);
            }
            if is_key_pressed(KeyCode::E) || is_key_pressed(KeyCode::Enter) {
                let ok = shop.try_buy(&mut self.player, self.game_seconds, &mut self.purchases);
                if ok {
                    self.audio.play_sfx(&self.audio.buy);
                    save::save(&self.to_save());
                }
            }
            if is_key_pressed(KeyCode::Escape) || is_key_pressed(KeyCode::Q) {
                self.shop = None;
                self.state = GameState::Playing;
            }
        }
    }

    fn update_dialog(&mut self, dt: f32) {
        let total_lines;
        let line_len;
        if let Some(d) = self.dialog.as_ref() {
            let npc = &self.npcs[d.npc_idx];
            total_lines = npc.dialog.len();
            line_len = npc.dialog.get(d.line).map(|s| s.chars().count()).unwrap_or(0);
        } else {
            self.state = GameState::Playing;
            return;
        }
        if let Some(d) = self.dialog.as_mut() {
            d.tick(dt, line_len);
            if is_key_pressed(KeyCode::E) || is_key_pressed(KeyCode::Enter) {
                if d.advance(total_lines) {
                    self.dialog = None;
                    self.state = GameState::Playing;
                }
            }
            if is_key_pressed(KeyCode::Escape) {
                self.dialog = None;
                self.state = GameState::Playing;
            }
        }
    }

    fn update_paused(&mut self) {
        if is_key_pressed(KeyCode::P) {
            self.state = GameState::Playing;
        }
        if is_key_pressed(KeyCode::Escape) {
            // Speichern & zum Hauptmenü
            save::save(&self.to_save());
            self.state = GameState::TitleMenu;
            self.menu_cursor = 0;
        }
    }

    fn update_gameover(&mut self) {
        if is_key_pressed(KeyCode::Enter) {
            self.respawn_at_checkpoint();
        }
        if is_key_pressed(KeyCode::Escape) {
            self.state = GameState::TitleMenu;
        }
    }

    fn update_victory(&mut self, dt: f32) {
        self.victory_scroll += dt * 22.0;
        if is_key_pressed(KeyCode::Enter) || is_key_pressed(KeyCode::Escape) {
            self.state = GameState::TitleMenu;
        }
    }

    fn update_intro(&mut self, dt: f32) {
        self.intro_scroll += dt * 18.0;
        let mut start = false;
        if is_key_pressed(KeyCode::Enter) || is_key_pressed(KeyCode::Space) {
            start = true;
        }
        // Auto-Übergang nach genug Scroll
        if self.intro_scroll > 240.0 + 22.0 * INTRO_TEXT.len() as f32 {
            start = true;
        }
        if start {
            self.state = GameState::Playing;
            // Initialer Quest-Hinweis sobald die Story losgeht.
            if self.quest_stage == 0 {
                self.quest_hint = "Quest: Sprich mit Stadtführer Klaus am Germarbrunnen.".to_string();
                self.quest_hint_t = 6.0;
            }
        }
    }

    fn update_title(&mut self, has_save: bool) {
        if is_key_pressed(KeyCode::W) || is_key_pressed(KeyCode::Up) {
            if self.menu_cursor > 0 {
                self.menu_cursor -= 1;
            }
            self.audio.play_sfx(&self.audio.menu_select);
        }
        if is_key_pressed(KeyCode::S) || is_key_pressed(KeyCode::Down) {
            if self.menu_cursor < 3 {
                self.menu_cursor += 1;
            }
            self.audio.play_sfx(&self.audio.menu_select);
        }
        if is_key_pressed(KeyCode::Enter) {
            match self.menu_cursor {
                0 => {
                    self.reset_for_new_game();
                    self.state = GameState::Intro;
                }
                1 => {
                    if has_save {
                        if let Some(d) = save::load() {
                            self.load_from_save(d);
                            self.state = GameState::Playing;
                        }
                    }
                }
                2 => {
                    self.state = GameState::Credits;
                    self.victory_scroll = 0.0;
                }
                3 => {
                    std::process::exit(0);
                }
                _ => {}
            }
        }
    }

    fn update_credits(&mut self, dt: f32) {
        self.victory_scroll += dt * 22.0;
        if is_key_pressed(KeyCode::Enter) || is_key_pressed(KeyCode::Escape) {
            self.state = GameState::TitleMenu;
        }
    }

    // --------------------------------------------------------------
    //  Render
    // --------------------------------------------------------------

    fn draw_world(&self) {
        // Sichtbarer Tile-Bereich aus Kamera
        let cam_x = self.cam.x;
        let cam_y = self.cam.y;
        let tx0 = (cam_x / TILE_SIZE) as i32 - 1;
        let ty0 = (cam_y / TILE_SIZE) as i32 - 1;
        let tx1 = ((cam_x + VIRTUAL_W as f32) / TILE_SIZE) as i32 + 1;
        let ty1 = ((cam_y + VIRTUAL_H as f32) / TILE_SIZE) as i32 + 1;

        for y in ty0.max(0)..ty1.min(self.world.h) {
            for x in tx0.max(0)..tx1.min(self.world.w) {
                let t = self.world.get(x, y);
                let (sx, sy) = self.cam.world_to_screen(x as f32 * TILE_SIZE, y as f32 * TILE_SIZE);
                draw_texture_ex(
                    self.tex.tile_texture(t),
                    sx.floor(),
                    sy.floor(),
                    WHITE,
                    DrawTextureParams {
                        dest_size: Some(vec2(TILE_SIZE, TILE_SIZE)),
                        ..Default::default()
                    },
                );
            }
        }

        // Slow-Spuren (vom Boss)
        for s in &self.slow_tiles {
            let (sx, sy) = self.cam.world_to_screen(s.x, s.y);
            draw_rectangle(sx, sy, 16.0, 16.0, Color::new(0.2, 0.4, 0.2, 0.7));
        }
        // Eis-Flecken
        for s in &self.ice_patches {
            let (sx, sy) = self.cam.world_to_screen(s.x, s.y);
            draw_rectangle(sx, sy, 16.0, 16.0, Color::new(0.8, 0.95, 1.0, 0.6));
        }
    }

    fn draw_entities(&self) {
        // Dungeon-Kristalle (die noch nicht gesammelten)
        for (i, &(cx_t, cy_t, _)) in CRYSTALS.iter().enumerate() {
            if (self.crystals & (1u8 << i)) != 0 {
                continue;
            }
            let (sx, sy) = self.cam.world_to_screen(
                cx_t as f32 * TILE_SIZE,
                cy_t as f32 * TILE_SIZE,
            );
            // sanftes Schweben & Glow
            let bob = (self.game_seconds * 2.5 + i as f32 * 1.7).sin() * 1.5;
            draw_circle(sx + 8.0, sy + 9.0, 9.0, Color::new(0.65, 0.30, 0.85, 0.25));
            draw_texture_ex(
                &self.tex.crystal,
                sx,
                sy + bob,
                WHITE,
                DrawTextureParams {
                    dest_size: Some(vec2(16.0, 16.0)),
                    ..Default::default()
                },
            );
        }

        // S-Bahn (prozedural — 3 rot-weiße Wagen, gleiten von links nach rechts)
        if self.train_anim > 0.0 {
            let progress = (self.train_anim / TRAIN_DURATION).clamp(0.0, 1.0);
            // Zug startet links außerhalb der Bahnhof-Area und verlässt sie rechts.
            let x0 = 92.0 * TILE_SIZE - 96.0; // Start (links)
            let x1 = 132.0 * TILE_SIZE + 16.0; // Ende (rechts)
            let train_world_x = x0 + (x1 - x0) * progress;
            let train_world_y = TRAIN_TRACK_TILE_Y as f32 * TILE_SIZE;
            let (tsx, tsy) = self.cam.world_to_screen(train_world_x, train_world_y);
            let car_w = 26.0;
            let car_h = 12.0;
            let gap = 2.0;
            // Lok (vorne, dunkler)
            draw_rectangle(tsx, tsy + 1.0, car_w, car_h, Color::new(0.6, 0.10, 0.10, 1.0));
            draw_rectangle(tsx + 2.0, tsy + 3.0, car_w - 4.0, 3.0, Color::new(0.95, 0.95, 0.95, 1.0));
            draw_rectangle(tsx + 2.0, tsy + 8.0, 5.0, 3.0, Color::new(0.18, 0.18, 0.20, 1.0));
            // 2 Anhängerwagen
            for i in 1..=2 {
                let cx = tsx - i as f32 * (car_w + gap);
                draw_rectangle(cx, tsy + 1.0, car_w, car_h, Color::new(0.85, 0.18, 0.18, 1.0));
                draw_rectangle(cx + 2.0, tsy + 3.0, car_w - 4.0, 2.0, Color::new(0.98, 0.97, 0.94, 1.0));
                // Fenster
                draw_rectangle(cx + 3.0, tsy + 6.0, 4.0, 3.0, Color::new(0.20, 0.30, 0.45, 1.0));
                draw_rectangle(cx + 10.0, tsy + 6.0, 4.0, 3.0, Color::new(0.20, 0.30, 0.45, 1.0));
                draw_rectangle(cx + 17.0, tsy + 6.0, 4.0, 3.0, Color::new(0.20, 0.30, 0.45, 1.0));
            }
            // Räder
            for i in 0..3 {
                let cx = tsx - i as f32 * (car_w + gap);
                draw_circle(cx + 5.0, tsy + car_h + 1.0, 2.0, BLACK);
                draw_circle(cx + car_w - 5.0, tsy + car_h + 1.0, 2.0, BLACK);
            }
        }

        // Münzen
        for c in &self.coins {
            if !c.alive {
                continue;
            }
            let (sx, sy) = self.cam.world_to_screen(c.x, c.y + c.bob_y());
            let tex = match c.kind {
                CoinKind::Copper => &self.tex.coin_copper,
                CoinKind::Silver => &self.tex.coin_silver,
                CoinKind::Gold => &self.tex.coin_gold,
                CoinKind::Brezel => &self.tex.coin_brezel,
            };
            // Doppelmünzen-Glow
            if self.player.powerups.double_coin > 0.0 {
                draw_circle(sx + 8.0, sy + 8.0, 7.0, Color::new(1.0, 0.85, 0.15, 0.3));
            }
            draw_texture_ex(
                tex,
                sx,
                sy,
                WHITE,
                DrawTextureParams {
                    dest_size: Some(vec2(16.0, 16.0)),
                    ..Default::default()
                },
            );
        }

        // NPCs (mit Idle-Bobbing, jede Person mit eigener Phase)
        for npc in &self.npcs {
            // Phase aus der Spawn-Position ableiten → jeder NPC wippt anders.
            let phase = npc.aabb.x * 0.13 + npc.aabb.y * 0.07;
            let bob = (self.game_seconds * 2.6 + phase).sin() * 0.6;
            let (sx, sy) = self.cam.world_to_screen(npc.aabb.x - 1.0, npc.aabb.y - 1.0 + bob);
            let t = match npc.kind {
                npc::NpcKind::MeisterIhle => &self.tex.npc_ihle,
                npc::NpcKind::Klaus => &self.tex.npc_klaus,
                npc::NpcKind::Gerhard => &self.tex.npc_gerhard,
                npc::NpcKind::OmaLiesl => &self.tex.npc_oma,
                npc::NpcKind::Franz => &self.tex.npc_franz,
            };
            draw_texture_ex(
                t,
                sx,
                sy,
                WHITE,
                DrawTextureParams {
                    dest_size: Some(vec2(16.0, 16.0)),
                    ..Default::default()
                },
            );
        }

        // Gegner
        for e in &self.enemies {
            if !e.alive() {
                continue;
            }
            let (sx, sy) = self.cam.world_to_screen(e.aabb.x - 1.0, e.aabb.y - 1.0);
            let t = match e.kind {
                EnemyKind::Mold => &self.tex.enemy_mold,
                EnemyKind::Blob => &self.tex.enemy_blob,
                EnemyKind::Rat => &self.tex.enemy_rat,
                EnemyKind::Beat => &self.tex.enemy_beat,
                EnemyKind::Ice => &self.tex.enemy_ice,
            };
            let tint = if e.hit_flash > 0.0 {
                Color::new(1.0, 0.5, 0.5, 1.0)
            } else {
                WHITE
            };
            draw_texture_ex(
                t,
                sx,
                sy,
                tint,
                DrawTextureParams {
                    dest_size: Some(vec2(16.0, 16.0)),
                    ..Default::default()
                },
            );
        }

        // Projektile
        for p in &self.projectiles {
            let (sx, sy) = self.cam.world_to_screen(p.x, p.y);
            draw_texture_ex(
                &self.tex.projectile,
                sx,
                sy,
                WHITE,
                DrawTextureParams {
                    dest_size: Some(vec2(16.0, 16.0)),
                    ..Default::default()
                },
            );
        }

        // Boss
        if let Some(b) = &self.boss {
            if b.alive() {
                let (sx, sy) = self.cam.world_to_screen(b.aabb.x, b.aabb.y);
                let mut tint = if b.hit_flash > 0.0 {
                    Color::new(1.0, 0.4, 0.4, 1.0)
                } else {
                    WHITE
                };
                // Phase 2: echter Boss blinkt schneller
                if b.phase == BossPhase::P2 {
                    if (b.anim_t * 8.0).sin() > 0.0 {
                        tint.a = 0.7;
                    }
                }
                draw_texture_ex(
                    &self.tex.boss,
                    sx,
                    sy,
                    tint,
                    DrawTextureParams {
                        dest_size: Some(vec2(48.0, 48.0)),
                        ..Default::default()
                    },
                );
                // Klone in Phase 2
                for c in &b.clones {
                    let (sx, sy) = self.cam.world_to_screen(c.aabb.x, c.aabb.y);
                    draw_texture_ex(
                        &self.tex.boss,
                        sx,
                        sy,
                        Color::new(0.6, 0.6, 0.7, 0.7),
                        DrawTextureParams {
                            dest_size: Some(vec2(48.0, 48.0)),
                            ..Default::default()
                        },
                    );
                }
            }
        }

        // Player
        let (psx, psy) = self.cam.world_to_screen(self.player.aabb.x - 2.0, self.player.aabb.y - 2.0);
        let tex = if self.player.on_water && self.player.powerups.fly <= 0.0 {
            &self.tex.player_swim
        } else {
            // Animation nur wenn der Spieler sich tatsächlich bewegt.
            let moving = self.player.vx.abs() > 6.0 || self.player.vy.abs() > 6.0;
            let alt = moving && ((self.player.anim_t * 8.0) as i32 % 2 == 1);
            match self.player.facing {
                Facing::Down => if alt { &self.tex.player_down_b } else { &self.tex.player_down_a },
                Facing::Up => if alt { &self.tex.player_up_b } else { &self.tex.player_up },
                Facing::Left => if alt { &self.tex.player_left_b } else { &self.tex.player_left },
                Facing::Right => if alt { &self.tex.player_right_b } else { &self.tex.player_right },
            }
        };
        let mut tint = WHITE;
        if self.player.powerups.invuln > 0.0 {
            // golden blinkend
            let t = (self.player.anim_t * 12.0).sin();
            if t > 0.0 {
                tint = Color::new(1.0, 0.85, 0.15, 1.0);
            }
        }
        if self.player.damage_flash > 0.0 {
            tint = Color::new(1.0, 0.5, 0.5, 1.0);
        }
        draw_texture_ex(
            tex,
            psx,
            psy,
            tint,
            DrawTextureParams {
                dest_size: Some(vec2(16.0, 16.0)),
                ..Default::default()
            },
        );

        // Powerup-Effekte
        if self.player.powerups.speed > 0.0 {
            draw_circle(psx + 8.0, psy + 14.0, 4.0, Color::new(0.9, 0.55, 0.20, 0.4));
        }
        if self.player.powerups.fly > 0.0 {
            // Flügel
            draw_circle(psx + 2.0, psy + 10.0, 3.0, Color::new(1.0, 1.0, 1.0, 0.8));
            draw_circle(psx + 14.0, psy + 10.0, 3.0, Color::new(1.0, 1.0, 1.0, 0.8));
        }
        if self.player.powerups.shield {
            draw_circle_lines(psx + 8.0, psy + 8.0, 10.0, 1.0, Color::new(0.85, 0.65, 0.20, 0.8));
        }

        // Angriffs-Animation: expandierender Slash-Bogen + Inner Flash
        if self.attack_anim_t > 0.0 {
            const ATTACK_TOTAL: f32 = 0.22;
            let p = ((ATTACK_TOTAL - self.attack_anim_t) / ATTACK_TOTAL).clamp(0.0, 1.0);
            let (cx, cy) = self.player.center();
            let (sx, sy) = self.cam.world_to_screen(cx, cy);
            let dir_angle = match self.player.facing {
                Facing::Down => std::f32::consts::FRAC_PI_2,
                Facing::Up => -std::f32::consts::FRAC_PI_2,
                Facing::Left => std::f32::consts::PI,
                Facing::Right => 0.0,
            };
            // 3 expanding Bögen mit Versatz (sieht aus wie eine wirbelnde Klinge)
            for ring in 0..3 {
                let phase = (p * 1.4 - ring as f32 * 0.18).clamp(0.0, 1.0);
                if phase <= 0.0 || phase >= 1.0 {
                    continue;
                }
                let radius = 6.0 + phase * 22.0;
                let alpha = (1.0 - phase) * 0.85;
                let col = Color::new(1.0, 0.95 - phase * 0.35, 0.40, alpha);
                let span = 1.6 - ring as f32 * 0.25;
                let start = dir_angle - span / 2.0;
                let steps = 16;
                for s in 0..steps {
                    let a0 = start + (s as f32 / steps as f32) * span;
                    let a1 = start + ((s + 1) as f32 / steps as f32) * span;
                    let x0 = sx + a0.cos() * radius;
                    let y0 = sy + a0.sin() * radius;
                    let x1 = sx + a1.cos() * radius;
                    let y1 = sy + a1.sin() * radius;
                    draw_line(x0, y0, x1, y1, 2.0, col);
                }
            }
            // Inner-Flash am Anfang
            if self.attack_anim_t > ATTACK_TOTAL * 0.55 {
                let flash_a = (self.attack_anim_t / ATTACK_TOTAL) * 0.8;
                draw_circle(sx, sy, 6.0, Color::new(1.0, 1.0, 0.85, flash_a));
            }
            // Speed-Lines hinter dem Spieler in Sprungrichtung (Lunge-Feel)
            if p < 0.5 {
                let trail_alpha = (1.0 - p * 2.0) * 0.6;
                let trail_len = 14.0;
                let (lx, ly) = match self.player.facing {
                    Facing::Down => (0.0, -trail_len),
                    Facing::Up => (0.0, trail_len),
                    Facing::Left => (trail_len, 0.0),
                    Facing::Right => (-trail_len, 0.0),
                };
                for i in 0..3 {
                    let off = (i as f32 - 1.0) * 3.0;
                    let (ox, oy) = if matches!(self.player.facing, Facing::Left | Facing::Right) {
                        (0.0, off)
                    } else {
                        (off, 0.0)
                    };
                    draw_line(
                        sx + ox,
                        sy + oy,
                        sx + ox + lx,
                        sy + oy + ly,
                        1.0,
                        Color::new(1.0, 0.95, 0.55, trail_alpha),
                    );
                }
            }
        }

        // Floating Damage / Coin Texte
        for f in &self.floating_texts {
            let (sx, sy) = self.cam.world_to_screen(f.x, f.y);
            let mut col = f.color;
            col.a *= f.alpha();
            // dunkler Schatten für Lesbarkeit
            let mut shadow = Color::new(0.0, 0.0, 0.0, 0.7 * f.alpha());
            shadow.a = shadow.a.min(0.7);
            draw_text(&f.text, sx + 1.0, sy + 1.0, 12.0, shadow);
            draw_text(&f.text, sx, sy, 12.0, col);
        }
    }

    fn draw_tint(&self) {
        let tod = time_of_day(self.game_seconds);
        let tint = tod.tint();
        if tint.a > 0.0 {
            draw_rectangle(0.0, 0.0, VIRTUAL_W as f32, VIRTUAL_H as f32, tint);
        }

        // Fenster-Licht bei Nacht
        if tod == TimeOfDay::Nacht {
            // Filiale-Schaufenster leuchten (geschlossen aber Licht an)
            for f in &FILIALEN {
                let (sx, sy) = self.cam.world_to_screen(
                    f.tile_x as f32 * TILE_SIZE - 16.0,
                    f.tile_y as f32 * TILE_SIZE - 4.0,
                );
                if sx > -32.0 && sx < VIRTUAL_W as f32 && sy > -32.0 && sy < VIRTUAL_H as f32 {
                    draw_rectangle(sx + 4.0, sy + 4.0, 24.0, 10.0, Color::new(1.0, 0.95, 0.55, 0.4));
                }
            }
        }
    }

    fn draw_interact_hints(&self, ui: &ui::UiCtx) {
        // Priorität: NPC > Filiale > Landmark > Museum — nur EIN Hint gleichzeitig.
        for npc in &self.npcs {
            if self.player.aabb.intersects(&npc.interact_zone()) {
                ui::draw_hint(ui, &format!("[E] mit {} sprechen", npc.name));
                return;
            }
        }
        if let Some(idx) = filiale_at(self.player.aabb.x, self.player.aabb.y) {
            let f = &FILIALEN[idx];
            ui::draw_hint(ui, &format!("[E] {} - {}", f.name, f.adresse));
            return;
        }
        let (pcx, pcy) = self.player.center();
        let landmarks: &[((i32, i32), &str)] = &[
            ((100, 60), "Germarbrunnen"),
            ((115, 35), "Jakobusbrunnen"),
            ((85, 65), "Mariensäule"),
            ((8, 50), "Römischer Ziegelbrennofen"),
            ((20, 105), "Cordobar-Ruine"),
        ];
        for &((lx, ly), name) in landmarks {
            let dx = pcx - lx as f32 * TILE_SIZE - 8.0;
            let dy = pcy - ly as f32 * TILE_SIZE - 8.0;
            if dx * dx + dy * dy < 28.0 * 28.0 {
                ui::draw_hint(ui, &format!("[E] {}", name));
                return;
            }
        }
        if !self.roman_artifact {
            let mdx = pcx - 115.0 * TILE_SIZE;
            let mdy = pcy - 58.0 * TILE_SIZE;
            if mdx * mdx + mdy * mdy < 24.0 * 24.0 {
                ui::draw_hint(ui, "[E] Stadtmuseum ZEIT+RAUM betreten");
            }
        }
    }

    /// Sind wir in einem State, der die Pixel-Welt rendert?
    fn has_world(&self) -> bool {
        !matches!(
            self.state,
            GameState::TitleMenu | GameState::Intro | GameState::Victory | GameState::Credits
        )
    }

    /// Render-Pass auf die virtuelle 480×270-Canvas — nur Pixel-Welt.
    fn draw_world_pass(&self) {
        clear_background(Color::new(0.05, 0.05, 0.08, 1.0));
        if self.has_world() {
            self.draw_world();
            self.draw_decorations();
            self.draw_entities();
            self.draw_tint();
        }
    }

    /// Welt-Dekorationen: Ihle-Logos + Café-Sitzbereich, reale Germering-Landmarks.
    fn draw_decorations(&self) {
        // --- Ihle-Filialen: Logo-Schild + Café-Tische + Stühle ---
        for f in FILIALEN.iter() {
            let bx = f.tile_x as f32 * TILE_SIZE;
            let by = f.tile_y as f32 * TILE_SIZE;

            // Logo-Schild über dem Gebäude (gelb mit roter "IHLE"-Schrift)
            let sign_w = 44.0;
            let sign_h = 13.0;
            let sign_world_x = bx - sign_w / 2.0 + 8.0;
            let sign_world_y = by - 23.0;
            let (sx, sy) = self.cam.world_to_screen(sign_world_x, sign_world_y);
            // Befestigungs-Stäbe (vom Schild zur Gebäudewand)
            draw_rectangle(
                sx + 4.0,
                sy + sign_h,
                1.5,
                7.0,
                Color::new(0.30, 0.30, 0.30, 1.0),
            );
            draw_rectangle(
                sx + sign_w - 5.5,
                sy + sign_h,
                1.5,
                7.0,
                Color::new(0.30, 0.30, 0.30, 1.0),
            );
            // Schild-Hintergrund + Border + Highlight
            draw_rectangle(sx, sy, sign_w, sign_h, Color::new(0.55, 0.12, 0.12, 1.0));
            draw_rectangle(sx + 1.0, sy + 1.0, sign_w - 2.0, sign_h - 2.0,
                Color::new(1.0, 0.85, 0.15, 1.0));
            draw_rectangle(sx + 1.0, sy + 1.0, sign_w - 2.0, 2.0,
                Color::new(1.0, 0.95, 0.55, 1.0)); // Glanzlinie oben
            // "IHLE"-Text mittig
            draw_text("IHLE", sx + 10.0, sy + 10.0, 12.0, Color::new(0.55, 0.12, 0.12, 1.0));

            // Café-Sitzbereich südlich der Tür (door = (tile_x, tile_y+2))
            let cafe_y = by + 56.0; // ~3.5 Kacheln unter Gebäudemitte
            let table_color = Color::new(0.55, 0.36, 0.20, 1.0);
            let table_top = Color::new(0.78, 0.55, 0.30, 1.0);
            let chair_color = Color::new(0.40, 0.26, 0.14, 1.0);
            let cloth_color = Color::new(0.95, 0.92, 0.85, 0.85);

            for &dx_tile in &[-22.0_f32, 20.0_f32] {
                let (tx, ty) = self.cam.world_to_screen(bx + dx_tile, cafe_y);
                // Stühle (4 um den Tisch, leicht versetzt für 3/4-Perspektive)
                draw_circle(tx - 8.0, ty - 1.0, 2.5, chair_color);
                draw_circle(tx + 8.0, ty - 1.0, 2.5, chair_color);
                draw_circle(tx - 1.0, ty - 8.0, 2.5, chair_color);
                draw_circle(tx - 1.0, ty + 6.0, 2.5, chair_color);
                // Tisch (Schatten + Beine + Platte + Tischtuch)
                draw_circle(tx, ty + 2.0, 5.5, Color::new(0.0, 0.0, 0.0, 0.3));
                draw_circle(tx, ty, 5.0, table_color);
                draw_circle(tx, ty - 0.5, 4.0, table_top);
                draw_circle(tx, ty - 1.0, 3.0, cloth_color);
                // Sonnenschirm-Andeutung als Kreis darüber (nur Hauptfiliale + GEP)
                if f.nummer <= 2 {
                    draw_circle(tx, ty - 3.0, 6.0, Color::new(0.85, 0.18, 0.18, 0.55));
                    draw_rectangle(tx - 0.5, ty - 3.0, 1.0, 4.0,
                        Color::new(0.3, 0.2, 0.1, 0.8));
                }
            }
        }

        // --- St. Jakobskirche ---
        // Über dem Kirchen-Building bei (113, 30, 5, 5) zeichnen wir den Turm + Kreuz.
        let (kx, ky) = self.cam.world_to_screen(115.0 * TILE_SIZE, 30.0 * TILE_SIZE);
        // Hauptturm (Glockenturm)
        draw_rectangle(kx - 6.0, ky - 24.0, 12.0, 24.0, Color::new(0.78, 0.74, 0.66, 1.0));
        draw_rectangle_lines(kx - 6.0, ky - 24.0, 12.0, 24.0, 1.0,
            Color::new(0.40, 0.36, 0.28, 1.0));
        // Spitzdach (Pyramide aus Dreieck)
        draw_triangle(
            macroquad::math::vec2(kx - 7.0, ky - 24.0),
            macroquad::math::vec2(kx + 7.0, ky - 24.0),
            macroquad::math::vec2(kx, ky - 38.0),
            Color::new(0.35, 0.18, 0.10, 1.0),
        );
        // Kreuz auf der Spitze
        draw_rectangle(kx - 0.5, ky - 46.0, 1.0, 9.0, Color::new(1.0, 0.85, 0.15, 1.0));
        draw_rectangle(kx - 2.5, ky - 42.0, 5.0, 1.0, Color::new(1.0, 0.85, 0.15, 1.0));
        // Glockenfenster (zwei Rundbögen)
        draw_rectangle(kx - 4.0, ky - 18.0, 3.0, 5.0, Color::new(0.18, 0.18, 0.30, 1.0));
        draw_rectangle(kx + 1.0, ky - 18.0, 3.0, 5.0, Color::new(0.18, 0.18, 0.30, 1.0));
        // Buntglasfenster unten
        draw_rectangle(kx - 3.0, ky - 8.0, 2.0, 5.0, Color::new(0.40, 0.20, 0.55, 1.0));
        draw_rectangle(kx + 1.0, ky - 8.0, 2.0, 5.0, Color::new(0.40, 0.20, 0.55, 1.0));
        // Schild
        draw_rectangle(kx - 22.0, ky + 8.0, 44.0, 9.0, Color::new(0.18, 0.18, 0.22, 0.9));
        draw_text("ST. JAKOBSKIRCHE", kx - 20.0, ky + 15.0, 8.0,
            Color::new(0.95, 0.95, 0.95, 1.0));

        // --- Stadthalle Banner ---
        let (sx, sy) = self.cam.world_to_screen(77.0 * TILE_SIZE, 49.0 * TILE_SIZE);
        draw_rectangle(sx, sy, 90.0, 14.0, Color::new(0.16, 0.24, 0.50, 1.0));
        draw_rectangle(sx + 1.0, sy + 1.0, 90.0 - 2.0, 12.0,
            Color::new(0.22, 0.32, 0.60, 1.0));
        draw_text("STADTHALLE GERMERING", sx + 4.0, sy + 11.0, 9.0,
            Color::new(0.95, 0.95, 0.95, 1.0));

        // --- Stadtmuseum ZEIT+RAUM Banner ---
        let (mx, my) = self.cam.world_to_screen(108.0 * TILE_SIZE, 49.0 * TILE_SIZE);
        draw_rectangle(mx, my, 76.0, 14.0, Color::new(0.40, 0.22, 0.10, 1.0));
        draw_rectangle(mx + 1.0, my + 1.0, 74.0, 12.0,
            Color::new(0.55, 0.32, 0.16, 1.0));
        draw_text("ZEIT+RAUM", mx + 14.0, my + 11.0, 10.0,
            Color::new(1.0, 0.88, 0.30, 1.0));

        // --- Polariom Eishalle ---
        let (px, py) = self.cam.world_to_screen(160.0 * TILE_SIZE, 75.0 * TILE_SIZE);
        draw_rectangle(px, py, 96.0, 16.0, Color::new(0.20, 0.45, 0.70, 1.0));
        draw_rectangle(px + 1.0, py + 1.0, 94.0, 14.0,
            Color::new(0.45, 0.75, 0.92, 1.0));
        draw_text("POLARIOM EISHALLE", px + 10.0, py + 12.0, 9.0,
            Color::new(0.10, 0.20, 0.40, 1.0));

        // --- Bahnhof S8-Schild ---
        let (bsx, bsy) = self.cam.world_to_screen(96.0 * TILE_SIZE, 13.0 * TILE_SIZE);
        draw_rectangle(bsx, bsy, 38.0, 14.0, Color::new(0.06, 0.30, 0.10, 1.0));
        draw_rectangle(bsx + 1.0, bsy + 1.0, 36.0, 12.0,
            Color::new(0.12, 0.55, 0.20, 1.0));
        draw_text("S8 BAHNHOF", bsx + 2.0, bsy + 11.0, 8.0,
            Color::new(1.0, 1.0, 1.0, 1.0));
    }

    /// Render-Pass auf den echten Bildschirm — gestochen scharfer UI-Text.
    fn draw_ui_pass(&self, ui: &ui::UiCtx) {
        match self.state {
            GameState::TitleMenu => {
                ui::draw_title_menu(ui, self.menu_cursor, save::exists());
            }
            GameState::Intro => {
                ui::draw_intro(ui, self.intro_scroll);
            }
            GameState::Credits | GameState::Victory => {
                ui::draw_victory(ui, self.victory_scroll);
            }
            _ => {
                if let Some(b) = &self.boss {
                    if b.alive() {
                        ui::draw_boss_bar(ui, b.hp, BOSS_HP_MAX, b.current_taunt, b.taunt_t);
                    }
                }
                ui::draw_hud(
                    ui,
                    &self.tex,
                    &self.player,
                    self.game_seconds,
                    &self.area_name,
                    self.area_fade,
                    self.cordobar_unlocked,
                    self.purchases,
                    self.crystals,
                    &self.quest_hint,
                    self.quest_hint_t,
                );
                if matches!(self.state, GameState::Playing) {
                    self.draw_interact_hints(ui);
                    if !self.save_message.is_empty() && self.save_message_t > 0.0 {
                        ui::draw_toast(ui, &self.save_message, Color::new(0.5, 1.0, 0.5, 1.0));
                    }
                }
                if self.minimap_open {
                    ui::draw_minimap(ui, &self.world, &self.player);
                }
                if matches!(self.state, GameState::Paused) {
                    ui::draw_paused(ui);
                }
                if matches!(self.state, GameState::Shopping) {
                    if let Some(s) = &self.shop {
                        ui::draw_shop(ui, &self.tex, s, &self.player, self.game_seconds, self.purchases);
                    }
                }
                if matches!(self.state, GameState::Dialog) {
                    if let Some(d) = &self.dialog {
                        ui::draw_dialog(ui, &self.tex, &self.npcs[d.npc_idx], d);
                    }
                }
                if matches!(self.state, GameState::GameOver) {
                    ui::draw_gameover(ui);
                }
            }
        }
    }
}

// ------------------------------------------------------------------------
//  Hauptschleife
// ------------------------------------------------------------------------

#[macroquad::main(window_conf)]
async fn main() {
    // Texturen
    let tex = build_textures().await;
    // Audio
    let audio = Audio::load().await;

    // Render-Target für virtuelle 480×270-Canvas
    let target = render_target(VIRTUAL_W as u32, VIRTUAL_H as u32);
    target.texture.set_filter(FilterMode::Nearest);

    // Kamera, die in den Render-Target zeichnet
    let mut virt_camera = Camera2D::from_display_rect(Rect::new(
        0.0,
        0.0,
        VIRTUAL_W as f32,
        VIRTUAL_H as f32,
    ));
    virt_camera.render_target = Some(target.clone());

    let mut game = Game::new(tex, audio);

    // 60-FPS-Cap: schlafen bis das Ziel-Frame-Zeit-Budget aufgebraucht ist.
    let target_frame_secs: f32 = 1.0 / 60.0;

    loop {
        let frame_start = std::time::Instant::now();
        let dt = get_frame_time().min(0.05);

        // ESC schließt nur im Hauptmenü oder Pause
        if is_key_pressed(KeyCode::Escape) {
            match game.state {
                GameState::TitleMenu => {
                    std::process::exit(0);
                }
                GameState::Playing => {
                    game.state = GameState::Paused;
                }
                _ => {}
            }
        }

        // State-Update
        match game.state {
            GameState::TitleMenu => game.update_title(save::exists()),
            GameState::Intro => game.update_intro(dt),
            GameState::Playing => game.update_playing(dt),
            GameState::Paused => game.update_paused(),
            GameState::Shopping => game.update_shopping(dt),
            GameState::Dialog => game.update_dialog(dt),
            GameState::GameOver => game.update_gameover(),
            GameState::Victory => game.update_victory(dt),
            GameState::Credits => game.update_credits(dt),
        }

        // === Pass 1: Pixel-Welt auf virtuelle 480×270-Canvas ===
        set_camera(&virt_camera);
        game.draw_world_pass();

        // === Pass 2: Bildschirm-Space — UI direkt in nativer Auflösung ===
        set_default_camera();
        clear_background(BLACK);

        let aspect = VIRTUAL_W as f32 / VIRTUAL_H as f32;
        let sw = screen_width();
        let sh = screen_height();
        let (w, h) = if sw / sh > aspect {
            (sh * aspect, sh)
        } else {
            (sw, sw / aspect)
        };
        let dx = (sw - w) / 2.0;
        let dy = (sh - h) / 2.0;

        // Pixel-Welt skaliert hochziehen (Nearest-Filter sorgt für scharfe Pixel)
        if game.has_world() {
            draw_texture_ex(
                &target.texture,
                dx,
                dy,
                WHITE,
                DrawTextureParams {
                    dest_size: Some(vec2(w, h)),
                    flip_y: true,
                    ..Default::default()
                },
            );
        }

        // UI in nativer Auflösung darüber — Text wird hier scharf gerastert
        let ui_ctx = ui::UiCtx::new();
        game.draw_ui_pass(&ui_ctx);

        // FPS-Cap: warte bis das 60-Hz-Slot vorbei ist (CPU/GPU-schonend).
        let elapsed = frame_start.elapsed().as_secs_f32();
        if elapsed < target_frame_secs {
            std::thread::sleep(std::time::Duration::from_secs_f32(target_frame_secs - elapsed));
        }

        next_frame().await
    }
}
