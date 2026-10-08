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
    CatBlush, CatEar, CatEye, CatEyeIris, CatEyePupil, CatEyeWing, CatHead, CatLeg, CatSquash,
    CatTailJoint, CatWhisker, LegType,
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

/// Spring-damper state for squash and stretch bounciness, including lateral belly jiggle.
#[derive(Resource)]
pub struct SquashPhysics {
    pub scale_offset: Vec3,
    pub velocity: Vec3,
    pub lateral_offset: f32,
    pub lateral_velocity: f32,
    pub last_vel_x: f32,
}

impl Default for SquashPhysics {
    fn default() -> Self {
        Self {
            scale_offset: Vec3::ZERO,
            velocity: Vec3::ZERO,
            lateral_offset: 0.0,
            lateral_velocity: 0.0,
            last_vel_x: 0.0,
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
/// - P3: Foot impact micro-squash on the torso when paws strike the ground.
fn animate_walk_cycle(
    time: Res<Time>,
    config: Res<CatConfig>,
    mut squash_phys: ResMut<SquashPhysics>,
    mut q_legs: Query<(&CatLeg, &mut Transform)>,
    mut prev_step_phase: Local<f32>,
) {
    let dt = time.delta_secs();
    let is_walking = config.state == CatAiState::Walking || config.state == CatAiState::Stopping;
    let phase = config.walk_phase;
    let posture = config.posture_blend;

    // P3: Foot impact detection — each quarter cycle (0.5 * PI) a paw enters stance phase
    if is_walking && config.settings.enable_roaming {
        let step_interval = std::f32::consts::FRAC_PI_2;
        let prev_step = (*prev_step_phase / step_interval).floor() as i32;
        let curr_step = (phase / step_interval).floor() as i32;
        if curr_step != prev_step && config.state == CatAiState::Walking {
            // Micro-squash impact on paw touchdown gives weighted, chonky step feel
            squash_phys.velocity.y -= 0.042;
        }
    }
    *prev_step_phase = phase;

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

    // 2. Breathing Rhythm and Purring Vibration
    let (breath_speed, breath_amount) = match config.state {
        CatAiState::Sleeping => (1.3, 0.035),
        CatAiState::Walking => (5.5, 0.018),
        CatAiState::Petting => (4.5, 0.024),
        _ => (2.6, 0.020),
    };
    let breath = (elapsed * breath_speed).sin() * breath_amount;
    let purr = if config.state == CatAiState::Petting {
        (elapsed * 32.0).sin() * 0.007
    } else {
        0.0
    };

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

    // 4. Spring Harmonic Physics for vertical impacts, landings, and petting
    let spring_stiffness = 85.0;
    let damping = 10.0;

    let force = -squash_phys.scale_offset * spring_stiffness - squash_phys.velocity * damping;
    squash_phys.velocity += force * dt;
    let vel = squash_phys.velocity;
    squash_phys.scale_offset += vel * dt;

    // 4b. P2: Lateral Inertial Belly Jiggle on Direction Change / Sudden Acceleration
    let curr_vx = config.current_velocity.x;
    let dvx = curr_vx - squash_phys.last_vel_x;
    if dvx.abs() > 10.0 {
        // Belly inertia kicks opposite to acceleration direction
        squash_phys.lateral_velocity -= (dvx / 140.0).clamp(-0.35, 0.35);
    }
    squash_phys.last_vel_x = curr_vx;

    // Lateral harmonic spring (softer and bouncier than vertical)
    let lateral_k = 50.0;
    let lateral_damping = 6.5;
    let lateral_force = -squash_phys.lateral_offset * lateral_k - squash_phys.lateral_velocity * lateral_damping;
    squash_phys.lateral_velocity += lateral_force * dt;
    let l_vel = squash_phys.lateral_velocity;
    squash_phys.lateral_offset += l_vel * dt;
    squash_phys.lateral_offset = squash_phys.lateral_offset.clamp(-0.40, 0.40);

    // 5. Compute Dynamic Translation (lowers down when sitting/sleeping + cuddle lean)
    let base_y = 0.22 - (config.posture_blend * 0.16) + walk_bob_y;
    let cuddle_x = config.cuddle_lean * 0.06;
    transform.translation = Vec3::new(walk_sway_x + cuddle_x, base_y, 0.0);
    // Subtle affectionate lean rotation when cuddled
    transform.rotation = Quat::from_rotation_z(-config.cuddle_lean * 0.05);

    // 6. Compute Dynamic Scale with Volume Preservation and Lateral Jiggle
    let dynamic_y = 1.0 + breath + purr + squash_phys.scale_offset.y - (config.posture_blend * 0.10);
    let dynamic_xz = 1.0 - (breath * 0.5) - (squash_phys.scale_offset.y * 0.45) + (config.posture_blend * 0.06);

    // Apply lateral jiggle: stretches along X when leaning, slight compression on Z
    let jiggle_x = squash_phys.lateral_offset * 0.14;
    let jiggle_z = -squash_phys.lateral_offset.abs() * 0.06;

    transform.scale = Vec3::new(
        base_scale * (dynamic_xz + jiggle_x),
        base_scale * dynamic_y,
        base_scale * (dynamic_xz + jiggle_z),
    );
}

/// Natural feline eye blinking, sleepy closed eyes, and S3 emotion-shaped irises/pupils.
///
/// All three eye layers (CatEye outer, CatEyeIris, CatEyePupil) are queried separately
/// so their X and Y scales can be driven independently per emotional state:
///
/// | State         | Eye outer | Iris          | Pupil                    |
/// |---------------|-----------|---------------|--------------------------|
/// | Happy/Petting | wide+flat | wide+squinty  | rounded cute             |
/// | Curious       | tall+wide | dilated large | narrow tall slit         |
/// | Startled      | very tall | very large    | fully round (dark flood) |
/// | Sleeping      | flat line | flat line     | flat line                |
fn update_eye_blinking(
    time: Res<Time>,
    mut config: ResMut<CatConfig>,
    mut q_eyes: Query<(&CatEye, &mut Transform)>,
    mut q_wings: Query<
        (&CatEyeWing, &mut Transform),
        (Without<CatEye>, Without<CatEyeIris>, Without<CatEyePupil>, Without<CatBlush>),
    >,
    mut q_iris: Query<
        (&CatEyeIris, &mut Transform),
        (Without<CatEye>, Without<CatEyeWing>, Without<CatEyePupil>, Without<CatBlush>),
    >,
    mut q_pupils: Query<
        (&CatEyePupil, &mut Transform),
        (Without<CatEye>, Without<CatEyeWing>, Without<CatEyeIris>, Without<CatBlush>),
    >,
    mut q_blush: Query<
        (&CatBlush, &mut Transform),
        (Without<CatEye>, Without<CatEyeWing>, Without<CatEyeIris>, Without<CatEyePupil>),
    >,
) {
    if !config.settings.enable_blinking {
        // Reset all layers to rest scale
        for (eye, mut transform) in &mut q_eyes {
            transform.scale = eye.rest_scale;
        }
        for (wing, mut transform) in &mut q_wings {
            transform.scale = wing.rest_scale;
        }
        for (iris, mut transform) in &mut q_iris {
            transform.scale = iris.rest_scale;
        }
        for (pupil, mut transform) in &mut q_pupils {
            transform.scale = pupil.rest_scale;
        }
        for (_blush, mut transform) in &mut q_blush {
            transform.scale = Vec3::ZERO;
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

    // Blink: vertical compression that applies to ALL layers equally
    let blink_y = (1.0 - config.blink_amount * 0.92).clamp(0.08, 1.0);

    // S3: Emotion-driven shape multipliers for the outer eye oval
    // emotion_x = horizontal stretch (> 1 = wider), emotion_y = vertical modifier
    let (eye_ex, eye_ey) = match config.state {
        CatAiState::Petting                                     => (1.25, 0.50), // happy squint
        CatAiState::Investigating | CatAiState::LookingAround   => (1.10, 1.20),
        CatAiState::Landing                                     => (1.14, 1.28),
        _                                                       => (1.00, 1.00),
    };

    // S3: Iris emotion — iris pops larger when curious, squints when happy
    let (iris_ex, iris_ey) = match config.state {
        CatAiState::Petting                                     => (1.18, 0.55),
        CatAiState::Investigating | CatAiState::LookingAround   => (1.12, 1.18),
        CatAiState::Landing                                     => (1.20, 1.25),
        _                                                       => (1.00, 1.00),
    };

    // S3: Pupil emotion — slit narrows when curious, rounds fully when startled
    let (pupil_ex, pupil_ey) = match config.state {
        CatAiState::Petting                                     => (1.10, 0.65), // rounded happy
        CatAiState::Investigating | CatAiState::LookingAround   => (0.85, 1.30), // tall thin slit
        CatAiState::Landing                                     => (1.60, 1.60), // fully dilated
        _                                                       => (1.00, 1.00),
    };

    // Apply blink + emotion to the eye white / outer oval
    for (eye, mut transform) in &mut q_eyes {
        transform.scale = Vec3::new(
            eye.rest_scale.x * eye_ex,
            eye.rest_scale.y * blink_y * eye_ey,
            eye.rest_scale.z,
        );
    }

    // Apply blink + emotion to the winged anime eyeliner
    for (wing, mut transform) in &mut q_wings {
        transform.scale = Vec3::new(
            wing.rest_scale.x * eye_ex,
            wing.rest_scale.y * blink_y * eye_ey,
            wing.rest_scale.z,
        );
    }

    // Apply blink + emotion to iris
    for (iris, mut transform) in &mut q_iris {
        transform.scale = Vec3::new(
            iris.rest_scale.x * iris_ex,
            iris.rest_scale.y * blink_y * iris_ey,
            iris.rest_scale.z,
        );
    }

    // Apply blink + emotion to pupil
    for (pupil, mut transform) in &mut q_pupils {
        transform.scale = Vec3::new(
            pupil.rest_scale.x * pupil_ex,
            pupil.rest_scale.y * blink_y * pupil_ey,
            pupil.rest_scale.z,
        );
    }

    // Smoothly fade in/out cheek blush during petting/happy
    let target_blush = if config.state == CatAiState::Petting { 1.0 } else { 0.0 };
    for (blush, mut transform) in &mut q_blush {
        transform.scale = transform.scale.lerp(blush.rest_scale * target_blush, 6.0 * dt);
    }
}

/// Head tracking, curious tilts, and look-around behavior with elastic limit spring (P4).
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

    // Blissful upward chin tilt when being petted, leaning affectionately into the hand
    if config.state == CatAiState::Petting {
        let pet_yaw = (config.cuddle_lean * 0.22).clamp(-0.35, 0.35);
        let pet_pitch = -0.18; // chin raised contentedly
        let pet_roll = (config.cuddle_lean * 0.10).clamp(-0.15, 0.15);
        let pet_rot = Quat::from_euler(EulerRot::YXZ, pet_yaw, pet_pitch, pet_roll);
        transform.rotation = transform.rotation.slerp(pet_rot, 7.5 * dt);
        return;
    }

    // Approximate head position in model-root local space for look-direction maths.
    let head_pos = Vec3::new(0.0, 0.70, 0.14);
    let to_target = (config.look_target - head_pos).normalize_or_zero();

    // Natural feline neck rotation limits with soft elastic overshoot (P4):
    // Softly yields past normal limits when tracking extreme angles
    let raw_yaw = to_target.x * 0.80;
    let hard_yaw = raw_yaw.clamp(-0.75, 0.75);
    let excess_yaw = raw_yaw - hard_yaw;
    let target_yaw = hard_yaw + excess_yaw * 0.18; // elastic neck strain

    let raw_pitch = -to_target.y * 0.65;
    let hard_pitch = raw_pitch.clamp(-0.45, 0.35);
    let excess_pitch = raw_pitch - hard_pitch;
    let target_pitch = hard_pitch + excess_pitch * 0.18;

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

/// Spring-damper tail physics: cascading lag per segment, wider tip sweep, and expressive wags (P1).
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

        // P1: Cascading spring stiffness & damping:
        // Base joint (0) is stiffer and leads; tip (2) is floppier with higher delay
        let spring_k = 22.0 / (1.0 + i * 0.65);
        let damping = (7.8 - i * 0.90).max(2.8);

        // Wave amplitude increases towards the tip (tip sweeps widest)
        let wave_amp = wag_amplitude * (1.0 + i * 0.50);

        // Progressive phase lag along the chain creates a natural traveling wave
        let wave_target_x = (elapsed * wag_speed - i * 0.85).sin() * wave_amp;
        let wave_target_y = (elapsed * (wag_speed * 0.5) - i * 0.50).cos() * (wave_amp * 0.35);

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

/// Ear dynamics: secondary motion with underdamped spring physics, sleep droop, and random twitches (P5).
fn update_ear_twitch(
    time: Res<Time>,
    config: Res<CatConfig>,
    mut q_ears: Query<(&mut CatEar, &mut Transform)>,
    mut twitch_timer: Local<f32>,
    mut twitch_active: Local<f32>,
) {
    let dt = time.delta_secs();
    *twitch_timer -= dt;

    if *twitch_timer <= 0.0 {
        // Schedule next twitch in 3.0 to 6.5 seconds
        *twitch_timer = 3.0 + ((time.elapsed_secs() * 3.7).sin().abs()) * 3.5;
        *twitch_active = 0.28;
    }

    let is_twitching = *twitch_active > 0.0;
    if is_twitching {
        *twitch_active -= dt;
    }

    // Swivel ears slightly with look direction
    let look_yaw = (config.look_target.x * 0.15).clamp(-0.15, 0.15);

    // Sleep droop: relaxed outward angle when sleeping
    let sleep_droop = if config.state == CatAiState::Sleeping { 0.22 } else { 0.0 };

    for (mut ear, mut transform) in &mut q_ears {
        // Kick spring velocity if twitching
        if is_twitching && ear.is_left {
            let kick = (time.elapsed_secs() * 42.0).sin() * 0.22;
            ear.dynamic_velocity += kick * 30.0 * dt;
        }

        // P5: Underdamped spring toward rest position (damping ratio < 1 -> natural bounciness)
        let spring_k = 38.0;
        let damping = 4.8;
        let force = -ear.dynamic_angle * spring_k - ear.dynamic_velocity * damping;
        ear.dynamic_velocity += force * dt;
        let vel = ear.dynamic_velocity;
        ear.dynamic_angle += vel * dt;
        ear.dynamic_angle = ear.dynamic_angle.clamp(-0.4, 0.4);

        let droop_angle = if ear.is_left { sleep_droop } else { -sleep_droop };
        let rot = ear.base_rotation
            * Quat::from_rotation_y(look_yaw)
            * Quat::from_rotation_z(ear.dynamic_angle + droop_angle);

        transform.rotation = transform.rotation.slerp(rot, 16.0 * dt);
    }
}
