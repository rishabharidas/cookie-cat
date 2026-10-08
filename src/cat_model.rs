//! 3D Cat Model System
//!
//! Procedural stylized anime/webtoon cat model inspired by Kayden (Eleceed):
//! 1. Expressive articulated head with winged anime feline eyes, luminous glowing irises,
//!    forehead tabby stripes, iconic white blaze, sweet pink nose, and stylized mouth lines.
//! 2. Chonky rounded body with a broad white chest bib, underbelly markings, and tiger flank stripes.
//! 3. 4 articulated white mitten paws with cute pink toe beans.
//! 4. 3-segment cascading physics tail with ginger base, tabby mid-stripe, and fluffy white tip.
//! 5. Dynamic squash-and-stretch bone for breathing, purring, sitting posture, and impact absorption.
//! 6. Dual-tone PBR coat customization supporting Kayden (Ginger Tabby), Carte (Snow White), and Smokey.

use bevy::prelude::*;
use crate::config::{CatConfig, CoatColor};

pub struct CatModelPlugin;

impl Plugin for CatModelPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, setup_cat_model)
            .add_systems(Update, (sync_cat_model, update_cat_materials));
    }
}

/// Root marker for the cat character.
#[derive(Component)]
pub struct CatRoot;

/// Bone for squash-and-stretch dynamic bounciness and posture crouch.
#[derive(Component)]
pub struct CatSquash;

/// Cat head bone (rotates to look around, tilt curiously, and follow cursor).
#[derive(Component)]
pub struct CatHead;

/// Marker for cat eyes (for natural blinking and sleepy closed eyes).
#[derive(Component)]
pub struct CatEye {
    #[allow(dead_code)]
    pub is_left: bool,
    pub rest_scale: Vec3,
}

/// Winged dark anime eyeliner / upper lid contour that moves with blinks and expressions.
#[derive(Component)]
pub struct CatEyeWing {
    #[allow(dead_code)]
    pub is_left: bool,
    pub rest_scale: Vec3,
}

/// Marker for the colored iris ring — dilates/contracts per emotional state.
#[derive(Component)]
pub struct CatEyeIris {
    #[allow(dead_code)]
    pub is_left: bool,
    pub rest_scale: Vec3,
}

/// Marker for the vertical-slit pupil — expands when curious or startled.
#[derive(Component)]
pub struct CatEyePupil {
    #[allow(dead_code)]
    pub is_left: bool,
    pub rest_scale: Vec3,
}

/// Marker for sweet pink blush on cheeks that fades in during petting.
#[derive(Component)]
pub struct CatBlush {
    #[allow(dead_code)]
    pub is_left: bool,
    pub rest_scale: Vec3,
}

/// Marker for cat whiskers (for sniffing and curiosity vibrations).
#[derive(Component)]
pub struct CatWhisker {
    #[allow(dead_code)]
    pub is_left: bool,
    pub base_rotation: Quat,
}

/// Marker and physics state for cat ears.
#[derive(Component)]
pub struct CatEar {
    pub is_left: bool,
    pub base_rotation: Quat,
    pub dynamic_angle: f32,
    pub dynamic_velocity: f32,
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
    pub stripe_material: Handle<StandardMaterial>,
    pub white_fur_material: Handle<StandardMaterial>,
    pub inner_ear_material: Handle<StandardMaterial>,
    pub nose_material: Handle<StandardMaterial>,
    pub eye_contour_material: Handle<StandardMaterial>,
    pub eye_white_material: Handle<StandardMaterial>,
    pub eye_shine_material: Handle<StandardMaterial>,
    pub whisker_material: Handle<StandardMaterial>,
    pub iris_material: Handle<StandardMaterial>,
    pub pupil_material: Handle<StandardMaterial>,
    pub mouth_material: Handle<StandardMaterial>,
    pub paw_pad_material: Handle<StandardMaterial>,
    pub blush_material: Handle<StandardMaterial>,
}

/// Spawns the root and initial cat hierarchy.
fn setup_cat_model(
    mut commands: Commands,
    mut materials: ResMut<Assets<StandardMaterial>>,
    config: Res<CatConfig>,
) {
    let body_material = materials.add(StandardMaterial {
        base_color: config.settings.coat.body_color(),
        perceptual_roughness: 0.72,
        reflectance: 0.18,
        ..default()
    });

    let stripe_material = materials.add(StandardMaterial {
        base_color: config.settings.coat.stripe_color(),
        perceptual_roughness: 0.74,
        reflectance: 0.18,
        ..default()
    });

    let white_fur_material = materials.add(StandardMaterial {
        base_color: config.settings.coat.white_fur_color(),
        perceptual_roughness: 0.65,
        reflectance: 0.22,
        ..default()
    });

    let inner_ear_material = materials.add(StandardMaterial {
        base_color: config.settings.coat.inner_ear_color(),
        perceptual_roughness: 0.60,
        reflectance: 0.20,
        ..default()
    });

    let nose_material = materials.add(StandardMaterial {
        base_color: config.settings.coat.nose_color(),
        perceptual_roughness: 0.35,
        reflectance: 0.45,
        ..default()
    });

    let eye_contour_material = materials.add(StandardMaterial {
        base_color: Color::srgb(0.08, 0.06, 0.09),
        perceptual_roughness: 0.20,
        reflectance: 0.50,
        ..default()
    });

    let eye_white_material = materials.add(StandardMaterial {
        base_color: Color::srgb(0.99, 0.99, 1.0),
        perceptual_roughness: 0.15,
        reflectance: 0.85,
        ..default()
    });

    let eye_shine_material = materials.add(StandardMaterial {
        base_color: Color::WHITE,
        unlit: true,
        ..default()
    });

    let whisker_material = materials.add(StandardMaterial {
        base_color: Color::srgb(0.32, 0.28, 0.26),
        perceptual_roughness: 0.40,
        ..default()
    });

    let iris_col = config.settings.coat.iris_color();
    let iris_material = materials.add(StandardMaterial {
        base_color: iris_col,
        emissive: match iris_col {
            Color::Srgba(s) => LinearRgba::new(s.red * 0.25, s.green * 0.25, s.blue * 0.25, 1.0),
            _ => LinearRgba::new(0.08, 0.32, 0.28, 1.0),
        },
        perceptual_roughness: 0.05,
        reflectance: 0.95,
        ..default()
    });

    let pupil_material = materials.add(StandardMaterial {
        base_color: Color::srgb(0.03, 0.02, 0.04),
        perceptual_roughness: 0.08,
        reflectance: 0.85,
        ..default()
    });

    let mouth_material = materials.add(StandardMaterial {
        base_color: Color::srgb(0.22, 0.14, 0.14),
        perceptual_roughness: 0.30,
        ..default()
    });

    let paw_pad_material = materials.add(StandardMaterial {
        base_color: Color::srgb(0.95, 0.62, 0.68),
        perceptual_roughness: 0.50,
        ..default()
    });

    let blush_material = materials.add(StandardMaterial {
        base_color: Color::srgba(1.0, 0.58, 0.68, 0.75),
        alpha_mode: AlphaMode::Blend,
        unlit: true,
        ..default()
    });

    let cat_materials = CatMaterials {
        body_material,
        stripe_material,
        white_fur_material,
        inner_ear_material,
        nose_material,
        eye_contour_material,
        eye_white_material,
        eye_shine_material,
        whisker_material,
        iris_material,
        pupil_material,
        mouth_material,
        paw_pad_material,
        blush_material,
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
            // Squash & stretch bone — lowered Y so the full body fits in the camera frame
            root.spawn((
                CatSquash,
                Transform::from_translation(Vec3::new(0.0, 0.22, 0.0))
                    .with_scale(Vec3::splat(config.settings.size.scale_factor())),
                Visibility::default(),
            ));
        });
}

/// Spawns the procedural cat model (faces +Z towards the screen).
///
/// Proportions and detailing inspired by Kayden from Eleceed (Reference Image 1):
/// - Chonky round body with a white chest bib and flank tiger stripes.
/// - Proportional head with a clean white muzzle, forehead blaze, and tabby stripes.
/// - Winged anime feline eyes with luminous teal irises and crisp specular shines.
/// - Dainty pink nose, cleft philtrum, and stylized feline mouth line.
/// - Triangular anime ears with dark backs, pink inner cavity, and base fluff.
/// - White mitten paws with cute pink toe beans.
/// - Arched 3-segment tail with a white fluffy tip.
fn spawn_procedural_model(
    parent: &mut ChildSpawnerCommands,
    meshes: &mut Assets<Mesh>,
    materials: &CatMaterials,
) {
    // ── Mesh primitives ──────────────────────────────────────────────────────
    // Torso & Anatomy
    let chest_mesh = meshes.add(Sphere::new(0.28).mesh().ico(5).unwrap());
    let torso_mesh = meshes.add(Sphere::new(0.26).mesh().ico(5).unwrap());
    let pelvis_mesh = meshes.add(Sphere::new(0.24).mesh().ico(4).unwrap());
    let haunch_mesh = meshes.add(Sphere::new(0.18).mesh().ico(4).unwrap());
    let neck_mesh = meshes.add(Sphere::new(0.19).mesh().ico(4).unwrap());

    // Bib & Markings
    let upper_bib_mesh = meshes.add(Sphere::new(0.22).mesh().ico(4).unwrap());
    let belly_bib_mesh = meshes.add(Sphere::new(0.20).mesh().ico(4).unwrap());
    let throat_bib_mesh = meshes.add(Sphere::new(0.14).mesh().ico(3).unwrap());
    let flank_stripe_mesh = meshes.add(Sphere::new(0.12).mesh().ico(3).unwrap());

    // Cranium & Jaw
    let cranium_mesh = meshes.add(Sphere::new(0.28).mesh().ico(5).unwrap());
    let jaw_mesh = meshes.add(Sphere::new(0.20).mesh().ico(4).unwrap());
    let cheek_fluff_mesh = meshes.add(Sphere::new(0.08).mesh().ico(4).unwrap());

    // Muzzle & Mouth (compact, proportional)
    let muzzle_base_mesh = meshes.add(Sphere::new(0.11).mesh().ico(4).unwrap());
    let whisker_pad_mesh = meshes.add(Sphere::new(0.052).mesh().ico(4).unwrap());
    let chin_mesh = meshes.add(Sphere::new(0.038).mesh().ico(4).unwrap());
    let white_blaze_mesh = meshes.add(Sphere::new(0.06).mesh().ico(3).unwrap());
    let forehead_stripe_mesh = meshes.add(Sphere::new(0.05).mesh().ico(3).unwrap());

    // Facial features
    let nose_mesh = meshes.add(Sphere::new(0.022).mesh().ico(4).unwrap());
    let philtrum_mesh = meshes.add(Cylinder::new(0.0055, 0.024));
    let mouth_line_mesh = meshes.add(Cylinder::new(0.0050, 0.034));
    let mouth_corner_mesh = meshes.add(Cylinder::new(0.0045, 0.018));
    let chin_cleft_mesh = meshes.add(Cylinder::new(0.0040, 0.014));
    let whisker_mesh = meshes.add(Cylinder::new(0.0035, 0.19));

    // Triangular Feline Ears
    let ear_outer_mesh = meshes.add(Cone { radius: 0.11, height: 0.20 });
    let ear_back_mesh = meshes.add(Cone { radius: 0.105, height: 0.19 });
    let ear_inner_mesh = meshes.add(Cone { radius: 0.085, height: 0.16 });
    let ear_tuft_mesh = meshes.add(Sphere::new(0.028).mesh().ico(3).unwrap());

    // Eyes (Unit sphere flattened with Transform scale)
    let unit_sphere_mesh = meshes.add(Sphere::new(1.0).mesh().ico(4).unwrap());

    // Legs & Paws
    let leg_shaft_mesh = meshes.add(Capsule3d::new(0.055, 0.14));
    let paw_mesh = meshes.add(Sphere::new(0.068).mesh().ico(4).unwrap());
    let paw_pad_mesh = meshes.add(Sphere::new(0.020).mesh().ico(3).unwrap());

    // 3-Segment Tail
    let tail_base_mesh = meshes.add(Sphere::new(0.085).mesh().ico(4).unwrap());
    let tail_mid_mesh = meshes.add(Sphere::new(0.080).mesh().ico(4).unwrap());
    let tail_tip_mesh = meshes.add(Sphere::new(0.075).mesh().ico(4).unwrap());

    parent
        .spawn((
            ActiveCatModelRoot,
            Transform::IDENTITY,
            Visibility::default(),
        ))
        .with_children(|model_root| {
            // ── 1. QUADRUPED BODY & TORSO HIERARCHY ──────────────────────────
            // 1a. Chest & Shoulders (broad thoracic foundation)
            model_root.spawn((
                Mesh3d(chest_mesh.clone()),
                MeshMaterial3d(materials.body_material.clone()),
                Transform::from_xyz(0.0, 0.08, 0.08)
                    .with_scale(Vec3::new(1.18, 0.96, 1.05)),
            ));

            // 1b. Elongated Spine / Mid-Torso / Ribcage
            model_root.spawn((
                Mesh3d(torso_mesh.clone()),
                MeshMaterial3d(materials.body_material.clone()),
                Transform::from_xyz(0.0, 0.04, -0.12)
                    .with_scale(Vec3::new(1.08, 0.94, 1.25)),
            ));

            // 1c. Pelvis / Rear Torso
            model_root.spawn((
                Mesh3d(pelvis_mesh.clone()),
                MeshMaterial3d(materials.body_material.clone()),
                Transform::from_xyz(0.0, 0.06, -0.32)
                    .with_scale(Vec3::new(1.02, 0.92, 0.95)),
            ));

            // 1d. Left Rear Haunch (muscular feline thigh)
            model_root.spawn((
                Mesh3d(haunch_mesh.clone()),
                MeshMaterial3d(materials.body_material.clone()),
                Transform::from_xyz(-0.22, 0.00, -0.26)
                    .with_scale(Vec3::new(0.68, 1.15, 0.95))
                    .with_rotation(Quat::from_rotation_z(-0.12)),
            ));

            // 1e. Right Rear Haunch
            model_root.spawn((
                Mesh3d(haunch_mesh.clone()),
                MeshMaterial3d(materials.body_material.clone()),
                Transform::from_xyz(0.22, 0.00, -0.26)
                    .with_scale(Vec3::new(0.68, 1.15, 0.95))
                    .with_rotation(Quat::from_rotation_z(0.12)),
            ));

            // 1f. Anatomical Neck / Throat (bridges chest smoothly to the skull)
            model_root.spawn((
                Mesh3d(neck_mesh.clone()),
                MeshMaterial3d(materials.body_material.clone()),
                Transform::from_xyz(0.0, 0.28, 0.12)
                    .with_scale(Vec3::new(0.96, 1.12, 0.88))
                    .with_rotation(Quat::from_rotation_x(0.15)),
            ));

            // 1g. White Fur Markings: Upper Chest Bib, Throat & Underbelly
            model_root.spawn((
                Mesh3d(upper_bib_mesh.clone()),
                MeshMaterial3d(materials.white_fur_material.clone()),
                Transform::from_xyz(0.0, 0.06, 0.20)
                    .with_scale(Vec3::new(0.78, 0.92, 0.50)),
            ));
            model_root.spawn((
                Mesh3d(throat_bib_mesh.clone()),
                MeshMaterial3d(materials.white_fur_material.clone()),
                Transform::from_xyz(0.0, 0.24, 0.19)
                    .with_scale(Vec3::new(0.65, 0.85, 0.45))
                    .with_rotation(Quat::from_rotation_x(0.15)),
            ));
            model_root.spawn((
                Mesh3d(belly_bib_mesh.clone()),
                MeshMaterial3d(materials.white_fur_material.clone()),
                Transform::from_xyz(0.0, -0.14, -0.06)
                    .with_scale(Vec3::new(0.72, 0.40, 1.15)),
            ));

            // 1h. Flank Tabby Stripes (slender markings hugging the ribcage)
            model_root.spawn((
                Mesh3d(flank_stripe_mesh.clone()),
                MeshMaterial3d(materials.stripe_material.clone()),
                Transform::from_xyz(-0.30, 0.08, -0.04)
                    .with_scale(Vec3::new(0.25, 0.55, 0.70))
                    .with_rotation(Quat::from_rotation_z(0.25)),
            ));
            model_root.spawn((
                Mesh3d(flank_stripe_mesh.clone()),
                MeshMaterial3d(materials.stripe_material.clone()),
                Transform::from_xyz(0.30, 0.08, -0.04)
                    .with_scale(Vec3::new(0.25, 0.55, 0.70))
                    .with_rotation(Quat::from_rotation_z(-0.25)),
            ));
            model_root.spawn((
                Mesh3d(flank_stripe_mesh.clone()),
                MeshMaterial3d(materials.stripe_material.clone()),
                Transform::from_xyz(-0.31, 0.02, -0.16)
                    .with_scale(Vec3::new(0.24, 0.50, 0.65))
                    .with_rotation(Quat::from_rotation_z(0.18)),
            ));
            model_root.spawn((
                Mesh3d(flank_stripe_mesh.clone()),
                MeshMaterial3d(materials.stripe_material.clone()),
                Transform::from_xyz(0.31, 0.02, -0.16)
                    .with_scale(Vec3::new(0.24, 0.50, 0.65))
                    .with_rotation(Quat::from_rotation_z(-0.18)),
            ));

            // ── 2. HEAD & FACIAL ANATOMY HIERARCHY ────────────────────────────
            model_root
                .spawn((
                    CatHead,
                    Mesh3d(cranium_mesh.clone()),
                    MeshMaterial3d(materials.body_material.clone()),
                    Transform::from_xyz(0.0, 0.48, 0.14)
                        .with_scale(Vec3::new(1.18, 0.94, 1.02)),
                    Visibility::default(),
                ))
                .with_children(|head| {
                    // 2a. Tapered Jaw / Chin Foundation
                    head.spawn((
                        Mesh3d(jaw_mesh.clone()),
                        MeshMaterial3d(materials.body_material.clone()),
                        Transform::from_xyz(0.0, -0.09, 0.06)
                            .with_scale(Vec3::new(1.10, 0.75, 0.90)),
                    ));

                    // 2b. Soft Layered Cheek Fluff (no sharp cone spikes!)
                    head.spawn((
                        Mesh3d(cheek_fluff_mesh.clone()),
                        MeshMaterial3d(materials.white_fur_material.clone()),
                        Transform::from_xyz(-0.25, -0.06, 0.08)
                            .with_scale(Vec3::new(1.20, 0.55, 0.50))
                            .with_rotation(Quat::from_rotation_z(0.25)),
                    ));
                    head.spawn((
                        Mesh3d(cheek_fluff_mesh.clone()),
                        MeshMaterial3d(materials.white_fur_material.clone()),
                        Transform::from_xyz(-0.23, -0.11, 0.06)
                            .with_scale(Vec3::new(1.10, 0.45, 0.45))
                            .with_rotation(Quat::from_rotation_z(0.15)),
                    ));
                    head.spawn((
                        Mesh3d(cheek_fluff_mesh.clone()),
                        MeshMaterial3d(materials.white_fur_material.clone()),
                        Transform::from_xyz(0.25, -0.06, 0.08)
                            .with_scale(Vec3::new(1.20, 0.55, 0.50))
                            .with_rotation(Quat::from_rotation_z(-0.25)),
                    ));
                    head.spawn((
                        Mesh3d(cheek_fluff_mesh.clone()),
                        MeshMaterial3d(materials.white_fur_material.clone()),
                        Transform::from_xyz(0.23, -0.11, 0.06)
                            .with_scale(Vec3::new(1.10, 0.45, 0.45))
                            .with_rotation(Quat::from_rotation_z(-0.15)),
                    ));

                    // 2c. Compact Muzzle Base (subtle backing behind pads)
                    head.spawn((
                        Mesh3d(muzzle_base_mesh.clone()),
                        MeshMaterial3d(materials.white_fur_material.clone()),
                        Transform::from_xyz(0.0, -0.085, 0.20)
                            .with_scale(Vec3::new(1.05, 0.75, 0.60)),
                    ));

                    // 2d. Left & Right Whisker Pads (vibrissal pads)
                    head.spawn((
                        Mesh3d(whisker_pad_mesh.clone()),
                        MeshMaterial3d(materials.white_fur_material.clone()),
                        Transform::from_xyz(-0.044, -0.078, 0.252)
                            .with_scale(Vec3::new(1.05, 0.88, 0.68)),
                    ));
                    head.spawn((
                        Mesh3d(whisker_pad_mesh.clone()),
                        MeshMaterial3d(materials.white_fur_material.clone()),
                        Transform::from_xyz(0.044, -0.078, 0.252)
                            .with_scale(Vec3::new(1.05, 0.88, 0.68)),
                    ));

                    // 2e. Small Chin below pads
                    head.spawn((
                        Mesh3d(chin_mesh.clone()),
                        MeshMaterial3d(materials.white_fur_material.clone()),
                        Transform::from_xyz(0.0, -0.130, 0.230)
                            .with_scale(Vec3::new(0.85, 0.65, 0.65)),
                    ));

                    // 2f. Flat Surface White Forehead Blaze (no bulging horn)
                    head.spawn((
                        Mesh3d(white_blaze_mesh.clone()),
                        MeshMaterial3d(materials.white_fur_material.clone()),
                        Transform::from_xyz(0.0, 0.035, 0.275)
                            .with_scale(Vec3::new(0.35, 0.85, 0.08)),
                    ));

                    // 2g. Flat Surface Forehead Tabby Stripes
                    head.spawn((
                        Mesh3d(forehead_stripe_mesh.clone()),
                        MeshMaterial3d(materials.stripe_material.clone()),
                        Transform::from_xyz(0.0, 0.165, 0.245)
                            .with_scale(Vec3::new(0.06, 0.45, 0.06)),
                    ));
                    head.spawn((
                        Mesh3d(forehead_stripe_mesh.clone()),
                        MeshMaterial3d(materials.stripe_material.clone()),
                        Transform::from_xyz(-0.085, 0.150, 0.235)
                            .with_scale(Vec3::new(0.055, 0.40, 0.06))
                            .with_rotation(Quat::from_rotation_z(0.20)),
                    ));
                    head.spawn((
                        Mesh3d(forehead_stripe_mesh.clone()),
                        MeshMaterial3d(materials.stripe_material.clone()),
                        Transform::from_xyz(0.085, 0.150, 0.235)
                            .with_scale(Vec3::new(0.055, 0.40, 0.06))
                            .with_rotation(Quat::from_rotation_z(-0.20)),
                    ));

                    // 2h. Triangular Feline Ears (outer shell, back accent, pink inner cavity, base fluff)
                    let left_ear_rot = Quat::from_euler(EulerRot::YXZ, 0.12, -0.14, -0.22);
                    head.spawn((
                        CatEar {
                            is_left: true,
                            base_rotation: left_ear_rot,
                            dynamic_angle: 0.0,
                            dynamic_velocity: 0.0,
                        },
                        Mesh3d(ear_outer_mesh.clone()),
                        MeshMaterial3d(materials.body_material.clone()),
                        Transform::from_xyz(-0.16, 0.24, 0.04)
                            .with_scale(Vec3::new(1.15, 1.0, 0.42))
                            .with_rotation(left_ear_rot),
                    ))
                    .with_children(|ear| {
                        ear.spawn((
                            Mesh3d(ear_back_mesh.clone()),
                            MeshMaterial3d(materials.stripe_material.clone()),
                            Transform::from_xyz(0.0, 0.0, -0.015),
                        ));
                        ear.spawn((
                            Mesh3d(ear_inner_mesh.clone()),
                            MeshMaterial3d(materials.inner_ear_material.clone()),
                            Transform::from_xyz(0.0, -0.01, 0.025)
                                .with_scale(Vec3::new(0.95, 0.95, 0.32)),
                        ));
                        ear.spawn((
                            Mesh3d(ear_tuft_mesh.clone()),
                            MeshMaterial3d(materials.white_fur_material.clone()),
                            Transform::from_xyz(0.0, -0.06, 0.035)
                                .with_scale(Vec3::new(1.2, 0.6, 0.7)),
                        ));
                    });

                    let right_ear_rot = Quat::from_euler(EulerRot::YXZ, -0.12, -0.14, 0.22);
                    head.spawn((
                        CatEar {
                            is_left: false,
                            base_rotation: right_ear_rot,
                            dynamic_angle: 0.0,
                            dynamic_velocity: 0.0,
                        },
                        Mesh3d(ear_outer_mesh.clone()),
                        MeshMaterial3d(materials.body_material.clone()),
                        Transform::from_xyz(0.16, 0.24, 0.04)
                            .with_scale(Vec3::new(1.15, 1.0, 0.42))
                            .with_rotation(right_ear_rot),
                    ))
                    .with_children(|ear| {
                        ear.spawn((
                            Mesh3d(ear_back_mesh.clone()),
                            MeshMaterial3d(materials.stripe_material.clone()),
                            Transform::from_xyz(0.0, 0.0, -0.015),
                        ));
                        ear.spawn((
                            Mesh3d(ear_inner_mesh.clone()),
                            MeshMaterial3d(materials.inner_ear_material.clone()),
                            Transform::from_xyz(0.0, -0.01, 0.025)
                                .with_scale(Vec3::new(0.95, 0.95, 0.32)),
                        ));
                        ear.spawn((
                            Mesh3d(ear_tuft_mesh.clone()),
                            MeshMaterial3d(materials.white_fur_material.clone()),
                            Transform::from_xyz(0.0, -0.06, 0.035)
                                .with_scale(Vec3::new(1.2, 0.6, 0.7)),
                        ));
                    });

                    // 2i. Expressive Anime Feline Eyes (LEFT)
                    let l_wing_rest = Vec3::new(0.062, 0.020, 0.010);
                    head.spawn((
                        CatEyeWing { is_left: true, rest_scale: l_wing_rest },
                        Mesh3d(unit_sphere_mesh.clone()),
                        MeshMaterial3d(materials.eye_contour_material.clone()),
                        Transform::from_xyz(-0.115, 0.055, 0.280)
                            .with_scale(l_wing_rest)
                            .with_rotation(Quat::from_rotation_z(0.16)),
                        Visibility::default(),
                    ));
                    let l_eye_rest = Vec3::new(0.058, 0.048, 0.012);
                    head.spawn((
                        CatEye { is_left: true, rest_scale: l_eye_rest },
                        Mesh3d(unit_sphere_mesh.clone()),
                        MeshMaterial3d(materials.eye_white_material.clone()),
                        Transform::from_xyz(-0.115, 0.035, 0.275).with_scale(l_eye_rest),
                        Visibility::default(),
                    ));
                    let l_iris_rest = Vec3::new(0.042, 0.044, 0.012);
                    head.spawn((
                        CatEyeIris { is_left: true, rest_scale: l_iris_rest },
                        Mesh3d(unit_sphere_mesh.clone()),
                        MeshMaterial3d(materials.iris_material.clone()),
                        Transform::from_xyz(-0.113, 0.034, 0.278).with_scale(l_iris_rest),
                        Visibility::default(),
                    ));
                    let l_pupil_rest = Vec3::new(0.011, 0.032, 0.010);
                    head.spawn((
                        CatEyePupil { is_left: true, rest_scale: l_pupil_rest },
                        Mesh3d(unit_sphere_mesh.clone()),
                        MeshMaterial3d(materials.pupil_material.clone()),
                        Transform::from_xyz(-0.113, 0.034, 0.282).with_scale(l_pupil_rest),
                        Visibility::default(),
                    ));
                    // Sparkles & Lid
                    head.spawn((
                        Mesh3d(unit_sphere_mesh.clone()),
                        MeshMaterial3d(materials.eye_shine_material.clone()),
                        Transform::from_xyz(-0.122, 0.045, 0.285)
                            .with_scale(Vec3::new(0.013, 0.013, 0.005)),
                    ));
                    head.spawn((
                        Mesh3d(unit_sphere_mesh.clone()),
                        MeshMaterial3d(materials.eye_shine_material.clone()),
                        Transform::from_xyz(-0.102, 0.026, 0.285)
                            .with_scale(Vec3::new(0.007, 0.007, 0.004)),
                    ));
                    head.spawn((
                        Mesh3d(unit_sphere_mesh.clone()),
                        MeshMaterial3d(materials.eye_contour_material.clone()),
                        Transform::from_xyz(-0.115, 0.014, 0.278)
                            .with_scale(Vec3::new(0.042, 0.007, 0.007)),
                    ));
                    let l_blush_rest = Vec3::new(0.048, 0.024, 0.007);
                    head.spawn((
                        CatBlush { is_left: true, rest_scale: l_blush_rest },
                        Mesh3d(unit_sphere_mesh.clone()),
                        MeshMaterial3d(materials.blush_material.clone()),
                        Transform::from_xyz(-0.155, -0.040, 0.260).with_scale(Vec3::ZERO),
                        Visibility::default(),
                    ));

                    // 2j. Expressive Anime Feline Eyes (RIGHT)
                    let r_wing_rest = Vec3::new(0.062, 0.020, 0.010);
                    head.spawn((
                        CatEyeWing { is_left: false, rest_scale: r_wing_rest },
                        Mesh3d(unit_sphere_mesh.clone()),
                        MeshMaterial3d(materials.eye_contour_material.clone()),
                        Transform::from_xyz(0.115, 0.055, 0.280)
                            .with_scale(r_wing_rest)
                            .with_rotation(Quat::from_rotation_z(-0.16)),
                        Visibility::default(),
                    ));
                    let r_eye_rest = Vec3::new(0.058, 0.048, 0.012);
                    head.spawn((
                        CatEye { is_left: false, rest_scale: r_eye_rest },
                        Mesh3d(unit_sphere_mesh.clone()),
                        MeshMaterial3d(materials.eye_white_material.clone()),
                        Transform::from_xyz(0.115, 0.035, 0.275).with_scale(r_eye_rest),
                        Visibility::default(),
                    ));
                    let r_iris_rest = Vec3::new(0.042, 0.044, 0.012);
                    head.spawn((
                        CatEyeIris { is_left: false, rest_scale: r_iris_rest },
                        Mesh3d(unit_sphere_mesh.clone()),
                        MeshMaterial3d(materials.iris_material.clone()),
                        Transform::from_xyz(0.113, 0.034, 0.278).with_scale(r_iris_rest),
                        Visibility::default(),
                    ));
                    let r_pupil_rest = Vec3::new(0.011, 0.032, 0.010);
                    head.spawn((
                        CatEyePupil { is_left: false, rest_scale: r_pupil_rest },
                        Mesh3d(unit_sphere_mesh.clone()),
                        MeshMaterial3d(materials.pupil_material.clone()),
                        Transform::from_xyz(0.113, 0.034, 0.282).with_scale(r_pupil_rest),
                        Visibility::default(),
                    ));
                    head.spawn((
                        Mesh3d(unit_sphere_mesh.clone()),
                        MeshMaterial3d(materials.eye_shine_material.clone()),
                        Transform::from_xyz(0.105, 0.045, 0.285)
                            .with_scale(Vec3::new(0.013, 0.013, 0.005)),
                    ));
                    head.spawn((
                        Mesh3d(unit_sphere_mesh.clone()),
                        MeshMaterial3d(materials.eye_shine_material.clone()),
                        Transform::from_xyz(0.125, 0.026, 0.285)
                            .with_scale(Vec3::new(0.007, 0.007, 0.004)),
                    ));
                    head.spawn((
                        Mesh3d(unit_sphere_mesh.clone()),
                        MeshMaterial3d(materials.eye_contour_material.clone()),
                        Transform::from_xyz(0.115, 0.014, 0.278)
                            .with_scale(Vec3::new(0.042, 0.007, 0.007)),
                    ));
                    let r_blush_rest = Vec3::new(0.048, 0.024, 0.007);
                    head.spawn((
                        CatBlush { is_left: false, rest_scale: r_blush_rest },
                        Mesh3d(unit_sphere_mesh.clone()),
                        MeshMaterial3d(materials.blush_material.clone()),
                        Transform::from_xyz(0.155, -0.040, 0.260).with_scale(Vec3::ZERO),
                        Visibility::default(),
                    ));

                    // 2k. Dainty Pink Nose Pad (proud of the whisker pads at Z = 0.284 -> 0.300)
                    head.spawn((
                        Mesh3d(nose_mesh.clone()),
                        MeshMaterial3d(materials.nose_material.clone()),
                        Transform::from_xyz(0.0, -0.054, 0.284)
                            .with_scale(Vec3::new(1.15, 0.82, 0.75)),
                    ));

                    // 2l. 100% VISIBLE SURFACE-ALIGNED FELINE MOUTH (`^ω^` smile contour)
                    // Placed at Z = 0.292 - 0.295, completely outside the whisker pads (Z = 0.287)
                    // Philtrum vertical seam
                    head.spawn((
                        Mesh3d(philtrum_mesh.clone()),
                        MeshMaterial3d(materials.mouth_material.clone()),
                        Transform::from_xyz(0.0, -0.075, 0.294),
                    ));
                    // Left smile line
                    head.spawn((
                        Mesh3d(mouth_line_mesh.clone()),
                        MeshMaterial3d(materials.mouth_material.clone()),
                        Transform::from_xyz(-0.016, -0.087, 0.293)
                            .with_rotation(Quat::from_rotation_z(-0.45)),
                    ));
                    // Left smile corner (upward flick)
                    head.spawn((
                        Mesh3d(mouth_corner_mesh.clone()),
                        MeshMaterial3d(materials.mouth_material.clone()),
                        Transform::from_xyz(-0.032, -0.083, 0.290)
                            .with_rotation(Quat::from_rotation_z(0.35)),
                    ));
                    // Right smile line
                    head.spawn((
                        Mesh3d(mouth_line_mesh.clone()),
                        MeshMaterial3d(materials.mouth_material.clone()),
                        Transform::from_xyz(0.016, -0.087, 0.293)
                            .with_rotation(Quat::from_rotation_z(0.45)),
                    ));
                    // Right smile corner (upward flick)
                    head.spawn((
                        Mesh3d(mouth_corner_mesh.clone()),
                        MeshMaterial3d(materials.mouth_material.clone()),
                        Transform::from_xyz(0.032, -0.083, 0.290)
                            .with_rotation(Quat::from_rotation_z(-0.35)),
                    ));
                    // Lower lip / chin cleft line
                    head.spawn((
                        Mesh3d(chin_cleft_mesh.clone()),
                        MeshMaterial3d(materials.mouth_material.clone()),
                        Transform::from_xyz(0.0, -0.102, 0.282),
                    ));

                    // 2m. Whiskers (fanning naturally from the whisker pads)
                    let whisker_rot_l1 = Quat::from_euler(EulerRot::YXZ, -0.14, 0.0, 1.48);
                    let whisker_rot_l2 = Quat::from_euler(EulerRot::YXZ, -0.14, 0.0, 1.70);
                    head.spawn((
                        CatWhisker { is_left: true, base_rotation: whisker_rot_l1 },
                        Mesh3d(whisker_mesh.clone()),
                        MeshMaterial3d(materials.whisker_material.clone()),
                        Transform::from_xyz(-0.14, -0.070, 0.260).with_rotation(whisker_rot_l1),
                    ));
                    head.spawn((
                        CatWhisker { is_left: true, base_rotation: whisker_rot_l2 },
                        Mesh3d(whisker_mesh.clone()),
                        MeshMaterial3d(materials.whisker_material.clone()),
                        Transform::from_xyz(-0.14, -0.092, 0.260).with_rotation(whisker_rot_l2),
                    ));
                    let whisker_rot_r1 = Quat::from_euler(EulerRot::YXZ, 0.14, 0.0, -1.48);
                    let whisker_rot_r2 = Quat::from_euler(EulerRot::YXZ, 0.14, 0.0, -1.70);
                    head.spawn((
                        CatWhisker { is_left: false, base_rotation: whisker_rot_r1 },
                        Mesh3d(whisker_mesh.clone()),
                        MeshMaterial3d(materials.whisker_material.clone()),
                        Transform::from_xyz(0.14, -0.070, 0.260).with_rotation(whisker_rot_r1),
                    ));
                    head.spawn((
                        CatWhisker { is_left: false, base_rotation: whisker_rot_r2 },
                        Mesh3d(whisker_mesh.clone()),
                        MeshMaterial3d(materials.whisker_material.clone()),
                        Transform::from_xyz(0.14, -0.092, 0.260).with_rotation(whisker_rot_r2),
                    ));
                });

            // ── 3. FOUR ARTICULATED LEGS WITH NATURAL SHOULDER / HAUNCH ORIGINS ─
            let legs = [
                (LegType::FrontLeft,  Vec3::new(-0.18, -0.22,  0.16)),
                (LegType::FrontRight, Vec3::new( 0.18, -0.22,  0.16)),
                (LegType::BackLeft,   Vec3::new(-0.22, -0.22, -0.24)),
                (LegType::BackRight,  Vec3::new( 0.22, -0.22, -0.24)),
            ];

            for (leg_type, pos) in legs {
                model_root
                    .spawn((
                        CatLeg { leg_type, rest_translation: pos },
                        Mesh3d(leg_shaft_mesh.clone()),
                        MeshMaterial3d(materials.body_material.clone()),
                        Transform::from_translation(pos),
                    ))
                    .with_children(|leg| {
                        leg.spawn((
                            Mesh3d(paw_mesh.clone()),
                            MeshMaterial3d(materials.white_fur_material.clone()),
                            Transform::from_xyz(0.0, -0.09, 0.02)
                                .with_scale(Vec3::new(1.0, 0.58, 1.22)),
                        ))
                        .with_children(|paw| {
                            paw.spawn((
                                Mesh3d(paw_pad_mesh.clone()),
                                MeshMaterial3d(materials.paw_pad_material.clone()),
                                Transform::from_xyz(-0.024, -0.045, 0.020),
                            ));
                            paw.spawn((
                                Mesh3d(paw_pad_mesh.clone()),
                                MeshMaterial3d(materials.paw_pad_material.clone()),
                                Transform::from_xyz(0.0, -0.045, 0.032),
                            ));
                            paw.spawn((
                                Mesh3d(paw_pad_mesh.clone()),
                                MeshMaterial3d(materials.paw_pad_material.clone()),
                                Transform::from_xyz(0.024, -0.045, 0.020),
                            ));
                        });
                    });
            }

            // ── 4. 3-SEGMENT TAIL ORIGINATING FROM REAR PELVIS ───────────────
            model_root
                .spawn((
                    CatTailJoint {
                        index: 0,
                        current_angle: Vec2::ZERO,
                        angular_velocity: Vec2::ZERO,
                    },
                    Mesh3d(tail_base_mesh.clone()),
                    MeshMaterial3d(materials.body_material.clone()),
                    Transform::from_xyz(0.0, 0.12, -0.42),
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
                            Mesh3d(tail_mid_mesh.clone()),
                            MeshMaterial3d(materials.stripe_material.clone()),
                            Transform::from_xyz(0.0, 0.10, -0.13),
                            Visibility::default(),
                        ))
                        .with_children(|tail1| {
                            tail1.spawn((
                                CatTailJoint {
                                    index: 2,
                                    current_angle: Vec2::ZERO,
                                    angular_velocity: Vec2::ZERO,
                                },
                                Mesh3d(tail_tip_mesh.clone()),
                                MeshMaterial3d(materials.white_fur_material.clone()),
                                Transform::from_xyz(0.0, 0.09, -0.12),
                                Visibility::default(),
                            ));
                        });
                });
        });
}

/// Ensures the procedural model is spawned.
fn sync_cat_model(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    cat_materials: Option<Res<CatMaterials>>,
    mut spawned: Local<bool>,
    q_squash: Query<Entity, With<CatSquash>>,
) {
    if *spawned {
        return;
    }

    let Ok(squash_entity) = q_squash.single() else {
        return;
    };

    let Some(materials) = cat_materials else {
        return;
    };

    commands.entity(squash_entity).with_children(|squash| {
        spawn_procedural_model(squash, &mut meshes, &materials);
    });

    *spawned = true;
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

    if last_coat.is_none() || *last_coat != Some(config.settings.coat) {
        *last_coat = Some(config.settings.coat);

        if let Some(mut mat) = materials.get_mut(&cat_mats.body_material) {
            mat.base_color = config.settings.coat.body_color();
        }
        if let Some(mut mat) = materials.get_mut(&cat_mats.stripe_material) {
            mat.base_color = config.settings.coat.stripe_color();
        }
        if let Some(mut mat) = materials.get_mut(&cat_mats.white_fur_material) {
            mat.base_color = config.settings.coat.white_fur_color();
        }
        if let Some(mut mat) = materials.get_mut(&cat_mats.inner_ear_material) {
            mat.base_color = config.settings.coat.inner_ear_color();
        }
        if let Some(mut mat) = materials.get_mut(&cat_mats.nose_material) {
            mat.base_color = config.settings.coat.nose_color();
        }
        if let Some(mut mat) = materials.get_mut(&cat_mats.iris_material) {
            let iris_col = config.settings.coat.iris_color();
            mat.base_color = iris_col;
            if let Color::Srgba(s) = iris_col {
                mat.emissive = LinearRgba::new(s.red * 0.25, s.green * 0.25, s.blue * 0.25, 1.0);
            }
        }
    }
}
