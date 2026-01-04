//! # New API Example
//!
//! This example demonstrates the new simplified API where you only need to add
//! one SettingsPlugin and register all your settings types with it.

use bevy::prelude::*;
use bevy_settings::{prelude::*, Settings};
use serde::{Deserialize, Serialize};

/// Example game settings with various configuration options
#[derive(Settings, Resource, Serialize, Deserialize, Default, Clone, PartialEq, Debug)]
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

/// Example graphics settings
#[derive(Settings, Resource, Serialize, Deserialize, Default, Clone, PartialEq, Debug)]
struct GraphicsSettings {
    /// Graphics quality level
    quality: GraphicsQuality,
    /// Enable vsync
    vsync: bool,
    /// Field of view
    fov: f32,
}

#[derive(Serialize, Deserialize, Default, Clone, PartialEq, Debug)]
enum GraphicsQuality {
    Low,
    #[default]
    Medium,
    High,
    Ultra,
}

/// Example audio settings
#[derive(Settings, Resource, Serialize, Deserialize, Default, Clone, PartialEq, Debug)]
struct AudioSettings {
    master: f32,
    music: f32,
    sfx: f32,
}

fn main() {
    println!("=== New API Example ===\n");

    App::new()
        .add_plugins(MinimalPlugins)
        // NEW API: Register all settings types with a single plugin!
        .add_plugins(
            SettingsPlugin::new("GameSettings")
                .format(SerializationFormat::Json)
                .version("0.1.0")
                .with_base_path("config")
                .register::<GameSettings>()
                .register::<GraphicsSettings>()
                .register::<AudioSettings>(),
        )
        .add_systems(Startup, print_initial_settings)
        .add_systems(Update, (modify_settings, check_and_exit).chain())
        .run();
}

fn print_initial_settings(
    game_settings: Res<GameSettings>,
    graphics_settings: Res<GraphicsSettings>,
    audio_settings: Res<AudioSettings>,
) {
    println!("Initial Settings:");
    println!("  Game: {:?}", *game_settings);
    println!("  Graphics: {:?}", *graphics_settings);
    println!("  Audio: {:?}\n", *audio_settings);
}

fn modify_settings(
    time: Res<Time>,
    mut game_settings: ResMut<GameSettings>,
    mut graphics_settings: ResMut<GraphicsSettings>,
    mut audio_settings: ResMut<AudioSettings>,
    mut modified: Local<bool>,
) {
    // Only modify once, after a short delay
    if !*modified && time.elapsed_secs() > 0.1 {
        *modified = true;

        // Modify game settings
        game_settings.volume = 0.8;
        game_settings.fullscreen = true;
        println!("Modified game settings: volume=0.8, fullscreen=true");

        // Modify graphics settings
        graphics_settings.quality = GraphicsQuality::High;
        graphics_settings.vsync = true;
        println!("Modified graphics settings: High quality, vsync=true");

        // Modify audio settings
        audio_settings.master = 0.9;
        audio_settings.music = 0.6;
        audio_settings.sfx = 0.8;
        println!("Modified audio settings: master=0.9, music=0.6, sfx=0.8\n");

        println!("Settings will be saved automatically!");
        println!("Check the 'config/' directory for saved files.\n");
    }
}

fn check_and_exit(time: Res<Time>) {
    if time.elapsed_secs() > 2.0 {
        println!("Settings have been saved. Exiting...");
        std::process::exit(0);
    }
}
