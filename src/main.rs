//! Germering Delivery — Ihle Sprinter Lieferfahrer Highscore Game

#![allow(
    clippy::type_complexity,
    clippy::too_many_arguments,
    clippy::field_reassign_with_default
)]

use std::time::{Duration, Instant};

use bevy::prelude::*;
use bevy::window::{PresentMode, WindowMode, WindowResolution};

mod game;

use game::GamePlugin;

const TARGET_FPS: f64 = 60.0;

#[derive(Resource)]
struct FpsLimiter {
    target_dt: Duration,
    last: Option<Instant>,
}

impl Default for FpsLimiter {
    fn default() -> Self {
        Self {
            target_dt: Duration::from_secs_f64(1.0 / TARGET_FPS),
            last: None,
        }
    }
}

fn limit_fps(mut limiter: ResMut<FpsLimiter>) {
    let now = Instant::now();
    if let Some(last) = limiter.last {
        let elapsed = now.duration_since(last);
        if elapsed < limiter.target_dt {
            std::thread::sleep(limiter.target_dt - elapsed);
        }
    }
    limiter.last = Some(Instant::now());
}

fn main() {
    App::new()
        .add_plugins(
            DefaultPlugins
                .set(WindowPlugin {
                    primary_window: Some(Window {
                        title: "Germering Delivery - Ihle Sprinter Highscore".into(),
                        resolution: WindowResolution::new(1280.0, 720.0),
                        present_mode: PresentMode::AutoVsync,
                        mode: WindowMode::BorderlessFullscreen,
                        resizable: true,
                        ..default()
                    }),
                    ..default()
                })
                .set(ImagePlugin::default_nearest()),
        )
        .insert_resource(ClearColor(Color::srgb(0.08, 0.08, 0.10)))
        .init_resource::<FpsLimiter>()
        .add_plugins(GamePlugin)
        .add_systems(Last, limit_fps)
        .run();
}
