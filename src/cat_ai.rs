//! Cat AI & Autonomous Roaming Systems
//!
//! Controls the cat's autonomous behavior: wandering around the screen,
//! looking around curiously, sitting, napping, reacting to petting,
//! and always returning to face the screen when resting.

use bevy::prelude::*;
use crate::cat_model::CatRoot;
use crate::config::{CatAiState, CatConfig};

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






/// Roaming area limits in 3D world space.
const ROAM_MIN_X: f32 = -1.6;
const ROAM_MAX_X: f32 = 1.6;
const ROAM_MIN_Z: f32 = -0.6;
const ROAM_MAX_Z: f32 = 0.6;

/// High-level AI state machine: decides when to walk, sit, sniff, or nap.
fn update_cat_ai(
    time: Res<Time>,
    mut config: ResMut<CatConfig>,
    q_root: Query<&Transform, With<CatRoot>>,
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
        let _current_pos = q_root
            .single()
            .map(|t| t.translation)
            .unwrap_or(Vec3::ZERO);

        match config.state {
            CatAiState::Idle | CatAiState::Sniffing | CatAiState::Sitting => {
                // Generate a pseudo-random new destination
                let seed = time.elapsed_secs();
                let rand_x = ((seed * 1.33).sin() * 0.5 + 0.5) * (ROAM_MAX_X - ROAM_MIN_X) + ROAM_MIN_X;
                let rand_z = ((seed * 2.71).cos() * 0.5 + 0.5) * (ROAM_MAX_Z - ROAM_MIN_Z) + ROAM_MIN_Z;

                config.roam_target = Vec3::new(rand_x, 0.0, rand_z);
                config.state = CatAiState::Walking;
                // Timeout in case it gets stuck
                config.state_timer = 6.0;
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

/// Moves and turns the cat towards its roaming destination.
/// When resting/idle, smoothly faces the screen towards the user.
fn update_roam_movement(
    time: Res<Time>,
    mut config: ResMut<CatConfig>,
    mut q_root: Query<&mut Transform, With<CatRoot>>,
) {
    let dt = time.delta_secs();
    let Ok(mut transform) = q_root.single_mut() else {
        return;
    };

    if config.state == CatAiState::Walking {
        let to_target = config.roam_target - transform.translation;
        let dist = Vec2::new(to_target.x, to_target.z).length();

        // Check if reached destination
        if dist < 0.15 {
            config.state = CatAiState::Idle;
            config.state_timer = 3.0;
            return;
        }

        // Smoothly turn towards moving direction (face points along travel direction)
        let target_angle = to_target.x.atan2(to_target.z);
        let _target_rot = Quat::from_rotation_y(target_angle);
        // 2D Sprite rotation handled via flip_x

        // Move forward along the direction the cat is facing (face leads the way!)
        let move_speed = 0.95;
        let forward = (transform.rotation * Vec3::Z).normalize_or_zero();
        transform.translation += Vec3::new(forward.x, 0.0, forward.z) * move_speed * dt;

        // Keep within roaming bounds
        transform.translation.x = transform.translation.x.clamp(ROAM_MIN_X, ROAM_MAX_X);
        transform.translation.z = transform.translation.z.clamp(ROAM_MIN_Z, ROAM_MAX_Z);

        // Advance walk cycle animation phase
        config.walk_phase += dt * 8.5;

        // Look in direction of travel
        config.look_target = config.roam_target + Vec3::new(0.0, 0.5, 0.0);
    } else {
        // When not walking (Idle, Sitting, Sniffing, Sleeping, Petting):
        // Smoothly turn to face the screen/user (towards +Z / camera)
        let _face_screen_rot = Quat::IDENTITY;
        // 2D Sprite rotation handled via flip_x

        // Reset walk phase smoothly
        config.walk_phase = 0.0;
    }
}
