#![feature(test)]

extern crate test;

use std::sync::atomic::{AtomicIsize, ATOMIC_ISIZE_INIT, Ordering};
use std::sync::{LockResult, Mutex, MutexGuard, TryLockError, TryLockResult};

#[cfg(test)]
mod tests;

pub struct AdaptiveMutex<T: ?Sized> {
    estimator: AtomicIsize,
    mutex: Mutex<T>,
}

impl<T> AdaptiveMutex<T> {
    pub fn new(t: T) -> AdaptiveMutex<T> {
        AdaptiveMutex {
            estimator: ATOMIC_ISIZE_INIT,
            mutex: Mutex::new(t),
        }
    }

    pub fn into_inner(self) -> LockResult<T> where T: Sized {
        self.mutex.into_inner()
    }
}

impl <T: ?Sized> AdaptiveMutex<T> {
    pub fn lock(&self) -> LockResult<MutexGuard<T>> {
        let mut num_spins = 0;

        loop { match self.try_lock() {
            Ok(g) => {
                self.update_estimator(num_spins);

                return Ok(g);
            },
            Err(TryLockError::Poisoned(e)) => return Err(e),
            Err(TryLockError::WouldBlock) => {
                num_spins += 1;

                let max_spins = 2 * self.estimator.load(Ordering::SeqCst);
                if num_spins >= max_spins {
                    return self.mutex.lock();
                }
            },
        } }
    }

    pub fn try_lock(&self) -> TryLockResult<MutexGuard<T>> {
        self.mutex.try_lock()
    }

    pub fn is_poisoned(&self) -> bool {
        self.mutex.is_poisoned()
    }

    pub fn get_mut(&mut self) -> LockResult<&mut T> {
        self.mutex.get_mut()
    }

    fn update_estimator(&self, num_spins: isize) -> isize {
        let current = self.estimator.load(Ordering::SeqCst);
        let to_add = (num_spins - current) / 8;

        self.estimator.fetch_add(to_add, Ordering::SeqCst)
    }
}

impl<T: ?Sized> std::panic::UnwindSafe for AdaptiveMutex<T> { }

impl<T: ?Sized> std::panic::RefUnwindSafe for AdaptiveMutex<T> { }

unsafe impl<T: ?Sized + Send> Send for AdaptiveMutex<T> { }

unsafe impl<T: ?Sized + Send> Sync for AdaptiveMutex<T> { }

impl<T> From<T> for AdaptiveMutex<T> {
    fn from(t: T) -> AdaptiveMutex<T> {
        AdaptiveMutex::new(t)
    }
}

impl<T: ?Sized + Default> Default for AdaptiveMutex<T> {
    fn default() -> AdaptiveMutex<T> {
        AdaptiveMutex::new(T::default())
    }
}

impl<T: ?Sized + std::fmt::Debug> std::fmt::Debug for AdaptiveMutex<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        f.debug_struct("AdaptiveMutex")
            .field("estimator", &self.estimator)
            .field("mutex", &&self.mutex)
            .finish()
    }
}
