use bevy::prelude::*;
use bevy_settings::{Settings, prelude::*};
use serde::{Deserialize, Serialize};

/// Example game settings with various configuration options
#[derive(Resource, Serialize, Deserialize, Default, Clone, PartialEq, Debug)]
struct GameSettings {
    /// Master volume (0.0 - 1.0)
    volume: f32,
    /// Window resolution
    resolution: (u32, u32),
    /// Enable fullscreen mode
    fullscreen: bool,
    /// Player name
    player_name: String,
}

impl Settings for GameSettings {
    const SECTION: &'static str = "game";
}

/// Example graphics settings
#[derive(Resource, Serialize, Deserialize, Default, Clone, PartialEq, Debug)]
struct GraphicsSettings {
    /// Graphics quality level
    quality: GraphicsQuality,
    /// Enable vsync
    vsync: bool,
    /// Field of view
    fov: f32,
}

impl Settings for GraphicsSettings {
    const SECTION: &'static str = "graphics";
}

#[derive(Serialize, Deserialize, Default, Clone, PartialEq, Debug)]
enum GraphicsQuality {
    Low,
    #[default]
    Medium,
    High,
    Ultra,
}

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        // Add settings plugin - registers multiple settings types in one file
        .add_plugins(
            SettingsPlugin::new()
                .with_path("config/basic.json", SerializationFormat::Json)
                .register::<GameSettings>()
                .register::<GraphicsSettings>(),
        )
        .add_systems(Startup, setup)
        .add_systems(Update, (handle_input, display_settings))
        .run();
}

fn setup(
    mut commands: Commands,
    game_settings: Res<GameSettings>,
    graphics_settings: Res<GraphicsSettings>,
) {
    info!("=== Settings Example ===");
    info!("Game Settings: {:?}", *game_settings);
    info!("Graphics Settings: {:?}", *graphics_settings);
    info!("\nControls:");
    info!("  V - Toggle volume (between 0.0 and 1.0)");
    info!("  F - Toggle fullscreen");
    info!("  Q - Cycle graphics quality");
    info!("  R - Reset to defaults");
    info!("  ESC - Exit");

    commands.spawn(Camera2d::default());
}

fn handle_input(
    keyboard: Res<ButtonInput<KeyCode>>,
    mut game_settings: ResMut<GameSettings>,
    mut graphics_settings: ResMut<GraphicsSettings>,
    mut exit: MessageWriter<AppExit>,
) {
    // Toggle volume
    if keyboard.just_pressed(KeyCode::KeyV) {
        game_settings.volume = if game_settings.volume > 0.5 { 0.0 } else { 1.0 };
        info!("Volume changed to: {}", game_settings.volume);
    }

    // Toggle fullscreen
    if keyboard.just_pressed(KeyCode::KeyF) {
        game_settings.fullscreen = !game_settings.fullscreen;
        info!("Fullscreen: {}", game_settings.fullscreen);
    }

    // Cycle graphics quality
    if keyboard.just_pressed(KeyCode::KeyQ) {
        graphics_settings.quality = match graphics_settings.quality {
            GraphicsQuality::Low => GraphicsQuality::Medium,
            GraphicsQuality::Medium => GraphicsQuality::High,
            GraphicsQuality::High => GraphicsQuality::Ultra,
            GraphicsQuality::Ultra => GraphicsQuality::Low,
        };
        info!("Graphics quality: {:?}", graphics_settings.quality);
    }

    // Reset to defaults
    if keyboard.just_pressed(KeyCode::KeyR) {
        *game_settings = GameSettings::default();
        *graphics_settings = GraphicsSettings::default();
        info!("Settings reset to defaults");
    }

    // Exit
    if keyboard.just_pressed(KeyCode::Escape) {
        exit.write(AppExit::Success);
    }
}

fn display_settings(game_settings: Res<GameSettings>, graphics_settings: Res<GraphicsSettings>) {
    // This function demonstrates that settings are reactive
    // When settings change, this will automatically be called
    // You can use this pattern to apply settings changes to your game

    if game_settings.is_changed() && !game_settings.is_added() {
        info!("Game settings changed! New settings: {:?}", *game_settings);
    }

    if graphics_settings.is_changed() && !graphics_settings.is_added() {
        info!(
            "Graphics settings changed! New settings: {:?}",
            *graphics_settings
        );
    }
}
