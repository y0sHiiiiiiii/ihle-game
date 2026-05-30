//! Navi-Text: Richtungs-Hinweis, Entfernung, BOOST-Anzeige.
//!
//! Die Navi-Anzeige ist jetzt Teil des kombinierten Minimap-Panels in `hud.rs`.
//! Diese Datei kuemmert sich nur noch um die Aktualisierung der Text-Inhalte.

use bevy::prelude::*;

use crate::game::assets::Rgba;
use crate::game::delivery::{ActiveDelivery, DeliveryPhase};
use crate::game::gamestate::GameState;
use crate::game::map::{GameMap, TileType, TILE_SIZE};
use crate::game::player::Player;

#[derive(Component)]
pub struct NaviDirectionText;

#[derive(Component)]
pub struct NaviDistanceText;

#[derive(Component)]
pub struct NaviBoostText;

pub struct NaviPlugin;

impl Plugin for NaviPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            Update,
            update_navi_text.run_if(in_state(GameState::Playing)),
        );
    }
}

pub fn tile_color_for_minimap(t: TileType) -> Rgba {
    match t {
        TileType::Grass | TileType::Park => (78, 138, 65, 255),
        TileType::Garden => (95, 160, 80, 255),
        TileType::Sidewalk => (175, 175, 180, 255),
        TileType::Cobble => (158, 150, 140, 255),
        TileType::Platform => (198, 198, 202, 255),
        TileType::Road
        | TileType::RoadH
        | TileType::RoadV
        | TileType::Crosswalk
        | TileType::Parking => (55, 55, 58, 255),
        TileType::Water => (60, 110, 200, 255),
        TileType::House | TileType::Roof => (180, 130, 80, 255),
        TileType::Hedge => (52, 110, 48, 255),
        TileType::IhleStore => (208, 110, 55, 255),
        TileType::JannickStore => (210, 35, 35, 255),
        TileType::Rathaus => (200, 170, 120, 255),
        TileType::Church => (196, 190, 176, 255),
        TileType::Bahnhof => (160, 130, 90, 255),
        TileType::Rails => (90, 80, 65, 255),
        TileType::Aldi => (40, 80, 180, 255),
        TileType::Rewe => (230, 60, 60, 255),
        TileType::Tree => (40, 110, 35, 255),
    }
}

fn update_navi_text(
    active: Option<Res<ActiveDelivery>>,
    player_q: Query<(&Transform, &Player)>,
    mut dir_q: Query<
        &mut Text,
        (
            With<NaviDirectionText>,
            Without<NaviDistanceText>,
            Without<NaviBoostText>,
        ),
    >,
    mut dist_q: Query<
        &mut Text,
        (
            With<NaviDistanceText>,
            Without<NaviDirectionText>,
            Without<NaviBoostText>,
        ),
    >,
    mut boost_q: Query<
        (&mut Text, &mut Style),
        (
            With<NaviBoostText>,
            Without<NaviDirectionText>,
            Without<NaviDistanceText>,
        ),
    >,
) {
    let Ok((tr, player)) = player_q.get_single() else {
        return;
    };
    let Some(delivery) = active else {
        if let Ok(mut text) = dir_q.get_single_mut() {
            text.sections[0].value = "-".to_string();
        }
        if let Ok(mut text) = dist_q.get_single_mut() {
            text.sections[0].value = "".to_string();
        }
        if let Ok((_, mut style)) = boost_q.get_single_mut() {
            style.display = Display::None;
        }
        return;
    };

    let target_tile = match delivery.phase {
        DeliveryPhase::GoToPickup => delivery.pickup.interact_tile,
        DeliveryPhase::GoToDropoff => delivery.dropoff.tile,
    };
    let target_world = GameMap::tile_to_world(target_tile);
    let player_pos = tr.translation.truncate();

    let to_target = target_world - player_pos;
    let dist_world = to_target.length();
    let dist_tiles = (dist_world / TILE_SIZE).round() as i32;
    let dist_meters = dist_tiles * 10;

    let target_angle = to_target.y.atan2(to_target.x);
    let mut rel = target_angle - player.facing;
    while rel > std::f32::consts::PI {
        rel -= std::f32::consts::TAU;
    }
    while rel < -std::f32::consts::PI {
        rel += std::f32::consts::TAU;
    }

    let label = if dist_world < 18.0 {
        "AM ZIEL"
    } else if rel.abs() < 0.39 {
        "-> GERADEAUS"
    } else if rel.abs() > std::f32::consts::PI - 0.39 {
        "<- WENDEN"
    } else if rel > 1.18 {
        "^ LINKS ABBIEGEN"
    } else if rel < -1.18 {
        "v RECHTS ABBIEGEN"
    } else if rel > 0.0 {
        "/ HALB LINKS"
    } else {
        "\\ HALB RECHTS"
    };

    if let Ok(mut text) = dir_q.get_single_mut() {
        text.sections[0].value = label.to_string();
    }
    if let Ok(mut text) = dist_q.get_single_mut() {
        let phase = match delivery.phase {
            DeliveryPhase::GoToPickup => format!("Abholen: {}", delivery.pickup.name),
            DeliveryPhase::GoToDropoff => format!("Liefern: {}", delivery.dropoff.name),
        };
        text.sections[0].value = format!("{}\n{}m", phase, dist_meters.max(0));
    }

    if let Ok((mut text, mut style)) = boost_q.get_single_mut() {
        if player.nitro_timer > 0.0 {
            style.display = Display::Flex;
            text.sections[0].value = format!("NITRO!  ({:.1}s)", player.nitro_timer.max(0.0));
        } else if player.speed_boost_timer > 0.0 {
            style.display = Display::Flex;
            text.sections[0].value =
                format!("BOOST AKTIV  ({:.1}s)", player.speed_boost_timer.max(0.0));
        } else {
            style.display = Display::None;
        }
    }
}
