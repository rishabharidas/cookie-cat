//! Cat AI & Autonomous Roaming Systems
//!
//! Controls the cat's autonomous behavior: wandering around the screen,
//! looking around curiously, sitting, napping, reacting to petting,
//! and always returning to face the screen when resting.

use bevy::prelude::*;
use bevy::window::{PrimaryWindow, WindowPosition};
use mouse_position::mouse_position::Mouse;
use crate::cat_model::CatRoot;
use crate::config::{CatAiState, CatConfig};

pub struct CatAiPlugin;

impl Plugin for CatAiPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, setup_particle_assets)
            .add_systems(
                Update,
                (
                    update_global_mouse,
                    update_cat_ai,
                    update_roam_movement,
                    cat_particles_3d_system,
                    update_particles_3d,
                ),
            );
    }
}

fn update_global_mouse(
    time: Res<Time>,
    mut config: ResMut<CatConfig>,
) {
    let dt = time.delta_secs();

    // Get mouse position
    let mut current_pos = config.last_mouse_pos;
    if let Mouse::Position { x, y } = Mouse::get_mouse_position() {
        current_pos = Vec2::new(x as f32, y as f32);
    }

    if current_pos.distance(config.last_mouse_pos) > 5.0 {
        config.mouse_idle_timer = 0.0;
        config.last_mouse_pos = current_pos;
    } else {
        config.mouse_idle_timer += dt;
    }

    config.global_mouse_pos = current_pos;
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

/// High-level AI state machine: decides when to walk, sit, sniff, or nap.
fn update_cat_ai(
    time: Res<Time>,
    mut config: ResMut<CatConfig>,
    q_window: Query<&Window, With<PrimaryWindow>>,
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

    // Check if mouse has been idle for > 6 seconds
    if config.mouse_idle_timer > 6.0 {
        if config.state != CatAiState::SeekingMouse && config.state != CatAiState::Sleeping && config.state != CatAiState::Petting {
            config.state = CatAiState::SeekingMouse;
            config.state_timer = 10.0;
        }
    } else if config.state == CatAiState::SeekingMouse || config.state == CatAiState::Sleeping {
        // Mouse moved! Wake up or stop seeking
        if config.state == CatAiState::Sleeping {
            println!("Cat woke up!");
        }
        config.state = CatAiState::Idle;
        config.state_timer = 2.5;
    }

    // If sleeping, do not randomly roam
    if config.state == CatAiState::Sleeping {
        return;
    }

    // In seeking mode, continuously update target to mouse pos (offset to be centered)
    if config.state == CatAiState::SeekingMouse {
        // Center the window (which is 480x340) on the mouse pointer
        config.desktop_target = config.global_mouse_pos - Vec2::new(240.0, 170.0);
        return;
    }

    config.state_timer -= dt;

    if config.state_timer <= 0.0 {
        match config.state {
            CatAiState::Idle | CatAiState::Sniffing | CatAiState::Sitting => {
                // Generate a pseudo-random new destination on the desktop
                let seed = time.elapsed_secs();
                let current_pos = config.desktop_pos.unwrap_or_else(|| {
                    q_window.single().map(|w| {
                        if let WindowPosition::At(pos) = w.position {
                            pos.as_vec2()
                        } else {
                            Vec2::ZERO
                        }
                    }).unwrap_or(Vec2::ZERO)
                });

                // Move left or right, and keep bottom of screen
                let dx = ((seed * 1.33).sin() * 600.0) - 300.0;

                // Assume 1080p height, window height is 340
                let bottom_y = 1080.0 - 340.0;

                config.desktop_target = current_pos + Vec2::new(dx, 0.0);

                // Roughly clamp to typical screen bounds so it doesn't wander off forever
                config.desktop_target.x = config.desktop_target.x.clamp(0.0, 1920.0 - 480.0);
                config.desktop_target.y = bottom_y;

                let action_seed = (time.elapsed_secs() * 3.1).cos().abs();
                if action_seed < 0.25 {
                    config.state = CatAiState::Running;
                    config.state_timer = 4.0;
                } else if action_seed < 0.40 {
                    config.state = CatAiState::Jumping;
                    config.state_timer = 2.0;
                } else {
                    config.state = CatAiState::Walking;
                    config.state_timer = 6.0;
                }
            }
            CatAiState::Walking | CatAiState::Running | CatAiState::Jumping => {
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

/// Moves the window and turns the cat to face the walking direction.
/// When resting/idle, smoothly faces the screen towards the user.
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

    // Initialize float pos if none
    if config.desktop_pos.is_none() {
        if let WindowPosition::At(pos) = window.position {
            config.desktop_pos = Some(pos.as_vec2());
        } else {
            config.desktop_pos = Some(Vec2::ZERO);
        }
    }

    let Ok(mut transform) = q_root.single_mut() else {
        return;
    };

    if config.state == CatAiState::Walking || config.state == CatAiState::Running || config.state == CatAiState::Jumping || config.state == CatAiState::SeekingMouse {
        let current_pos = config.desktop_pos.unwrap();
        let to_target = config.desktop_target - current_pos;
        let dist = to_target.length();

        // Check if reached destination
        if dist < 5.0 {
            if config.state == CatAiState::SeekingMouse {
                config.state = CatAiState::Petting;
                config.petting_timer = 2.0;
                config.state_timer = 4.0;
            } else {
                config.state = CatAiState::Idle;
                config.state_timer = 3.0;
            }
            return;
        }

        // Determine orientation: if moving right, face right (pure 2D side view). If left, face left.
        let target_angle = if to_target.x > 0.0 {
            std::f32::consts::PI / 2.0 // Facing right directly
        } else {
            -std::f32::consts::PI / 2.0 // Facing left directly
        };

        let target_rot = Quat::from_rotation_y(target_angle);
        transform.rotation = transform.rotation.slerp(target_rot, 7.0 * dt);

        // Determine move speed based on state
        let move_speed = match config.state {
            CatAiState::Running => 250.0,
            CatAiState::SeekingMouse => 200.0,
            _ => 100.0, // Walking or Jumping base horizontal speed
        };

        let direction = to_target.normalize_or_zero();
        let new_pos = current_pos + direction * move_speed * dt;

        // Jumping animation (vertical window bounce)
        if config.state == CatAiState::Jumping {
            // Parabola based on state timer (2.0 down to 0.0)
            let progress = (2.0 - config.state_timer) / 2.0; // 0 to 1
            let jump_height = (progress * std::f32::consts::PI).sin() * 80.0;
            // new_pos.y is modified to bounce up, but target y is at the bottom.
            // Actually, modifying window position Y for jump can be jarring.
            // Instead, we just adjust the translation of the cat root.
            transform.translation.y = jump_height * 0.01; // scale pixel jump to world space roughly
        } else {
            // Smoothly return translation to 0
            transform.translation.y = transform.translation.y * (1.0 - 5.0 * dt);
        }

        config.desktop_pos = Some(new_pos);
        window.position = WindowPosition::At(new_pos.as_ivec2());

        // Advance walk cycle animation phase faster if running
        let walk_speed = if config.state == CatAiState::Running { 15.0 } else { 8.5 };
        config.walk_phase += dt * walk_speed;

        // Look in direction of travel (in local cat space)
        config.look_target = transform.translation + Vec3::new(direction.x, 0.5, 3.0);
    } else {
        // When not moving (Idle, Sitting, Sniffing, Sleeping, Petting):
        // Smoothly turn to face the screen/user (towards +Z / camera)
        let face_screen_rot = Quat::IDENTITY;
        transform.rotation = transform.rotation.slerp(face_screen_rot, 3.5 * dt);

        // Smoothly return Y translation to 0 (in case we stopped jumping)
        transform.translation.y = transform.translation.y * (1.0 - 5.0 * dt);

        // Reset walk phase smoothly
        config.walk_phase = 0.0;

        // Ensure desktop pos stays in sync with window if user drags it
        if let WindowPosition::At(pos) = window.position {
            config.desktop_pos = Some(pos.as_vec2());
        }
    }
}

/// Spawns 3D floating hearts and zzz particles using billboard quads in 3D.
fn cat_particles_3d_system(
    mut commands: Commands,
    particle_assets: Option<Res<ParticleAssets>>,
    time: Res<Time>,
    mut config: ResMut<CatConfig>,
    q_root: Query<&Transform, With<CatRoot>>,
) {
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
                config.particle_timer = 0.9;

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
        // Billboard facing camera
        transform.rotation = camera_rot;

        // Shrink smoothly as lifetime expires
        let progress = (particle.lifetime / particle.max_lifetime).clamp(0.0, 1.0);
        transform.scale = particle.initial_scale * (0.35 + 0.65 * progress);
    }
}
