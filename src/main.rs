//! Desktop 3D Cat Companion
//!
//! A lightweight, transparent, always-on-top 3D procedural physics cat companion.
//! Runs as a background desktop companion with native system tray integration,
//! organic feline behavioral AI, realistic 4-beat locomotion, eye blinking,
//! desktop gravity physics, and an extensible settings dashboard.
//!
//! Multi-OS compatible: Compiles natively for macOS, Windows, and Linux.

mod config;
mod cat_model;
mod cat_physics;
mod cat_ai;
mod tray;
mod dashboard;
mod window_control;

use bevy::camera::ClearColorConfig;
use bevy::prelude::*;
use bevy::window::{CompositeAlphaMode, WindowLevel};

use cat_model::CatModelPlugin;
use cat_physics::CatPhysicsPlugin;
use cat_ai::CatAiPlugin;
mod platform;

use tray::TrayPlugin;
use dashboard::DashboardPlugin;
use window_control::WindowControlPlugin;

fn main() {
    // 1. Configure macOS application as background accessory (no Dock tile, no top menu bar entry)
    platform::configure_as_background_accessory();

    // 2. Query initial display to position window dynamically on its usable ground
    let desktop_env = platform::create_desktop_environment();
    let initial_display = desktop_env
        .displays()
        .into_iter()
        .find(|d| d.is_primary)
        .or_else(|| desktop_env.displays().into_iter().next());

    let (initial_x, initial_y) = if let Some(ref display) = initial_display {
        println!(
            "Display detected:\n  id: {}\n  name: {}\n  position: ({}, {})\n  size: {}x{}\n  work area: ({}, {}, {}x{})\n  scale factor: {}",
            display.id,
            display.name,
            display.bounds.x,
            display.bounds.y,
            display.bounds.width,
            display.bounds.height,
            display.work_area.x,
            display.work_area.y,
            display.work_area.width,
            display.work_area.height,
            display.scale_factor
        );

        let initial_metrics = platform::DesktopMetrics::from_display(
            display,
            480.0,
            340.0,
            340.0 * 0.75,
            20.0,
            0.0,
        );
        let center_x = (display.work_area.left() + (display.work_area.width - 480.0) / 2.0)
            .clamp(initial_metrics.min_x, initial_metrics.max_x) as i32;
        (center_x, initial_metrics.ground_y as i32)
    } else {
        (100, 500)
    };

    App::new()
        // Window configuration: transparent, borderless, always on top, grounded at bottom
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
                position: bevy::window::WindowPosition::At(IVec2::new(initial_x, initial_y)),
                resizable: false,
                ..default()
            }),
            ..default()
        }))
        .insert_resource(platform::DesktopEnvironmentResource(desktop_env))
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
        .add_systems(Startup, (setup_3d_scene, ensure_platform_accessory))
        .add_systems(Update, ensure_platform_accessory)
        // Core Plugins
        .add_plugins(CatModelPlugin)
        .add_plugins(CatPhysicsPlugin)
        .add_plugins(CatAiPlugin)
        .add_plugins(TrayPlugin)
        .add_plugins(DashboardPlugin)
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

    // Fill / Rim Light (soft cool rim light from back left for beautiful contours)
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

/// Enforces background accessory activation policy and suppresses top menu bar.
fn ensure_platform_accessory(mut frame_count: Local<u32>) {
    if *frame_count < 20 {
        *frame_count += 1;
        platform::configure_as_background_accessory();
    }
}
