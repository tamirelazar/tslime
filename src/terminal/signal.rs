//! Global signal handling for application shutdown.
//!
//! Provides a thread-safe atomic flag to coordinate shutdown across the application,
//! primarily triggered by Unix signals (SIGINT, SIGTERM) caught in `screen.rs`.

use std::sync::atomic::{AtomicBool, Ordering};

/// Global flag indicating if a shutdown has been requested (e.g., via Ctrl+C).
pub static SHUTDOWN_REQUESTED: AtomicBool = AtomicBool::new(false);

/// Signal that the application should shut down.
///
/// Called from the SIGINT/SIGTERM handlers registered in `screen.rs`.
#[cfg(unix)]
pub fn request_shutdown() {
    SHUTDOWN_REQUESTED.store(true, Ordering::SeqCst);
}

/// Check if a shutdown has been requested.
pub fn is_shutdown_requested() -> bool {
    SHUTDOWN_REQUESTED.load(Ordering::SeqCst)
}

/// Reset the shutdown flag (for testing purposes).
#[cfg(test)]
pub fn clear_shutdown_request() {
    SHUTDOWN_REQUESTED.store(false, Ordering::SeqCst);
}

#[cfg(all(unix, test))]
mod tests {
    use super::*;
    use std::sync::Mutex;

    static TEST_LOCK: Mutex<()> = Mutex::new(());

    #[test]
    fn test_shutdown_flag_initial_state() {
        let _lock = TEST_LOCK.lock().unwrap();
        clear_shutdown_request();
        assert!(!is_shutdown_requested());
    }

    #[test]
    fn test_request_shutdown() {
        let _lock = TEST_LOCK.lock().unwrap();
        clear_shutdown_request();
        request_shutdown();
        assert!(is_shutdown_requested());
        clear_shutdown_request();
    }

    #[test]
    fn test_clear_shutdown_request() {
        let _lock = TEST_LOCK.lock().unwrap();
        request_shutdown();
        assert!(is_shutdown_requested());
        clear_shutdown_request();
        assert!(!is_shutdown_requested());
    }

    #[test]
    fn test_thread_safe_access() {
        let _lock = TEST_LOCK.lock().unwrap();
        clear_shutdown_request();

        let handle = std::thread::spawn(|| {
            for _ in 0..1000 {
                request_shutdown();
                std::thread::sleep(std::time::Duration::from_micros(1));
            }
        });

        for _ in 0..1000 {
            if is_shutdown_requested() {
                clear_shutdown_request();
            }
            std::thread::sleep(std::time::Duration::from_micros(1));
        }

        handle.join().unwrap();
    }
}
