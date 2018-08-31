extern crate adaptive_mutex;

use adaptive_mutex::AdaptiveMutex;

use std::fs::File;
use std::io::Write;
use std::os::unix::io::FromRawFd;
use std::sync::Arc;
use std::thread::spawn;

type Mutex<T> = AdaptiveMutex<T>;

fn main() {
    static NUM_THREADS: usize = 8;

    let stdout = unsafe { File::from_raw_fd(1) };

    let mutex_ptr = Arc::new(Mutex::new(stdout));

    let threads = (0..NUM_THREADS).map(|_| {
        let cloned = mutex_ptr.clone();

        spawn(move || {
            loop {
                let mut guard = cloned.lock().unwrap();
                writeln!(*guard, "hello, world!");
            }
        })
    });

    for thread in threads {
        thread.join().unwrap();
    }
}
