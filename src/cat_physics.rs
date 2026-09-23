//! Physics & Secondary Motion Systems
//!
//! Implements:
//! 1. Spring-damper physics for multi-joint tail sway and inertial lag.
//! 2. Ear twitch dynamics.
//! 3. Procedural 4-beat trot walk cycle with paw stepping and torso bobbing.
//! 4. Squash-and-stretch dynamic bounciness for jumps, landings, and breathing.
//! 5. Head look-at target tracking.

use std::f32::consts::PI;
use bevy::prelude::*;
use crate::cat_model::{CatEar, CatHead, CatLeg, CatSquash, CatTailJoint, LegType};
use crate::config::{CatAiState, CatConfig};

pub struct CatPhysicsPlugin;

impl Plugin for CatPhysicsPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<SquashPhysics>()
            .add_systems(
                Update,
                (
                    animate_walk_cycle,
                    update_tail_physics,
                    update_ear_twitch,
                    update_squash_and_stretch,
                    update_head_look,
                ),
            );
    }
}

/// Spring-damper state for squash and stretch bounciness.
#[derive(Resource)]
pub struct SquashPhysics {
    pub scale_offset: Vec3,
    pub velocity: Vec3,
}

impl Default for SquashPhysics {
    fn default() -> Self {
        Self {
            scale_offset: Vec3::ZERO,
            velocity: Vec3::ZERO,
        }
    }
}

/// Procedural 4-beat walk cycle: animates paw positions and torso bobbing.
fn animate_walk_cycle(
    time: Res<Time>,
    config: Res<CatConfig>,
    mut q_legs: Query<(&CatLeg, &mut Transform)>,
) {
    let dt = time.delta_secs();
    let is_walking = config.state == CatAiState::Walking;
    let phase = config.walk_phase;

    for (leg, mut transform) in &mut q_legs {
        if is_walking {
            // Determine phase offset based on diagonal gait
            let leg_phase = match leg.leg_type {
                LegType::FrontLeft | LegType::BackRight => phase,
                LegType::FrontRight | LegType::BackLeft => phase + PI,
            };

            // Forward / backward stride
            let step_z = leg_phase.cos() * 0.14;
            // Vertical lift during swing phase
            let step_y = leg_phase.sin().max(0.0) * 0.10;

            transform.translation = leg.rest_translation + Vec3::new(0.0, step_y, step_z);
        } else {
            // Smoothly return paws to resting pose
            transform.translation = transform.translation.lerp(leg.rest_translation, 10.0 * dt);
        }
    }
}

/// Spring-damper tail physics: simulates inertia, body turns, and happy harmonic wagging.
fn update_tail_physics(
    time: Res<Time>,
    config: Res<CatConfig>,
    mut q_tail: Query<(&mut CatTailJoint, &mut Transform)>,
) {
    let dt = time.delta_secs();
    let elapsed = time.elapsed_secs();

    // Determine tail wag speed and amplitude based on state
    let (wag_speed, wag_amplitude) = match config.state {
        CatAiState::Petting => (12.0, 0.45), // Excited rapid wag
        CatAiState::Walking => (6.0, 0.28),  // Balanced trot sway
        CatAiState::Sitting => (2.5, 0.20),  // Gentle tip swish
        CatAiState::Sleeping => (1.2, 0.08), // Almost still
        _ => (3.5, 0.22),                    // Idle relaxed wag
    };

    for (mut joint, mut transform) in &mut q_tail {
        let i = joint.index as f32;
        // Harmonic wave propagated along tail chain
        let wave_target_x = (elapsed * wag_speed - i * 0.6).sin() * wag_amplitude;
        let wave_target_y = (elapsed * (wag_speed * 0.5) - i * 0.3).cos() * (wag_amplitude * 0.4);

        // Spring-damper tracking to target
        let spring_k = 18.0;
        let damping = 6.0;

        let diff_x = wave_target_x - joint.current_angle.x;
        let diff_y = wave_target_y - joint.current_angle.y;

        joint.angular_velocity.x += (diff_x * spring_k - joint.angular_velocity.x * damping) * dt;
        joint.angular_velocity.y += (diff_y * spring_k - joint.angular_velocity.y * damping) * dt;

        let ang_vel = joint.angular_velocity;
        joint.current_angle += ang_vel * dt;

        // Apply rotation to joint transform
        transform.rotation = Quat::from_euler(
            EulerRot::YXZ,
            joint.current_angle.x,
            joint.current_angle.y,
            0.0,
        );
    }
}

/// Ear twitch dynamics: secondary motion on the ears with random quick twitches.
fn update_ear_twitch(
    time: Res<Time>,
    mut q_ears: Query<(&CatEar, &mut Transform)>,
    mut twitch_timer: Local<f32>,
    mut twitch_active: Local<f32>,
) {
    let dt = time.delta_secs();
    *twitch_timer -= dt;

    if *twitch_timer <= 0.0 {
        // Schedule next twitch in 2.5 to 5.0 seconds
        *twitch_timer = 2.5 + ((time.elapsed_secs() * 3.7).sin().abs()) * 2.5;
        *twitch_active = 0.35; // Duration of twitch
    }

    let is_twitching = *twitch_active > 0.0;
    if is_twitching {
        *twitch_active -= dt;
    }

    for (ear, mut transform) in &mut q_ears {
        let mut rot = ear.base_rotation;
        if is_twitching && ear.is_left {
            let wobble = (time.elapsed_secs() * 35.0).sin() * 0.18;
            rot *= Quat::from_rotation_z(wobble);
        }
        transform.rotation = transform.rotation.slerp(rot, 15.0 * dt);
    }
}

/// Dynamic squash-and-stretch: plush bouncy deformation for jumps and landings.
fn update_squash_and_stretch(
    time: Res<Time>,
    config: Res<CatConfig>,
    mut squash_phys: ResMut<SquashPhysics>,
    mut q_squash: Query<&mut Transform, With<CatSquash>>,
) {
    let dt = time.delta_secs();
    let elapsed = time.elapsed_secs();
    let base_scale = config.size.scale_factor();

    let Ok(mut transform) = q_squash.single_mut() else {
        return;
    };

    // Idle breathing rhythm
    let (breath_speed, breath_amount) = match config.state {
        CatAiState::Sleeping => (1.4, 0.035),
        CatAiState::Walking => (6.0, 0.020),
        _ => (2.8, 0.025),
    };
    let breath = (elapsed * breath_speed).sin() * breath_amount;

    // Spring harmonic physics for impact/petting squash
    let spring_stiffness = 80.0;
    let damping = 9.0;

    let force = -squash_phys.scale_offset * spring_stiffness - squash_phys.velocity * damping;
    squash_phys.velocity += force * dt;
        let vel = squash_phys.velocity;
    squash_phys.scale_offset += vel * dt;

    let dynamic_y = 1.0 + breath + squash_phys.scale_offset.y;
    // Volume preservation: when squashing in Y, expand in X & Z
    let dynamic_xz = 1.0 - (breath * 0.5) - (squash_phys.scale_offset.y * 0.45);

    transform.scale = Vec3::new(
        base_scale * dynamic_xz,
        base_scale * dynamic_y,
        base_scale * dynamic_xz,
    );
}

/// Head tracking: rotates head smoothly to face look target.
fn update_head_look(
    time: Res<Time>,
    config: Res<CatConfig>,
    mut q_head: Query<&mut Transform, With<CatHead>>,
) {
    let Ok(mut transform) = q_head.single_mut() else {
        return;
    };

    let dt = time.delta_secs();

    // Calculate direction from head to look target
    let head_pos = Vec3::new(0.0, 0.9, 0.5);
    let to_target = (config.look_target - head_pos).normalize_or_zero();

    // Calculate yaw and pitch within comfortable neck limits
    let target_yaw = (-to_target.x).clamp(-0.65, 0.65);
    let target_pitch = to_target.y.clamp(-0.40, 0.40);

    let target_rot = Quat::from_euler(EulerRot::YXZ, target_yaw, target_pitch, 0.0);
    transform.rotation = transform.rotation.slerp(target_rot, 5.0 * dt);
}
