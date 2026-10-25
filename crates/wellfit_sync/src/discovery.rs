#![forbid(unsafe_code)]

//! Device discovery module.

use crate::config::PeerDevice;
use std::net::TcpStream;
use std::time::Duration;

pub fn discover_devices(devices: &[PeerDevice]) -> Vec<PeerDevice> {
    let mut available = Vec::new();
    for device in devices {
        if device.enabled {
            // Simple TCP connectivity check
            let addr = format!("{}:{}", device.address, device.port);
            if TcpStream::connect_timeout(&addr.parse().unwrap(), Duration::from_secs(3)).is_ok() {
                available.push(device.clone());
            }
        }
    }
    available
}
