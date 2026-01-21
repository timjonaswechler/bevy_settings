# bevy_settings

Persistent settings for **Bevy** resources with:

- **Delta encoding**: only store values that differ from `Default` (small, human-friendly files)
- **Path templates**: store settings via a `bevy_paths::TypedPath` template (supports `{param}` placeholders)
- **Auto-save**: resources are persisted automatically when they change
- **Multiple file formats**: `json`, `toml`, `ron`, and `bin` (bincode)

This repository is a small workspace:

- `bevy_settings`: the runtime crate (Bevy plugin, load/save logic, delta encoding)
- `bevy_settings_derive`: the proc-macro crate providing `#[derive(SettingsGroup)]`
- `bevy_settings_meta` (optional): metadata types for UI hints / localization and validation (feature-gated)

> Current workspace version: `0.1.1`
> Bevy version used in this workspace: `0.17.3`

---

## Why this crate?

Most games/apps have a bunch of settings resources:
graphics, audio, input bindings, save-game slots, etc.

`bevy_settings` lets you model each settings file as a **plain Bevy resource**, and handles:

- where it is saved (via a path template)
- what is saved (delta to defaults)
- when it is saved (automatic on change, plus manual triggers)

---

## Core concepts

### 1) Settings group = one file

A “settings group” is a single Bevy resource that maps to one file on disk.

You define it by deriving `SettingsGroup` and providing a path template:

- `#[settings("settings/global.toml")]` (static path)
- `#[settings("saves/{slot_id}/game.json")]` (dynamic path; `{slot_id}` comes from a struct field)

### 2) Path parameters

If your template contains placeholders (e.g. `{slot_id}`), the derive macro:

- extracts `slot_id` as a *path parameter*
- registers it via `SettingsGroupTrait::path_params()`

On save, path parameter fields are **removed from the serialized payload** (they belong in the path).
On load, path parameter fields are **preserved** from the current resource.

### 3) Delta encoding

When saving, `bevy_settings` serializes:

- `T::default()` and current settings `T`
- computes a “delta” object containing only changed fields

If the delta would be empty, the file is removed (if it exists).

---

## Installation

This workspace depends on a local sibling crate `bevy_paths` (path dependency). If you want to use `bevy_settings` outside of this repo, you’ll need `bevy_paths` available as well.

In your `Cargo.toml`:

```toml
[dependencies]
bevy_settings = "0.1.1"
```

If you use this repository as a git dependency or path dependency, ensure `bevy_paths` is resolvable.

---

## Quick start

### 1) Define a settings resource

```rust
use bevy::prelude::*;
use bevy_settings::*;
use serde::{Deserialize, Serialize};

#[derive(SettingsGroup, Resource, Serialize, Deserialize, Default, Clone, Reflect, PartialEq)]
#[reflect(Resource)]
#[settings("settings/global.toml")]
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
```

### 2) Add plugins and register your group

`bevy_settings` relies on `bevy_paths` to resolve `TypedPath` templates, so make sure the path registry plugin is installed.

```rust
use bevy::prelude::*;
use bevy_settings::*;
use bevy_paths::prelude::*;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_plugins(PathRegistryPlugin::new("MyStudio", "MyGame", "App"))
        .add_plugins(SettingsPlugin::default().register::<GlobalSettings>())
        .run();
}
```

### 3) Load/save at runtime (optional)

You can trigger load/save explicitly via `Commands`:

```rust
fn load_on_startup(mut commands: Commands) {
    commands.load_settings::<GlobalSettings>();
}

fn save_now(mut commands: Commands) {
    commands.save_settings::<GlobalSettings>();
}
```

### 4) Auto-save on changes

When you register a type, the plugin adds an auto-save system in `PostUpdate`:

- if the resource `T` is `changed` (and not just `added`), it queues a save.

---

## Dynamic paths (save slots)

Example of a per-slot save file:

```rust
use bevy::prelude::*;
use bevy_settings::*;
use serde::{Deserialize, Serialize};

#[derive(SettingsGroup, Resource, Serialize, Deserialize, Default, Clone, Reflect, PartialEq)]
#[reflect(Resource)]
#[settings("saves/{slot_id}/game.json")]
struct SaveGame {
    slot_id: String, // used for path resolution, not persisted in file
    inventory: InventoryData,
}

#[derive(Serialize, Deserialize, Default, Clone, Reflect, PartialEq)]
struct InventoryData {
    gold: u32,
    items: Vec<String>,
}
```

Important behavior:

- `slot_id` must be present and non-empty when saving (validated at runtime)
- when loading, `slot_id` is preserved from the current resource, so selecting a slot is simply:
  set `slot_id`, then call `load_settings::<SaveGame>()`

There is a runnable example in this repository:
`bevy_settings/bevy_settings/examples/demo.rs`.

---

## Supported formats

The file format is inferred from the file extension of the resolved path:

- `.json` → JSON (pretty-printed)
- `.toml` → TOML (pretty)
- `.ron` → RON (pretty)
- `.bin` → bincode (binary)

If the extension is missing, it defaults to JSON behavior.

---

## Crate overview

### `bevy_settings`

Key public API:

- `SettingsPlugin`: registers settings resources and auto-save systems
- `SettingsCommandsExt`: `Commands::load_settings::<T>()` / `Commands::save_settings::<T>()`
- `SettingsGroupTrait`: trait implemented by the derive macro (ties together resource + serde + `TypedPath`)

### `bevy_settings_derive`

Provides:

- `#[derive(SettingsGroup)]` + attribute `#[settings("...")]`

Generates implementations for:

- `bevy_settings::SettingsGroupTrait`
- `bevy_paths::TypedPath`

---

## Feature flags

`bevy_settings`:

- `meta`: enables re-exports from `bevy_settings_meta` (metadata types like `SettingDescriptor`, `UiHint`, etc.)

---

## Roadmap / Known limitations

See `todo.md` for current priorities, including:

- a way to mark groups as transient / not persisted
- preventing duplicate registrations from adding duplicate auto-save systems
- settings migration/versioning support for struct changes

---

## License

MIT. See `LICENSE`.
