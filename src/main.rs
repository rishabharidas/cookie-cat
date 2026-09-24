//! Desktop 2D Cat Widget
//!
//! A lightweight, transparent, always-on-top 2D cat companion
//! inspired by drifty.so. Built with Rust and Bevy.
//!
//! Multi-OS compatible: Compiles natively for macOS, Windows, and Linux.

mod config;
mod cat_model;
mod cat_physics;
mod cat_ai;
mod window_control;

use bevy::camera::ClearColorConfig;
use bevy::prelude::*;
use bevy::window::{CompositeAlphaMode, WindowLevel};

use cat_model::CatModelPlugin;
use cat_physics::CatPhysicsPlugin;
use cat_ai::CatAiPlugin;
use window_control::WindowControlPlugin;

fn main() {
    App::new()
        // Window configuration: transparent, borderless, always on top
        .add_plugins(DefaultPlugins.set(WindowPlugin {
            primary_window: Some(Window {
                title: "Desktop 2D Cat".into(),
                transparent: true,
                decorations: false,
                window_level: WindowLevel::AlwaysOnTop,
                #[cfg(target_os = "macos")]
                composite_alpha_mode: CompositeAlphaMode::PostMultiplied,
                #[cfg(target_os = "linux")]
                composite_alpha_mode: CompositeAlphaMode::PreMultiplied,
                #[cfg(target_os = "windows")]
                composite_alpha_mode: CompositeAlphaMode::Auto,
                resolution: (256, 256).into(),
                resizable: false,
                ..default()
            }),
            ..default()
        }))
        // Transparent clear color for window
        .insert_resource(ClearColor(Color::NONE))
        .init_resource::<config::CatConfig>()
        // 2D Scene setup (Camera)
        .add_systems(Startup, setup_2d_scene)
        // Plugins
        .add_plugins(CatModelPlugin)
        .add_plugins(CatPhysicsPlugin)
        .add_plugins(CatAiPlugin)
        .add_plugins(WindowControlPlugin)
        .run();
}

/// Sets up 2D camera
fn setup_2d_scene(mut commands: Commands) {
    // 2D Camera with transparent clear color
    commands.spawn((
        Camera2d,
        Camera {
            clear_color: ClearColorConfig::Custom(Color::NONE),
            ..default()
        },
    ));
}
