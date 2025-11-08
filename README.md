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
- **Version Migration**: Semantic versioning support with automatic migration between versions
- **Custom Sections**: Define custom section names for settings in the unified file

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
        // Register all settings with a single plugin - creates one unified file
        .add_plugins(
            SettingsPlugin::new("GameSettings")
                .format(SerializationFormat::Json)
                .with_base_path("config")
                .register::<GameSettings>()
                .register::<GraphicsSettings>()
        )
        .run();
}
```

That's it! Your settings will be:
- Stored in a single unified file `config/GameSettings.json`
- Loaded on startup (or defaults if file doesn't exist)
- Automatically saved when modified
- Only stored if they differ from defaults

The file will look like:
```json
{
  "_versions": {
    "gamesettings": "1.0.0"
  },
  "gamesettings": {
    "volume": 0.8,
    "resolution": [1920, 1080],
    "fullscreen": true
  },
  "graphicssettings": {
    "quality": 2
  }
}
```

Note: Version tracking is optional. Use `register_with_version` to enable it for specific settings.

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
use bevy_settings::{SettingsPlugin, SerializationFormat};

App::new()
    .add_plugins(
        SettingsPlugin::new("GameSettings")
            .format(SerializationFormat::Json)
            .with_base_path("config")
            .register::<MySettings>()
    )
    .run();
```

### Custom Settings Path

```rust
SettingsPlugin::new("GameSettings")
    .format(SerializationFormat::Json)
    .with_base_path("custom/path")
    .register::<MySettings>()
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

// If settings differ from defaults, only those fields are saved
let modified = MySettings { volume: 0.5, ..default() };
// Only the "volume" field will be saved to the file
```

## Serialization Formats

### JSON (Human-Readable)

```rust
SerializationFormat::Json
```

Creates human-readable `.json` files with all registered settings in a unified structure:
```json
{
  "mysettings": {
    "volume": 0.8,
    "resolution": [1920, 1080],
    "fullscreen": true
  }
}
```

If version tracking is enabled, a `_versions` field is also included.

### Binary (Compact)

```rust
SerializationFormat::Binary
```

Creates compact `.bin` files using [bincode](https://github.com/bincode-org/bincode).

## Examples

### Multiple Settings

You can register multiple settings types with a single plugin, and they'll all be saved to one unified file:

```rust
#[derive(Settings, Resource, Serialize, Deserialize, Default, Clone, PartialEq)]
struct GameSettings { /* ... */ }

#[derive(Settings, Resource, Serialize, Deserialize, Default, Clone, PartialEq)]
struct GraphicsSettings { /* ... */ }

App::new()
    .add_plugins(
        SettingsPlugin::new("GameSettings")
            .format(SerializationFormat::Json)
            .with_base_path("config")
            .register::<GameSettings>()
            .register::<GraphicsSettings>()
    )
    .run();
```

This creates a single file `config/GameSettings.json`:
```json
{
  "gamesettings": { /* ... */ },
  "graphicssettings": { /* ... */ }
}
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

## Version Migration

Settings can implement version-aware migrations to handle breaking changes between versions.

### Basic Setup

Register settings with a version:

```rust
use bevy_settings::{SettingsPlugin, Settings};
use serde::{Deserialize, Serialize};

#[derive(Resource, Serialize, Deserialize, Clone, PartialEq, Debug)]
struct NetworkSettings {
    server_url: String,
    port: u16,
    #[serde(default)]
    timeout_seconds: Option<u32>,  // Added in v2.0.0
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

    // Optional: Custom section name (default is lowercase type name)
    const SECTION: &'static str = "network";

    // Optional: Implement migration logic
    fn migrate(
        file_version: Option<&semver::Version>,
        target_version: &semver::Version,
        mut data: serde_json::Value,
    ) -> Result<(serde_json::Value, bool), bevy_settings::SettingsError> {
        let mut changed = false;

        // Migrate from v1.x to v2.0.0+: add timeout_seconds field
        if let Some(file_ver) = file_version {
            if file_ver < &semver::Version::new(2, 0, 0) 
                && target_version >= &semver::Version::new(2, 0, 0) 
            {
                if let serde_json::Value::Object(ref mut map) = data {
                    if !map.contains_key("timeout_seconds") {
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

// Register with version tracking
App::new()
    .add_plugins(
        SettingsPlugin::new("GameSettings")
            .format(SerializationFormat::Json)
            .with_base_path("config")
            .register_with_version::<NetworkSettings>("2.0.0")
    )
    .run();
```

### How Migration Works

1. When settings are loaded, the system checks the version stored in the file
2. If the file version differs from the target version, `migrate()` is called
3. The migration function receives:
   - `file_version`: The version stored in the file (None if not present)
   - `target_version`: The current version you're migrating to
   - `data`: The raw settings data as JSON
4. Return the migrated data and whether changes were made
5. Migrated settings are automatically saved with the new version

### Custom Section Names

By default, section names are the lowercase type name. You can customize this:

```rust
impl Settings for NetworkSettings {
    const SECTION: &'static str = "network";  // instead of "networksettings"
    // ...
}
```

This affects how the settings appear in the unified file:

```json
{
  "_versions": {
    "network": "2.0.0"
  },
  "network": {
    "server_url": "https://example.com",
    "port": 8080,
    "timeout_seconds": 30
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

# Migration example demonstrating version migrations and custom sections
cargo run --example migration
```

## How It Works

1. **Startup**: The plugin loads settings from a unified file on disk, or uses defaults if the file doesn't exist
2. **Runtime**: Settings are available as Bevy resources
3. **Modification**: When settings are modified (via `ResMut`), Bevy's change detection triggers
4. **Persistence**: The plugin automatically saves all settings to a single unified file
5. **Delta Persistence**: Only fields that differ from defaults are saved; if all settings equal defaults, the file is deleted

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
