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
        SettingsPlugin::<TestSettings>::new("test_settings", SerializationFormat::Json)
            .with_base_path(get_test_path(test_name).to_str().unwrap()),
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
        SettingsPlugin::<TestSettings>::new("test_settings", SerializationFormat::Json)
            .with_base_path(get_test_path(test_name).to_str().unwrap()),
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

    // Check if file was created
    let settings_file = get_test_path(test_name).join("test_settings.json");
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
            SettingsPlugin::<TestSettings>::new("test_settings", SerializationFormat::Json)
                .with_base_path(get_test_path(test_name).to_str().unwrap()),
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
            SettingsPlugin::<TestSettings>::new("test_settings", SerializationFormat::Json)
                .with_base_path(get_test_path(test_name).to_str().unwrap()),
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
        SettingsPlugin::<TestSettings>::new("test_settings", SerializationFormat::Json)
            .with_base_path(get_test_path(test_name).to_str().unwrap()),
    );

    app.update();

    let settings_file = get_test_path(test_name).join("test_settings.json");

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
        SettingsPlugin::<TestSettings>::new("test_settings", SerializationFormat::Binary)
            .with_base_path(get_test_path(test_name).to_str().unwrap()),
    );

    app.update();

    // Modify settings
    {
        let mut settings = app.world_mut().resource_mut::<TestSettings>();
        settings.value = 999;
        settings.name = "binary".to_string();
    }

    app.update();

    // Check if .bin file was created
    let settings_file = get_test_path(test_name).join("test_settings.bin");
    assert!(settings_file.exists());

    cleanup_test(test_name);
}
