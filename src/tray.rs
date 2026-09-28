//! System Tray Integration Module
//!
//! Provides a native status bar / system tray icon and menu for the desktop cat.
//! Supports:
//! - "Open Dashboard"
//! - "Show/Hide Cat"
//! - "Pause/Resume Cat"
//! - "Exit"

use bevy::app::AppExit;
use bevy::prelude::*;
use bevy::window::PrimaryWindow;
use tray_icon::menu::{Menu, MenuEvent, MenuItem, PredefinedMenuItem};
use tray_icon::{Icon, TrayIcon, TrayIconBuilder, TrayIconEvent};

use crate::config::CatConfig;

pub struct TrayPlugin;

impl Plugin for TrayPlugin {
    fn build(&self, app: &mut App) {
        if let Some(tray_state) = setup_system_tray() {
            app.insert_non_send(tray_state);
        }

        app.init_resource::<DashboardRequested>()
            .add_systems(Update, handle_tray_events);
    }
}

/// Resource indicating whether the dashboard has been requested from tray or shortcut.
#[derive(Resource, Default)]
pub struct DashboardRequested(pub bool);

/// Holds references to the tray icon and its menu item IDs.
/// NonSend because TrayIcon interacts with OS main thread APIs.
pub struct TrayState {
    pub _tray_icon: TrayIcon,
    pub open_dashboard_id: tray_icon::menu::MenuId,
    pub toggle_cat_id: tray_icon::menu::MenuId,
    pub pause_cat_id: tray_icon::menu::MenuId,
    pub exit_id: tray_icon::menu::MenuId,
}

fn setup_system_tray() -> Option<TrayState> {
    let icon = create_cat_tray_icon();

    let tray_menu = Menu::new();

    let open_dashboard_item = MenuItem::new("Open Dashboard", true, None);
    let toggle_cat_item = MenuItem::new("Show/Hide Cat", true, None);
    let pause_cat_item = MenuItem::new("Pause/Resume Cat", true, None);
    let separator = PredefinedMenuItem::separator();
    let exit_item = MenuItem::new("Exit", true, None);

    let open_dashboard_id = open_dashboard_item.id().clone();
    let toggle_cat_id = toggle_cat_item.id().clone();
    let pause_cat_id = pause_cat_item.id().clone();
    let exit_id = exit_item.id().clone();

    let _ = tray_menu.append(&open_dashboard_item);
    let _ = tray_menu.append(&toggle_cat_item);
    let _ = tray_menu.append(&pause_cat_item);
    let _ = tray_menu.append(&separator);
    let _ = tray_menu.append(&exit_item);

    let mut builder = TrayIconBuilder::new()
        .with_menu(Box::new(tray_menu))
        .with_tooltip("Desktop Cat");

    if let Some(tray_icon) = icon {
        builder = builder.with_icon(tray_icon);
    }

    match builder.build() {
        Ok(tray_icon) => {
            println!("System tray icon initialized successfully.");
            Some(TrayState {
                _tray_icon: tray_icon,
                open_dashboard_id,
                toggle_cat_id,
                pause_cat_id,
                exit_id,
            })
        }
        Err(err) => {
            eprintln!("Warning: Failed to create system tray icon: {err}");
            None
        }
    }
}

/// Polls tray menu events and tray icon clicks.
fn handle_tray_events(
    tray_state: Option<NonSend<TrayState>>,
    mut dashboard_req: ResMut<DashboardRequested>,
    mut config: ResMut<CatConfig>,
    mut q_window: Query<&mut Window, With<PrimaryWindow>>,
    mut exit: MessageWriter<AppExit>,
) {
    // Drain raw tray icon click events without triggering actions;
    // native tray behavior opens the menu on click.
    while let Ok(_event) = TrayIconEvent::receiver().try_recv() {}

    // 2. Process Tray Menu item selection
    let Some(state) = tray_state else {
        return;
    };

    while let Ok(event) = MenuEvent::receiver().try_recv() {
        if event.id == state.open_dashboard_id {
            dashboard_req.0 = true;
        } else if event.id == state.toggle_cat_id {
            config.settings.cat_visible = !config.settings.cat_visible;
            if let Ok(mut window) = q_window.single_mut() {
                window.visible = config.settings.cat_visible;
            }
            config.save_to_disk();
        } else if event.id == state.pause_cat_id {
            config.settings.paused = !config.settings.paused;
            println!("Cat paused: {}", config.settings.paused);
            config.save_to_disk();
        } else if event.id == state.exit_id {
            println!("Terminating Desktop Cat via tray menu.");
            exit.write(AppExit::Success);
        }
    }
}

/// Generates a cute 32x32 RGBA cat icon.
fn create_cat_tray_icon() -> Option<Icon> {
    const WIDTH: u32 = 32;
    const HEIGHT: u32 = 32;
    let mut rgba = vec![0u8; (WIDTH * HEIGHT * 4) as usize];

    // Draw cute cat face silhouette
    for y in 0..HEIGHT {
        for x in 0..WIDTH {
            let idx = ((y * WIDTH + x) * 4) as usize;
            let fx = x as f32;
            let fy = y as f32;

            // Head circle: center (16, 19), radius 10.5
            let dx = fx - 16.0;
            let dy = fy - 19.0;
            let in_head = (dx * dx + dy * dy) <= 10.5 * 10.5;

            // Left ear triangle: points (6, 17), (7, 5), (14, 11)
            let in_left_ear = is_point_in_triangle(
                Vec2::new(fx, fy),
                Vec2::new(6.0, 17.0),
                Vec2::new(7.0, 5.0),
                Vec2::new(14.0, 11.0),
            );

            // Right ear triangle: points (26, 17), (25, 5), (18, 11)
            let in_right_ear = is_point_in_triangle(
                Vec2::new(fx, fy),
                Vec2::new(26.0, 17.0),
                Vec2::new(25.0, 5.0),
                Vec2::new(18.0, 11.0),
            );

            // Left ear inner pink
            let in_left_inner = is_point_in_triangle(
                Vec2::new(fx, fy),
                Vec2::new(8.0, 15.0),
                Vec2::new(8.5, 8.0),
                Vec2::new(13.0, 12.0),
            );

            // Right ear inner pink
            let in_right_inner = is_point_in_triangle(
                Vec2::new(fx, fy),
                Vec2::new(24.0, 15.0),
                Vec2::new(23.5, 8.0),
                Vec2::new(19.0, 12.0),
            );

            // Eyes: small circles at (11.5, 17.5) and (20.5, 17.5), r = 1.8
            let d_le = ((fx - 11.5).powi(2) + (fy - 17.5).powi(2)).sqrt();
            let d_re = ((fx - 20.5).powi(2) + (fy - 17.5).powi(2)).sqrt();
            let in_eyes = d_le <= 1.8 || d_re <= 1.8;

            // Nose: tiny triangle at (16, 21)
            let d_nose = ((fx - 16.0).powi(2) + (fy - 21.0).powi(2)).sqrt();
            let in_nose = d_nose <= 1.4;

            if in_eyes {
                rgba[idx] = 40;
                rgba[idx + 1] = 40;
                rgba[idx + 2] = 50;
                rgba[idx + 3] = 255;
            } else if in_nose {
                rgba[idx] = 245;
                rgba[idx + 1] = 130;
                rgba[idx + 2] = 155;
                rgba[idx + 3] = 255;
            } else if in_left_inner || in_right_inner {
                rgba[idx] = 250;
                rgba[idx + 1] = 180;
                rgba[idx + 2] = 190;
                rgba[idx + 3] = 255;
            } else if in_head || in_left_ear || in_right_ear {
                rgba[idx] = 245;
                rgba[idx + 1] = 245;
                rgba[idx + 2] = 248;
                rgba[idx + 3] = 255;
            }
        }
    }

    Icon::from_rgba(rgba, WIDTH, HEIGHT).ok()
}

fn is_point_in_triangle(pt: Vec2, v1: Vec2, v2: Vec2, v3: Vec2) -> bool {
    fn sign(p1: Vec2, p2: Vec2, p3: Vec2) -> f32 {
        (p1.x - p3.x) * (p2.y - p3.y) - (p2.x - p3.x) * (p1.y - p3.y)
    }

    let d1 = sign(pt, v1, v2);
    let d2 = sign(pt, v2, v3);
    let d3 = sign(pt, v3, v1);

    let has_neg = (d1 < 0.0) || (d2 < 0.0) || (d3 < 0.0);
    let has_pos = (d1 > 0.0) || (d2 > 0.0) || (d3 > 0.0);

    !(has_neg && has_pos)
}
