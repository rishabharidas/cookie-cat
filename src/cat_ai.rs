//! Cat Behavioral AI & Autonomous Roaming Systems
//!
//! Features:
//! 1. Organic behavioral state machine: Idle, LookingAround, Walking, Stopping,
//!    Sitting, Standing, Sleeping, Investigating, Stretching, and Petting.
//! 2. Contextual probabilistic transitions with bounded randomized timing.
//! 3. Grounded horizontal desktop roaming with smooth acceleration, turning, and stopping deceleration.
//! 4. 3D emotion particle effects (floating hearts & zzz) with toggle support.

use std::f32::consts::PI;
use bevy::prelude::*;
use bevy::window::{PrimaryWindow, WindowPosition};
use crate::cat_model::CatRoot;
use crate::config::{CatAiState, CatConfig};

pub struct CatAiPlugin;

impl Plugin for CatAiPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, setup_particle_assets)
            .add_systems(
                Update,
                (
                    update_cat_behavior_ai,
                    update_roam_movement,
                    cat_particles_3d_system,
                    update_particles_3d,
                ),
            );
    }
}

/// Shared 3D mesh and material assets for emotion particles.
#[derive(Resource)]
pub struct ParticleAssets {
    pub quad_mesh: Handle<Mesh>,
    pub heart_material: Handle<StandardMaterial>,
    pub zzz_material: Handle<StandardMaterial>,
}

fn setup_particle_assets(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    let quad_mesh = meshes.add(Rectangle::new(0.28, 0.28));

    let heart_tex = asset_server.load("cat/heart.png");
    let heart_material = materials.add(StandardMaterial {
        base_color_texture: Some(heart_tex),
        alpha_mode: AlphaMode::Blend,
        unlit: true,
        cull_mode: None,
        ..default()
    });

    let zzz_tex = asset_server.load("cat/zzz.png");
    let zzz_material = materials.add(StandardMaterial {
        base_color_texture: Some(zzz_tex),
        alpha_mode: AlphaMode::Blend,
        unlit: true,
        cull_mode: None,
        ..default()
    });

    commands.insert_resource(ParticleAssets {
        quad_mesh,
        heart_material,
        zzz_material,
    });
}

/// Marker for 3D billboard emotion particles (hearts, zzz).
#[derive(Component)]
pub struct EmotionParticle3d {
    pub lifetime: f32,
    pub max_lifetime: f32,
    pub velocity: Vec3,
    pub initial_scale: Vec3,
}

/// High-level AI state machine: organically transitions between feline states.
fn update_cat_behavior_ai(
    time: Res<Time>,
    mut config: ResMut<CatConfig>,
    _q_window: Query<&Window, With<PrimaryWindow>>,
) {
    let dt = time.delta_secs();

    // Check if the cat is currently paused by the user
    if config.settings.paused {
        config.state = CatAiState::Idle;
        return;
    }

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

    // In falling or landing states, let physics manage completion
    if config.state == CatAiState::Falling {
        return;
    }

    if config.state == CatAiState::Landing {
        config.state_timer -= dt;
        if config.state_timer <= 0.0 {
            config.state = CatAiState::Idle;
            config.state_timer = 2.0;
        }
        return;
    }

    // Do not auto-transition if sleeping unless state timer has expired
    if config.state == CatAiState::Sleeping {
        config.state_timer -= dt;
        if config.state_timer <= 0.0 {
            // Wake up gently and stretch
            config.state = CatAiState::Stretching;
            config.state_timer = 2.5;
        }
        return;
    }

    config.state_timer -= dt;

    if config.state_timer <= 0.0 {
        let elapsed = time.elapsed_secs();
        let seed = ((elapsed * 2.37).sin() * 43758.5453).fract().abs();

        match config.state {
            CatAiState::Idle => {
                // If roaming is disabled, pick only in-place behaviors
                if !config.settings.enable_roaming {
                    if seed < 0.40 {
                        start_looking_around(&mut config, seed);
                    } else if seed < 0.70 {
                        config.state = CatAiState::Sitting;
                        config.state_timer = 6.0;
                    } else if seed < 0.85 {
                        config.state = CatAiState::Investigating;
                        config.state_timer = 2.8;
                        config.look_target = Vec3::new(0.0, -0.1, 1.2);
                    } else {
                        config.state = CatAiState::Stretching;
                        config.state_timer = 2.4;
                    }
                    return;
                }

                // Weighted contextual state decision
                if seed < 0.32 {
                    // Start looking around curiously
                    start_looking_around(&mut config, seed);
                } else if seed < 0.65 {
                    // Walk to a new position on the bottom ground
                    start_walking(&mut config, elapsed);
                } else if seed < 0.82 {
                    // Settle down into sitting posture
                    config.state = CatAiState::Sitting;
                    config.state_timer = 6.0 + seed * 6.0;
                } else if seed < 0.92 {
                    // Sniff / investigate
                    config.state = CatAiState::Investigating;
                    config.state_timer = 2.6;
                    config.look_target = Vec3::new(0.0, -0.1, 1.2);
                } else {
                    // Morning stretch
                    config.state = CatAiState::Stretching;
                    config.state_timer = 2.2;
                }
            }

            CatAiState::LookingAround => {
                // Return to Idle or transition into Walking
                config.head_tilt = 0.0;
                if seed < 0.40 && config.settings.enable_roaming {
                    start_walking(&mut config, elapsed);
                } else {
                    config.state = CatAiState::Idle;
                    config.state_timer = config.settings.idle_frequency.random_idle_duration(seed);
                    if config.mouse_attention_timer <= 0.0 {
                        config.look_target = Vec3::new(0.0, 0.45, 2.5);
                    }
                }
            }

            CatAiState::Walking => {
                // Timed out before reaching: smoothly decelerate to a stop
                config.state = CatAiState::Stopping;
                config.state_timer = 0.40;
            }

            CatAiState::Stopping => {
                // Finished stopping: settle into idle or small action
                if seed < 0.35 {
                    start_looking_around(&mut config, seed);
                } else if seed < 0.65 {
                    config.state = CatAiState::Sitting;
                    config.state_timer = 5.0 + seed * 5.0;
                } else {
                    config.state = CatAiState::Idle;
                    config.state_timer = config.settings.idle_frequency.random_idle_duration(seed);
                }
            }

            CatAiState::Sitting => {
                if seed < 0.35 {
                    // Take a peaceful nap
                    config.state = CatAiState::Sleeping;
                    config.state_timer = 18.0 + seed * 16.0;
                } else if seed < 0.65 {
                    // Look around while sitting
                    start_looking_around(&mut config, seed);
                } else {
                    // Stand up
                    config.state = CatAiState::Standing;
                    config.state_timer = 0.55;
                }
            }

            CatAiState::Standing => {
                config.state = CatAiState::Idle;
                config.state_timer = 2.0;
            }

            CatAiState::Investigating | CatAiState::Stretching => {
                config.state = CatAiState::Idle;
                config.state_timer = config.settings.idle_frequency.random_idle_duration(seed);
                if config.mouse_attention_timer <= 0.0 {
                    config.look_target = Vec3::new(0.0, 0.45, 2.5);
                }
            }

            _ => {
                config.state = CatAiState::Idle;
                config.state_timer = 2.5;
            }
        }
    }
}

fn start_looking_around(config: &mut CatConfig, seed: f32) {
    config.state = CatAiState::LookingAround;
    config.state_timer = 2.2 + seed * 1.5;

    // Pick a curious look target: glancing left, right, or upward
    let look_x = if seed < 0.5 { -1.8 - seed * 0.8 } else { 1.8 + seed * 0.8 };
    let look_y = 0.5 + (seed * 1.2);
    config.look_target = Vec3::new(look_x, look_y, 2.0);

    // Inquisitive head tilt
    config.head_tilt = if seed < 0.5 { -0.10 } else { 0.10 };
}

fn start_walking(
    config: &mut CatConfig,
    elapsed: f32,
) {
    let (min_x, max_x, ground_y) = if let Some(m) = &config.current_metrics {
        (m.min_x, m.max_x, m.ground_y)
    } else {
        (0.0, 1000.0, config.ground_y)
    };

    let current_pos = config.desktop_pos.unwrap_or(Vec2::new((min_x + max_x) * 0.5, ground_y));

    let seed = ((elapsed * 3.71).sin() * 43_758.547).fract().abs();

    // Turn inward when nearing screen boundaries
    let move_dir = if current_pos.x <= min_x + 90.0 {
        1.0 // Near left edge -> walk right
    } else if current_pos.x >= max_x - 90.0 || seed < 0.50 {
        -1.0 // Near right edge or random -> walk left
    } else {
        1.0
    };

    // Realistic feline stroll distances along desktop
    let dist = if seed < 0.40 {
        110.0 + seed * 260.0
    } else if seed < 0.80 {
        220.0 + (seed - 0.40) * 450.0
    } else {
        140.0 + (seed - 0.80) * 550.0
    };

    let target_x = (current_pos.x + move_dir * dist).clamp(min_x, max_x);
    config.desktop_target = Vec2::new(target_x, ground_y);

    config.state = CatAiState::Walking;
    config.state_timer = (dist / config.settings.speed.pixels_per_second() + 1.8).clamp(3.0, 8.5);
}

/// Moves the cat window smoothly across the desktop along the ground surface,
/// turns into the travel direction, and decelerates upon arrival.
fn update_roam_movement(
    time: Res<Time>,
    mut config: ResMut<CatConfig>,
    mut q_root: Query<&mut Transform, With<CatRoot>>,
    mut q_window: Query<&mut Window, With<PrimaryWindow>>,
) {
    let dt = time.delta_secs();

    let Ok(mut window) = q_window.single_mut() else {
        return;
    };

    // Ensure initial placement at bottom ground above the dock on first frame
    if !config.initial_spawned {
        if let Some(m) = &config.current_metrics {
            let initial_x = (m.min_x + m.max_x) * 0.5;
            let initial_pos = Vec2::new(initial_x, m.ground_y);
            window.position = WindowPosition::At(initial_pos.as_ivec2());
            config.desktop_pos = Some(initial_pos);
            config.initial_spawned = true;
        } else if let WindowPosition::At(pos) = window.position {
            config.desktop_pos = Some(pos.as_vec2());
            config.initial_spawned = true;
        }
    }

    // Keep desktop pos in sync with window if dragging
    if config.is_dragged {
        if let WindowPosition::At(pos) = window.position {
            config.desktop_pos = Some(pos.as_vec2());
        }
        return;
    }

    let Ok(mut transform) = q_root.single_mut() else {
        return;
    };

    let Some(current_pos) = config.desktop_pos else {
        return;
    };

    let (min_x, max_x, ground_y) = if let Some(m) = &config.current_metrics {
        (m.min_x, m.max_x, m.ground_y)
    } else {
        (0.0, 1000.0, config.ground_y)
    };

    if config.state == CatAiState::Walking {
        let to_target = config.desktop_target - current_pos;
        let dist = to_target.x.abs();

        // Arrival check: within 12 pixels of destination
        if dist < 12.0 {
            config.state = CatAiState::Stopping;
            config.state_timer = 0.40;
            return;
        }

        let direction_x = to_target.x.signum();
        let target_speed = config.settings.speed.pixels_per_second();

        // Smooth acceleration
        config.current_velocity.x = config.current_velocity.x.lerp(direction_x * target_speed, 6.0 * dt);

        let new_x = (current_pos.x + config.current_velocity.x * dt).clamp(min_x, max_x);
        let new_pos = Vec2::new(new_x, ground_y);
        config.desktop_pos = Some(new_pos);

        window.position = WindowPosition::At(new_pos.as_ivec2());

        // Facing direction: smooth banking rotation
        let target_angle = if direction_x > 0.0 {
            PI / 3.2
        } else {
            -PI / 3.2
        };
        let target_rot = Quat::from_rotation_y(target_angle);
        transform.rotation = transform.rotation.slerp(target_rot, 7.5 * dt);

        // Advance walk animation phase proportional to velocity
        let speed_ratio = config.current_velocity.x.abs() / 100.0;
        config.walk_phase += dt * (7.5 * speed_ratio).max(2.0);

        // Head looks forward along travel direction unless actively watching mouse
        if config.mouse_attention_timer <= 0.0 {
            config.look_target = Vec3::new(direction_x * 1.5, 0.45, 2.5);
        }
    } else if config.state == CatAiState::Stopping {
        // Decelerate smoothly
        config.current_velocity.x = config.current_velocity.x.lerp(0.0, 8.0 * dt);
        let new_x = (current_pos.x + config.current_velocity.x * dt).clamp(min_x, max_x);
        let new_pos = Vec2::new(new_x, ground_y);
        config.desktop_pos = Some(new_pos);
        window.position = WindowPosition::At(new_pos.as_ivec2());

        // Advance walk cycle to finish step
        let speed_ratio = config.current_velocity.x.abs() / 100.0;
        config.walk_phase += dt * (5.0 * speed_ratio);
    } else {
        // Resting / Idle / Sitting / Sleeping: face towards screen
        config.current_velocity = Vec2::ZERO;

        let face_screen_rot = Quat::IDENTITY;
        transform.rotation = transform.rotation.slerp(face_screen_rot, 3.8 * dt);

        // Reset walk phase smoothly to zero
        config.walk_phase = 0.0;

        // Apply updated position from gravity falling if applicable
        if config.state == CatAiState::Falling || config.state == CatAiState::Landing {
            if let Some(pos) = config.desktop_pos {
                window.position = WindowPosition::At(pos.as_ivec2());
            }
        }
    }
}

/// Spawns 3D floating hearts and zzz particles.
fn cat_particles_3d_system(
    mut commands: Commands,
    particle_assets: Option<Res<ParticleAssets>>,
    time: Res<Time>,
    mut config: ResMut<CatConfig>,
    q_root: Query<&Transform, With<CatRoot>>,
) {
    if !config.settings.enable_particles {
        return;
    }

    let Some(assets) = particle_assets else {
        return;
    };

    config.particle_timer -= time.delta_secs();

    let cat_pos = q_root
        .single()
        .map(|t| t.translation)
        .unwrap_or(Vec3::ZERO);

    match config.state {
        CatAiState::Petting => {
            if config.particle_timer <= 0.0 {
                config.particle_timer = 0.35;

                let offset_x = (time.elapsed_secs() * 6.0).sin() * 0.25;
                let spawn_pos = cat_pos + Vec3::new(offset_x, 1.15, 0.25);

                commands.spawn((
                    EmotionParticle3d {
                        lifetime: 1.3,
                        max_lifetime: 1.3,
                        velocity: Vec3::new(0.08, 0.75, 0.0),
                        initial_scale: Vec3::splat(1.0),
                    },
                    Mesh3d(assets.quad_mesh.clone()),
                    MeshMaterial3d(assets.heart_material.clone()),
                    Transform::from_translation(spawn_pos),
                ));
            }
        }
        CatAiState::Sleeping => {
            if config.particle_timer <= 0.0 {
                config.particle_timer = 0.95;

                let offset_x = 0.2 + (time.elapsed_secs() * 2.5).sin() * 0.15;
                let spawn_pos = cat_pos + Vec3::new(offset_x, 0.85, 0.20);

                commands.spawn((
                    EmotionParticle3d {
                        lifetime: 1.6,
                        max_lifetime: 1.6,
                        velocity: Vec3::new(0.10, 0.50, 0.0),
                        initial_scale: Vec3::splat(0.9),
                    },
                    Mesh3d(assets.quad_mesh.clone()),
                    MeshMaterial3d(assets.zzz_material.clone()),
                    Transform::from_translation(spawn_pos),
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

/// Updates 3D emotion particles: floating upward, facing camera, and shrinking on expiry.
fn update_particles_3d(
    mut commands: Commands,
    time: Res<Time>,
    q_camera: Query<&Transform, (With<Camera3d>, Without<EmotionParticle3d>)>,
    mut q_particles: Query<(Entity, &mut EmotionParticle3d, &mut Transform)>,
) {
    let dt = time.delta_secs();
    let camera_rot = q_camera.single().map(|t| t.rotation).unwrap_or(Quat::IDENTITY);

    for (entity, mut particle, mut transform) in &mut q_particles {
        particle.lifetime -= dt;
        if particle.lifetime <= 0.0 {
            commands.entity(entity).despawn();
            continue;
        }

        transform.translation += particle.velocity * dt;
        transform.rotation = camera_rot;

        let progress = (particle.lifetime / particle.max_lifetime).clamp(0.0, 1.0);
        transform.scale = particle.initial_scale * (0.35 + 0.65 * progress);
    }
}
