//! 3D Cat Model System
//!
//! Supports:
//! 1. Loading realistic 3D models exported from Blender (`assets/models/cat.glb`).
//! 2. Procedural stylized chibi cat fallback.
//! 3. Dynamic runtime switching between models (press 'M').
//! 4. Automatic screen-facing orientation and head tracking.

use std::f32::consts::PI;
use bevy::gltf::GltfAssetLabel;
use bevy::prelude::*;
use crate::config::{CatConfig, CoatColor, ModelType};

pub struct CatModelPlugin;

impl Plugin for CatModelPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, setup_cat_model)
            .add_systems(Update, (sync_cat_model, update_cat_materials, attach_head_to_gltf));
    }
}

/// Root marker for the cat character.
#[derive(Component)]
pub struct CatRoot;

/// Bone for squash-and-stretch dynamic bounciness.
#[derive(Component)]
pub struct CatSquash;

/// Cat head bone (rotates to look around and follow cursor).
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

/// Marker for the active model root under squash.
#[derive(Component)]
pub struct ActiveCatModelRoot;

/// Handles for shared PBR materials.
#[derive(Resource)]
pub struct CatMaterials {
    pub body_material: Handle<StandardMaterial>,
    pub inner_ear_material: Handle<StandardMaterial>,
    pub nose_material: Handle<StandardMaterial>,
    pub eye_material: Handle<StandardMaterial>,
    pub eye_shine_material: Handle<StandardMaterial>,
    pub whisker_material: Handle<StandardMaterial>,
}

/// Spawns the root and initial cat model.
fn setup_cat_model(
    mut commands: Commands,
    mut materials: ResMut<Assets<StandardMaterial>>,
    config: Res<CatConfig>,
) {
    // 1. Create shared PBR materials
    let body_material = materials.add(StandardMaterial {
        base_color: config.coat.body_color(),
        perceptual_roughness: 0.60,
        reflectance: 0.20,
        ..default()
    });

    let inner_ear_material = materials.add(StandardMaterial {
        base_color: config.coat.inner_ear_color(),
        perceptual_roughness: 0.70,
        ..default()
    });

    let nose_material = materials.add(StandardMaterial {
        base_color: config.coat.nose_color(),
        perceptual_roughness: 0.40,
        ..default()
    });

    let eye_material = materials.add(StandardMaterial {
        base_color: Color::srgb(0.06, 0.06, 0.08),
        perceptual_roughness: 0.10,
        reflectance: 0.90,
        ..default()
    });

    let eye_shine_material = materials.add(StandardMaterial {
        base_color: Color::WHITE,
        unlit: true,
        ..default()
    });

    let whisker_material = materials.add(StandardMaterial {
        base_color: Color::srgb(0.35, 0.32, 0.30),
        perceptual_roughness: 0.50,
        ..default()
    });

    let cat_materials = CatMaterials {
        body_material,
        inner_ear_material,
        nose_material,
        eye_material,
        eye_shine_material,
        whisker_material,
    };
    commands.insert_resource(cat_materials);

    // 2. Spawn Cat Hierarchy
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
            ));
        });
}

/// Spawns the 3D model exported from Blender (cat.glb).
fn spawn_gltf_model(parent: &mut ChildSpawnerCommands, asset_server: &AssetServer) {
    let scene_handle: Handle<WorldAsset> =
        asset_server.load(GltfAssetLabel::Scene(0).from_asset("models/cat.glb"));

    parent
        .spawn((
            ActiveCatModelRoot,
            // Rotate 180 degrees around Y so the model faces the camera/screen (+Z)
            Transform::from_xyz(0.0, -0.45, 0.0).with_rotation(Quat::from_rotation_y(PI)),
            Visibility::default(),
        ))
        .with_children(|gltf_root| {
            // Center and scale assets/models/cat.glb (which has ~30 unit dimensions)
            // Model bounding box is centered at X: ~14.7, Z: ~-14.2
            let scale = 0.042;
            gltf_root.spawn((
                WorldAssetRoot(scene_handle),
                Transform::from_translation(Vec3::new(-14.7 * scale, 0.0, 14.2 * scale))
                    .with_scale(Vec3::splat(scale)),
            ));
        });
}

/// Spawns the cute procedural cat model (faces +Z towards the screen).
fn spawn_procedural_model(
    parent: &mut ChildSpawnerCommands,
    meshes: &mut Assets<Mesh>,
    materials: &CatMaterials,
) {
    let body_mesh = meshes.add(Sphere::new(0.46).mesh().ico(5).unwrap());
    let head_mesh = meshes.add(Sphere::new(0.40).mesh().ico(5).unwrap());
    let ear_mesh = meshes.add(Cone { radius: 0.12, height: 0.22 });
    let inner_ear_mesh = meshes.add(Cone { radius: 0.08, height: 0.16 });
    let cheek_mesh = meshes.add(Sphere::new(0.09).mesh().ico(4).unwrap());
    let eye_mesh = meshes.add(Sphere::new(0.050).mesh().ico(4).unwrap());
    let eye_shine_mesh = meshes.add(Sphere::new(0.016).mesh().ico(3).unwrap());
    let nose_mesh = meshes.add(Sphere::new(0.032).mesh().ico(4).unwrap());
    let leg_mesh = meshes.add(Capsule3d::new(0.10, 0.22));
    let paw_mesh = meshes.add(Sphere::new(0.11).mesh().ico(4).unwrap());
    let tail_mesh = meshes.add(Sphere::new(0.09).mesh().ico(4).unwrap());
    let whisker_mesh = meshes.add(Cylinder::new(0.004, 0.22));

    parent
        .spawn((
            ActiveCatModelRoot,
            Transform::IDENTITY,
            Visibility::default(),
        ))
        .with_children(|model_root| {
            // Torso / Body (Chubby ellipsoid, slightly tilted forward)
            model_root.spawn((
                Mesh3d(body_mesh.clone()),
                MeshMaterial3d(materials.body_material.clone()),
                Transform::from_scale(Vec3::new(0.92, 0.86, 1.20))
                    .with_rotation(Quat::from_rotation_x(0.06)),
            ));

            // Head (Positioned forward at +Z so it faces the screen)
            model_root
                .spawn((
                    CatHead,
                    Mesh3d(head_mesh.clone()),
                    MeshMaterial3d(materials.body_material.clone()),
                    Transform::from_xyz(0.0, 0.32, 0.48),
                    Visibility::default(),
                ))
                .with_children(|head| {
                    // Left Ear
                    let left_ear_rot = Quat::from_euler(EulerRot::YXZ, 0.18, -0.18, -0.28);
                    head.spawn((
                        CatEar { is_left: true, base_rotation: left_ear_rot },
                        Mesh3d(ear_mesh.clone()),
                        MeshMaterial3d(materials.body_material.clone()),
                        Transform::from_xyz(-0.20, 0.34, 0.04).with_rotation(left_ear_rot),
                    ))
                    .with_children(|ear| {
                        ear.spawn((
                            Mesh3d(inner_ear_mesh.clone()),
                            MeshMaterial3d(materials.inner_ear_material.clone()),
                            Transform::from_xyz(0.0, -0.01, 0.03),
                        ));
                    });

                    // Right Ear
                    let right_ear_rot = Quat::from_euler(EulerRot::YXZ, -0.18, -0.18, 0.28);
                    head.spawn((
                        CatEar { is_left: false, base_rotation: right_ear_rot },
                        Mesh3d(ear_mesh.clone()),
                        MeshMaterial3d(materials.body_material.clone()),
                        Transform::from_xyz(0.20, 0.34, 0.04).with_rotation(right_ear_rot),
                    ))
                    .with_children(|ear| {
                        ear.spawn((
                            Mesh3d(inner_ear_mesh.clone()),
                            MeshMaterial3d(materials.inner_ear_material.clone()),
                            Transform::from_xyz(0.0, -0.01, 0.03),
                        ));
                    });

                    // Left Cheek
                    head.spawn((
                        Mesh3d(cheek_mesh.clone()),
                        MeshMaterial3d(materials.body_material.clone()),
                        Transform::from_xyz(-0.09, -0.07, 0.34).with_scale(Vec3::new(1.1, 0.9, 0.9)),
                    ));

                    // Right Cheek
                    head.spawn((
                        Mesh3d(cheek_mesh.clone()),
                        MeshMaterial3d(materials.body_material.clone()),
                        Transform::from_xyz(0.09, -0.07, 0.34).with_scale(Vec3::new(1.1, 0.9, 0.9)),
                    ));

                    // Left Eye
                    head.spawn((
                        Mesh3d(eye_mesh.clone()),
                        MeshMaterial3d(materials.eye_material.clone()),
                        Transform::from_xyz(-0.15, 0.04, 0.35),
                    ))
                    .with_children(|eye| {
                        // Eye shine highlight
                        eye.spawn((
                            Mesh3d(eye_shine_mesh.clone()),
                            MeshMaterial3d(materials.eye_shine_material.clone()),
                            Transform::from_xyz(0.014, 0.016, 0.040),
                        ));
                    });

                    // Right Eye
                    head.spawn((
                        Mesh3d(eye_mesh.clone()),
                        MeshMaterial3d(materials.eye_material.clone()),
                        Transform::from_xyz(0.15, 0.04, 0.35),
                    ))
                    .with_children(|eye| {
                        // Eye shine highlight
                        eye.spawn((
                            Mesh3d(eye_shine_mesh.clone()),
                            MeshMaterial3d(materials.eye_shine_material.clone()),
                            Transform::from_xyz(0.014, 0.016, 0.040),
                        ));
                    });

                    // Nose
                    head.spawn((
                        Mesh3d(nose_mesh.clone()),
                        MeshMaterial3d(materials.nose_material.clone()),
                        Transform::from_xyz(0.0, -0.04, 0.40),
                    ));

                    // Whiskers (Left & Right)
                    let whisker_rot_l1 = Quat::from_euler(EulerRot::YXZ, -0.2, 0.0, 1.45);
                    let whisker_rot_l2 = Quat::from_euler(EulerRot::YXZ, -0.2, 0.0, 1.70);
                    head.spawn((
                        Mesh3d(whisker_mesh.clone()),
                        MeshMaterial3d(materials.whisker_material.clone()),
                        Transform::from_xyz(-0.20, -0.04, 0.30).with_rotation(whisker_rot_l1),
                    ));
                    head.spawn((
                        Mesh3d(whisker_mesh.clone()),
                        MeshMaterial3d(materials.whisker_material.clone()),
                        Transform::from_xyz(-0.20, -0.08, 0.30).with_rotation(whisker_rot_l2),
                    ));

                    let whisker_rot_r1 = Quat::from_euler(EulerRot::YXZ, 0.2, 0.0, -1.45);
                    let whisker_rot_r2 = Quat::from_euler(EulerRot::YXZ, 0.2, 0.0, -1.70);
                    head.spawn((
                        Mesh3d(whisker_mesh.clone()),
                        MeshMaterial3d(materials.whisker_material.clone()),
                        Transform::from_xyz(0.20, -0.04, 0.30).with_rotation(whisker_rot_r1),
                    ));
                    head.spawn((
                        Mesh3d(whisker_mesh.clone()),
                        MeshMaterial3d(materials.whisker_material.clone()),
                        Transform::from_xyz(0.20, -0.08, 0.30).with_rotation(whisker_rot_r2),
                    ));
                });

            // 4 Paws / Legs
            let legs = [
                (LegType::FrontLeft, Vec3::new(-0.24, -0.32, 0.30)),
                (LegType::FrontRight, Vec3::new(0.24, -0.32, 0.30)),
                (LegType::BackLeft, Vec3::new(-0.26, -0.32, -0.30)),
                (LegType::BackRight, Vec3::new(0.26, -0.32, -0.30)),
            ];

            for (leg_type, pos) in legs {
                model_root
                    .spawn((
                        CatLeg { leg_type, rest_translation: pos },
                        Mesh3d(leg_mesh.clone()),
                        MeshMaterial3d(materials.body_material.clone()),
                        Transform::from_translation(pos),
                    ))
                    .with_children(|leg| {
                        // Cute foot paw at base
                        leg.spawn((
                            Mesh3d(paw_mesh.clone()),
                            MeshMaterial3d(materials.body_material.clone()),
                            Transform::from_xyz(0.0, -0.11, 0.03).with_scale(Vec3::new(1.0, 0.65, 1.15)),
                        ));
                    });
            }

            // 3-Segment Physics Tail
            model_root
                .spawn((
                    CatTailJoint {
                        index: 0,
                        current_angle: Vec2::ZERO,
                        angular_velocity: Vec2::ZERO,
                    },
                    Mesh3d(tail_mesh.clone()),
                    MeshMaterial3d(materials.body_material.clone()),
                    Transform::from_xyz(0.0, 0.08, -0.54),
                    Visibility::default(),
                ))
                .with_children(|tail0| {
                    tail0
                        .spawn((
                            CatTailJoint {
                                index: 1,
                                current_angle: Vec2::ZERO,
                                angular_velocity: Vec2::ZERO,
                            },
                            Mesh3d(tail_mesh.clone()),
                            MeshMaterial3d(materials.body_material.clone()),
                            Transform::from_xyz(0.0, 0.10, -0.14),
                            Visibility::default(),
                        ))
                        .with_children(|tail1| {
                            tail1.spawn((
                                CatTailJoint {
                                    index: 2,
                                    current_angle: Vec2::ZERO,
                                    angular_velocity: Vec2::ZERO,
                                },
                                Mesh3d(tail_mesh.clone()),
                                MeshMaterial3d(materials.body_material.clone()),
                                Transform::from_xyz(0.0, 0.12, -0.12).with_scale(Vec3::splat(0.85)),
                            ));
                        });
                });
        });
}

/// Keeps active model in sync with config.model_type.
fn sync_cat_model(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    mut meshes: ResMut<Assets<Mesh>>,
    cat_materials: Option<Res<CatMaterials>>,
    config: Res<CatConfig>,
    mut last_model_type: Local<Option<ModelType>>,
    q_squash: Query<Entity, With<CatSquash>>,
    q_active_models: Query<Entity, With<ActiveCatModelRoot>>,
) {
    if *last_model_type == Some(config.model_type) {
        return;
    }
    *last_model_type = Some(config.model_type);

    let Ok(squash_entity) = q_squash.single() else {
        return;
    };

    // Despawn previous model children
    for model_entity in &q_active_models {
        commands.entity(model_entity).despawn();
    }

    let Some(materials) = cat_materials else {
        return;
    };

    commands.entity(squash_entity).with_children(|squash| {
        match config.model_type {
            ModelType::Gltf => spawn_gltf_model(squash, &asset_server),
            ModelType::Procedural => spawn_procedural_model(squash, &mut meshes, &materials),
        }
    });
}

/// Automatically attaches CatHead component to any node with 'head' in its name from a loaded Blender armature.
fn attach_head_to_gltf(
    mut commands: Commands,
    q_nodes: Query<(Entity, &Name), (Without<CatHead>, Added<Name>)>,
) {
    for (entity, name) in &q_nodes {
        let name_str = name.as_str().to_lowercase();
        if name_str.contains("head") {
            commands.entity(entity).insert(CatHead);
        }
    }
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

        if let Some(mut mat) = materials.get_mut(&cat_mats.body_material) {
            mat.base_color = config.coat.body_color();
        }

        if let Some(mut mat) = materials.get_mut(&cat_mats.inner_ear_material) {
            mat.base_color = config.coat.inner_ear_color();
        }
    }
}
