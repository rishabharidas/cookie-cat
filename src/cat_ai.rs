//! Cat AI & Autonomous Roaming Systems
//!
//! Controls the cat's autonomous behavior: wandering left and right,
//! sitting, napping, and reacting to petting.

use bevy::prelude::*;
use crate::cat_model::CatRoot;
use crate::config::{CatAiState, CatConfig};
use bevy::window::PrimaryWindow;

pub struct CatAiPlugin;

impl Plugin for CatAiPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            Update,
            (
                update_cat_ai,
                update_roam_movement,
            ),
        );
    }
}

/// High-level AI state machine: decides when to walk left/right, sit, sniff, or nap.
fn update_cat_ai(
    time: Res<Time>,
    mut config: ResMut<CatConfig>,
    _q_window: Query<&Window, With<PrimaryWindow>>,
) {
    let dt = time.delta_secs();

    // Check petting timer
    if config.petting_timer > 0.0 {
        config.petting_timer -= dt;
        if config.petting_timer <= 0.0 {
            config.petting_timer = 0.0;
            if config.state == CatAiState::Petting {
                config.state = CatAiState::Idle;
                config.state_timer = 2.5;
            }
        }
        return;
    }

    // If sleeping, do not randomly roam
    if config.state == CatAiState::Sleeping {
        return;
    }

    config.state_timer -= dt;

    if config.state_timer <= 0.0 {
        match config.state {
            CatAiState::Idle | CatAiState::Sniffing | CatAiState::Sitting => {
                // Pick a direction (left or right)
                let seed = time.elapsed_secs();
                let direction = if (seed * 1.33).sin() > 0.0 { 1.0 } else { -1.0 };

                // Set roam target on the X axis (local coordinates)
                // The window will actually move on the screen, but we simulate it by setting a target
                // For simplicity, we just set a velocity or target.
                // Let's set roam_target.x to +1000.0 or -1000.0 and just walk that way for a bit.
                config.roam_target = Vec3::new(direction * 1000.0, 0.0, 0.0);
                config.state = CatAiState::Walking;
                config.state_timer = 2.0 + (seed * 2.0).sin().abs() * 3.0; // Walk for 2-5 seconds
            }
            CatAiState::Walking => {
                // Arrived or timed out: pick next idle action
                let seed = (time.elapsed_secs() * 5.0).sin().abs();
                if seed < 0.35 {
                    config.state = CatAiState::Sitting;
                    config.state_timer = 4.0;
                } else if seed < 0.65 {
                    config.state = CatAiState::Sniffing;
                    config.state_timer = 2.5;
                } else {
                    config.state = CatAiState::Idle;
                    config.state_timer = 3.0;
                }
            }
            _ => {
                config.state = CatAiState::Idle;
                config.state_timer = 2.5;
            }
        }
    }
}

/// Updates walk phase when walking.
/// We keep this simple, as actual window movement happens in window_control.rs.
fn update_roam_movement(
    time: Res<Time>,
    mut config: ResMut<CatConfig>,
    _q_root: Query<&mut Transform, With<CatRoot>>,
) {
    let dt = time.delta_secs();

    if config.state == CatAiState::Walking {
        // Advance walk cycle animation phase
        config.walk_phase += dt * 8.5;
    } else {
        // Reset walk phase smoothly
        config.walk_phase = 0.0;
    }
}
