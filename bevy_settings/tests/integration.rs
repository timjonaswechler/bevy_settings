use bevy::prelude::*;
use bevy_settings::{prelude::*, Settings};
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;

#[derive(Settings, Resource, Serialize, Deserialize, Clone, PartialEq, Debug)]
struct TestSettings {
    value: i32,
    name: String,
}

impl Default for TestSettings {
    fn default() -> Self {
        Self {
            value: 42,
            name: "default".to_string(),
        }
    }
}

fn get_test_path(test_name: &str) -> PathBuf {
    PathBuf::from("/tmp/bevy_settings_integration").join(test_name)
}

fn cleanup_test(test_name: &str) {
    let path = get_test_path(test_name);
    let _ = fs::remove_dir_all(&path);
}

#[test]
fn test_plugin_loads_defaults() {
    let test_name = "test_plugin_loads_defaults";
    cleanup_test(test_name);

    let mut app = App::new();
    app.add_plugins(MinimalPlugins).add_plugins(
        SettingsPlugin::new("TestSettings")
            .format(SerializationFormat::Json)
            .with_base_path(get_test_path(test_name).to_str().unwrap())
            .register::<TestSettings>(),
    );

    app.update();

    let settings = app.world().resource::<TestSettings>();
    assert_eq!(settings.value, 42);
    assert_eq!(settings.name, "default");

    cleanup_test(test_name);
}

#[test]
fn test_plugin_saves_on_change() {
    let test_name = "test_plugin_saves_on_change";
    cleanup_test(test_name);

    let mut app = App::new();
    app.add_plugins(MinimalPlugins).add_plugins(
        SettingsPlugin::new("TestSettings")
            .format(SerializationFormat::Json)
            .with_base_path(get_test_path(test_name).to_str().unwrap())
            .register::<TestSettings>(),
    );

    app.update();

    // Modify settings
    {
        let mut settings = app.world_mut().resource_mut::<TestSettings>();
        settings.value = 100;
        settings.name = "modified".to_string();
    }

    // Run another update to trigger the save system
    app.update();

    // Check if file was created (file with plugin name)
    let settings_file = get_test_path(test_name).join("TestSettings.json");
    assert!(settings_file.exists());

    // Verify file contents
    let content = fs::read_to_string(&settings_file).unwrap();
    assert!(content.contains("100"));
    assert!(content.contains("modified"));

    cleanup_test(test_name);
}

#[test]
fn test_plugin_loads_saved_settings() {
    let test_name = "test_plugin_loads_saved_settings";
    cleanup_test(test_name);

    // First app: save settings
    {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins).add_plugins(
            SettingsPlugin::new("TestSettings")
                .format(SerializationFormat::Json)
                .with_base_path(get_test_path(test_name).to_str().unwrap())
                .register::<TestSettings>(),
        );

        app.update();

        {
            let mut settings = app.world_mut().resource_mut::<TestSettings>();
            settings.value = 200;
            settings.name = "saved".to_string();
        }

        app.update();
    }

    // Second app: load settings
    {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins).add_plugins(
            SettingsPlugin::new("TestSettings")
                .format(SerializationFormat::Json)
                .with_base_path(get_test_path(test_name).to_str().unwrap())
                .register::<TestSettings>(),
        );

        app.update();

        let settings = app.world().resource::<TestSettings>();
        assert_eq!(settings.value, 200);
        assert_eq!(settings.name, "saved");
    }

    cleanup_test(test_name);
}

#[test]
fn test_delta_persistence() {
    let test_name = "test_delta_persistence";
    cleanup_test(test_name);

    let mut app = App::new();
    app.add_plugins(MinimalPlugins).add_plugins(
        SettingsPlugin::new("TestSettings")
            .format(SerializationFormat::Json)
            .with_base_path(get_test_path(test_name).to_str().unwrap())
            .register::<TestSettings>(),
    );

    app.update();

    let settings_file = get_test_path(test_name).join("TestSettings.json");

    // Default settings should not create a file
    assert!(!settings_file.exists());

    // Modify settings
    {
        let mut settings = app.world_mut().resource_mut::<TestSettings>();
        settings.value = 100;
    }

    app.update();

    // File should now exist
    assert!(settings_file.exists());

    // Reset to defaults
    {
        let mut settings = app.world_mut().resource_mut::<TestSettings>();
        *settings = TestSettings::default();
    }

    app.update();

    // File should be deleted
    assert!(!settings_file.exists());

    cleanup_test(test_name);
}

#[test]
fn test_binary_format() {
    let test_name = "test_binary_format";
    cleanup_test(test_name);

    let mut app = App::new();
    app.add_plugins(MinimalPlugins).add_plugins(
        SettingsPlugin::new("TestSettings")
            .format(SerializationFormat::Binary)
            .with_base_path(get_test_path(test_name).to_str().unwrap())
            .register::<TestSettings>(),
    );

    app.update();

    // Modify settings
    {
        let mut settings = app.world_mut().resource_mut::<TestSettings>();
        settings.value = 999;
        settings.name = "binary".to_string();
    }

    app.update();

    // Check if .bin file was created (file with plugin name)
    let settings_file = get_test_path(test_name).join("TestSettings.bin");
    assert!(settings_file.exists());

    cleanup_test(test_name);
}

// Test settings with custom SECTION and migration support
#[derive(Resource, Serialize, Deserialize, Clone, PartialEq, Debug)]
struct MigratableSettings {
    value: i32,
    name: String,
    #[serde(default)]
    new_field: Option<String>,
}

impl Default for MigratableSettings {
    fn default() -> Self {
        Self {
            value: 100,
            name: "default".to_string(),
            new_field: Some("new".to_string()),
        }
    }
}

impl Settings for MigratableSettings {
    fn type_name() -> &'static str {
        "MigratableSettings"
    }

    const SECTION: &'static str = "migratable";

    fn migrate(
        file_version: Option<&semver::Version>,
        target_version: &semver::Version,
        mut data: serde_json::Value,
    ) -> Result<(serde_json::Value, bool), SettingsError> {
        let mut changed = false;

        // If file version is less than 2.0.0, add the new_field
        if let Some(file_ver) = file_version {
            if file_ver < &semver::Version::new(2, 0, 0) 
                && target_version >= &semver::Version::new(2, 0, 0) 
            {
                if let serde_json::Value::Object(ref mut map) = data {
                    if !map.contains_key("new_field") {
                        map.insert(
                            "new_field".to_string(),
                            serde_json::Value::String("migrated".to_string()),
                        );
                        changed = true;
                    }
                }
            }
        }

        Ok((data, changed))
    }
}

#[test]
fn test_section_constant() {
    // Test that SECTION constant is correctly set
    assert_eq!(TestSettings::SECTION, "testsettings");
    assert_eq!(MigratableSettings::SECTION, "migratable");
}

#[test]
fn test_migration_adds_new_field() {
    let test_name = "test_migration_adds_new_field";
    cleanup_test(test_name);

    // First, create a settings file with version 1.0.0
    {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins).add_plugins(
            SettingsPlugin::new("TestSettings")
                .format(SerializationFormat::Json)
                .with_base_path(get_test_path(test_name).to_str().unwrap())
                .register_with_version::<MigratableSettings>("1.0.0"),
        );

        app.update();

        {
            let mut settings = app.world_mut().resource_mut::<MigratableSettings>();
            settings.value = 200;
            settings.name = "old_version".to_string();
            // Explicitly set new_field to None to simulate old version without this field
            settings.new_field = None;
        }

        app.update();
    }

    // Manually edit the file to remove new_field and set version
    let settings_file = get_test_path(test_name).join("TestSettings.json");
    let content = fs::read_to_string(&settings_file).unwrap();
    let mut json: serde_json::Value = serde_json::from_str(&content).unwrap();
    
    // Remove new_field from the migratable section to simulate old data
    if let Some(migratable) = json.get_mut("migratable") {
        if let serde_json::Value::Object(ref mut map) = migratable {
            map.remove("new_field");
        }
    }
    
    fs::write(&settings_file, serde_json::to_string_pretty(&json).unwrap()).unwrap();

    // Now load with version 2.0.0 - migration should add new_field
    {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins).add_plugins(
            SettingsPlugin::new("TestSettings")
                .format(SerializationFormat::Json)
                .with_base_path(get_test_path(test_name).to_str().unwrap())
                .register_with_version::<MigratableSettings>("2.0.0"),
        );

        app.update();

        let settings = app.world().resource::<MigratableSettings>();
        assert_eq!(settings.value, 200);
        assert_eq!(settings.name, "old_version");
        // Migration should have added this field
        assert_eq!(settings.new_field, Some("migrated".to_string()));
    }

    cleanup_test(test_name);
}

#[test]
fn test_version_tracking() {
    let test_name = "test_version_tracking";
    cleanup_test(test_name);

    let mut app = App::new();
    app.add_plugins(MinimalPlugins).add_plugins(
        SettingsPlugin::new("TestSettings")
            .format(SerializationFormat::Json)
            .with_base_path(get_test_path(test_name).to_str().unwrap())
            .register_with_version::<TestSettings>("1.0.0"),
    );

    app.update();

    {
        let mut settings = app.world_mut().resource_mut::<TestSettings>();
        settings.value = 999;
    }

    app.update();

    // Check that version is saved in the file
    let settings_file = get_test_path(test_name).join("TestSettings.json");
    let content = fs::read_to_string(&settings_file).unwrap();
    let json: serde_json::Value = serde_json::from_str(&content).unwrap();
    
    // Check for version info
    assert!(json.get("_versions").is_some());
    let versions = json.get("_versions").unwrap();
    assert_eq!(
        versions.get("testsettings").and_then(|v| v.as_str()),
        Some("1.0.0")
    );

    cleanup_test(test_name);
}
