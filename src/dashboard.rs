//! Settings & Control Dashboard Module
//!
//! Provides a dedicated lightweight dashboard window with controls for:
//! - Cat status (active/paused, visible/hidden, coat color, size, speed, idle frequency)
//! - Behavior toggles (random roaming, head/face look, natural blinking, gravity, emotion particles)
//! - Quick actions (reset to ground, save settings)
//!
//! Designed so that closing the dashboard never terminates the application.

use bevy::camera::ClearColorConfig;
use bevy::prelude::*;
use bevy::ui::UiTargetCamera;
use bevy::window::{PrimaryWindow, WindowCloseRequested, WindowRef};

use crate::config::CatConfig;
use crate::tray::DashboardRequested;

pub struct DashboardPlugin;

impl Plugin for DashboardPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            Update,
            (
                check_open_dashboard_request,
                handle_dashboard_buttons,
                handle_dashboard_window_close,
                update_dashboard_ui_labels,
            ),
        );
    }
}

/// Marker for the secondary dashboard window entity.
#[derive(Component)]
pub struct DashboardWindow;

/// Marker for the 2D camera rendering the dashboard UI.
#[derive(Component)]
pub struct DashboardCamera;

/// Marker for the root UI container of the dashboard.
#[derive(Component)]
pub struct DashboardRoot;

/// Actions that buttons inside the dashboard can trigger.
#[derive(Component, Clone, Copy, PartialEq, Eq)]
pub enum DashboardAction {
    TogglePause,
    ToggleVisibility,
    CycleCoat,
    CycleSize,
    CycleSpeed,
    CycleIdleFreq,
    ToggleRoaming,
    ToggleHeadLook,
    ToggleBlinking,
    ToggleGravity,
    ToggleParticles,
    ResetToGround,
}

/// Label marker component for dynamic text updates.
#[derive(Component)]
pub struct DashboardLabel(pub DashboardAction);

/// Marker for the live cat behavior state label in the dashboard.
#[derive(Component)]
pub struct LiveStateLabel;

/// Spawns the dashboard window if requested and not already open.
fn check_open_dashboard_request(
    mut commands: Commands,
    mut dashboard_req: ResMut<DashboardRequested>,
    q_existing: Query<Entity, With<DashboardWindow>>,
    config: Res<CatConfig>,
) {
    if !dashboard_req.0 {
        return;
    }
    dashboard_req.0 = false;

    // If dashboard window already exists, do not open another
    if !q_existing.is_empty() {
        println!("Dashboard is already open.");
        return;
    }

    // 1. Spawn secondary window for dashboard
    let window_entity = commands
        .spawn((
            Window {
                title: "Desktop Cat Dashboard".into(),
                resolution: (380, 560).into(),
                resizable: false,
                decorations: true,
                ..default()
            },
            DashboardWindow,
        ))
        .id();

    // 2. Spawn 2D camera targeting this specific window
    let camera_entity = commands
        .spawn((
            Camera2d,
            Camera {
                clear_color: ClearColorConfig::Custom(Color::srgb(0.11, 0.12, 0.15)),
                ..default()
            },
            bevy::camera::RenderTarget::Window(WindowRef::Entity(window_entity)),
            DashboardCamera,
        ))
        .id();

    // 3. Build Dashboard UI hierarchy
    build_dashboard_ui(&mut commands, camera_entity, &config);
    println!("Dashboard window opened.");
}

/// Closes the dashboard window when its close button is clicked, without exiting the app.
fn handle_dashboard_window_close(
    mut commands: Commands,
    mut close_events: MessageReader<WindowCloseRequested>,
    q_dashboard_win: Query<Entity, With<DashboardWindow>>,
    q_dashboard_cam: Query<Entity, With<DashboardCamera>>,
    q_dashboard_root: Query<Entity, With<DashboardRoot>>,
) {
    for event in close_events.read() {
        if let Ok(dash_win) = q_dashboard_win.get(event.window) {
            // Despawn dashboard window, camera, and root UI
            commands.entity(dash_win).despawn();

            for cam in &q_dashboard_cam {
                commands.entity(cam).despawn();
            }
            for root in &q_dashboard_root {
                commands.entity(root).despawn();
            }

            println!("Dashboard closed. Desktop Cat continues running.");
        }
    }
}

/// Handles button interactions inside the dashboard.
#[allow(clippy::type_complexity)]
fn handle_dashboard_buttons(
    mut interaction_query: Query<
        (&Interaction, &DashboardAction, &mut BackgroundColor),
        (Changed<Interaction>, With<Button>),
    >,
    mut config: ResMut<CatConfig>,
    mut q_primary_window: Query<&mut Window, (With<PrimaryWindow>, Without<DashboardWindow>)>,
) {
    for (interaction, action, mut bg_color) in &mut interaction_query {
        match *interaction {
            Interaction::Pressed => {
                *bg_color = BackgroundColor(Color::srgb(0.24, 0.28, 0.38));

                match action {
                    DashboardAction::TogglePause => {
                        config.settings.paused = !config.settings.paused;
                    }
                    DashboardAction::ToggleVisibility => {
                        config.settings.cat_visible = !config.settings.cat_visible;
                        if let Ok(mut win) = q_primary_window.single_mut() {
                            win.visible = config.settings.cat_visible;
                        }
                    }
                    DashboardAction::CycleCoat => {
                        config.settings.coat = config.settings.coat.next();
                    }
                    DashboardAction::CycleSize => {
                        config.settings.size = config.settings.size.next();
                    }
                    DashboardAction::CycleSpeed => {
                        config.settings.speed = config.settings.speed.next();
                    }
                    DashboardAction::CycleIdleFreq => {
                        config.settings.idle_frequency = config.settings.idle_frequency.next();
                    }
                    DashboardAction::ToggleRoaming => {
                        config.settings.enable_roaming = !config.settings.enable_roaming;
                    }
                    DashboardAction::ToggleHeadLook => {
                        config.settings.enable_head_look = !config.settings.enable_head_look;
                    }
                    DashboardAction::ToggleBlinking => {
                        config.settings.enable_blinking = !config.settings.enable_blinking;
                    }
                    DashboardAction::ToggleGravity => {
                        config.settings.enable_gravity = !config.settings.enable_gravity;
                    }
                    DashboardAction::ToggleParticles => {
                        config.settings.enable_particles = !config.settings.enable_particles;
                    }
                    DashboardAction::ResetToGround => {
                        if let Some(pos) = config.desktop_pos {
                            let ground_y = config.ground_y;
                            config.desktop_pos = Some(Vec2::new(pos.x, ground_y));
                            config.vertical_velocity = 0.0;
                            if let Ok(mut win) = q_primary_window.single_mut() {
                                win.position = bevy::window::WindowPosition::At(
                                    bevy::math::IVec2::new(pos.x as i32, ground_y as i32),
                                );
                            }
                        }
                    }
                }

                // Automatically persist changed settings to disk
                config.save_to_disk();
            }
            Interaction::Hovered => {
                *bg_color = BackgroundColor(Color::srgb(0.20, 0.23, 0.30));
            }
            Interaction::None => {
                *bg_color = BackgroundColor(Color::srgb(0.15, 0.17, 0.22));
            }
        }
    }
}

/// Updates UI text values dynamically when settings change.
fn update_dashboard_ui_labels(
    config: Res<CatConfig>,
    mut q_labels: Query<(&DashboardLabel, &mut Text), Without<LiveStateLabel>>,
    mut q_live: Query<&mut Text, With<LiveStateLabel>>,
) {
    for mut text in &mut q_live {
        **text = format!("Current Action: {}", config.state.display_name());
    }

    if !config.is_changed() {
        return;
    }

    for (label, mut text) in &mut q_labels {
        let new_text = match label.0 {
            DashboardAction::TogglePause => {
                if config.settings.paused {
                    "Status: [PAUSED]".to_string()
                } else {
                    "Status: [ACTIVE]".to_string()
                }
            }
            DashboardAction::ToggleVisibility => {
                if config.settings.cat_visible {
                    "Cat Visibility: [VISIBLE]".to_string()
                } else {
                    "Cat Visibility: [HIDDEN]".to_string()
                }
            }
            DashboardAction::CycleCoat => {
                format!("Coat: {}", config.settings.coat.display_name())
            }
            DashboardAction::CycleSize => {
                format!("Size: {}", config.settings.size.display_name())
            }
            DashboardAction::CycleSpeed => {
                format!("Speed: {}", config.settings.speed.display_name())
            }
            DashboardAction::CycleIdleFreq => {
                format!("Idle Frequency: {}", config.settings.idle_frequency.display_name())
            }
            DashboardAction::ToggleRoaming => {
                if config.settings.enable_roaming {
                    "Random Roaming: [ON]".to_string()
                } else {
                    "Random Roaming: [OFF]".to_string()
                }
            }
            DashboardAction::ToggleHeadLook => {
                if config.settings.enable_head_look {
                    "Head & Eye Tracking: [ON]".to_string()
                } else {
                    "Head & Eye Tracking: [OFF]".to_string()
                }
            }
            DashboardAction::ToggleBlinking => {
                if config.settings.enable_blinking {
                    "Natural Blinking: [ON]".to_string()
                } else {
                    "Natural Blinking: [OFF]".to_string()
                }
            }
            DashboardAction::ToggleGravity => {
                if config.settings.enable_gravity {
                    "Desktop Gravity: [ON]".to_string()
                } else {
                    "Desktop Gravity: [OFF]".to_string()
                }
            }
            DashboardAction::ToggleParticles => {
                if config.settings.enable_particles {
                    "Emotion Particles: [ON]".to_string()
                } else {
                    "Emotion Particles: [OFF]".to_string()
                }
            }
            DashboardAction::ResetToGround => "Snap to Ground".to_string(),
        };

        **text = new_text;
    }
}

/// Builds the UI layout using modern dark aesthetics.
fn build_dashboard_ui(
    commands: &mut Commands,
    camera_entity: Entity,
    config: &CatConfig,
) {
    commands
        .spawn((
            DashboardRoot,
            UiTargetCamera(camera_entity),
            Node {
                width: Val::Percent(100.0),
                height: Val::Percent(100.0),
                flex_direction: FlexDirection::Column,
                padding: UiRect::all(Val::Px(16.0)),
                row_gap: Val::Px(10.0),
                ..default()
            },
            BackgroundColor(Color::srgb(0.09, 0.10, 0.13)),
        ))
        .with_children(|root| {
            // Header
            root.spawn((
                Text::new("🐱 Desktop Cat Dashboard"),
                TextFont::from_font_size(18.0),
                TextColor(Color::srgb(0.95, 0.95, 0.98)),
            ));

            root.spawn((
                Text::new("Configure behavior, appearance & physics"),
                TextFont::from_font_size(11.5),
                TextColor(Color::srgb(0.60, 0.63, 0.70)),
            ));

            root.spawn((
                LiveStateLabel,
                Text::new(format!("Current Action: {}", config.state.display_name())),
                TextFont::from_font_size(12.0),
                TextColor(Color::srgb(0.98, 0.78, 0.45)),
            ));

            // Section 1: Cat & Motion
            spawn_section_header(root, "CAT CONTROLS");
            spawn_dashboard_button(
                root,
                DashboardAction::TogglePause,
                if config.settings.paused { "Status: [PAUSED]" } else { "Status: [ACTIVE]" },
            );
            spawn_dashboard_button(
                root,
                DashboardAction::ToggleVisibility,
                if config.settings.cat_visible { "Cat Visibility: [VISIBLE]" } else { "Cat Visibility: [HIDDEN]" },
            );
            spawn_dashboard_button(
                root,
                DashboardAction::CycleCoat,
                &format!("Coat: {}", config.settings.coat.display_name()),
            );
            spawn_dashboard_button(
                root,
                DashboardAction::CycleSize,
                &format!("Size: {}", config.settings.size.display_name()),
            );
            spawn_dashboard_button(
                root,
                DashboardAction::CycleSpeed,
                &format!("Speed: {}", config.settings.speed.display_name()),
            );
            spawn_dashboard_button(
                root,
                DashboardAction::CycleIdleFreq,
                &format!("Idle Frequency: {}", config.settings.idle_frequency.display_name()),
            );

            // Section 2: Behavior Toggles
            spawn_section_header(root, "BEHAVIOR & PHYSICS");
            spawn_dashboard_button(
                root,
                DashboardAction::ToggleRoaming,
                if config.settings.enable_roaming { "Random Roaming: [ON]" } else { "Random Roaming: [OFF]" },
            );
            spawn_dashboard_button(
                root,
                DashboardAction::ToggleHeadLook,
                if config.settings.enable_head_look { "Head & Eye Tracking: [ON]" } else { "Head & Eye Tracking: [OFF]" },
            );
            spawn_dashboard_button(
                root,
                DashboardAction::ToggleBlinking,
                if config.settings.enable_blinking { "Natural Blinking: [ON]" } else { "Natural Blinking: [OFF]" },
            );
            spawn_dashboard_button(
                root,
                DashboardAction::ToggleGravity,
                if config.settings.enable_gravity { "Desktop Gravity: [ON]" } else { "Desktop Gravity: [OFF]" },
            );
            spawn_dashboard_button(
                root,
                DashboardAction::ToggleParticles,
                if config.settings.enable_particles { "Emotion Particles: [ON]" } else { "Emotion Particles: [OFF]" },
            );

            // Section 3: Actions
            spawn_section_header(root, "ACTIONS");
            spawn_dashboard_button(root, DashboardAction::ResetToGround, "Snap to Ground");
        });
}

fn spawn_section_header(parent: &mut ChildSpawnerCommands, title: &str) {
    parent.spawn((
        Text::new(title),
        TextFont::from_font_size(11.0),
        TextColor(Color::srgb(0.50, 0.54, 0.65)),
        Node {
            margin: UiRect::top(Val::Px(6.0)),
            ..default()
        },
    ));
}

fn spawn_dashboard_button(
    parent: &mut ChildSpawnerCommands,
    action: DashboardAction,
    label: &str,
) {
    parent
        .spawn((
            Button,
            action,
            Node {
                width: Val::Percent(100.0),
                height: Val::Px(28.0),
                justify_content: JustifyContent::FlexStart,
                align_items: AlignItems::Center,
                padding: UiRect::axes(Val::Px(10.0), Val::Px(2.0)),
                border: UiRect::all(Val::Px(1.0)),
                border_radius: BorderRadius::all(Val::Px(5.0)),
                ..default()
            },
            BackgroundColor(Color::srgb(0.15, 0.17, 0.22)),
            BorderColor::all(Color::srgb(0.24, 0.27, 0.35)),
        ))
        .with_children(|btn| {
            btn.spawn((
                DashboardLabel(action),
                Text::new(label),
                TextFont::from_font_size(12.0),
                TextColor(Color::srgb(0.90, 0.92, 0.96)),
            ));
        });
}
