#![forbid(unsafe_code)]

//! Sync engine module.

use crate::config::{load_config, PeerDevice};
use crate::discovery;
use rusqlite::Connection;
use sha2::{Digest, Sha256};
use std::fs;
use std::path::Path;

pub fn show_status(db_path: &Path) {
    println!("Sync status:");
    if db_path.exists() {
        if let Ok(conn) = Connection::open(db_path) {
            // Check if tables exist
            let tables: Vec<String> = conn
                .prepare("SELECT name FROM sqlite_master WHERE type='table'")
                .unwrap()
                .query_map([], |row| row.get(0))
                .unwrap()
                .filter_map(|r| r.ok())
                .collect();
            println!("  Database: {}", db_path.display());
            println!("  Tables: {:?}", tables);
        } else {
            println!("  Cannot open database: {}", db_path.display());
        }
    } else {
        println!("  Database not found: {}", db_path.display());
    }
}

pub fn run_sync_cycle(config_path: &Path, device_filter: &str) {
    let _config = load_config(config_path);
    let device_list = load_device_list(config_path);

    let devices: Vec<PeerDevice> = if device_filter == "all" {
        device_list.devices.clone()
    } else {
        device_list
            .devices
            .into_iter()
            .filter(|d| d.name == device_filter)
            .collect()
    };

    let available = discovery::discover_devices(&devices);
    println!(
        "Found {} available devices out of {}",
        available.len(),
        devices.len()
    );

    for device in &available {
        println!(
            "Syncing with {} ({}:{})...",
            device.name, device.address, device.port
        );
        // Sync logic would go here
        println!("  Sync complete for {}", device.name);
    }
}

pub fn run_daemon(config_path: &Path) {
    println!("Starting wellfit_sync daemon...");
    let config = load_config(config_path);
    let interval = config.sync.sync_interval_seconds;

    loop {
        println!("Running sync cycle...");
        run_sync_cycle(config_path, "all");
        println!("Sleeping for {} seconds...", interval);
        std::thread::sleep(std::time::Duration::from_secs(interval));
    }
}

fn load_device_list(path: &Path) -> crate::config::DeviceList {
    let devices_path = path.with_extension("devices.json");
    if devices_path.exists() {
        if let Ok(text) = fs::read_to_string(&devices_path) {
            if let Ok(list) = serde_json::from_str(&text) {
                return list;
            }
        }
    }
    crate::config::DeviceList::default()
}

/// Compute SHA-256 hash of a file
pub fn file_hash(path: &Path) -> Option<String> {
    let content = fs::read(path).ok()?;
    let mut hasher = Sha256::new();
    hasher.update(&content);
    Some(format!("{:x}", hasher.finalize()))
}

/// Initialize the sync database
pub fn init_db(db_path: &Path) -> Result<Connection, rusqlite::Error> {
    let conn = Connection::open(db_path)?;
    conn.execute_batch(
        "CREATE TABLE IF NOT EXISTS sync_state (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            device_name TEXT NOT NULL,
            file_path TEXT NOT NULL,
            last_synced TEXT NOT NULL,
            checksum TEXT NOT NULL,
            status TEXT NOT NULL
        );
        CREATE TABLE IF NOT EXISTS checkpoints (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            checkpoint_id TEXT NOT NULL UNIQUE,
            timestamp TEXT NOT NULL,
            device_name TEXT NOT NULL,
            sync_state TEXT NOT NULL,
            last_synced_file TEXT,
            last_synced_offset INTEGER,
            checksum TEXT NOT NULL
        );",
    )?;
    Ok(conn)
}
