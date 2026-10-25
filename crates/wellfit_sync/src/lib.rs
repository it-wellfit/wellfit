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
