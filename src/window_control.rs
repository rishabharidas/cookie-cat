//! Window Control & Desktop Interaction Module
//!
//! Handles:
//! 1. Whole-screen global mouse attention tracking:
//!    - Cat notices mouse movements anywhere across the desktop.
//!    - Smooth, subtle head gaze and inquisitive tilts within natural neck limits.
//!    - Cat occasionally ignores the cursor to feel natural and living.
//!    - Returns to natural idle gaze once cursor stops moving.
//! 2. Native OS window dragging with smooth release into gravity physics.
//! 3. Interactive petting, coat cycling, resizing, and dashboard hotkeys.

use bevy::{
    app::AppExit,
    prelude::*,
    window::{CursorOptions, PrimaryWindow, WindowPosition},
};
use crate::cat_model::CatRoot;
use crate::cat_physics::SquashPhysics;
use crate::config::{CatAiState, CatConfig};
use crate::tray::DashboardRequested;

pub struct WindowControlPlugin;

impl Plugin for WindowControlPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<CursorScreenPos>()
            .add_systems(
                Update,
                (
                    update_display_metrics,
                    update_cursor_pos,
                    update_cursor_hit_test_3d,
                    update_global_mouse_attention,
                    handle_mouse_interactions_3d,
                    handle_keyboard_shortcuts,
                )
                    .chain(),
            );
    }
}

/// Stores the cursor position in 2D window pixels (for local clicks).
#[derive(Resource, Default)]
pub struct CursorScreenPos(pub Option<Vec2>);

/// Hitbox radius in window pixels around the cat's screen position.
const CAT_SCREEN_RADIUS: f32 = 95.0;

/// Monitors the cat window position across multi-monitor virtual desktop space.
/// When the cat moves or is dragged to another display, dynamically detects the new
/// display and updates metrics (work area, bounds, min_x, max_x, ground_y).
fn update_display_metrics(
    desktop_env: Res<crate::platform::DesktopEnvironmentResource>,
    mut config: ResMut<CatConfig>,
    q_window: Query<&Window, With<PrimaryWindow>>,
) {
    let Ok(window) = q_window.single() else {
        return;
    };
    let WindowPosition::At(pos) = window.position else {
        return;
    };

    let win_w = window.resolution.width();
    let win_h = window.resolution.height();
    let window_rect = crate::platform::DesktopRect::new(pos.x as f32, pos.y as f32, win_w, win_h);

    if let Some(current_display) = desktop_env.0.display_for_window(window_rect) {
        let display_changed = match &config.current_metrics {
            Some(existing) => existing.display_id != current_display.id,
            None => true,
        };

        if display_changed {
            if let Some(old) = &config.current_metrics {
                println!(
                    "Cat moved from display '{}' to '{}'. Updating desktop metrics...",
                    old.display_name, current_display.name
                );
            } else {
                println!(
                    "Display detected: id='{}' name='{}' bounds={:?} work_area={:?} scale={}",
                    current_display.id,
                    current_display.name,
                    current_display.bounds,
                    current_display.work_area,
                    current_display.scale_factor
                );
            }

            let paws_y_offset = win_h * 0.75;
            let new_metrics = crate::platform::DesktopMetrics::from_display(
                &current_display,
                win_w,
                win_h,
                paws_y_offset,
                20.0,
                0.0,
            );

            config.ground_y = new_metrics.ground_y;

            // If not dragging and not falling, adjust desktop target so it stays within new display
            if !config.is_dragged && config.state != CatAiState::Falling {
                config.desktop_target.x = config.desktop_target.x.clamp(new_metrics.min_x, new_metrics.max_x);
                config.desktop_target.y = new_metrics.ground_y;
            }

            config.current_metrics = Some(new_metrics);
        }
    }
}

/// Reads local cursor position from the window for clicks.
fn update_cursor_pos(
    mut cursor_pos: ResMut<CursorScreenPos>,
    q_window: Query<&Window, With<PrimaryWindow>>,
) {
    let Ok(window) = q_window.single() else {
        cursor_pos.0 = None;
        return;
    };
    cursor_pos.0 = window.cursor_position();
}

/// Ensures hit testing is enabled for window interactions and dragging.
fn update_cursor_hit_test_3d(
    mut q_window: Query<&mut CursorOptions, With<PrimaryWindow>>,
) {
    if let Ok(mut cursor_options) = q_window.single_mut() {
        cursor_options.hit_test = true;
    }
}

/// Whole-screen global mouse attention tracking.
/// Cat notices cursor movement across the entire display and subtly glances over.
fn update_global_mouse_attention(
    time: Res<Time>,
    desktop_env: Res<crate::platform::DesktopEnvironmentResource>,
    mut config: ResMut<CatConfig>,
    q_window: Query<&Window, With<PrimaryWindow>>,
) {
    let dt = time.delta_secs();

    if !config.settings.enable_head_look {
        return;
    }

    // Query global cursor position across the virtual desktop display
    let global_mouse_opt = desktop_env.0.global_cursor_position();

    if let Some(mouse_px) = global_mouse_opt {
        let delta = if let Some(last_pos) = config.last_global_mouse {
            (mouse_px - last_pos).length()
        } else {
            0.0
        };

        config.last_global_mouse = Some(mouse_px);

        // Cat notices significant cursor movement (> 12 pixels) across the screen.
        // Sleeping cats ignore it unless woken.
        if delta > 12.0 && config.state != CatAiState::Sleeping && config.state != CatAiState::Falling {
            let elapsed = time.elapsed_secs();
            let seed = ((elapsed * 5.13).sin() * 43_758.547).fract().abs();

            // 75% chance to notice and look over (nonchalant feline behavior)
            if seed < 0.75 {
                config.mouse_attention_timer = 2.5 + seed * 1.5;

                // Cat center in screen coordinates
                let (win_w, win_h) = q_window
                    .single()
                    .map(|w| (w.resolution.width(), w.resolution.height()))
                    .unwrap_or((480.0, 340.0));

                let cat_screen_x = config.desktop_pos.map(|p| p.x + win_w * 0.5).unwrap_or(0.0);
                let cat_screen_y = config.desktop_pos.map(|p| p.y + win_h * 0.75).unwrap_or(config.ground_y + win_h * 0.75);

                let dx = ((mouse_px.x - cat_screen_x) / 550.0).clamp(-1.8, 1.8);
                let dy = ((cat_screen_y - mouse_px.y) / 450.0).clamp(-0.8, 1.2);

                config.look_target = Vec3::new(dx, 0.45 + dy * 0.45, 2.5);
                config.head_tilt = (dx * 0.08).clamp(-0.10, 0.10);
            }
        }
    }

    // Decay mouse attention timer
    if config.mouse_attention_timer > 0.0 {
        config.mouse_attention_timer -= dt;
        if config.mouse_attention_timer <= 0.0 {
            // Natural return to forward resting gaze or idle glance
            config.head_tilt = 0.0;
            if config.state == CatAiState::Idle || config.state == CatAiState::Sitting {
                config.look_target = Vec3::new(0.0, 0.45, 2.5);
            }
        }
    }
}

/// Handles mouse clicks: left-click to drag/pet, right-click to cycle coat.
fn handle_mouse_interactions_3d(
    mouse_button: Res<ButtonInput<MouseButton>>,
    cursor_pos: Res<CursorScreenPos>,
    mut config: ResMut<CatConfig>,
    mut squash_phys: ResMut<SquashPhysics>,
    mut q_window: Query<&mut Window, With<PrimaryWindow>>,
    q_cat: Query<&Transform, With<CatRoot>>,
    q_camera: Query<(&Camera, &GlobalTransform)>,
) {
    let Ok(mut window) = q_window.single_mut() else {
        return;
    };

    // Track dragging state and release into gravity
    if mouse_button.just_released(MouseButton::Left) && config.is_dragged {
        config.is_dragged = false;

        // If dropped above ground, enable falling
        if let WindowPosition::At(pos) = window.position {
            config.desktop_pos = Some(pos.as_vec2());
            if pos.y < config.ground_y as i32 - 10 && config.settings.enable_gravity {
                config.state = CatAiState::Falling;
            }
        }
    }

    let Some(mouse_px) = cursor_pos.0 else {
        return;
    };

    let Ok(cat_transform) = q_cat.single() else {
        return;
    };

    let Ok((camera, camera_transform)) = q_camera.single() else {
        return;
    };

    let cat_3d_pos = cat_transform.translation + Vec3::new(0.0, 0.45, 0.0);
    let Ok(cat_screen_px) = camera.world_to_viewport(camera_transform, cat_3d_pos) else {
        return;
    };

    let effective_radius = CAT_SCREEN_RADIUS * config.settings.size.scale_factor();
    let is_on_cat = cat_screen_px.distance(mouse_px) <= effective_radius;

    if !is_on_cat {
        return;
    }

    // Left Click: Drag window & Trigger petting bounce reaction
    if mouse_button.just_pressed(MouseButton::Left) {
        config.is_dragged = true;
        window.start_drag_move();

        // Trigger joyful petting reaction & spring jump
        config.state = CatAiState::Petting;
        config.petting_timer = 2.0;

        squash_phys.scale_offset.y += 0.35;
        squash_phys.velocity.y += 3.2;
    }

    // Right Click: Quick cycle coat color
    if mouse_button.just_pressed(MouseButton::Right) {
        config.settings.coat = config.settings.coat.next();
        println!("Switched coat to: {}", config.settings.coat.display_name());
        config.save_to_disk();
    }
}

/// Keyboard shortcuts for interactive controls.
fn handle_keyboard_shortcuts(
    keyboard: Res<ButtonInput<KeyCode>>,
    mut config: ResMut<CatConfig>,
    mut dashboard_req: ResMut<DashboardRequested>,
    mut exit: MessageWriter<AppExit>,
) {
    // 'C' -> Cycle Coat Color (Biscuit -> White -> Grey)
    if keyboard.just_pressed(KeyCode::KeyC) {
        config.settings.coat = config.settings.coat.next();
        println!("Coat changed to: {}", config.settings.coat.display_name());
        config.save_to_disk();
    }

    // 'S' -> Cycle Size (Small -> Normal -> Large -> ExtraLarge)
    if keyboard.just_pressed(KeyCode::KeyS) {
        config.settings.size = config.settings.size.next();
        println!("Size changed to: {}", config.settings.size.display_name());
        config.save_to_disk();
    }

    // 'D' -> Open Dashboard Window
    if keyboard.just_pressed(KeyCode::KeyD) {
        dashboard_req.0 = true;
    }

    // 'P' -> Toggle Pause / Resume
    if keyboard.just_pressed(KeyCode::KeyP) {
        config.settings.paused = !config.settings.paused;
        println!("Cat paused: {}", config.settings.paused);
        config.save_to_disk();
    }

    // 'Space' -> Toggle Sleep / Awake
    if keyboard.just_pressed(KeyCode::Space) {
        config.state = if config.state == CatAiState::Sleeping {
            CatAiState::Idle
        } else {
            CatAiState::Sleeping
        };
        println!("Cat state: {:?}", config.state);
    }

    // 'H' -> Print Help / Controls
    if keyboard.just_pressed(KeyCode::KeyH) {
        println!("=== Desktop Cat Controls ===");
        println!("Left Click + Drag: Move window (releases with gravity)");
        println!("Left Click: Pet cat & bounce with hearts");
        println!("Right Click or 'C': Cycle coat color");
        println!("'S': Cycle size (Small / Normal / Large / ExtraLarge)");
        println!("'D': Open Settings Dashboard");
        println!("'P': Pause / Resume cat behavior");
        println!("Space: Sleep / Awake toggle");
        println!("'Q' or Esc: Quit");
    }

    // 'Escape' or 'Q' -> Quit Application
    if keyboard.just_pressed(KeyCode::Escape) || keyboard.just_pressed(KeyCode::KeyQ) {
        exit.write(AppExit::Success);
    }
}
