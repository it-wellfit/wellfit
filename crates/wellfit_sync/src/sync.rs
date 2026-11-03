#![forbid(unsafe_code)]

//! Sync engine module.

use crate::config::{load_config, load_device_list, PeerDevice};
use crate::discovery;
use rusqlite::Connection;
use sha2::{Digest, Sha256};
use std::fs;
use std::path::Path;

pub fn show_status(db_path: &Path) -> Result<(), String> {
    println!("Sync status:");
    if db_path.exists() {
        let conn = Connection::open(db_path)
            .map_err(|error| format!("cannot open database {}: {error}", db_path.display()))?;
        let mut statement = conn
            .prepare("SELECT name FROM sqlite_master WHERE type='table'")
            .map_err(|error| format!("cannot query database {}: {error}", db_path.display()))?;
        let tables = statement
            .query_map([], |row| row.get::<_, String>(0))
            .map_err(|error| format!("cannot read database {}: {error}", db_path.display()))?
            .collect::<Result<Vec<_>, _>>()
            .map_err(|error| format!("cannot read database {}: {error}", db_path.display()))?;
        println!("  Database: {}", db_path.display());
        println!("  Tables: {:?}", tables);
    } else {
        println!("  Database not found: {}", db_path.display());
    }
    Ok(())
}

pub fn run_sync_cycle(config_path: &Path, device_filter: &str) -> Result<(), String> {
    let _config = load_config(config_path)?;
    let device_list = load_device_list(config_path)?;

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

    if available.is_empty() {
        return Ok(());
    }

    let names = available
        .iter()
        .map(|device| device.name.as_str())
        .collect::<Vec<_>>()
        .join(", ");
    Err(format!(
        "data transfer is not implemented; no records were synchronized with: {names}"
    ))
}

pub fn run_daemon(config_path: &Path) -> Result<(), String> {
    println!("Starting wellfit_sync daemon...");
    let config = load_config(config_path)?;
    let interval = config.sync.sync_interval_seconds;

    loop {
        println!("Running sync cycle...");
        if let Err(error) = run_sync_cycle(config_path, "all") {
            eprintln!("Sync cycle failed: {error}");
        }
        println!("Sleeping for {} seconds...", interval);
        std::thread::sleep(std::time::Duration::from_secs(interval));
    }
}

/// Compute SHA-256 hash of a file
pub fn file_hash(path: &Path) -> Result<String, std::io::Error> {
    let content = fs::read(path)?;
    let mut hasher = Sha256::new();
    hasher.update(&content);
    Ok(format!("{:x}", hasher.finalize()))
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
