use std::sync::{Arc, Mutex, MutexGuard};

/// helper trait to simplify the error handling when locking a Mutex.
pub trait MutexUtil<T> {
    fn acquire_mutex(&self) -> MutexGuard<'_, T>;
}

impl<T> MutexUtil<T> for Arc<Mutex<T>> {
    fn acquire_mutex(&self) -> MutexGuard<'_, T> {
        match self.lock() {
            Ok(locked_mutex) => locked_mutex,
            Err(e) => {
                panic!("poisoned mutex: {e}")
            }
        }
    }
}
