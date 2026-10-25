#![cfg(test)]

use crate::config::PeerDevice;
use crate::discovery;

#[test]
fn test_discover_devices_empty_list() {
    let devices: Vec<PeerDevice> = Vec::new();
    let result = discovery::discover_devices(&devices);
    assert!(result.is_empty());
}

#[test]
fn test_discover_devices_all_disabled() {
    let devices = vec![
        PeerDevice {
            name: "device-1".to_string(),
            address: "192.168.1.10".to_string(),
            port: 8080,
            enabled: false,
        },
        PeerDevice {
            name: "device-2".to_string(),
            address: "192.168.1.11".to_string(),
            port: 8081,
            enabled: false,
        },
    ];
    let result = discovery::discover_devices(&devices);
    assert!(result.is_empty());
}

#[test]
fn test_discover_devices_mixed_enabled_disabled() {
    let devices = vec![
        PeerDevice {
            name: "device-1".to_string(),
            address: "127.0.0.1".to_string(),
            port: 22,
            enabled: true,
        },
        PeerDevice {
            name: "device-2".to_string(),
            address: "192.168.1.11".to_string(),
            port: 9999,
            enabled: true,
        },
        PeerDevice {
            name: "device-3".to_string(),
            address: "127.0.0.1".to_string(),
            port: 22,
            enabled: false,
        },
    ];
    let result = discovery::discover_devices(&devices);
    // Disabled devices must never appear in results
    assert!(!result.iter().any(|d| d.name == "device-3"));
    // Only enabled devices are candidates
    assert!(result.iter().all(|d| d.enabled));
}

#[test]
fn test_discover_devices_empty_string_address() {
    let devices = vec![PeerDevice {
        name: "device-1".to_string(),
        address: "".to_string(),
        port: 8080,
        enabled: true,
    }];
    let result = discovery::discover_devices(&devices);
    assert!(result.is_empty());
}
