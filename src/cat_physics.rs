//! Physics & Secondary Motion Systems
//!
//! Implements:
//! 1. Squash-and-stretch dynamic bounciness for clicks and breathing.

use bevy::prelude::*;
use crate::cat_model::CatSquash;
use crate::config::{CatAiState, CatConfig};

pub struct CatPhysicsPlugin;

impl Plugin for CatPhysicsPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<SquashPhysics>()
            .add_systems(
                Update,
                (
                    update_squash_and_stretch,
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

/// Dynamic squash-and-stretch: plush bouncy deformation for interactions and breathing.
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
    // Volume preservation: when squashing in Y, expand in X
    let dynamic_x = 1.0 - (breath * 0.5) - (squash_phys.scale_offset.y * 0.45);

    transform.scale = Vec3::new(
        base_scale * dynamic_x,
        base_scale * dynamic_y,
        1.0,
    );
}
