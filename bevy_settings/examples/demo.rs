use bevy::prelude::*;
use bevy_settings::prelude::*; // Import everything
use serde::{Deserialize, Serialize};

// --- User Code ---

// 1. Definition einer Settings-Gruppe
#[derive(SettingsGroup, Resource, Serialize, Deserialize, Default, Clone, Reflect, PartialEq)]
#[reflect(Resource)]
#[settings(file = "settings/global.toml")]
struct GlobalSettings {
    audio: AudioConfig,
    graphics: GraphicsConfig,
}

#[derive(Serialize, Deserialize, Default, Clone, Reflect, PartialEq)]
struct AudioConfig {
    volume: f32,
    muted: bool,
}

#[derive(Serialize, Deserialize, Default, Clone, Reflect, PartialEq)]
struct GraphicsConfig {
    resolution_scale: f32,
    fullscreen: bool,
}

// 2. Definition einer dynamischen Gruppe (Savegame)
#[derive(SettingsGroup, Resource, Serialize, Deserialize, Default, Clone, Reflect, PartialEq)]
#[reflect(Resource)]
#[settings(file = "saves/{slot_id}/game.json")]
struct SaveGame {
    // Parameter wird automatisch erkannt und beim Speichern gefiltert
    slot_id: String,

    inventory: InventoryData,
}

#[derive(Serialize, Deserialize, Default, Clone, Reflect, PartialEq)]
struct InventoryData {
    gold: u32,
    items: Vec<String>,
}

// --- App Setup ---

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        // WICHTIG: Path Registry muss da sein
        .add_plugins(bevy_paths::prelude::PathRegistryPlugin::new(
            "MyStudio", "MyGame", "App",
        ))
        .add_plugins(
            SettingsPlugin::default()
                .register::<GlobalSettings>()
                .register::<SaveGame>(),
        )
        .add_systems(Update, (change_volume, load_savegame_input))
        .run();
}

fn change_volume(mut settings: ResMut<GlobalSettings>, time: Res<Time>) {
    // Ändere Volume dynamisch
    if time.elapsed_secs() > 2.0 && settings.audio.volume != 0.8 {
        info!("Changing volume...");
        settings.audio.volume = 0.8;
    }
}

fn load_savegame_input(
    mut commands: Commands,
    mut save_game: ResMut<SaveGame>,
    input: Res<ButtonInput<KeyCode>>,
) {
    if input.just_pressed(KeyCode::KeyL) {
        info!("Loading save slot 1...");
        save_game.slot_id = "slot_1".to_string();
        commands.load_settings::<SaveGame>();
    }

    if input.just_pressed(KeyCode::KeyS) {
        info!("Saving save slot 1 (manual trigger)...");
        commands.save_settings::<SaveGame>();
    }

    if input.just_pressed(KeyCode::KeyI) {
        save_game.inventory.gold += 10;
        info!("Added gold: {}", save_game.inventory.gold);
    }
}
