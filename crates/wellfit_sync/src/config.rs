#![forbid(unsafe_code)]

//! Configuration management for wellfit_sync.

use serde::{Deserialize, Serialize};
use std::fs;
use std::path::Path;

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct SyncConfig {
    pub device: DeviceConfig,
    pub sync: SyncSettings,
    pub security: SecurityConfig,
    pub monitoring: MonitoringConfig,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct DeviceConfig {
    pub name: String,
    pub id: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SyncSettings {
    pub sync_interval_seconds: u64,
    pub max_concurrent_syncs: usize,
    pub retry_max_attempts: u32,
    pub retry_base_delay_ms: u64,
    pub checkpoint_interval_seconds: u64,
    pub bandwidth_limit_mbps: f64,
}

impl Default for SyncSettings {
    fn default() -> Self {
        Self {
            sync_interval_seconds: 300,
            max_concurrent_syncs: 4,
            retry_max_attempts: 3,
            retry_base_delay_ms: 1000,
            checkpoint_interval_seconds: 60,
            bandwidth_limit_mbps: 10.0,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct SecurityConfig {
    pub tls_enabled: bool,
    pub token: String,
    pub encryption_enabled: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct MonitoringConfig {
    pub health_check_port: u16,
    pub metrics_enabled: bool,
    pub log_level: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PeerDevice {
    pub name: String,
    pub address: String,
    pub port: u16,
    pub enabled: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct DeviceList {
    pub devices: Vec<PeerDevice>,
}

pub fn load_config(path: &Path) -> SyncConfig {
    if path.exists() {
        if let Ok(text) = fs::read_to_string(path) {
            if let Ok(config) = serde_json::from_str(&text) {
                return config;
            }
        }
    }
    SyncConfig::default()
}

pub fn save_config(path: &Path, config: &SyncConfig) {
    if let Ok(text) = serde_json::to_string_pretty(config) {
        let _ = fs::write(path, text);
    }
}

pub fn add_device(path: &Path, name: &str, address: &str, port: u16) {
    let mut device_list = load_device_list(path);
    device_list.devices.push(PeerDevice {
        name: name.to_string(),
        address: address.to_string(),
        port,
        enabled: true,
    });
    save_device_list(path, &device_list);
    println!("Device '{}' added: {}:{}", name, address, port);
}

pub fn remove_device(path: &Path, name: &str) {
    let mut device_list = load_device_list(path);
    device_list.devices.retain(|d| d.name != name);
    save_device_list(path, &device_list);
    println!("Device '{}' removed", name);
}

pub fn list_devices(path: &Path) {
    let device_list = load_device_list(path);
    println!("Configured devices:");
    for d in &device_list.devices {
        println!(
            "  {} - {}:{} (enabled: {})",
            d.name, d.address, d.port, d.enabled
        );
    }
}

pub fn load_device_list(path: &Path) -> DeviceList {
    let devices_path = path.with_extension("devices.json");
    if devices_path.exists() {
        if let Ok(text) = fs::read_to_string(&devices_path) {
            if let Ok(list) = serde_json::from_str(&text) {
                return list;
            }
        }
    }
    DeviceList::default()
}

fn save_device_list(path: &Path, list: &DeviceList) {
    let devices_path = path.with_extension("devices.json");
    if let Ok(text) = serde_json::to_string_pretty(list) {
        let _ = fs::write(&devices_path, text);
    }
}
