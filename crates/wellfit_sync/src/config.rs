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

pub fn load_config(path: &Path) -> Result<SyncConfig, String> {
    let text = match fs::read_to_string(path) {
        Ok(text) => text,
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => {
            return Ok(SyncConfig::default());
        }
        Err(error) => {
            return Err(format!("cannot read config {}: {error}", path.display()));
        }
    };
    serde_json::from_str(&text)
        .map_err(|error| format!("cannot parse config {}: {error}", path.display()))
}

pub fn save_config(path: &Path, config: &SyncConfig) -> Result<(), String> {
    let text = serde_json::to_string_pretty(config)
        .map_err(|error| format!("cannot serialize config {}: {error}", path.display()))?;
    wellfit_core::text::write_atomic(path, text.as_bytes())
        .map_err(|error| format!("cannot write config {}: {error}", path.display()))
}

pub fn add_device(path: &Path, name: &str, address: &str, port: u16) -> Result<(), String> {
    let mut device_list = load_device_list(path)?;
    device_list.devices.push(PeerDevice {
        name: name.to_string(),
        address: address.to_string(),
        port,
        enabled: true,
    });
    save_device_list(path, &device_list)?;
    println!("Device '{}' added: {}:{}", name, address, port);
    Ok(())
}

pub fn remove_device(path: &Path, name: &str) -> Result<(), String> {
    let mut device_list = load_device_list(path)?;
    device_list.devices.retain(|d| d.name != name);
    save_device_list(path, &device_list)?;
    println!("Device '{}' removed", name);
    Ok(())
}

pub fn list_devices(path: &Path) -> Result<(), String> {
    let device_list = load_device_list(path)?;
    println!("Configured devices:");
    for d in &device_list.devices {
        println!(
            "  {} - {}:{} (enabled: {})",
            d.name, d.address, d.port, d.enabled
        );
    }
    Ok(())
}

pub fn load_device_list(path: &Path) -> Result<DeviceList, String> {
    let devices_path = path.with_extension("devices.json");
    let text = match fs::read_to_string(&devices_path) {
        Ok(text) => text,
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => {
            return Ok(DeviceList::default());
        }
        Err(error) => {
            return Err(format!(
                "cannot read device list {}: {error}",
                devices_path.display()
            ));
        }
    };
    serde_json::from_str(&text).map_err(|error| {
        format!(
            "cannot parse device list {}: {error}",
            devices_path.display()
        )
    })
}

fn save_device_list(path: &Path, list: &DeviceList) -> Result<(), String> {
    let devices_path = path.with_extension("devices.json");
    let text = serde_json::to_string_pretty(list).map_err(|error| {
        format!(
            "cannot serialize device list {}: {error}",
            devices_path.display()
        )
    })?;
    wellfit_core::text::write_atomic(&devices_path, text.as_bytes()).map_err(|error| {
        format!(
            "cannot write device list {}: {error}",
            devices_path.display()
        )
    })
}
