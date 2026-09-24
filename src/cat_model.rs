//! 2D Cat Model System
//!
//! Spawns the 2D sprite for the desktop cat.

use bevy::prelude::*;

pub struct CatModelPlugin;

impl Plugin for CatModelPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, setup_cat_model);
    }
}

/// Root marker for the cat character.
#[derive(Component)]
pub struct CatRoot;

/// Marker for squash-and-stretch dynamic bounciness.
#[derive(Component)]
pub struct CatSquash;

/// Marker for the cat sprite so it can be flipped.
#[derive(Component)]
pub struct CatSprite;

/// Spawns the root entity and the 2D sprite.
fn setup_cat_model(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
) {
    let texture_handle = asset_server.load("feature-desktop-pet-tail-01.png");

    // Root entity for logical positioning/movement
    commands
        .spawn((
            CatRoot,
            Transform::default(),
            Visibility::default(),
        ))
        .with_children(|root| {
            // Squash bone for physics
            root.spawn((
                CatSquash,
                Transform::default(),
                Visibility::default(),
            ))
            .with_children(|squash| {
                // 2D Sprite
                squash.spawn((
                    CatSprite,
                    Sprite {
                        image: texture_handle,
                        custom_size: Some(Vec2::new(180.0, 180.0)),
                        ..default()
                    },
                    Transform::default(),
                ));
            });
        });
}
