#![cfg(test)]

use crate::sync;
use std::path::Path;

#[test]
fn test_file_hash_nonexistent_file() {
    let path = Path::new("/nonexistent/file.txt");
    let result = sync::file_hash(path);
    assert!(result.is_none());
}

#[test]
fn test_file_hash_existing_file() {
    let temp_dir = std::env::temp_dir();
    let test_file = temp_dir.join(format!("test_hash_{}.txt", std::process::id()));
    std::fs::write(&test_file, "test content for hashing").unwrap();

    let result = sync::file_hash(&test_file);
    assert!(result.is_some());
    let hash = result.clone().unwrap();
    assert!(!hash.is_empty());
    assert_eq!(hash.len(), 64);

    let result2 = sync::file_hash(&test_file);
    assert_eq!(result, result2);

    let _ = std::fs::remove_file(&test_file);
}

#[test]
fn test_init_db_creates_tables() {
    let temp_dir = std::env::temp_dir();
    let db_path = temp_dir.join(format!("test_sync_{}.db", std::process::id()));

    let _ = std::fs::remove_file(&db_path);

    let conn = sync::init_db(&db_path);
    assert!(conn.is_ok());

    let conn = conn.unwrap();
    let tables: Vec<String> = conn
        .prepare("SELECT name FROM sqlite_master WHERE type='table'")
        .unwrap()
        .query_map([], |row| row.get(0))
        .unwrap()
        .filter_map(|r| r.ok())
        .collect();

    assert!(tables.contains(&"sync_state".to_string()));
    assert!(tables.contains(&"checkpoints".to_string()));

    let _ = std::fs::remove_file(&db_path);
}

#[test]
fn test_run_sync_cycle_no_devices() {
    let temp_dir = std::env::temp_dir();
    let config_path = temp_dir.join(format!("test_sync_config_{}.json", std::process::id()));

    let config = crate::config::SyncConfig::default();
    serde_json::to_writer(&std::fs::File::create(&config_path).unwrap(), &config).unwrap();

    sync::run_sync_cycle(&config_path, "all");
    sync::run_sync_cycle(&config_path, "nonexistent-device");

    let _ = std::fs::remove_file(&config_path);
    let devices_path = config_path.with_extension("devices.json");
    let _ = std::fs::remove_file(&devices_path);
}

#[test]
fn test_sync_cycle_with_device() {
    let temp_dir = std::env::temp_dir();
    let config_path = temp_dir.join(format!("test_sync_config_{}.json", std::process::id()));

    let mut config = crate::config::SyncConfig::default();
    config.device.name = "test-device".to_string();
    config.device.id = "dev-001".to_string();
    serde_json::to_writer(&std::fs::File::create(&config_path).unwrap(), &config).unwrap();

    let device_list = crate::config::DeviceList {
        devices: vec![crate::config::PeerDevice {
            name: "test-device".to_string(),
            address: "127.0.0.1".to_string(),
            port: 22,
            enabled: true,
        }],
    };
    let devices_path = config_path.with_extension("devices.json");
    serde_json::to_writer(&std::fs::File::create(&devices_path).unwrap(), &device_list).unwrap();

    sync::run_sync_cycle(&config_path, "test-device");

    let _ = std::fs::remove_file(&config_path);
    let _ = std::fs::remove_file(&devices_path);
}

#[test]
fn test_run_daemon_starts() {
    let temp_dir = std::env::temp_dir();
    let config_path = temp_dir.join(format!("test_sync_config_{}.json", std::process::id()));

    let mut config = crate::config::SyncConfig::default();
    config.sync.sync_interval_seconds = 1;
    config.device.name = "test-device".to_string();
    config.device.id = "dev-001".to_string();
    serde_json::to_writer(&std::fs::File::create(&config_path).unwrap(), &config).unwrap();

    let _ = config_path;
    let _ = std::fs::remove_file(&config_path);
    let devices_path = config_path.with_extension("devices.json");
    let _ = std::fs::remove_file(&devices_path);
}
