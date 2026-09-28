//! Physics, Locomotion & Secondary Motion Systems
//!
//! Implements:
//! 1. Physically-grounded feline 4-beat walk cycle with accurate swing/stance phases,
//!    torso bobbing, lateral weight sway, and speed synchronization.
//! 2. Desktop gravity, vertical drop acceleration, and spring-cushioned landing impacts.
//! 3. Natural eye blinking, slow-blinking, and sleepy closed eyes.
//! 4. Inquisitive head tracking, neck pitch/yaw limits, and curious head tilts.
//! 5. Posture blending for smooth sitting, standing, stretching, and resting poses.
//! 6. Spring-damper wave propagation tail physics and reactive ear twitches.

use std::f32::consts::PI;
use bevy::prelude::*;
use crate::cat_model::{
    CatEar, CatEye, CatHead, CatLeg, CatSquash, CatTailJoint, CatWhisker, LegType,
};
use crate::config::{CatAiState, CatConfig};

pub struct CatPhysicsPlugin;

impl Plugin for CatPhysicsPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<SquashPhysics>()
            .add_systems(
                Update,
                (
                    update_desktop_gravity,
                    animate_walk_cycle,
                    update_posture_and_squash,
                    update_eye_blinking,
                    update_head_look_and_tilt,
                    update_whisker_twitch,
                    update_tail_physics,
                    update_ear_twitch,
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

/// Applies desktop gravity when the cat is above ground level.
fn update_desktop_gravity(
    time: Res<Time>,
    mut config: ResMut<CatConfig>,
    mut squash_phys: ResMut<SquashPhysics>,
) {
    if !config.settings.enable_gravity || config.is_dragged {
        return;
    }

    let dt = time.delta_secs();

    // Use dynamic desktop work-area ground baseline
    if let Some(m) = &config.current_metrics {
        config.ground_y = m.ground_y;
    }

    if let Some(pos) = config.desktop_pos {
        // Check if cat is currently above ground level
        if pos.y < config.ground_y - 2.0 {
            if config.state != CatAiState::Falling && config.state != CatAiState::Petting {
                config.state = CatAiState::Falling;
            }

            // Gravitational acceleration (pixels/sec^2)
            let gravity = 1350.0;
            config.vertical_velocity += gravity * dt;
            // Terminal falling velocity clamp
            config.vertical_velocity = config.vertical_velocity.min(1800.0);

            let mut new_y = pos.y + config.vertical_velocity * dt;

            // Check landing collision with ground
            if new_y >= config.ground_y {
                new_y = config.ground_y;
                let impact_vel = config.vertical_velocity;
                config.vertical_velocity = 0.0;

                // Landing squash impact proportional to impact velocity
                let squash_strength = (impact_vel / 900.0).clamp(0.15, 0.45);
                squash_phys.scale_offset.y = -squash_strength;
                squash_phys.velocity.y = -squash_strength * 6.0;

                config.state = CatAiState::Landing;
                config.state_timer = 0.35;
            }

            config.desktop_pos = Some(Vec2::new(pos.x, new_y));
        } else if config.state == CatAiState::Falling {
            // Grounded
            config.vertical_velocity = 0.0;
            config.state = CatAiState::Landing;
            config.state_timer = 0.25;
        }
    }
}

/// Physically-grounded feline 4-beat walk cycle.
/// Accurate feline gait:
/// - Stance phase: paw stays firmly on the ground ($Y = \text{rest}$) pushing backward.
/// - Swing phase: paw lifts into a parabolic step arc ($Y > \text{rest}$) reaching forward.
/// - Leg timing: 4-beat lateral sequence with 0.25 phase intervals.
fn animate_walk_cycle(
    time: Res<Time>,
    config: Res<CatConfig>,
    mut q_legs: Query<(&CatLeg, &mut Transform)>,
) {
    let dt = time.delta_secs();
    let is_walking = config.state == CatAiState::Walking || config.state == CatAiState::Stopping;
    let phase = config.walk_phase;
    let posture = config.posture_blend;

    // Stride parameters based on current speed
    let stride_z = 0.16;
    let step_lift = 0.12;

    for (leg, mut transform) in &mut q_legs {
        if is_walking && config.settings.enable_roaming {
            // Feline 4-beat gait phase distribution:
            // BackLeft (0.0) -> FrontLeft (0.25) -> BackRight (0.50) -> FrontRight (0.75)
            let leg_phase_offset = match leg.leg_type {
                LegType::BackLeft => 0.0,
                LegType::FrontLeft => 0.25 * 2.0 * PI,
                LegType::BackRight => 0.50 * 2.0 * PI,
                LegType::FrontRight => 0.75 * 2.0 * PI,
            };

            let leg_phase = (phase + leg_phase_offset).rem_euclid(2.0 * PI);

            // Stance phase is 60% of cycle (contact with ground)
            // Swing phase is 40% of cycle (lifted and swinging forward)
            let (step_y, step_z) = if leg_phase < PI {
                // Swing phase: paw lifts and reaches forward
                let norm = leg_phase / PI;
                let lift = (norm * PI).sin() * step_lift;
                let z = -stride_z + norm * (stride_z * 2.0);
                (lift, z)
            } else {
                // Stance phase: paw stays on ground, pushing backward
                let norm = (leg_phase - PI) / PI;
                let z = stride_z - norm * (stride_z * 2.0);
                (0.0, z) // Paw stays flat on the ground plane!
            };

            let target_pos = leg.rest_translation + Vec3::new(0.0, step_y, step_z);
            transform.translation = transform.translation.lerp(target_pos, 16.0 * dt);
        } else {
            // Not walking: apply posture tuck or relax to rest translation
            let mut target_pos = leg.rest_translation;

            if posture > 0.05 {
                // Sitting / Sleeping posture: tuck paws smoothly
                match leg.leg_type {
                    LegType::FrontLeft | LegType::FrontRight => {
                        // Front paws tuck slightly under chest
                        target_pos.y += 0.04 * posture;
                        target_pos.z += 0.06 * posture;
                    }
                    LegType::BackLeft | LegType::BackRight => {
                        // Back legs fold up beside torso
                        target_pos.y += 0.12 * posture;
                        target_pos.z += 0.08 * posture;
                    }
                }
            } else if config.state == CatAiState::Stretching {
                // Stretching posture: front paws reach far forward
                match leg.leg_type {
                    LegType::FrontLeft | LegType::FrontRight => {
                        target_pos.z += 0.14;
                        target_pos.y -= 0.02;
                    }
                    LegType::BackLeft | LegType::BackRight => {
                        target_pos.z -= 0.08;
                        target_pos.y += 0.02;
                    }
                }
            }

            transform.translation = transform.translation.lerp(target_pos, 8.0 * dt);
        }
    }
}

/// Updates posture blending and squash-and-stretch dynamic bounciness.
fn update_posture_and_squash(
    time: Res<Time>,
    mut config: ResMut<CatConfig>,
    mut squash_phys: ResMut<SquashPhysics>,
    mut q_squash: Query<&mut Transform, With<CatSquash>>,
) {
    let dt = time.delta_secs();
    let elapsed = time.elapsed_secs();
    let base_scale = config.settings.size.scale_factor();

    let Ok(mut transform) = q_squash.single_mut() else {
        return;
    };

    // 1. Posture Target Blend Factor (0.0 = standing, 1.0 = sitting/sleeping)
    let target_posture = match config.state {
        CatAiState::Sitting => 0.85,
        CatAiState::Sleeping => 1.00,
        _ => 0.0,
    };
    config.posture_blend = config.posture_blend.lerp(target_posture, 4.0 * dt);

    // 2. Breathing Rhythm
    let (breath_speed, breath_amount) = match config.state {
        CatAiState::Sleeping => (1.3, 0.035),
        CatAiState::Walking => (5.5, 0.018),
        CatAiState::Petting => (4.5, 0.030),
        _ => (2.6, 0.022),
    };
    let breath = (elapsed * breath_speed).sin() * breath_amount;

    // 3. Torso Vertical Bobbing & Swaying while walking
    let (walk_bob_y, walk_sway_x) = if config.state == CatAiState::Walking {
        // Double frequency bobbing (2 bobs per full 4-beat cycle)
        let bob = (config.walk_phase * 2.0).sin().abs() * 0.045;
        // Subtle lateral sway
        let sway = config.walk_phase.sin() * 0.025;
        (bob, sway)
    } else {
        (0.0, 0.0)
    };

    // 4. Spring Harmonic Physics for impacts, landings, and petting
    let spring_stiffness = 85.0;
    let damping = 10.0;

    let force = -squash_phys.scale_offset * spring_stiffness - squash_phys.velocity * damping;
    squash_phys.velocity += force * dt;
    let vel = squash_phys.velocity;
    squash_phys.scale_offset += vel * dt;

    // 5. Compute Dynamic Translation (lowers down when sitting/sleeping)
    let base_y = 0.55 - (config.posture_blend * 0.16) + walk_bob_y;
    transform.translation = Vec3::new(walk_sway_x, base_y, 0.0);

    // 6. Compute Dynamic Scale with Volume Preservation
    let dynamic_y = 1.0 + breath + squash_phys.scale_offset.y - (config.posture_blend * 0.10);
    let dynamic_xz = 1.0 - (breath * 0.5) - (squash_phys.scale_offset.y * 0.45) + (config.posture_blend * 0.06);

    transform.scale = Vec3::new(
        base_scale * dynamic_xz,
        base_scale * dynamic_y,
        base_scale * dynamic_xz,
    );
}

/// Natural feline eye blinking and sleepy closed eyes.
fn update_eye_blinking(
    time: Res<Time>,
    mut config: ResMut<CatConfig>,
    mut q_eyes: Query<(&CatEye, &mut Transform)>,
) {
    if !config.settings.enable_blinking {
        // Reset eyes to open
        for (eye, mut transform) in &mut q_eyes {
            transform.scale = eye.rest_scale;
        }
        return;
    }

    let dt = time.delta_secs();

    if config.state == CatAiState::Sleeping {
        // Eyes held peacefully closed while sleeping
        config.blink_amount = 0.94;
    } else {
        config.blink_timer -= dt;

        if config.blink_timer <= 0.0 {
            // Trigger a quick blink (duration ~0.14s)
            config.blink_timer = 3.5 + ((time.elapsed_secs() * 1.7).sin().abs()) * 4.0;
            config.blink_amount = 1.0;
        }

        // Decay blink smoothly
        if config.blink_amount > 0.0 {
            config.blink_amount = (config.blink_amount - dt * 8.0).max(0.0);
        }

        // Half-blink of trust when sitting peacefully
        if config.state == CatAiState::Sitting && config.blink_amount <= 0.0 {
            let slow_blink = ((time.elapsed_secs() * 0.6).sin()).max(0.0) * 0.40;
            config.blink_amount = slow_blink;
        }
    }

    // Apply eye vertical compression
    let y_scale_factor = (1.0 - config.blink_amount * 0.92).clamp(0.08, 1.0);

    for (eye, mut transform) in &mut q_eyes {
        transform.scale = Vec3::new(
            eye.rest_scale.x,
            eye.rest_scale.y * y_scale_factor,
            eye.rest_scale.z,
        );
    }
}

/// Head tracking, curious tilts, and look-around behavior.
fn update_head_look_and_tilt(
    time: Res<Time>,
    config: Res<CatConfig>,
    mut q_head: Query<&mut Transform, With<CatHead>>,
) {
    let Ok(mut transform) = q_head.single_mut() else {
        return;
    };

    let dt = time.delta_secs();

    if !config.settings.enable_head_look {
        transform.rotation = transform.rotation.slerp(Quat::IDENTITY, 4.0 * dt);
        return;
    }

    // Calculate direction from head to look target
    let head_pos = Vec3::new(0.0, 0.85, 0.45);
    let to_target = (config.look_target - head_pos).normalize_or_zero();

    // Natural feline neck rotation limits:
    // Yaw: +/- 45 deg, Pitch: -25 to +20 deg
    let target_yaw = (to_target.x * 0.80).clamp(-0.75, 0.75);
    let target_pitch = (-to_target.y * 0.65).clamp(-0.45, 0.35);
    let target_roll = config.head_tilt.clamp(-0.25, 0.25);

    let target_rot = Quat::from_euler(EulerRot::YXZ, target_yaw, target_pitch, target_roll);
    transform.rotation = transform.rotation.slerp(target_rot, 5.5 * dt);
}

/// Subtle whisker vibration during sniffing / investigating.
fn update_whisker_twitch(
    time: Res<Time>,
    config: Res<CatConfig>,
    mut q_whiskers: Query<(&CatWhisker, &mut Transform)>,
) {
    let dt = time.delta_secs();
    let is_investigating = config.state == CatAiState::Investigating;

    for (whisker, mut transform) in &mut q_whiskers {
        let mut target_rot = whisker.base_rotation;

        if is_investigating {
            let vib = (time.elapsed_secs() * 24.0).sin() * 0.08;
            target_rot *= Quat::from_rotation_y(vib);
        }

        transform.rotation = transform.rotation.slerp(target_rot, 12.0 * dt);
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
        CatAiState::Petting => (12.5, 0.48), // Excited rapid wag
        CatAiState::Walking => (6.5, 0.30),  // Balanced trot sway
        CatAiState::Sitting => (2.2, 0.16),  // Gentle tip swish curled near paws
        CatAiState::Sleeping => (1.0, 0.06), // Almost still
        CatAiState::Investigating => (4.5, 0.25),
        _ => (3.2, 0.20),                    // Idle relaxed wag
    };

    for (mut joint, mut transform) in &mut q_tail {
        let i = joint.index as f32;
        // Harmonic wave propagated along tail chain
        let wave_target_x = (elapsed * wag_speed - i * 0.65).sin() * wag_amplitude;
        let wave_target_y = (elapsed * (wag_speed * 0.5) - i * 0.35).cos() * (wag_amplitude * 0.35);

        // Spring-damper tracking to target
        let spring_k = 18.0;
        let damping = 6.5;

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
    config: Res<CatConfig>,
    mut q_ears: Query<(&CatEar, &mut Transform)>,
    mut twitch_timer: Local<f32>,
    mut twitch_active: Local<f32>,
) {
    let dt = time.delta_secs();
    *twitch_timer -= dt;

    if *twitch_timer <= 0.0 {
        // Schedule next twitch in 3.0 to 6.5 seconds
        *twitch_timer = 3.0 + ((time.elapsed_secs() * 3.7).sin().abs()) * 3.5;
        *twitch_active = 0.30;
    }

    let is_twitching = *twitch_active > 0.0;
    if is_twitching {
        *twitch_active -= dt;
    }

    // Swivel ears slightly with look direction
    let look_yaw = (config.look_target.x * 0.15).clamp(-0.15, 0.15);

    for (ear, mut transform) in &mut q_ears {
        let mut rot = ear.base_rotation * Quat::from_rotation_y(look_yaw);

        if is_twitching && ear.is_left {
            let wobble = (time.elapsed_secs() * 36.0).sin() * 0.16;
            rot *= Quat::from_rotation_z(wobble);
        }

        transform.rotation = transform.rotation.slerp(rot, 14.0 * dt);
    }
}
