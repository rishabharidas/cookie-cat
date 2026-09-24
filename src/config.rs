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
    /// Returns the primary body PBR color.
    pub fn body_color(&self) -> Color {
        match self {
            // Warm cream tone matching drifty.so
            CoatColor::Biscuit => Color::srgb(0.93, 0.91, 0.85),
            // Clean soft white with subtle warmth
            CoatColor::White => Color::srgb(0.97, 0.97, 0.98),
            // Soft slate grey / British Shorthair
            CoatColor::Grey => Color::srgb(0.58, 0.60, 0.64),
        }
    }

    /// Returns the inner ear accent color.
    pub fn inner_ear_color(&self) -> Color {
        match self {
            CoatColor::Biscuit => Color::srgb(0.96, 0.78, 0.78),
            CoatColor::White => Color::srgb(0.98, 0.82, 0.84),
            CoatColor::Grey => Color::srgb(0.85, 0.72, 0.76),
        }
    }

    /// Returns the cute nose color.
    pub fn nose_color(&self) -> Color {
        Color::srgb(0.92, 0.55, 0.62)
    }

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
    Running,
    Jumping,
    Sniffing,
    Sitting,
    SeekingMouse,
    Petting,
    Sleeping,
}

/// Global cat settings and live state resource.
#[derive(Resource)]
pub struct CatConfig {
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
    pub particle_timer: f32,
    /// Target roaming position on screen
    pub desktop_target: Vec2,
    /// Current position of the window on screen (stored as f32 for smooth movement)
    pub desktop_pos: Option<Vec2>,
    /// Global OS mouse position
    pub global_mouse_pos: Vec2,
    /// Last seen global OS mouse position
    pub last_mouse_pos: Vec2,
    /// Mouse idle time counter
    pub mouse_idle_timer: f32,
    /// Walk cycle phase accumulator
    pub walk_phase: f32,
    /// Look target for the head (e.g. mouse cursor in 3D, defaults towards the screen)
    pub look_target: Vec3,
}

impl Default for CatConfig {
    fn default() -> Self {
        Self {
            coat: CoatColor::default(),
            size: CatSize::default(),
            state: CatAiState::default(),
            state_timer: 3.0,
            petting_timer: 0.0,
            particle_timer: 0.0,
            desktop_target: Vec2::ZERO,
            desktop_pos: None,
            global_mouse_pos: Vec2::ZERO,
            last_mouse_pos: Vec2::ZERO,
            mouse_idle_timer: 0.0,
            walk_phase: 0.0,
            look_target: Vec3::new(0.0, 0.5, 3.0),
        }
    }
}
