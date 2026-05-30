//! Germering Delivery — Game-Modul-Wurzel

use bevy::prelude::*;

pub mod assets;
pub mod audio;
pub mod delivery;
pub mod fx;
pub mod gamestate;
pub mod highscore;
pub mod hud;
pub mod map;
pub mod navi;
pub mod npc;
pub mod player;
pub mod shop;
pub mod speech;

pub use gamestate::GameState;

pub struct GamePlugin;

impl Plugin for GamePlugin {
    fn build(&self, app: &mut App) {
        app.init_state::<GameState>()
            .add_plugins((
                assets::AssetsPlugin,
                audio::AudioPlugin,
                gamestate::GameStatePlugin,
                map::MapPlugin,
                player::PlayerPlugin,
                delivery::DeliveryPlugin,
                fx::FxPlugin,
                navi::NaviPlugin,
                npc::NpcPlugin,
                shop::ShopPlugin,
                speech::SpeechPlugin,
                highscore::HighscorePlugin,
                hud::HudPlugin,
            ));
    }
}
