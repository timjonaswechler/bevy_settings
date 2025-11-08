use bevy::prelude::*;
use bevy_settings::{prelude::*, Settings};
use serde::{Deserialize, Serialize};

/// Network settings with versioning and migration support
///
/// This example demonstrates:
/// 1. Using a custom SECTION name
/// 2. Implementing migration logic for version changes
/// 3. Tracking versions per settings section
#[derive(Resource, Serialize, Deserialize, Clone, PartialEq, Debug)]
struct NetworkSettings {
    server_url: String,
    port: u16,
    #[serde(default)]
    timeout_seconds: Option<u32>,
}

impl Default for NetworkSettings {
    fn default() -> Self {
        Self {
            server_url: "https://example.com".to_string(),
            port: 8080,
            timeout_seconds: Some(30),
        }
    }
}

impl Settings for NetworkSettings {
    fn type_name() -> &'static str {
        "NetworkSettings"
    }

    // Custom section name instead of default "networksettings"
    const SECTION: &'static str = "network";

    fn migrate(
        file_version: Option<&semver::Version>,
        target_version: &semver::Version,
        mut data: serde_json::Value,
    ) -> Result<(serde_json::Value, bool), SettingsError> {
        let mut changed = false;

        println!(
            "Migrating network settings from {:?} to {}",
            file_version, target_version
        );

        // Example migration: Version 1.x didn't have timeout_seconds
        // In version 2.0.0+, we added this field
        if let Some(file_ver) = file_version {
            if file_ver < &semver::Version::new(2, 0, 0)
                && target_version >= &semver::Version::new(2, 0, 0)
            {
                if let serde_json::Value::Object(ref mut map) = data {
                    if !map.contains_key("timeout_seconds") {
                        println!("Adding timeout_seconds field during migration");
                        map.insert(
                            "timeout_seconds".to_string(),
                            serde_json::Value::Number(30.into()),
                        );
                        changed = true;
                    }
                }
            }
        }

        Ok((data, changed))
    }
}

/// Game settings demonstrating another settings type
#[derive(Settings, Resource, Serialize, Deserialize, Default, Clone, PartialEq, Debug)]
struct GameSettings {
    volume: f32,
    fullscreen: bool,
}

fn main() {
    println!("=== Migration Example ===");
    println!();
    println!("This example demonstrates settings migration between versions.");
    println!();

    App::new()
        .add_plugins(MinimalPlugins)
        // Register multiple settings types with individual versions
        .add_plugins(
            SettingsPlugin::new("GameSettings")
                .format(SerializationFormat::Json)
                .with_base_path("config")
                // Network settings are at version 2.0.0 (has migration logic)
                .register_with_version::<NetworkSettings>("2.0.0")
                // Game settings don't need versioning yet
                .register::<GameSettings>(),
        )
        .add_systems(Startup, print_settings)
        .add_systems(Update, (modify_settings, check_and_exit))
        .run();
}

fn print_settings(network: Res<NetworkSettings>, game: Res<GameSettings>) {
    println!("=== Current Settings ===");
    println!();
    println!("Network Settings (section: {})", NetworkSettings::SECTION);
    println!("  Server URL: {}", network.server_url);
    println!("  Port: {}", network.port);
    println!("  Timeout: {:?} seconds", network.timeout_seconds);
    println!();
    println!("Game Settings (section: {})", GameSettings::SECTION);
    println!("  Volume: {}", game.volume);
    println!("  Fullscreen: {}", game.fullscreen);
    println!();
    println!("Settings file: config/GameSettings.json");
    println!();
}

fn modify_settings(
    mut network: ResMut<NetworkSettings>,
    mut game: ResMut<GameSettings>,
    time: Res<Time>,
) {
    // Modify settings after 0.5 seconds (only once)
    if time.elapsed_seconds() > 0.5 && time.elapsed_seconds() < 0.51 {
        network.server_url = "https://api.example.com".to_string();
        network.port = 9000;
        network.timeout_seconds = Some(60);

        game.volume = 0.8;
        game.fullscreen = false;

        println!("Settings modified! They will be saved automatically.");
        println!();
        println!("To test migration:");
        println!("1. Check the created file: config/GameSettings.json");
        println!("2. Manually edit the file:");
        println!("   - Remove 'timeout_seconds' from the 'network' section");
        println!("   - Change '_versions' -> 'network' to '1.0.0'");
        println!("3. Run this example again");
        println!("4. The migration will add 'timeout_seconds' back!");
        println!();
    }
}

fn check_and_exit(time: Res<Time>) {
    // Exit after 2 seconds
    if time.elapsed_seconds() > 2.0 {
        println!("Example complete!");
        std::process::exit(0);
    }
}
