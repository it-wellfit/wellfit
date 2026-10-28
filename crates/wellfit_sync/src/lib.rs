#![forbid(unsafe_code)]
#![allow(dead_code)]

//! Wellfit Sync - Lightweight multi-platform sync daemon.
//!
//! Provides device discovery, data synchronization, health monitoring,
//! and fault recovery with minimal resource usage.

pub mod config;
pub mod discovery;
pub mod health;
pub mod sync;

pub use config::*;
pub use discovery::*;
pub use health::*;
pub use sync::*;

#[cfg(test)]
mod config_tests;
#[cfg(test)]
mod discovery_tests;
#[cfg(test)]
mod health_tests;
#[cfg(test)]
mod sync_tests;
