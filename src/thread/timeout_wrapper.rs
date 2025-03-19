use crate::thread::TimeoutError;
use anyhow::Result;
use std::{
    thread,
    time::{Duration, Instant},
};

pub fn timeout_wrapper<F, T>(f: F, timeout_duration: Duration) -> Result<T>
where
    F: Fn() -> T + Send + 'static,
    T: Send + 'static,
{
    let start = Instant::now();
    let handle = thread::spawn(f);

    loop {
        if start.elapsed() >= timeout_duration {
            return Err(TimeoutError.into());
        }

        if handle.is_finished() {
            return match handle.join() {
                Ok(result) => Ok(result),
                Err(_) => Err(TimeoutError.into()), // Treat panic as timeout for simplicity in this example
            };
        }
        thread::sleep(Duration::from_millis(10));
    }
}
