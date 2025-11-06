//! # SettingsStore as Resource Example
//!
//! This example demonstrates using SettingsStore as a Resource,
//! giving you manual control over settings management.

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

fn main() {
    println!("=== SettingsStore as Resource Example ===\n");

    // Create settings store as a resource
    let settings_store = SettingsStore::new("GameSettings")
        .format(SerializationFormat::Json)
        .version("0.1.0")
        .with_base_path("settings")
        .register::<Input>()
        .register::<Audio>();

    App::new()
        .add_plugins(MinimalPlugins)
        // Insert the settings store as a resource
        .insert_resource(settings_store)
        .add_systems(Startup, print_store_info)
        .run();
}

fn print_store_info(store: Res<SettingsStore>) {
    println!("SettingsStore Information:");
    println!("  Name: {}", store.get_name());
    println!("  Version: {:?}", store.get_version());
    println!("  Format: {:?}", store.get_format());
    println!("  Base Path: {:?}", store.get_base_path_option());
    println!("\nSettingsStore is now available as a resource!");
    println!("You can access it in any system using Res<SettingsStore>");
    
    std::process::exit(0);
}
