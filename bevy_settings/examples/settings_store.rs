//! # SettingsStore API Example
//!
//! This example demonstrates the new fluent SettingsStore API that closely
//! matches Bevy's builder pattern style.

use bevy::prelude::*;
use bevy_settings::{prelude::*, Settings};
use serde::{Deserialize, Serialize};

/// Input settings example
#[derive(Settings, Resource, Serialize, Deserialize, Default, Clone, PartialEq, Debug)]
struct Input {
    mouse_sensitivity: f32,
    invert_y: bool,
}

/// Audio settings example
#[derive(Settings, Resource, Serialize, Deserialize, Default, Clone, PartialEq, Debug)]
struct Audio {
    master_volume: f32,
    music_volume: f32,
    sfx_volume: f32,
}

/// Graphics settings example
#[derive(Settings, Resource, Serialize, Deserialize, Default, Clone, PartialEq, Debug)]
struct Graphics {
    quality: u8,
    vsync: bool,
}

fn main() {
    println!("=== SettingsStore API Example ===\n");

    App::new()
        .add_plugins(MinimalPlugins)
        // Use the new SettingsStore API - register all settings with a single store
        .add_plugins(
            SettingsStore::new("GameSettings")
                .format(SerializationFormat::Json)
                .version("0.1.0")
                .with_base_path("settings")
                .register::<Input>()
                .register::<Audio>()
                .register::<Graphics>(),
        )
        .add_systems(Startup, print_initial_settings)
        .add_systems(Update, (modify_settings, check_and_exit).chain())
        .run();
}

fn print_initial_settings(
    input: Res<Input>,
    audio: Res<Audio>,
    graphics: Res<Graphics>,
) {
    println!("Initial Settings (from 'settings' folder):");
    println!("  Input: {:?}", *input);
    println!("  Audio: {:?}", *audio);
    println!("  Graphics: {:?}\n", *graphics);
}

fn modify_settings(
    time: Res<Time>,
    mut input: ResMut<Input>,
    mut audio: ResMut<Audio>,
    mut graphics: ResMut<Graphics>,
    mut modified: Local<bool>,
) {
    // Only modify once, after a short delay
    if !*modified && time.elapsed_seconds() > 0.1 {
        *modified = true;

        println!("Modifying settings...\n");

        // Modify settings
        input.mouse_sensitivity = 2.5;
        input.invert_y = true;
        println!("Modified Input settings: sensitivity=2.5, invert_y=true");

        audio.master_volume = 0.8;
        audio.music_volume = 0.6;
        audio.sfx_volume = 0.7;
        println!("Modified Audio settings: master=0.8, music=0.6, sfx=0.7");

        graphics.quality = 3;
        graphics.vsync = true;
        println!("Modified Graphics settings: quality=3, vsync=true\n");

        println!("Settings will be saved automatically!");
        println!("Check the 'settings/' directory for:");
        println!("  - Input.json");
        println!("  - Audio.json");
        println!("  - Graphics.json\n");
    }
}

fn check_and_exit(time: Res<Time>) {
    if time.elapsed_seconds() > 2.0 {
        println!("\nSettings have been saved. Exiting...");
        std::process::exit(0);
    }
}
