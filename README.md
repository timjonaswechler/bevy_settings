# bevy_settings

A settings management system for [Bevy](https://bevyengine.org/) that:
- 🎯 Manages settings as Bevy resources
- 💾 Persists only deviations from default values (delta persistence)
- 📦 Supports JSON and binary (bincode) serialization formats
- 🚀 Provides a derive macro to reduce boilerplate
- 🔄 Automatically saves settings when they change

## Features

- **Delta Persistence**: Only values that differ from defaults are saved to disk, keeping settings files minimal
- **Multiple Formats**: Choose between human-readable JSON or compact binary format
- **Automatic Saving**: Settings are automatically saved when modified
- **Type-Safe**: Full Rust type safety with derive macros
- **Bevy Integration**: Works seamlessly with Bevy's resource system

## Installation

Add to your `Cargo.toml`:

```toml
[dependencies]
bevy_settings = "0.1"
```

## Quick Start

```rust
use bevy::prelude::*;
use bevy_settings::{prelude::*, Settings};
use serde::{Deserialize, Serialize};

// Define your settings structs
#[derive(Settings, Resource, Serialize, Deserialize, Default, Clone, PartialEq)]
struct GameSettings {
    volume: f32,
    resolution: (u32, u32),
    fullscreen: bool,
}

#[derive(Settings, Resource, Serialize, Deserialize, Default, Clone, PartialEq)]
struct GraphicsSettings {
    quality: i32,
}

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        // Register all settings with a single plugin
        .add_plugins(
            SettingsPlugin::new()
                .register::<GameSettings>(
                    SettingsConfig::new("game_settings", SerializationFormat::Json)
                )
                .register::<GraphicsSettings>(
                    SettingsConfig::new("graphics_settings", SerializationFormat::Json)
                )
        )
        .run();
}
```

That's it! Your settings will be:
- Loaded from `settings/*.json` on startup (or defaults if files don't exist)
- Automatically saved when modified
- Only stored if they differ from defaults

## Usage

### Defining Settings

Settings must implement several traits. The `Settings` derive macro handles this automatically:

```rust
#[derive(Settings, Resource, Serialize, Deserialize, Default, Clone, PartialEq)]
struct MySettings {
    value: i32,
}
```

Required trait implementations:
- `Settings` - Our derive macro
- `Resource` - Bevy resource
- `Serialize` + `Deserialize` - Serde serialization
- `Default` - Provides default values
- `Clone` - For copying settings
- `PartialEq` - For detecting changes from defaults

### Adding to Your App

```rust
use bevy_settings::{SettingsPlugin, SettingsConfig, SerializationFormat};

App::new()
    .add_plugins(
        SettingsPlugin::new()
            .register::<MySettings>(
                SettingsConfig::new("my_settings", SerializationFormat::Json)
            )
    )
    .run();
```

### Custom Settings Path

```rust
SettingsPlugin::new()
    .register::<MySettings>(
        SettingsConfig::new("my_settings", SerializationFormat::Json)
            .with_base_path("custom/path")
    )
```

### Reading Settings

Settings are available as Bevy resources:

```rust
fn my_system(settings: Res<MySettings>) {
    println!("Volume: {}", settings.volume);
}
```

### Modifying Settings

Modify settings like any Bevy resource. They'll be automatically saved:

```rust
fn modify_settings(mut settings: ResMut<MySettings>) {
    settings.volume = 0.8;
    // Settings will be saved automatically!
}
```

### Delta Persistence

The system only saves values that differ from defaults:

```rust
// If settings equal defaults, no file is created
let defaults = MySettings::default();
// File is deleted if it exists

// If settings differ from defaults, only those settings are saved
let modified = MySettings { volume: 0.5, ..default() };
// File is created with the modified values
```

## Serialization Formats

### JSON (Human-Readable)

```rust
SerializationFormat::Json
```

Creates human-readable `.json` files:
```json
{
  "volume": 0.8,
  "resolution": [1920, 1080],
  "fullscreen": true
}
```

### Binary (Compact)

```rust
SerializationFormat::Binary
```

Creates compact `.bin` files using [bincode](https://github.com/bincode-org/bincode).

## Examples

### Multiple Settings

You can register multiple settings types with a single plugin:

```rust
#[derive(Settings, Resource, Serialize, Deserialize, Default, Clone, PartialEq)]
struct GameSettings { /* ... */ }

#[derive(Settings, Resource, Serialize, Deserialize, Default, Clone, PartialEq)]
struct GraphicsSettings { /* ... */ }

App::new()
    .add_plugins(
        SettingsPlugin::new()
            .register::<GameSettings>(
                SettingsConfig::new("game", SerializationFormat::Json)
            )
            .register::<GraphicsSettings>(
                SettingsConfig::new("graphics", SerializationFormat::Json)
            )
    )
    .run();
```

### Reacting to Changes

Use Bevy's change detection:

```rust
fn on_settings_change(settings: Res<MySettings>) {
    if settings.is_changed() && !settings.is_added() {
        println!("Settings changed!");
        // Apply settings to your game
    }
}
```

## Examples

Run the examples:

```bash
# Simple automated example
cargo run --example simple

# Basic interactive example
cargo run --example basic

# Advanced example with nested structs, enums, and multiple formats
cargo run --example advanced

# New API example showing the simplified registration
cargo run --example new_api
```

## How It Works

1. **Startup**: The plugin loads settings from disk, or uses defaults if the file doesn't exist
2. **Runtime**: Settings are available as Bevy resources
3. **Modification**: When settings are modified (via `ResMut`), Bevy's change detection triggers
4. **Persistence**: The plugin automatically saves settings to disk
5. **Delta Persistence**: If settings equal defaults, the file is deleted

## API Documentation

For detailed API documentation, run:

```bash
cargo doc --open
```

## Contributing

Contributions are welcome! Please feel free to submit a Pull Request.

## License

Licensed under either of:

- Apache License, Version 2.0 ([LICENSE-APACHE](LICENSE) or http://www.apache.org/licenses/LICENSE-2.0)
- MIT license ([LICENSE-MIT](LICENSE) or http://opensource.org/licenses/MIT)

at your option.
