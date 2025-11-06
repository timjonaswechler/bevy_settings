//! # Advanced Features Example
//!
//! This example demonstrates:
//! - Using nested structs in settings
//! - Using enums in settings
//! - Using vectors in settings
//! - Multiple settings types
//! - Custom base path for settings storage

use bevy::prelude::*;
use bevy_settings::{prelude::*, Settings};
use serde::{Deserialize, Serialize};

/// Video settings with nested configuration
#[derive(Settings, Resource, Serialize, Deserialize, Default, Clone, PartialEq, Debug)]
struct VideoSettings {
    resolution: Resolution,
    display_mode: DisplayMode,
    vsync: bool,
    fps_limit: Option<u32>,
}

#[derive(Serialize, Deserialize, Clone, PartialEq, Debug)]
struct Resolution {
    width: u32,
    height: u32,
}

impl Default for Resolution {
    fn default() -> Self {
        Self {
            width: 1920,
            height: 1080,
        }
    }
}

#[derive(Serialize, Deserialize, Default, Clone, PartialEq, Debug)]
enum DisplayMode {
    #[default]
    Windowed,
    Fullscreen,
    BorderlessFullscreen,
}

/// Audio settings with multiple channels
#[derive(Settings, Resource, Serialize, Deserialize, Default, Clone, PartialEq, Debug)]
struct AudioSettings {
    master: f32,
    music: f32,
    sfx: f32,
    voice: f32,
    muted_channels: Vec<String>,
}

/// Gameplay settings with preferences
#[derive(Settings, Resource, Serialize, Deserialize, Default, Clone, PartialEq, Debug)]
struct GameplaySettings {
    difficulty: Difficulty,
    auto_save: bool,
    mouse_sensitivity: f32,
    invert_y: bool,
}

#[derive(Serialize, Deserialize, Default, Clone, PartialEq, Debug)]
enum Difficulty {
    Easy,
    #[default]
    Normal,
    Hard,
    Nightmare,
}

fn main() {
    println!("=== Advanced Settings Example ===\n");

    let mut app = App::new();

    app.add_plugins(MinimalPlugins)
        // Add multiple settings with different formats using the new API
        .add_plugins(
            SettingsPlugin::new()
                .register::<VideoSettings>(
                    SettingsConfig::new("video", SerializationFormat::Json)
                        .with_base_path("config"),
                )
                .register::<AudioSettings>(
                    SettingsConfig::new("audio", SerializationFormat::Json)
                        .with_base_path("config"),
                )
                .register::<GameplaySettings>(
                    SettingsConfig::new("gameplay", SerializationFormat::Binary)
                        .with_base_path("config"),
                ),
        )
        .add_systems(Startup, print_initial_settings)
        .add_systems(Update, (modify_settings, check_and_exit).chain());

    app.run();
}

fn print_initial_settings(
    video: Res<VideoSettings>,
    audio: Res<AudioSettings>,
    gameplay: Res<GameplaySettings>,
) {
    println!("Initial Settings:");
    println!("  Video: {:?}", *video);
    println!("  Audio: {:?}", *audio);
    println!("  Gameplay: {:?}\n", *gameplay);
}

fn modify_settings(
    time: Res<Time>,
    mut video: ResMut<VideoSettings>,
    mut audio: ResMut<AudioSettings>,
    mut gameplay: ResMut<GameplaySettings>,
    mut modified: Local<bool>,
) {
    // Only modify once, after a short delay to ensure the app is fully initialized
    if !*modified && time.elapsed_seconds() > 0.1 {
        *modified = true;

        // Modify video settings
        video.resolution = Resolution {
            width: 2560,
            height: 1440,
        };
        video.display_mode = DisplayMode::BorderlessFullscreen;
        video.fps_limit = Some(144);
        println!("Modified video settings to 2560x1440, borderless fullscreen, 144 fps");

        // Modify audio settings
        audio.master = 0.8;
        audio.music = 0.6;
        audio.sfx = 0.9;
        audio.muted_channels = vec!["ambient".to_string()];
        println!("Modified audio settings: master=0.8, music=0.6, sfx=0.9");

        // Modify gameplay settings
        gameplay.difficulty = Difficulty::Hard;
        gameplay.mouse_sensitivity = 1.5;
        gameplay.invert_y = true;
        println!("Modified gameplay settings: Hard difficulty, 1.5 sensitivity, inverted Y\n");

        println!("Settings will be saved automatically!");
        println!("Check the 'config/' directory for:");
        println!("  - video.json (JSON format)");
        println!("  - audio.json (JSON format)");
        println!("  - gameplay.bin (Binary format)\n");
    }
}

fn check_and_exit(time: Res<Time>) {
    if time.elapsed_seconds() > 2.0 {
        println!("Settings have been saved. Exiting...");
        std::process::exit(0);
    }
}
