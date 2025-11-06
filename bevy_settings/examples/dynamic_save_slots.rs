//! # Dynamic Save Game Slots Example
//!
//! This example demonstrates using placeholder names in SettingsStore
//! for dynamic save game management. The placeholder name can be used
//! to organize settings by context (e.g., different save game slots).

use bevy::prelude::*;
use bevy_settings::{prelude::*, Settings};
use serde::{Deserialize, Serialize};

/// Player progress settings for a save game
#[derive(Settings, Resource, Serialize, Deserialize, Default, Clone, PartialEq, Debug)]
struct PlayerProgress {
    level: u32,
    experience: u64,
    gold: u32,
}

/// World state settings for a save game
#[derive(Settings, Resource, Serialize, Deserialize, Default, Clone, PartialEq, Debug)]
struct WorldState {
    day: u32,
    completed_quests: Vec<String>,
}

fn main() {
    println!("=== Dynamic Save Game Slots Example ===\n");

    // Simulate loading save game slot 1
    // The [slot1] placeholder will be used as a prefix for settings files
    // Files will be named: slot1_PlayerProgress.json, slot1_WorldState.json
    println!("Loading Save Game Slot 1...\n");

    App::new()
        .add_plugins(MinimalPlugins)
        .add_plugins(
            SettingsStore::new("[slot1]")
                .format(SerializationFormat::Json)
                .version("0.1.0")
                .with_base_path("SaveGames")
                .register::<PlayerProgress>()
                .register::<WorldState>(),
        )
        .add_systems(Startup, print_initial_state)
        .add_systems(Update, (simulate_gameplay, check_and_exit).chain())
        .run();
}

fn print_initial_state(progress: Res<PlayerProgress>, world: Res<WorldState>) {
    println!("Initial Game State:");
    println!("  Player: {:?}", *progress);
    println!("  World: {:?}\n", *world);
}

fn simulate_gameplay(
    time: Res<Time>,
    mut progress: ResMut<PlayerProgress>,
    mut world: ResMut<WorldState>,
    mut modified: Local<bool>,
) {
    // Simulate some gameplay after a short delay
    if !*modified && time.elapsed_seconds() > 0.1 {
        *modified = true;

        println!("Simulating gameplay progress...\n");

        // Update player progress
        progress.level = 5;
        progress.experience = 1234;
        progress.gold = 500;
        println!("Player Progress: level={}, exp={}, gold={}", 
                 progress.level, progress.experience, progress.gold);

        // Update world state
        world.day = 10;
        world.completed_quests = vec![
            "Tutorial".to_string(),
            "FirstBoss".to_string(),
        ];
        println!("World State: day={}, quests={:?}\n", 
                 world.day, world.completed_quests);

        println!("Game state will be saved automatically!");
        println!("Check the 'SaveGames/' directory for:");
        println!("  - slot1_PlayerProgress.json");
        println!("  - slot1_WorldState.json\n");
        println!("Note: The '[slot1]' placeholder becomes 'slot1_' prefix in file names.");
        println!("This allows you to have multiple save slots (e.g., [slot2], [slot3])");
        println!("in the same directory without conflicts.\n");
    }
}

fn check_and_exit(time: Res<Time>) {
    if time.elapsed_seconds() > 2.0 {
        println!("Game state saved. Exiting...");
        std::process::exit(0);
    }
}
