//! Configuration & Settings Persistence Module
//!
//! Provides customizable settings for the desktop cat, including coat color,
//! size, movement speed, idle frequency, and behavior toggles.
//! Settings automatically persist to a local JSON file.

use std::fs;
use std::path::{Path, PathBuf};
use bevy::prelude::*;
use serde::{Deserialize, Serialize};

/// Available coat colors for the desktop cat.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
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
            CoatColor::Biscuit => Color::srgb(0.93, 0.91, 0.85),
            CoatColor::White => Color::srgb(0.97, 0.97, 0.98),
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
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
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

/// Movement speed setting.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
pub enum SpeedSetting {
    Relaxed,
    #[default]
    Normal,
    Playful,
}

impl SpeedSetting {
    pub fn pixels_per_second(&self) -> f32 {
        match self {
            SpeedSetting::Relaxed => 65.0,
            SpeedSetting::Normal => 105.0,
            SpeedSetting::Playful => 160.0,
        }
    }

    pub fn display_name(&self) -> &'static str {
        match self {
            SpeedSetting::Relaxed => "Relaxed (65 px/s)",
            SpeedSetting::Normal => "Normal (105 px/s)",
            SpeedSetting::Playful => "Playful (160 px/s)",
        }
    }

    pub fn next(&self) -> Self {
        match self {
            SpeedSetting::Relaxed => SpeedSetting::Normal,
            SpeedSetting::Normal => SpeedSetting::Playful,
            SpeedSetting::Playful => SpeedSetting::Relaxed,
        }
    }
}

/// How frequently the cat transitions or takes action when idle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
pub enum IdleFrequency {
    Low,
    #[default]
    Medium,
    High,
}

impl IdleFrequency {
    pub fn random_idle_duration(&self, seed: f32) -> f32 {
        let frac = (seed.fract() + 1.0).fract();
        match self {
            IdleFrequency::Low => 5.0 + frac * 5.0,     // 5 to 10s
            IdleFrequency::Medium => 3.0 + frac * 4.0,  // 3 to 7s
            IdleFrequency::High => 1.8 + frac * 2.5,    // 1.8 to 4.3s
        }
    }

    pub fn display_name(&self) -> &'static str {
        match self {
            IdleFrequency::Low => "Low (Calm)",
            IdleFrequency::Medium => "Medium (Balanced)",
            IdleFrequency::High => "High (Lively)",
        }
    }

    pub fn next(&self) -> Self {
        match self {
            IdleFrequency::Low => IdleFrequency::Medium,
            IdleFrequency::Medium => IdleFrequency::High,
            IdleFrequency::High => IdleFrequency::Low,
        }
    }
}

/// Active behavioral and AI state of the cat.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
pub enum CatAiState {
    #[default]
    Idle,
    LookingAround,
    Walking,
    Stopping,
    Sitting,
    Standing,
    Sleeping,
    Investigating,
    Stretching,
    Petting,
    Falling,
    Landing,
}

impl CatAiState {
    pub fn display_name(&self) -> &'static str {
        match self {
            CatAiState::Idle => "Idle",
            CatAiState::LookingAround => "Looking Around",
            CatAiState::Walking => "Walking",
            CatAiState::Stopping => "Stopping",
            CatAiState::Sitting => "Sitting",
            CatAiState::Standing => "Standing Up",
            CatAiState::Sleeping => "Sleeping / Napping",
            CatAiState::Investigating => "Investigating / Sniffing",
            CatAiState::Stretching => "Stretching",
            CatAiState::Petting => "Happy / Being Petted",
            CatAiState::Falling => "Falling (Gravity)",
            CatAiState::Landing => "Landing",
        }
    }
}

/// Persistent user settings saved across sessions.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CatSettings {
    pub coat: CoatColor,
    pub size: CatSize,
    pub speed: SpeedSetting,
    pub idle_frequency: IdleFrequency,
    pub enable_roaming: bool,
    pub enable_head_look: bool,
    pub enable_blinking: bool,
    pub enable_particles: bool,
    pub enable_gravity: bool,
    pub cat_visible: bool,
    pub paused: bool,
}

impl Default for CatSettings {
    fn default() -> Self {
        Self {
            coat: CoatColor::default(),
            size: CatSize::default(),
            speed: SpeedSetting::default(),
            idle_frequency: IdleFrequency::default(),
            enable_roaming: true,
            enable_head_look: true,
            enable_blinking: true,
            enable_particles: true,
            enable_gravity: true,
            cat_visible: true,
            paused: false,
        }
    }
}

/// Global cat settings and live state resource.
#[derive(Resource)]
pub struct CatConfig {
    /// User settings (persisted)
    pub settings: CatSettings,
    /// Current behavioral state
    pub state: CatAiState,
    /// Previous state for transitions
    #[allow(dead_code)]
    pub prev_state: CatAiState,
    /// State timer countdown
    pub state_timer: f32,
    /// Petting reaction timer
    pub petting_timer: f32,
    /// Particle spawn cooldown timer (hearts / zzz)
    pub particle_timer: f32,
    /// Target roaming position on screen
    pub desktop_target: Vec2,
    /// Current position of the window on screen
    pub desktop_pos: Option<Vec2>,
    /// Vertical falling velocity in pixels/sec for gravity
    pub vertical_velocity: f32,
    /// Whether user is currently dragging the cat
    pub is_dragged: bool,
    /// Previous drag position for measuring drop velocity
    #[allow(dead_code)]
    pub last_drag_pos: Option<Vec2>,
    /// Screen and dock geometry metrics for the display the cat is currently on
    pub current_metrics: Option<crate::platform::DesktopMetrics>,
    /// Timer for looking at global mouse movements
    pub mouse_attention_timer: f32,
    /// Last detected global mouse position
    pub last_global_mouse: Option<Vec2>,
    /// Whether the cat window has been positioned at the bottom on launch
    pub initial_spawned: bool,
    /// Calculated ground level Y on screen
    pub ground_y: f32,
    /// Walk cycle phase accumulator
    pub walk_phase: f32,
    /// Smoothly blended walk speed
    pub current_velocity: Vec2,
    /// Look target for the head in 3D local coordinates
    pub look_target: Vec3,
    /// Inquisitive head tilt angle (roll around Z)
    pub head_tilt: f32,
    /// Posture blend factor: 0.0 = standing, 1.0 = sitting/sleeping
    pub posture_blend: f32,
    /// Blink timer countdown
    pub blink_timer: f32,
    /// Blink progress: 0.0 = open, 1.0 = closed
    pub blink_amount: f32,
    /// Path where settings are saved
    pub config_path: PathBuf,
}

impl Default for CatConfig {
    fn default() -> Self {
        let config_path = get_default_config_path();
        let settings = load_settings(&config_path);
        let env = crate::platform::create_desktop_environment();
        let current_metrics = env
            .displays()
            .into_iter()
            .find(|d| d.is_primary)
            .or_else(|| env.displays().into_iter().next())
            .map(|d| {
                crate::platform::DesktopMetrics::from_display(
                    &d,
                    480.0,
                    340.0,
                    255.0,
                    20.0,
                    0.0,
                )
            });

        let ground_y = current_metrics.as_ref().map(|m| m.ground_y).unwrap_or(0.0);
        let initial_x = current_metrics.as_ref().map(|m| (m.min_x + m.max_x) * 0.5).unwrap_or(0.0);

        Self {
            settings,
            state: CatAiState::default(),
            prev_state: CatAiState::default(),
            state_timer: 4.0,
            petting_timer: 0.0,
            particle_timer: 0.0,
            desktop_target: Vec2::new(initial_x, ground_y),
            desktop_pos: None,
            vertical_velocity: 0.0,
            is_dragged: false,
            last_drag_pos: None,
            current_metrics,
            mouse_attention_timer: 0.0,
            last_global_mouse: None,
            initial_spawned: false,
            ground_y,
            walk_phase: 0.0,
            current_velocity: Vec2::ZERO,
            look_target: Vec3::new(0.0, 0.45, 2.5),
            head_tilt: 0.0,
            posture_blend: 0.0,
            blink_timer: 3.0,
            blink_amount: 0.0,
            config_path,
        }
    }
}

impl CatConfig {
    /// Saves current settings to disk.
    pub fn save_to_disk(&self) {
        if let Ok(json) = serde_json::to_string_pretty(&self.settings) {
            if let Some(parent) = self.config_path.parent() {
                let _ = fs::create_dir_all(parent);
            }
            if let Err(err) = fs::write(&self.config_path, json) {
                eprintln!("Failed to save cat settings: {err}");
            }
        }
    }
}

fn get_default_config_path() -> PathBuf {
    // If standard HOME is available, use ~/.config/cookie-cat/settings.json
    if let Ok(home) = std::env::var("HOME") {
        let p = Path::new(&home).join(".config").join("cookie-cat").join("settings.json");
        return p;
    }
    // Fallback to local working directory
    PathBuf::from("cat_settings.json")
}

fn load_settings(path: &Path) -> CatSettings {
    if path.exists() {
        if let Ok(data) = fs::read_to_string(path) {
            if let Ok(settings) = serde_json::from_str::<CatSettings>(&data) {
                return settings;
            }
        }
    }
    CatSettings::default()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_coat_cycling() {
        let mut coat = CoatColor::Biscuit;
        coat = coat.next();
        assert_eq!(coat, CoatColor::White);
        coat = coat.next();
        assert_eq!(coat, CoatColor::Grey);
        coat = coat.next();
        assert_eq!(coat, CoatColor::Biscuit);
    }

    #[test]
    fn test_cat_size_scale_factors() {
        assert_eq!(CatSize::Small.scale_factor(), 0.80);
        assert_eq!(CatSize::Normal.scale_factor(), 1.00);
        assert_eq!(CatSize::Large.scale_factor(), 1.25);
        assert_eq!(CatSize::ExtraLarge.scale_factor(), 1.50);

        let mut size = CatSize::Small;
        size = size.next();
        assert_eq!(size, CatSize::Normal);
    }

    #[test]
    fn test_speed_settings() {
        assert!(SpeedSetting::Relaxed.pixels_per_second() < SpeedSetting::Normal.pixels_per_second());
        assert!(SpeedSetting::Normal.pixels_per_second() < SpeedSetting::Playful.pixels_per_second());
        assert_eq!(SpeedSetting::Relaxed.next(), SpeedSetting::Normal);
    }

    #[test]
    fn test_idle_frequency() {
        let dur = IdleFrequency::Medium.random_idle_duration(0.5);
        assert!(dur >= 3.0 && dur <= 7.0);
    }

    #[test]
    fn test_settings_serialization_roundtrip() {
        let settings = CatSettings {
            coat: CoatColor::White,
            size: CatSize::Large,
            speed: SpeedSetting::Playful,
            idle_frequency: IdleFrequency::High,
            enable_roaming: false,
            enable_head_look: true,
            enable_blinking: true,
            enable_particles: false,
            enable_gravity: true,
            cat_visible: true,
            paused: false,
        };

        let json = serde_json::to_string(&settings).expect("Serialization failed");
        let deserialized: CatSettings = serde_json::from_str(&json).expect("Deserialization failed");

        assert_eq!(deserialized.coat, CoatColor::White);
        assert_eq!(deserialized.size, CatSize::Large);
        assert_eq!(deserialized.speed, SpeedSetting::Playful);
        assert_eq!(deserialized.idle_frequency, IdleFrequency::High);
        assert!(!deserialized.enable_roaming);
        assert!(!deserialized.enable_particles);
    }

    #[test]
    fn test_cat_ai_state_display_names() {
        assert_eq!(CatAiState::Idle.display_name(), "Idle");
        assert_eq!(CatAiState::Walking.display_name(), "Walking");
        assert_eq!(CatAiState::Sleeping.display_name(), "Sleeping / Napping");
        assert_eq!(CatAiState::LookingAround.display_name(), "Looking Around");
        assert_eq!(CatAiState::Investigating.display_name(), "Investigating / Sniffing");
    }
}
