#![forbid(unsafe_code)]
#![allow(dead_code)]

//! Wellfit Sync - Lightweight multi-platform sync daemon.
//!
//! Runs on laptops, desktops, servers, and NAS devices.
//! Provides device discovery, data synchronization, health monitoring,
//! and fault recovery with minimal resource usage.

mod config;
mod discovery;
mod health;
mod sync;

use clap::{Parser, Subcommand};
use std::path::PathBuf;

const VERSION: &str = "0.1.0";

#[derive(Parser)]
#[command(name = "wellfit_sync", version = VERSION, about = "Lightweight multi-platform sync daemon")]
struct Cli {
    /// Configuration file path
    #[arg(short, long, default_value = "sync.toml")]
    config: PathBuf,

    /// Run as daemon (background service)
    #[arg(short, long)]
    daemon: bool,

    /// Sub-command
    #[command(subcommand)]
    command: Option<Commands>,
}

#[derive(Subcommand)]
enum Commands {
    /// Show sync status
    Status {
        /// Database path
        #[arg(short, long, default_value = "data/sync.db")]
        db: PathBuf,
    },
    /// Add a device to sync with
    AddDevice {
        /// Device name
        #[arg(short, long)]
        name: String,
        /// Device address (IP or hostname)
        #[arg(short, long)]
        address: String,
        /// Device port
        #[arg(short, long, default_value = "8080")]
        port: u16,
    },
    /// Remove a device
    RemoveDevice {
        /// Device name
        #[arg(short, long)]
        name: String,
    },
    /// List all devices
    ListDevices,
    /// Run a single sync cycle
    Sync {
        /// Device name (or "all" for all devices)
        #[arg(short, long, default_value = "all")]
        device: String,
    },
    /// Run health check
    Health,
    /// Install as system service
    Install,
    /// Uninstall system service
    Uninstall,
}

fn main() {
    let cli = Cli::parse();

    match cli.command {
        Some(Commands::Status { db }) => {
            sync::show_status(&db);
        }
        Some(Commands::AddDevice {
            name,
            address,
            port,
        }) => {
            config::add_device(&cli.config, &name, &address, port);
        }
        Some(Commands::RemoveDevice { name }) => {
            config::remove_device(&cli.config, &name);
        }
        Some(Commands::ListDevices) => {
            config::list_devices(&cli.config);
        }
        Some(Commands::Sync { device }) => {
            sync::run_sync_cycle(&cli.config, &device);
        }
        Some(Commands::Health) => {
            health::run_health_check();
        }
        Some(Commands::Install) => {
            platform::install_service();
        }
        Some(Commands::Uninstall) => {
            platform::uninstall_service();
        }
        None => {
            if cli.daemon {
                sync::run_daemon(&cli.config);
            } else {
                println!("wellfit_sync v{}", VERSION);
                println!("Use --help for available commands");
            }
        }
    }
}

mod platform {
    pub fn install_service() {
        #[cfg(target_os = "windows")]
        {
            println!("Installing Windows Service...");
            // sc.exe create wellfit_sync binPath= "wellfit_sync.exe --daemon" start= auto
            println!("Run: sc.exe create wellfit_sync binPath= \"wellfit_sync.exe --daemon\" start= auto");
        }
        #[cfg(target_os = "linux")]
        {
            println!("Installing systemd service...");
            println!("Run: sudo cp wellfit-sync.service /etc/systemd/system/");
            println!("Run: sudo systemctl enable wellfit_sync");
            println!("Run: sudo systemctl start wellfit_sync");
        }
        #[cfg(target_os = "macos")]
        {
            println!("Installing launchd service...");
            println!("Run: cp wellfit-sync.plist ~/Library/LaunchAgents/");
            println!("Run: launchctl load ~/Library/LaunchAgents/wellfit-sync.plist");
        }
    }

    pub fn uninstall_service() {
        #[cfg(target_os = "windows")]
        {
            println!("Uninstalling Windows Service...");
            println!("Run: sc.exe delete wellfit_sync");
        }
        #[cfg(target_os = "linux")]
        {
            println!("Uninstalling systemd service...");
            println!("Run: sudo systemctl stop wellfit_sync");
            println!("Run: sudo systemctl disable wellfit_sync");
            println!("Run: sudo rm /etc/systemd/system/wellfit-sync.service");
        }
        #[cfg(target_os = "macos")]
        {
            println!("Uninstalling launchd service...");
            println!("Run: launchctl unload ~/Library/LaunchAgents/wellfit-sync.plist");
            println!("Run: rm ~/Library/LaunchAgents/wellfit-sync.plist");
        }
    }
}
