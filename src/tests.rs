use super::*;

use std::sync::TryLockError;
use std::panic::catch_unwind;
use test::Bencher;

#[test]
fn lock() {
    let mutex = AdaptiveMutex::new(0);
    let mut guard = mutex.lock().unwrap();
    *guard += 1;

    assert!(mutex.try_lock().is_err());
}

#[test]
fn poison() {
    let mutex = AdaptiveMutex::new(0);
    assert!(!mutex.is_poisoned());

    assert!(catch_unwind(|| {
        let _guard = mutex.lock().unwrap();
        panic!();
    }).is_err());

    assert!(mutex.is_poisoned());
    assert!(mutex.lock().is_err());

    let try_lock_result = mutex.try_lock();

    assert!(try_lock_result.is_err());
    assert!(match try_lock_result.err().unwrap() {
        TryLockError::Poisoned(_) => true,
        _ => false,
    });
}

#[bench]
fn increment(b: &mut Bencher) {
    let mutex = AdaptiveMutex::new(0);

    b.iter(|| {
        let mut guard = mutex.lock().unwrap();
        *guard += 1
    });
}

#[bench]
fn increment_mutex(b: &mut Bencher) {
    use std::sync::Mutex;

    let mutex = Mutex::new(0);

    b.iter(|| {
        let mut guard = mutex.lock().unwrap();
        *guard += 1
    });
}
