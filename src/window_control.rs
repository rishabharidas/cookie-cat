//! Window Control & Interaction Module (3D)
//!
//! Handles:
//! 1. Native OS window dragging with zero latency.
//! 2. Dynamic click-through hit testing based on 3D cat screen projection.
//! 3. Interactive petting, coat cycling, resizing, and sleep toggles.

use bevy::{
    app::AppExit,
    prelude::*,
    window::{CursorOptions, PrimaryWindow},
};
use crate::cat_model::CatRoot;
use crate::cat_physics::SquashPhysics;
use crate::config::{CatAiState, CatConfig};

pub struct WindowControlPlugin;

impl Plugin for WindowControlPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<CursorScreenPos>()
            .add_systems(
                Update,
                (
                    update_cursor_pos,
                    update_cursor_hit_test_3d,
                    handle_mouse_interactions_3d,
                    handle_keyboard_shortcuts,
                )
                    .chain(),
            );
    }
}

/// Stores the cursor position in 2D window pixels.
#[derive(Resource, Default)]
pub struct CursorScreenPos(pub Option<Vec2>);

/// Hitbox radius in window pixels around the cat's screen position.
const CAT_SCREEN_RADIUS: f32 = 90.0;

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
fn update_cursor_hit_test_3d(
    mut q_window: Query<&mut CursorOptions, With<PrimaryWindow>>,
) {
    if let Ok(mut cursor_options) = q_window.single_mut() {
        cursor_options.hit_test = true;
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

    let effective_radius = CAT_SCREEN_RADIUS * config.size.scale_factor();
    let is_on_cat = cat_screen_px.distance(mouse_px) <= effective_radius;

    // Update cat look target towards cursor if cursor is near
    if is_on_cat {
        let dx = (mouse_px.x - cat_screen_px.x) * 0.012;
        let dy = (cat_screen_px.y - mouse_px.y) * 0.012; // screen Y is inverted relative to world Y
        config.look_target = cat_transform.translation + Vec3::new(dx, 0.5 + dy, 2.5);
    }

    if !is_on_cat {
        return;
    }

    // Left Click: Drag window & Trigger petting bounce reaction
    if mouse_button.just_pressed(MouseButton::Left) {
        if let Ok(mut window) = q_window.single_mut() {
            window.start_drag_move();
        }

        // Trigger joyful petting reaction & spring jump
        config.state = CatAiState::Petting;
        config.petting_timer = 2.0;

        squash_phys.scale_offset.y += 0.35;
        squash_phys.velocity.y += 3.2;
    }

    // Right Click: Quick cycle coat color
    if mouse_button.just_pressed(MouseButton::Right) {
        config.coat = config.coat.next();
        println!("Switched coat to: {}", config.coat.display_name());
    }
}

/// Keyboard shortcuts for interactive controls.
fn handle_keyboard_shortcuts(
    keyboard: Res<ButtonInput<KeyCode>>,
    mut config: ResMut<CatConfig>,
    mut exit: MessageWriter<AppExit>,
) {
    // 'C' -> Cycle Coat Color (Biscuit -> White -> Grey)
    if keyboard.just_pressed(KeyCode::KeyC) {
        config.coat = config.coat.next();
        println!("Coat changed to: {}", config.coat.display_name());
    }

    // 'S' -> Cycle Size (Small -> Normal -> Large -> ExtraLarge)
    if keyboard.just_pressed(KeyCode::KeyS) {
        config.size = config.size.next();
        println!("Size changed to: {}", config.size.display_name());
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
        println!("Left Click + Drag: Move window");
        println!("Left Click: Pet cat & bounce with hearts");
        println!("Right Click or 'C': Cycle coat color");
        println!("'S': Cycle size (Small / Normal / Large / ExtraLarge)");
        println!("Space: Sleep / Awake toggle");
        println!("'Q' or Esc: Quit");
    }

    // 'Escape' or 'Q' -> Quit Application
    if keyboard.just_pressed(KeyCode::Escape) || keyboard.just_pressed(KeyCode::KeyQ) {
        exit.write(AppExit::Success);
    }
}
