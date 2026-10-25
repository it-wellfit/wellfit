#![forbid(unsafe_code)]

//! Wellfit Core - Desktop Session Mutex & Resilient Timing Manager
//!
//! Coordinates Windows Fuyi ERP desktop session locks to prevent focus contention,
//! dirty reads, and deadlock states.
//! Designed with:
//! - Automated Stale-Lock Self-Healing (reclaims locks from crashed processes)
//! - Exponential Backoff Contention Prevention (eliminates CPU spinning)
//! - Extensible WorkWave Policy (supports arbitrary custom work waves)

use std::fs::{self, OpenOptions};
use std::io::Write;
use std::path::{Path, PathBuf};
use std::thread;
use std::time::{Duration, Instant};

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum WorkWave {
    Wave1ShopeeInbound,   // ~08:30 Morning Inbound
    Wave2InternalReports, // ~10:00 Internal Daily Reports
    Wave3MtrixOutbound,   // ~17:00 EOD MTRIX Submission
    Custom(String),       // Dynamic/extensible custom wave
}

impl WorkWave {
    pub fn name(&self) -> &str {
        match self {
            Self::Wave1ShopeeInbound => "WAVE_1_SHOPEE_INBOUND",
            Self::Wave2InternalReports => "WAVE_2_INTERNAL_REPORTS",
            Self::Wave3MtrixOutbound => "WAVE_3_MTRIX_OUTBOUND",
            Self::Custom(s) => s.as_str(),
        }
    }
}

#[derive(Debug)]
pub enum MutexError {
    Timeout(Duration, String),
    Io(std::io::Error),
}

impl std::fmt::Display for MutexError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Timeout(dur, wave) => {
                write!(
                    f,
                    "Desktop session mutex contention timeout ({dur:?}) for {wave}"
                )
            }
            Self::Io(e) => write!(f, "IO error acquiring desktop mutex: {e}"),
        }
    }
}

impl std::error::Error for MutexError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::Io(e) => Some(e),
            _ => None,
        }
    }
}

impl From<std::io::Error> for MutexError {
    fn from(e: std::io::Error) -> Self {
        Self::Io(e)
    }
}

pub struct MutexGuard {
    lock_file: PathBuf,
}

impl Drop for MutexGuard {
    fn drop(&mut self) {
        let _ = fs::remove_file(&self.lock_file);
    }
}

pub struct DesktopSessionMutex {
    lock_path: PathBuf,
    stale_lock_threshold: Duration,
}

impl DesktopSessionMutex {
    pub fn new<P: AsRef<Path>>(lock_path: P) -> Self {
        Self {
            lock_path: lock_path.as_ref().to_path_buf(),
            stale_lock_threshold: Duration::from_secs(900), // 15 minutes default stale boundary
        }
    }

    pub fn with_stale_threshold<P: AsRef<Path>>(lock_path: P, threshold: Duration) -> Self {
        Self {
            lock_path: lock_path.as_ref().to_path_buf(),
            stale_lock_threshold: threshold,
        }
    }

    pub fn default_mutex() -> Self {
        Self::new("wellfit_desktop.lock")
    }

    /// Acquire desktop session lock with adaptive exponential backoff and stale-lock recovery.
    pub fn acquire(&self, wave: WorkWave, timeout: Duration) -> Result<MutexGuard, MutexError> {
        let start = Instant::now();
        let mut backoff = Duration::from_millis(20);

        loop {
            // Stale lock recovery. Age alone is not a sufficient reason to break
            // a lock: a long running wave keeps a valid lock file that is
            // simply old. The lock is only broken when it is old **and** the
            // process that owns it is gone.
            if self.lock_path.exists() && self.lock_is_stale() {
                let _ = fs::remove_file(&self.lock_path);
            }

            match OpenOptions::new()
                .write(true)
                .create_new(true)
                .open(&self.lock_path)
            {
                Ok(mut file) => {
                    let msg = format!(
                        "LOCKED_BY={};PID={};TIME={:?}\n",
                        wave.name(),
                        std::process::id(),
                        Instant::now()
                    );
                    let _ = file.write_all(msg.as_bytes());
                    return Ok(MutexGuard {
                        lock_file: self.lock_path.clone(),
                    });
                }
                Err(ref e) if e.kind() == std::io::ErrorKind::AlreadyExists => {
                    if start.elapsed() >= timeout {
                        return Err(MutexError::Timeout(timeout, wave.name().to_string()));
                    }
                    thread::sleep(backoff);
                    // Exponential backoff capped at 150ms to prevent CPU saturation
                    backoff = std::cmp::min(backoff * 2, Duration::from_millis(150));
                }
                Err(e) => return Err(MutexError::Io(e)),
            }
        }
    }

    /// Whether the lock file may be broken.
    ///
    /// The file must be older than the configured threshold and the process
    /// that created it must no longer exist. When the owner cannot be
    /// determined the age rule alone decides, which keeps the behaviour on
    /// platforms without a process table query.
    fn lock_is_stale(&self) -> bool {
        let Ok(metadata) = fs::metadata(&self.lock_path) else {
            return false;
        };
        let Ok(modified) = metadata.modified() else {
            return false;
        };
        let Ok(elapsed) = modified.elapsed() else {
            return false;
        };
        if elapsed <= self.stale_lock_threshold {
            return false;
        }
        match lock_owner_pid(&self.lock_path) {
            Some(pid) => {
                // Only a lock whose owner is known to be gone may be broken.
                // If the liveness of the owner cannot be established, the lock
                // is left alone and the caller waits for its timeout.
                matches!(process_is_running(pid), Some(false))
            }
            None => {
                // No owner PID could be parsed from the lock file: treat the
                // lock as stale and breakable, since its owner cannot be
                // established as live.
                true
            }
        }
    }
}

/// Read the owner process id from a lock file.
fn lock_owner_pid(path: &Path) -> Option<u32> {
    let content = fs::read_to_string(path).ok()?;
    for part in content.split(';') {
        let part = part.trim();
        if let Some(value) = part.strip_prefix("PID=") {
            return value.trim().parse::<u32>().ok();
        }
    }
    None
}

/// Whether a process id is still running.
///
/// Returns `None` when the platform does not allow the question to be
/// answered, so that the caller can fall back to the age rule.
fn process_is_running(pid: u32) -> Option<bool> {
    #[cfg(windows)]
    {
        let output = std::process::Command::new("tasklist")
            .args(["/FI", &format!("PID eq {pid}"), "/NH"])
            .output()
            .ok()?;
        let text = String::from_utf8_lossy(&output.stdout);
        if text.contains(&pid.to_string()) {
            Some(true)
        } else {
            Some(false)
        }
    }
    #[cfg(not(windows))]
    {
        let _ = pid;
        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Lock file of a test, placed in the temporary folder so that a failing
    /// test cannot leave a lock file inside the repository.
    fn test_lock_path(name: &str) -> String {
        let dir = std::env::temp_dir();
        dir.join(name).to_string_lossy().into_owned()
    }

    #[test]
    fn test_mutex_acquire_and_release() {
        let lock_path = test_lock_path("wellfit_test_mutex_raii.lock");
        let _ = fs::remove_file(&lock_path);

        let mutex = DesktopSessionMutex::new(&lock_path);
        {
            let _guard = mutex
                .acquire(WorkWave::Wave1ShopeeInbound, Duration::from_secs(10))
                .unwrap();
            assert!(Path::new(&lock_path).exists());
        }
        assert!(!Path::new(&lock_path).exists());
    }

    #[test]
    fn test_mutex_contention_timeout() {
        let lock_path = test_lock_path("wellfit_test_mutex_contention.lock");
        let _ = fs::remove_file(&lock_path);

        let mutex1 = DesktopSessionMutex::new(&lock_path);
        let mutex2 = DesktopSessionMutex::new(&lock_path);

        // The first holder gets a generous budget: the point of the test is the
        // timeout of the second one, not how fast the first one acquires.
        let _guard = mutex1
            .acquire(WorkWave::Wave1ShopeeInbound, Duration::from_secs(30))
            .unwrap();
        let res = mutex2.acquire(WorkWave::Wave2InternalReports, Duration::from_millis(100));
        assert!(matches!(res, Err(MutexError::Timeout(_, _))));
    }

    #[test]
    fn test_a_live_lock_is_not_broken_even_when_the_file_is_old() {
        let lock_path = test_lock_path("wellfit_test_mutex_live_owner.lock");
        let _ = fs::remove_file(&lock_path);

        // A zero threshold makes the file "old" by age, but the owner is this
        // very process: the lock must survive.
        let mutex = DesktopSessionMutex::with_stale_threshold(&lock_path, Duration::from_millis(0));
        let _guard = mutex
            .acquire(WorkWave::Wave1ShopeeInbound, Duration::from_secs(10))
            .unwrap();

        let other = DesktopSessionMutex::with_stale_threshold(&lock_path, Duration::from_millis(0));
        let res = other.acquire(WorkWave::Wave2InternalReports, Duration::from_millis(100));
        assert!(
            matches!(res, Err(MutexError::Timeout(_, _))),
            "a lock owned by a live process must not be broken"
        );
    }

    #[test]
    fn test_custom_wave_and_stale_lock_recovery() {
        let lock_path = test_lock_path("wellfit_test_mutex_stale.lock");
        let _ = fs::remove_file(&lock_path);

        // Simulate stale lock created with 0 threshold
        fs::write(&lock_path, "LOCKED_BY=DEAD_PROCESS;PID=99999\n").unwrap();
        let mutex = DesktopSessionMutex::with_stale_threshold(&lock_path, Duration::from_millis(1));
        thread::sleep(Duration::from_millis(10));

        let custom_wave = WorkWave::Custom("WAVE_CUSTOM_COUPANG".to_string());
        let guard = mutex.acquire(custom_wave, Duration::from_secs(10));
        assert!(guard.is_ok(), "Stale lock must be cleanly recovered");
        drop(guard);
        assert!(!Path::new(&lock_path).exists());
    }
}
