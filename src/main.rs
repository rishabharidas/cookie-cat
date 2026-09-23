//! Desktop 3D Cat Widget
//!
//! A lightweight, transparent, always-on-top 3D procedural physics cat companion
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
                title: "Desktop 3D Cat".into(),
                transparent: true,
                decorations: false,
                window_level: WindowLevel::AlwaysOnTop,
                #[cfg(target_os = "macos")]
                composite_alpha_mode: CompositeAlphaMode::PostMultiplied,
                #[cfg(target_os = "linux")]
                composite_alpha_mode: CompositeAlphaMode::PreMultiplied,
                #[cfg(target_os = "windows")]
                composite_alpha_mode: CompositeAlphaMode::Auto,
                resolution: (480, 340).into(),
                resizable: false,
                ..default()
            }),
            ..default()
        }))
        // Transparent clear color for window
        .insert_resource(ClearColor(Color::NONE))
        // Ambient studio lighting
        .insert_resource(GlobalAmbientLight {
            color: Color::srgb(1.0, 0.98, 0.95),
            brightness: 1200.0,
            ..default()
        })
        .init_resource::<config::CatConfig>()
        // 3D Scene setup (Camera and lights)
        .add_systems(Startup, setup_3d_scene)
        // Plugins
        .add_plugins(CatModelPlugin)
        .add_plugins(CatPhysicsPlugin)
        .add_plugins(CatAiPlugin)
        .add_plugins(WindowControlPlugin)
        .run();
}

/// Sets up 3D camera and studio key/fill/rim lights.
fn setup_3d_scene(mut commands: Commands) {
    // 3D Camera with transparent clear color
    commands.spawn((
        Camera3d::default(),
        Camera {
            clear_color: ClearColorConfig::Custom(Color::NONE),
            ..default()
        },
        Transform::from_xyz(0.0, 3.2, 4.6).looking_at(Vec3::new(0.0, 0.4, 0.0), Vec3::Y),
    ));

    // Key Light (warm sunlight from upper right)
    commands.spawn((
        DirectionalLight {
            color: Color::srgb(1.0, 0.97, 0.92),
            illuminance: 6500.0,
            shadow_maps_enabled: false,
            ..default()
        },
        Transform::from_xyz(4.0, 7.0, 5.0).looking_at(Vec3::ZERO, Vec3::Y),
    ));

    // Fill / Rim Light (soft cool rim light from back left for beautiful chibi contours)
    commands.spawn((
        DirectionalLight {
            color: Color::srgb(0.90, 0.94, 1.0),
            illuminance: 3200.0,
            shadow_maps_enabled: false,
            ..default()
        },
        Transform::from_xyz(-4.0, 4.0, -3.0).looking_at(Vec3::ZERO, Vec3::Y),
    ));
}
