#![cfg(test)]

use crate::sync;
use std::path::Path;

fn test_config_path(test_name: &str) -> std::path::PathBuf {
    std::env::temp_dir().join(format!(
        "test_sync_config_{test_name}_{}.json",
        std::process::id()
    ))
}

#[test]
fn test_file_hash_nonexistent_file() {
    let path = Path::new("/nonexistent/file.txt");
    assert!(sync::file_hash(path).is_err());
}

#[test]
fn test_file_hash_existing_file() {
    let temp_dir = std::env::temp_dir();
    let test_file = temp_dir.join(format!("test_hash_{}.txt", std::process::id()));
    std::fs::write(&test_file, "test content for hashing").unwrap();

    let result = sync::file_hash(&test_file).unwrap();
    let hash = result.clone();
    assert!(!hash.is_empty());
    assert_eq!(hash.len(), 64);

    let result2 = sync::file_hash(&test_file).unwrap();
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
fn test_show_status_reports_database_errors() {
    let path = std::env::temp_dir();
    let error = sync::show_status(&path).unwrap_err();
    assert!(error.contains("cannot open database"));
}

#[test]
fn test_run_sync_cycle_no_devices() {
    let config_path = test_config_path("no_devices");

    let config = crate::config::SyncConfig::default();
    serde_json::to_writer(&std::fs::File::create(&config_path).unwrap(), &config).unwrap();

    sync::run_sync_cycle(&config_path, "all").unwrap();
    sync::run_sync_cycle(&config_path, "nonexistent-device").unwrap();

    let _ = std::fs::remove_file(&config_path);
    let devices_path = config_path.with_extension("devices.json");
    let _ = std::fs::remove_file(&devices_path);
}

#[test]
fn test_sync_cycle_with_device() {
    let config_path = test_config_path("with_device");
    let listener = std::net::TcpListener::bind("127.0.0.1:0").unwrap();
    let port = listener.local_addr().unwrap().port();

    let mut config = crate::config::SyncConfig::default();
    config.device.name = "test-device".to_string();
    config.device.id = "dev-001".to_string();
    serde_json::to_writer(&std::fs::File::create(&config_path).unwrap(), &config).unwrap();

    let device_list = crate::config::DeviceList {
        devices: vec![crate::config::PeerDevice {
            name: "test-device".to_string(),
            address: "127.0.0.1".to_string(),
            port,
            enabled: true,
        }],
    };
    let devices_path = config_path.with_extension("devices.json");
    serde_json::to_writer(&std::fs::File::create(&devices_path).unwrap(), &device_list).unwrap();

    let result = sync::run_sync_cycle(&config_path, "test-device");
    let error = result.unwrap_err();
    assert!(error.contains("data transfer is not implemented"));
    drop(listener);

    let _ = std::fs::remove_file(&config_path);
    let _ = std::fs::remove_file(&devices_path);
}

#[test]
fn test_run_daemon_rejects_invalid_config() {
    let config_path = test_config_path("invalid_daemon");
    std::fs::write(&config_path, "{invalid json").unwrap();
    let error = sync::run_daemon(&config_path).unwrap_err();
    assert!(error.contains("cannot parse config"));
    let _ = std::fs::remove_file(&config_path);
}
