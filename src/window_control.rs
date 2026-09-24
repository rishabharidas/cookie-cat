//! Window Control & Interaction Module (2D)
//!
//! Handles:
//! 1. Moving the OS window dynamically based on the AI state.
//! 2. Basic mouse hit testing and interactions.
//! 3. Keyboard shortcuts.

use bevy::{
    app::AppExit,
    prelude::*,
    window::{CursorOptions, PrimaryWindow, WindowPosition},
};
use crate::cat_model::CatSprite;
use crate::cat_physics::SquashPhysics;
use crate::config::{CatAiState, CatConfig};

pub struct WindowControlPlugin;

impl Plugin for WindowControlPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<CursorScreenPos>()
            .init_resource::<MonitorSize>()
            .add_systems(Startup, init_window_position)
            .add_systems(
                Update,
                (
                    update_cursor_pos,
                    update_cursor_hit_test,
                    handle_mouse_interactions,
                    handle_keyboard_shortcuts,
                    move_window_with_ai,
                )
                    .chain(),
            );
    }
}

/// Stores the cursor position in 2D window pixels.
#[derive(Resource, Default)]
pub struct CursorScreenPos(pub Option<Vec2>);

/// Stores the cached monitor size to clamp to the bottom.
#[derive(Resource)]
pub struct MonitorSize {
    pub width: i32,
    pub height: i32,
}

impl Default for MonitorSize {
    fn default() -> Self {
        // Fallback size, real size should ideally be fetched from the monitor.
        Self { width: 1920, height: 1080 }
    }
}

fn init_window_position(
    mut q_window: Query<&mut Window, With<PrimaryWindow>>,
    _monitor_size: ResMut<MonitorSize>,
) {
    if let Ok(mut window) = q_window.single_mut() {
        // Here we'd ideally get monitor size, but it might not be ready immediately.
        // We'll set a default starting position near the bottom center.
        let x = 1920 / 2 - 128;
        let y = 1080 - 256 - 40; // bottom of screen, roughly accounting for taskbar
        window.position = WindowPosition::At(IVec2::new(x, y));
    }
}

/// Reads cursor position from the window.
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
fn update_cursor_hit_test(
    mut q_window: Query<&mut CursorOptions, With<PrimaryWindow>>,
) {
    if let Ok(mut cursor_options) = q_window.single_mut() {
        cursor_options.hit_test = true;
    }
}

/// Hitbox radius in window pixels.
const CAT_SCREEN_RADIUS: f32 = 100.0;

/// Handles mouse clicks: left-click to drag/pet.
fn handle_mouse_interactions(
    mouse_button: Res<ButtonInput<MouseButton>>,
    cursor_pos: Res<CursorScreenPos>,
    mut config: ResMut<CatConfig>,
    mut squash_phys: ResMut<SquashPhysics>,
    mut q_window: Query<&mut Window, With<PrimaryWindow>>,
) {
    let Some(mouse_px) = cursor_pos.0 else {
        return;
    };

    // Center of 256x256 window
    let center = Vec2::new(128.0, 128.0);
    let is_on_cat = mouse_px.distance(center) <= CAT_SCREEN_RADIUS;

    if !is_on_cat {
        return;
    }

    // Left Click: Drag window & Trigger petting bounce reaction
    if mouse_button.just_pressed(MouseButton::Left) {
        if let Ok(mut window) = q_window.single_mut() {
            window.start_drag_move();
        }

        config.state = CatAiState::Petting;
        config.petting_timer = 2.0;

        squash_phys.scale_offset.y += 0.35;
        squash_phys.velocity.y += 3.2;
    }
}

/// Keyboard shortcuts for interactive controls.
fn handle_keyboard_shortcuts(
    keyboard: Res<ButtonInput<KeyCode>>,
    mut config: ResMut<CatConfig>,
    mut exit: MessageWriter<AppExit>,
) {
    if keyboard.just_pressed(KeyCode::Space) {
        config.state = if config.state == CatAiState::Sleeping {
            CatAiState::Idle
        } else {
            CatAiState::Sleeping
        };
        println!("Cat state: {:?}", config.state);
    }

    if keyboard.just_pressed(KeyCode::Escape) || keyboard.just_pressed(KeyCode::KeyQ) {
        exit.write(AppExit::Success);
    }
}

/// Moves the actual OS window left/right when the cat AI state is Walking.
fn move_window_with_ai(
    time: Res<Time>,
    config: Res<CatConfig>,
    mut q_window: Query<&mut Window, With<PrimaryWindow>>,
    mut q_sprite: Query<&mut Sprite, With<CatSprite>>,
    mut fraction_acc: Local<f32>,
) {
    let dt = time.delta_secs();

    let Ok(mut window) = q_window.single_mut() else {
        return;
    };

    if config.state == CatAiState::Walking {
        // Check which direction to walk based on roam_target.x
        let direction = if config.roam_target.x > 0.0 { 1.0 } else { -1.0 };

        let move_speed = 100.0; // pixels per second
        let dx = direction * move_speed * dt;

        *fraction_acc += dx;
        let move_pixels = fraction_acc.trunc() as i32;
        *fraction_acc -= move_pixels as f32;

        if let WindowPosition::At(mut pos) = window.position {
            pos.x += move_pixels;
            window.position = WindowPosition::At(pos);
        } else if let WindowPosition::Automatic = window.position {
            // fallback if it wasn't At
             window.position = WindowPosition::At(IVec2::new(1920/2, 1080 - 256 - 40));
        }

        // Flip sprite based on direction
        if let Ok(mut sprite) = q_sprite.single_mut() {
            sprite.flip_x = direction > 0.0;
        }
    } else {
        *fraction_acc = 0.0;
    }
}
