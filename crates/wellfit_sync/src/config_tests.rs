#![cfg(test)]

use crate::config;
use crate::config::{DeviceList, PeerDevice, SyncConfig};

fn test_config_path() -> std::path::PathBuf {
    std::env::temp_dir().join(format!(
        "test_config_{}_{}.json",
        std::process::id(),
        "save_load"
    ))
}

fn test_add_list_path() -> std::path::PathBuf {
    std::env::temp_dir().join(format!(
        "test_config_{}_{}.json",
        std::process::id(),
        "add_list"
    ))
}

#[test]
fn test_config_default_values() {
    let config = config::SyncConfig::default();
    assert_eq!(config.device.name, "");
    assert_eq!(config.device.id, "");
    assert_eq!(config.sync.sync_interval_seconds, 300);
    assert_eq!(config.sync.max_concurrent_syncs, 4);
    assert_eq!(config.sync.retry_max_attempts, 3);
    assert_eq!(config.sync.retry_base_delay_ms, 1000);
    assert_eq!(config.sync.checkpoint_interval_seconds, 60);
    assert_eq!(config.sync.bandwidth_limit_mbps, 10.0);
    assert!(!config.security.tls_enabled);
    assert_eq!(config.security.token, "");
    assert!(!config.security.encryption_enabled);
    assert_eq!(config.monitoring.health_check_port, 0);
    assert!(!config.monitoring.metrics_enabled);
    assert_eq!(config.monitoring.log_level, "");
}

#[test]
fn test_peer_device_creation() {
    let device = PeerDevice {
        name: "test-device".to_string(),
        address: "192.168.1.100".to_string(),
        port: 8080,
        enabled: true,
    };
    assert_eq!(device.name, "test-device");
    assert_eq!(device.address, "192.168.1.100");
    assert_eq!(device.port, 8080);
    assert!(device.enabled);
}

#[test]
fn test_device_list_default() {
    let list = DeviceList::default();
    assert!(list.devices.is_empty());
}

#[test]
fn test_config_save_and_load() {
    let path = test_config_path();
    let mut config = SyncConfig::default();
    config.device.name = "test-device".to_string();
    config.device.id = "dev-001".to_string();
    config.sync.sync_interval_seconds = 600;
    config.sync.max_concurrent_syncs = 8;
    config.security.tls_enabled = true;
    config.security.token = "test-token-123".to_string();
    config.security.encryption_enabled = true;
    config.monitoring.health_check_port = 8080;
    config.monitoring.metrics_enabled = true;
    config.monitoring.log_level = "debug".to_string();

    config::save_config(&path, &config);
    let loaded = config::load_config(&path);

    assert_eq!(loaded.device.name, "test-device");
    assert_eq!(loaded.device.id, "dev-001");
    assert_eq!(loaded.sync.sync_interval_seconds, 600);
    assert_eq!(loaded.sync.max_concurrent_syncs, 8);
    assert_eq!(loaded.sync.retry_max_attempts, 3);
    assert_eq!(loaded.sync.retry_base_delay_ms, 1000);
    assert_eq!(loaded.sync.checkpoint_interval_seconds, 60);
    assert_eq!(loaded.sync.bandwidth_limit_mbps, 10.0);
    assert!(loaded.security.tls_enabled);
    assert_eq!(loaded.security.token, "test-token-123");
    assert!(loaded.security.encryption_enabled);
    assert_eq!(loaded.monitoring.health_check_port, 8080);
    assert!(loaded.monitoring.metrics_enabled);
    assert_eq!(loaded.monitoring.log_level, "debug");

    let _ = std::fs::remove_file(&path);
    let devices_path = path.with_extension("devices.json");
    let _ = std::fs::remove_file(&devices_path);
}

#[test]
fn test_add_and_list_devices() {
    let path = test_add_list_path();
    let devices_path = path.with_extension("devices.json");

    let _ = std::fs::remove_file(&path);
    let _ = std::fs::remove_file(&devices_path);

    config::add_device(&path, "device-1", "192.168.1.10", 8080);
    config::add_device(&path, "device-2", "192.168.1.11", 8081);

    assert!(devices_path.exists());

    let device_list = config::load_device_list(&path);
    assert_eq!(device_list.devices.len(), 2);
    assert_eq!(device_list.devices[0].name, "device-1");
    assert_eq!(device_list.devices[0].address, "192.168.1.10");
    assert_eq!(device_list.devices[0].port, 8080);
    assert!(device_list.devices[0].enabled);
    assert_eq!(device_list.devices[1].name, "device-2");
    assert_eq!(device_list.devices[1].address, "192.168.1.11");
    assert_eq!(device_list.devices[1].port, 8081);
    assert!(device_list.devices[1].enabled);

    config::remove_device(&path, "device-1");
    let device_list = config::load_device_list(&path);
    assert_eq!(device_list.devices.len(), 1);
    assert_eq!(device_list.devices[0].name, "device-2");

    let _ = std::fs::remove_file(&path);
    let _ = std::fs::remove_file(&devices_path);
}

#[test]
fn test_config_file_not_found_returns_default() {
    let path = std::path::Path::new("/nonexistent/path/config.json");
    let config = config::load_config(path);
    assert_eq!(config.device.name, "");
    assert_eq!(config.sync.sync_interval_seconds, 300);
    assert_eq!(config.sync.max_concurrent_syncs, 4);
}

#[test]
fn test_invalid_json_returns_default() {
    let path = std::env::temp_dir().join(format!("test_invalid_json_{}.json", std::process::id()));
    // Ensure clean state
    let _ = std::fs::remove_file(&path);
    // Write invalid JSON
    std::fs::write(&path, "{invalid json").unwrap();
    let config = config::load_config(&path);
    assert_eq!(config.device.name, "");
    let _ = std::fs::remove_file(&path);
}
