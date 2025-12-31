use std::sync::atomic::{AtomicBool, Ordering};

pub static SHUTDOWN_REQUESTED: AtomicBool = AtomicBool::new(false);

pub fn request_shutdown() {
    SHUTDOWN_REQUESTED.store(true, Ordering::SeqCst);
}

pub fn is_shutdown_requested() -> bool {
    SHUTDOWN_REQUESTED.load(Ordering::SeqCst)
}

#[allow(dead_code)]
pub fn clear_shutdown_request() {
    SHUTDOWN_REQUESTED.store(false, Ordering::SeqCst);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_shutdown_flag_initial_state() {
        clear_shutdown_request();
        assert!(!is_shutdown_requested());
    }

    #[test]
    fn test_request_shutdown() {
        clear_shutdown_request();
        request_shutdown();
        assert!(is_shutdown_requested());
        clear_shutdown_request();
    }

    #[test]
    fn test_clear_shutdown_request() {
        request_shutdown();
        assert!(is_shutdown_requested());
        clear_shutdown_request();
        assert!(!is_shutdown_requested());
    }

    #[test]
    fn test_thread_safe_access() {
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
