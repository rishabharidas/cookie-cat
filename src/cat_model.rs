//! 3D Procedural Chibi Cat Model
//!
//! Spawns an adorable, stylized 3D cat hierarchy using Bevy primitives
//! (spheres, capsules, cones) with PBR materials and joints for physics.

use bevy::prelude::*;
use crate::config::{CatConfig, CoatColor};

pub struct CatModelPlugin;

impl Plugin for CatModelPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, setup_cat_model)
            .add_systems(Update, update_cat_materials);
    }
}

/// Root marker for the cat character.
#[derive(Component)]
pub struct CatRoot;

/// Bone for squash-and-stretch dynamic bounciness.
#[derive(Component)]
pub struct CatSquash;

/// Cat head bone (can rotate to look around and follow cursor).
#[derive(Component)]
pub struct CatHead;

/// Marker for cat ears.
#[derive(Component)]
pub struct CatEar {
    pub is_left: bool,
    pub base_rotation: Quat,
}

/// Type of leg for walk cycle animation.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LegType {
    FrontLeft,
    FrontRight,
    BackLeft,
    BackRight,
}

/// Leg bone for walk cycle animation.
#[derive(Component)]
pub struct CatLeg {
    pub leg_type: LegType,
    pub rest_translation: Vec3,
}

/// Tail segment for spring-damper physics.
#[derive(Component)]
pub struct CatTailJoint {
    pub index: usize,
    pub current_angle: Vec2,
    pub angular_velocity: Vec2,
}

/// Handles for shared PBR materials.
#[derive(Resource)]
#[allow(dead_code)]
pub struct CatMaterials {
    pub body_material: Handle<StandardMaterial>,
    pub inner_ear_material: Handle<StandardMaterial>,
    pub nose_material: Handle<StandardMaterial>,
    pub eye_material: Handle<StandardMaterial>,
}

/// Spawns the complete 3D procedural cat model.
fn setup_cat_model(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    config: Res<CatConfig>,
) {
    // 1. Create shared PBR materials
    let body_material = materials.add(StandardMaterial {
        base_color: config.coat.body_color(),
        perceptual_roughness: 0.65,
        reflectance: 0.25,
        ..default()
    });

    let inner_ear_material = materials.add(StandardMaterial {
        base_color: config.coat.inner_ear_color(),
        perceptual_roughness: 0.70,
        ..default()
    });

    let nose_material = materials.add(StandardMaterial {
        base_color: config.coat.nose_color(),
        perceptual_roughness: 0.50,
        ..default()
    });

    let eye_material = materials.add(StandardMaterial {
        base_color: Color::srgb(0.08, 0.08, 0.10),
        perceptual_roughness: 0.15,
        reflectance: 0.85,
        ..default()
    });

    commands.insert_resource(CatMaterials {
        body_material: body_material.clone(),
        inner_ear_material: inner_ear_material.clone(),
        nose_material: nose_material.clone(),
        eye_material: eye_material.clone(),
    });

    // 2. Mesh assets
    let body_mesh = meshes.add(Sphere::new(0.48).mesh().ico(5).unwrap());
    let head_mesh = meshes.add(Sphere::new(0.42).mesh().ico(5).unwrap());
    let ear_mesh = meshes.add(Cone { radius: 0.13, height: 0.24 });
    let inner_ear_mesh = meshes.add(Cone { radius: 0.09, height: 0.18 });
    let eye_mesh = meshes.add(Sphere::new(0.052).mesh().ico(4).unwrap());
    let nose_mesh = meshes.add(Sphere::new(0.038).mesh().ico(4).unwrap());
    let leg_mesh = meshes.add(Capsule3d::new(0.11, 0.20));
    let tail_mesh = meshes.add(Sphere::new(0.10).mesh().ico(4).unwrap());

    // 3. Spawn Cat Hierarchy
    commands
        .spawn((
            CatRoot,
            Transform::from_xyz(0.0, 0.0, 0.0),
            Visibility::default(),
        ))
        .with_children(|root| {
            // Squash & stretch bone
            root.spawn((
                CatSquash,
                Transform::from_translation(Vec3::new(0.0, 0.55, 0.0))
                    .with_scale(Vec3::splat(config.size.scale_factor())),
                Visibility::default(),
            ))
            .with_children(|squash| {
                // Torso / Body (Chubby ellipsoid)
                squash.spawn((
                    Mesh3d(body_mesh.clone()),
                    MeshMaterial3d(body_material.clone()),
                    Transform::from_scale(Vec3::new(0.92, 0.88, 1.25)),
                ));

                // Head
                squash
                    .spawn((
                        CatHead,
                        Mesh3d(head_mesh.clone()),
                        MeshMaterial3d(body_material.clone()),
                        Transform::from_xyz(0.0, 0.38, 0.54),
                        Visibility::default(),
                    ))
                    .with_children(|head| {
                        // Left Ear
                        let left_ear_rot = Quat::from_euler(EulerRot::YXZ, 0.15, -0.22, -0.25);
                        head.spawn((
                            CatEar { is_left: true, base_rotation: left_ear_rot },
                            Mesh3d(ear_mesh.clone()),
                            MeshMaterial3d(body_material.clone()),
                            Transform::from_xyz(-0.21, 0.38, 0.05).with_rotation(left_ear_rot),
                        ))
                        .with_children(|ear| {
                            ear.spawn((
                                Mesh3d(inner_ear_mesh.clone()),
                                MeshMaterial3d(inner_ear_material.clone()),
                                Transform::from_xyz(0.0, -0.01, 0.04),
                            ));
                        });

                        // Right Ear
                        let right_ear_rot = Quat::from_euler(EulerRot::YXZ, -0.15, -0.22, 0.25);
                        head.spawn((
                            CatEar { is_left: false, base_rotation: right_ear_rot },
                            Mesh3d(ear_mesh.clone()),
                            MeshMaterial3d(body_material.clone()),
                            Transform::from_xyz(0.21, 0.38, 0.05).with_rotation(right_ear_rot),
                        ))
                        .with_children(|ear| {
                            ear.spawn((
                                Mesh3d(inner_ear_mesh.clone()),
                                MeshMaterial3d(inner_ear_material.clone()),
                                Transform::from_xyz(0.0, -0.01, 0.04),
                            ));
                        });

                        // Left Eye
                        head.spawn((
                            Mesh3d(eye_mesh.clone()),
                            MeshMaterial3d(eye_material.clone()),
                            Transform::from_xyz(-0.16, 0.03, 0.36),
                        ));

                        // Right Eye
                        head.spawn((
                            Mesh3d(eye_mesh.clone()),
                            MeshMaterial3d(eye_material.clone()),
                            Transform::from_xyz(0.16, 0.03, 0.36),
                        ));

                        // Nose
                        head.spawn((
                            Mesh3d(nose_mesh.clone()),
                            MeshMaterial3d(nose_material.clone()),
                            Transform::from_xyz(0.0, -0.05, 0.42),
                        ));
                    });

                // 4 Paws / Legs
                let legs = [
                    (LegType::FrontLeft, Vec3::new(-0.26, -0.32, 0.34)),
                    (LegType::FrontRight, Vec3::new(0.26, -0.32, 0.34)),
                    (LegType::BackLeft, Vec3::new(-0.28, -0.32, -0.34)),
                    (LegType::BackRight, Vec3::new(0.28, -0.32, -0.34)),
                ];

                for (leg_type, pos) in legs {
                    squash.spawn((
                        CatLeg { leg_type, rest_translation: pos },
                        Mesh3d(leg_mesh.clone()),
                        MeshMaterial3d(body_material.clone()),
                        Transform::from_translation(pos),
                    ));
                }

                // 3-Segment Physics Tail
                // Base Joint
                squash
                    .spawn((
                        CatTailJoint {
                            index: 0,
                            current_angle: Vec2::ZERO,
                            angular_velocity: Vec2::ZERO,
                        },
                        Mesh3d(tail_mesh.clone()),
                        MeshMaterial3d(body_material.clone()),
                        Transform::from_xyz(0.0, 0.08, -0.58),
                        Visibility::default(),
                    ))
                    .with_children(|tail0| {
                        // Mid Joint
                        tail0
                            .spawn((
                                CatTailJoint {
                                    index: 1,
                                    current_angle: Vec2::ZERO,
                                    angular_velocity: Vec2::ZERO,
                                },
                                Mesh3d(tail_mesh.clone()),
                                MeshMaterial3d(body_material.clone()),
                                Transform::from_xyz(0.0, 0.12, -0.16),
                                Visibility::default(),
                            ))
                            .with_children(|tail1| {
                                // Tip Joint
                                tail1.spawn((
                                    CatTailJoint {
                                        index: 2,
                                        current_angle: Vec2::ZERO,
                                        angular_velocity: Vec2::ZERO,
                                    },
                                    Mesh3d(tail_mesh.clone()),
                                    MeshMaterial3d(body_material.clone()),
                                    Transform::from_xyz(0.0, 0.14, -0.14)
                                        .with_scale(Vec3::splat(0.85)),
                                ));
                            });
                    });
            });
        });
}

/// Updates PBR material colors when coat color changes.
fn update_cat_materials(
    config: Res<CatConfig>,
    cat_materials: Option<Res<CatMaterials>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut last_coat: Local<Option<CoatColor>>,
) {
    let Some(cat_mats) = cat_materials else {
        return;
    };

    if last_coat.is_none() || *last_coat != Some(config.coat) {
        *last_coat = Some(config.coat);

        // Update body color
        if let Some(mut mat) = materials.get_mut(&cat_mats.body_material) {
            mat.base_color = config.coat.body_color();
        }

        // Update inner ear color
        if let Some(mut mat) = materials.get_mut(&cat_mats.inner_ear_material) {
            mat.base_color = config.coat.inner_ear_color();
        }
    }
}
