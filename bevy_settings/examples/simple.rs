use bevy::prelude::*;
use bevy_settings::{prelude::*, Settings};
use serde::{Deserialize, Serialize};

/// Simple audio settings example
#[derive(Settings, Resource, Serialize, Deserialize, Default, Clone, PartialEq, Debug)]
struct AudioSettings {
    #[serde(default)]
    master_volume: f32,
    #[serde(default)]
    music_volume: f32,
    #[serde(default)]
    sfx_volume: f32,
}

fn main() {
    App::new()
        .add_plugins(MinimalPlugins)
        // Add the settings plugin - it will automatically load settings on startup
        // and save them when they change
        .add_plugins(
            SettingsPlugin::new("AudioSettings")
                .format(SerializationFormat::Json)
                .with_base_path("config")
                .register::<AudioSettings>(),
        )
        .add_systems(Startup, print_settings)
        .add_systems(Update, modify_settings)
        .run();
}

fn print_settings(settings: Res<AudioSettings>) {
    info!("Current audio settings: {:?}", *settings);
    info!("Settings file will be saved to: config/AudioSettings.json");
}

fn modify_settings(mut settings: ResMut<AudioSettings>, time: Res<Time>) {
    // Modify settings after 2 seconds
    if time.elapsed_secs() > 2.0 && time.elapsed_secs() < 2.1 {
        settings.master_volume = 0.8;
        settings.music_volume = 0.6;
        settings.sfx_volume = 0.9;
        info!("Settings modified! They will be saved automatically.");
    }

    // Exit after 3 seconds
    if time.elapsed_secs() > 3.0 {
        std::process::exit(0);
    }
}
