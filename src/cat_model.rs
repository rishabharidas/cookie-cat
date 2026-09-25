//! Cat Model Module
//!
//! Handles rendering the 2D sprite of the cat and flipping it.

use bevy::{
    prelude::*,
};
use crate::config::{CatConfig, CoatColor};

pub struct CatModelPlugin;

impl Plugin for CatModelPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, setup_cat_model)
            .add_systems(Update, update_cat_sprite_flip);
    }
}

/// Root marker for the whole cat hierarchy
#[derive(Component)]
pub struct CatRoot;

#[derive(Resource)]
pub struct CatSprite {
    pub biscuit: Handle<Image>,
    pub white: Handle<Image>,
    pub grey: Handle<Image>,
}

fn setup_cat_model(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    config: Res<CatConfig>,
) {
    let biscuit_handle = asset_server.load("cat/cat_biscuit.png");
    let white_handle = asset_server.load("cat/cat_white.png");
    let grey_handle = asset_server.load("cat/cat_grey.png");

    let active_texture = match config.coat {
        CoatColor::Biscuit => biscuit_handle.clone(),
        CoatColor::White => white_handle.clone(),
        CoatColor::Grey => grey_handle.clone(),
    };

    commands.insert_resource(CatSprite {
        biscuit: biscuit_handle.clone(),
        white: white_handle.clone(),
        grey: grey_handle.clone(),
    });

    // Adjust scale to fit the window appropriately for a 2D sprite
    // Base scale is smaller since 1 unit = 1 pixel. Image is 370x270.
    commands
        .spawn((
            CatRoot,
            Sprite::from_image(active_texture),
            Transform::from_scale(Vec3::splat(config.size.scale_factor() * 0.4)),
        ));
}

fn update_cat_sprite_flip(
    config: Res<CatConfig>,
    cat_sprite: Option<Res<CatSprite>>,
    mut q_sprite: Query<(&mut Sprite, &Transform), With<CatRoot>>,
) {
    let Some(sprites) = cat_sprite else { return };
    for (mut sprite, transform) in &mut q_sprite {
        let expected_texture = match config.coat {
            CoatColor::Biscuit => &sprites.biscuit,
            CoatColor::White => &sprites.white,
            CoatColor::Grey => &sprites.grey,
        };

        if sprite.image != *expected_texture {
            sprite.image = expected_texture.clone();
        }

        // Face direction based on look_target X relative to current pos
        if config.look_target.x < transform.translation.x {
            sprite.flip_x = true;
        } else {
            sprite.flip_x = false;
        }
    }
}
