//! Cat AI & Autonomous Roaming Systems
//!
//! Controls the cat's autonomous behavior: wandering around the screen,
//! looking around curiously, sitting, napping, and reacting to petting.

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
                cat_particles_3d_system,
                update_particles_3d,
            ),
        );
    }
}

/// Marker for 3D billboard emotion particles (hearts, zzz).
#[derive(Component)]
pub struct EmotionParticle3d {
    pub lifetime: f32,
    pub max_lifetime: f32,
    pub velocity: Vec3,
}

/// Roaming area limits in 3D world space.
const ROAM_MIN_X: f32 = -1.8;
const ROAM_MAX_X: f32 = 1.8;
const ROAM_MIN_Z: f32 = -0.7;
const ROAM_MAX_Z: f32 = 0.7;

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
                config.state_timer = 2.0;
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
                    config.state_timer = 3.5;
                } else if seed < 0.65 {
                    config.state = CatAiState::Sniffing;
                    config.state_timer = 2.2;
                } else {
                    config.state = CatAiState::Idle;
                    config.state_timer = 2.0;
                }
            }
            _ => {
                config.state = CatAiState::Idle;
                config.state_timer = 2.0;
            }
        }
    }
}

/// Moves and turns the cat towards its roaming destination.
fn update_roam_movement(
    time: Res<Time>,
    mut config: ResMut<CatConfig>,
    mut q_root: Query<&mut Transform, With<CatRoot>>,
) {
    if config.state != CatAiState::Walking {
        return;
    }

    let dt = time.delta_secs();
    let Ok(mut transform) = q_root.single_mut() else {
        return;
    };

    let to_target = config.roam_target - transform.translation;
    let dist = Vec2::new(to_target.x, to_target.z).length();

    // Check if reached destination
    if dist < 0.15 {
        config.state = CatAiState::Idle;
        config.state_timer = 2.5;
        return;
    }

    // Smoothly turn towards moving direction
    let target_angle = to_target.x.atan2(to_target.z);
    let target_rot = Quat::from_rotation_y(target_angle);
    transform.rotation = transform.rotation.slerp(target_rot, 6.0 * dt);

    // Move forward
    let move_speed = 0.95;
    let forward = transform.forward().as_vec3();
    transform.translation += Vec3::new(forward.x, 0.0, forward.z) * move_speed * dt;

    // Keep within roaming bounds
    transform.translation.x = transform.translation.x.clamp(ROAM_MIN_X, ROAM_MAX_X);
    transform.translation.z = transform.translation.z.clamp(ROAM_MIN_Z, ROAM_MAX_Z);

    // Advance walk cycle animation phase
    config.walk_phase += dt * 8.5;

    // Update look target to face travel destination
    config.look_target = config.roam_target + Vec3::new(0.0, 0.5, 0.0);
}

/// Spawns 3D floating hearts and zzz particles.
fn cat_particles_3d_system(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    time: Res<Time>,
    mut config: ResMut<CatConfig>,
    q_root: Query<&Transform, With<CatRoot>>,
) {
    config.particle_timer -= time.delta_secs();

    let cat_pos = q_root
        .single()
        .map(|t| t.translation)
        .unwrap_or(Vec3::ZERO);

    match config.state {
        CatAiState::Petting => {
            if config.particle_timer <= 0.0 {
                config.particle_timer = 0.35;
                let heart_tex = asset_server.load("cat/heart.png");

                let offset_x = (time.elapsed_secs() * 6.0).sin() * 0.25;
                let spawn_pos = cat_pos + Vec3::new(offset_x, 1.2, 0.2);

                commands.spawn((
                    EmotionParticle3d {
                        lifetime: 1.2,
                        max_lifetime: 1.2,
                        velocity: Vec3::new(0.1, 0.85, 0.0),
                    },
                    Sprite::from_image(heart_tex),
                    Transform::from_translation(spawn_pos).with_scale(Vec3::splat(0.0035)),
                ));
            }
        }
        CatAiState::Sleeping => {
            if config.particle_timer <= 0.0 {
                config.particle_timer = 0.9;
                let zzz_tex = asset_server.load("cat/zzz.png");

                let offset_x = 0.2 + (time.elapsed_secs() * 2.5).sin() * 0.15;
                let spawn_pos = cat_pos + Vec3::new(offset_x, 0.9, 0.1);

                commands.spawn((
                    EmotionParticle3d {
                        lifetime: 1.6,
                        max_lifetime: 1.6,
                        velocity: Vec3::new(0.12, 0.55, 0.0),
                    },
                    Sprite::from_image(zzz_tex),
                    Transform::from_translation(spawn_pos).with_scale(Vec3::splat(0.0035)),
                ));
            }
        }
        _ => {
            if config.particle_timer < 0.0 {
                config.particle_timer = 0.0;
            }
        }
    }
}

/// Updates 3D emotion particles: floating upward and fading out.
fn update_particles_3d(
    mut commands: Commands,
    time: Res<Time>,
    mut q_particles: Query<(Entity, &mut EmotionParticle3d, &mut Transform, &mut Sprite)>,
) {
    let dt = time.delta_secs();

    for (entity, mut particle, mut transform, mut sprite) in &mut q_particles {
        particle.lifetime -= dt;
        if particle.lifetime <= 0.0 {
            commands.entity(entity).despawn();
            continue;
        }

        transform.translation += particle.velocity * dt;

        let alpha = (particle.lifetime / particle.max_lifetime).clamp(0.0, 1.0);
        sprite.color = Color::srgba(1.0, 1.0, 1.0, alpha);
    }
}
