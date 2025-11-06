use bevy::prelude::*;
use bevy_settings::{prelude::*, Settings};
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;

#[derive(Settings, Resource, Serialize, Deserialize, Clone, PartialEq, Debug)]
struct TestSettingsA {
    value: i32,
    name: String,
}

impl Default for TestSettingsA {
    fn default() -> Self {
        Self {
            value: 100,
            name: "test_a".to_string(),
        }
    }
}

#[derive(Settings, Resource, Serialize, Deserialize, Clone, PartialEq, Debug)]
struct TestSettingsB {
    count: u32,
    enabled: bool,
}

impl Default for TestSettingsB {
    fn default() -> Self {
        Self {
            count: 0,
            enabled: false,
        }
    }
}

fn get_test_path(test_name: &str) -> PathBuf {
    std::env::temp_dir()
        .join("bevy_settings_store_tests")
        .join(test_name)
}

fn cleanup_test(test_name: &str) {
    let path = get_test_path(test_name);
    let _ = fs::remove_dir_all(&path);
}

#[test]
fn test_settings_store_as_plugin() {
    let test_name = "test_settings_store_as_plugin";
    cleanup_test(test_name);

    let mut app = App::new();
    app.add_plugins(MinimalPlugins).add_plugins(
        SettingsStore::new("TestStore")
            .format(SerializationFormat::Json)
            .version("1.0.0")
            .with_base_path(get_test_path(test_name).to_str().unwrap())
            .register::<TestSettingsA>()
            .register::<TestSettingsB>(),
    );

    app.update();

    // Check that resources were inserted
    let settings_a = app.world().resource::<TestSettingsA>();
    assert_eq!(settings_a.value, 100);
    assert_eq!(settings_a.name, "test_a");

    let settings_b = app.world().resource::<TestSettingsB>();
    assert_eq!(settings_b.count, 0);
    assert_eq!(settings_b.enabled, false);

    cleanup_test(test_name);
}

#[test]
fn test_settings_store_as_resource() {
    let test_name = "test_settings_store_as_resource";
    cleanup_test(test_name);

    let store = SettingsStore::new("TestStore")
        .format(SerializationFormat::Json)
        .version("1.0.0")
        .with_base_path(get_test_path(test_name).to_str().unwrap())
        .register::<TestSettingsA>();

    let mut app = App::new();
    app.add_plugins(MinimalPlugins).insert_resource(store);

    app.update();

    // Check that the SettingsStore resource exists
    let store_res = app.world().resource::<SettingsStore>();
    assert_eq!(store_res.get_name(), "TestStore");
    assert_eq!(store_res.get_version(), Some("1.0.0"));
    assert_eq!(store_res.get_format(), SerializationFormat::Json);

    cleanup_test(test_name);
}

#[test]
fn test_settings_store_saves_changes() {
    let test_name = "test_settings_store_saves_changes";
    cleanup_test(test_name);

    let mut app = App::new();
    app.add_plugins(MinimalPlugins).add_plugins(
        SettingsStore::new("TestStore")
            .format(SerializationFormat::Json)
            .with_base_path(get_test_path(test_name).to_str().unwrap())
            .register::<TestSettingsA>(),
    );

    app.update();

    // Modify settings
    {
        let mut settings = app.world_mut().resource_mut::<TestSettingsA>();
        settings.value = 200;
        settings.name = "modified".to_string();
    }

    app.update();

    // Check if file was created
    let settings_file = get_test_path(test_name).join("TestSettingsA.json");
    assert!(settings_file.exists());

    // Verify file contents
    let content = fs::read_to_string(&settings_file).unwrap();
    assert!(content.contains("200"));
    assert!(content.contains("modified"));

    cleanup_test(test_name);
}

#[test]
fn test_settings_store_with_placeholder() {
    let test_name = "test_settings_store_with_placeholder";
    cleanup_test(test_name);

    let mut app = App::new();
    app.add_plugins(MinimalPlugins).add_plugins(
        SettingsStore::new("[slot1]")
            .format(SerializationFormat::Json)
            .with_base_path(get_test_path(test_name).to_str().unwrap())
            .register::<TestSettingsA>(),
    );

    app.update();

    // Modify settings
    {
        let mut settings = app.world_mut().resource_mut::<TestSettingsA>();
        settings.value = 999;
    }

    app.update();

    // Check if file was created with placeholder prefix
    let settings_file = get_test_path(test_name).join("slot1_TestSettingsA.json");
    assert!(
        settings_file.exists(),
        "Expected file with placeholder prefix: {:?}",
        settings_file
    );

    cleanup_test(test_name);
}

#[test]
fn test_settings_store_loads_existing() {
    let test_name = "test_settings_store_loads_existing";
    cleanup_test(test_name);

    // First app: save settings
    {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins).add_plugins(
            SettingsStore::new("TestStore")
                .format(SerializationFormat::Json)
                .with_base_path(get_test_path(test_name).to_str().unwrap())
                .register::<TestSettingsA>(),
        );

        app.update();

        {
            let mut settings = app.world_mut().resource_mut::<TestSettingsA>();
            settings.value = 500;
            settings.name = "persisted".to_string();
        }

        app.update();
    }

    // Second app: load settings
    {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins).add_plugins(
            SettingsStore::new("TestStore")
                .format(SerializationFormat::Json)
                .with_base_path(get_test_path(test_name).to_str().unwrap())
                .register::<TestSettingsA>(),
        );

        app.update();

        let settings = app.world().resource::<TestSettingsA>();
        assert_eq!(settings.value, 500);
        assert_eq!(settings.name, "persisted");
    }

    cleanup_test(test_name);
}

#[test]
fn test_settings_store_binary_format() {
    let test_name = "test_settings_store_binary_format";
    cleanup_test(test_name);

    let mut app = App::new();
    app.add_plugins(MinimalPlugins).add_plugins(
        SettingsStore::new("TestStore")
            .format(SerializationFormat::Binary)
            .with_base_path(get_test_path(test_name).to_str().unwrap())
            .register::<TestSettingsA>(),
    );

    app.update();

    // Modify settings
    {
        let mut settings = app.world_mut().resource_mut::<TestSettingsA>();
        settings.value = 777;
    }

    app.update();

    // Check if .bin file was created
    let settings_file = get_test_path(test_name).join("TestSettingsA.bin");
    assert!(settings_file.exists());

    cleanup_test(test_name);
}
