//! Configuration & Customization Module
//!
//! Designed to be clean, modular, and easily extensible for adding
//! custom colors, cat breeds, accessories, and behavior settings in the future.

use bevy::prelude::*;

/// Available coat colors for the desktop cat.
/// Currently supports Biscuit (Drifty cream), White, and Grey.
/// Designed for easy addition of custom hex/RGB colors or new coats!
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum CoatColor {
    #[default]
    Biscuit,
    White,
    Grey,
}

impl CoatColor {
    /// User-friendly display name.
    pub fn display_name(&self) -> &'static str {
        match self {
            CoatColor::Biscuit => "Biscuit (Cream)",
            CoatColor::White => "White",
            CoatColor::Grey => "Grey",
        }
    }

    /// Cycles to the next available coat color.
    pub fn next(&self) -> Self {
        match self {
            CoatColor::Biscuit => CoatColor::White,
            CoatColor::White => CoatColor::Grey,
            CoatColor::Grey => CoatColor::Biscuit,
        }
    }
}

/// Available size scales for the desktop cat.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum CatSize {
    Small,
    #[default]
    Normal,
    Large,
    ExtraLarge,
}

impl CatSize {
    /// Returns the uniform scale factor for rendering.
    pub fn scale_factor(&self) -> f32 {
        match self {
            CatSize::Small => 0.80,
            CatSize::Normal => 1.00,
            CatSize::Large => 1.25,
            CatSize::ExtraLarge => 1.50,
        }
    }

    /// Display label for current size.
    pub fn display_name(&self) -> &'static str {
        match self {
            CatSize::Small => "Small (80%)",
            CatSize::Normal => "Normal (100%)",
            CatSize::Large => "Large (125%)",
            CatSize::ExtraLarge => "Extra Large (150%)",
        }
    }

    /// Cycles to the next size option.
    pub fn next(&self) -> Self {
        match self {
            CatSize::Small => CatSize::Normal,
            CatSize::Normal => CatSize::Large,
            CatSize::Large => CatSize::ExtraLarge,
            CatSize::ExtraLarge => CatSize::Small,
        }
    }
}

/// Active behavioral and AI state of the cat.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum CatAiState {
    #[default]
    Idle,
    Walking,
    Sniffing,
    Sitting,
    Petting,
    Sleeping,
}

/// Model source type: either procedural stylized model or realistic 3D GLTF model (.glb).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum ModelType {
    /// Realistic 3D mesh loaded from assets/models/cat.glb (exported from Blender)
    #[default]
    Gltf,
    /// Stylized procedural cat built from geometric shapes
    Procedural,
}

impl ModelType {
    pub fn toggle(&self) -> Self {
        match self {
            ModelType::Gltf => ModelType::Procedural,
            ModelType::Procedural => ModelType::Gltf,
        }
    }

    pub fn display_name(&self) -> &'static str {
        match self {
            ModelType::Gltf => "3D Blender Model (assets/models/cat.glb)",
            ModelType::Procedural => "Procedural Stylized",
        }
    }
}

/// Global cat settings and live state resource.
#[derive(Resource)]
pub struct CatConfig {
    /// Active 3D model type
    pub model_type: ModelType,
    /// Current coat color
    pub coat: CoatColor,
    /// Current size scale
    pub size: CatSize,
    /// Current behavioral state
    pub state: CatAiState,
    /// Timer for state transitions
    pub state_timer: f32,
    /// Timer for petting bounce reaction
    pub petting_timer: f32,
    /// Particle spawn cooldown timer (hearts / zzz)

    /// Target roaming position in 3D space
    pub roam_target: Vec3,
    /// Walk cycle phase accumulator
    pub walk_phase: f32,
    /// Look target for the head (e.g. mouse cursor in 3D, defaults towards the screen)
    pub look_target: Vec3,
}

impl Default for CatConfig {
    fn default() -> Self {
        Self {
            model_type: ModelType::default(),
            coat: CoatColor::default(),
            size: CatSize::default(),
            state: CatAiState::default(),
            state_timer: 3.0,
            petting_timer: 0.0,

            roam_target: Vec3::ZERO,
            walk_phase: 0.0,
            look_target: Vec3::new(0.0, 0.5, 3.0),
        }
    }
}
