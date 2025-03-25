// TODO!

// use std::sync::atomic::{AtomicBool, Ordering};
// use log::{Record, Level, Metadata};

// struct MyLogger {
//     enabled: AtomicBool,
// }

// impl MyLogger {
//     const fn new(enabled: bool) -> MyLogger {
//         MyLogger {
//             enabled: AtomicBool::new(enabled),
//         }
//     }

//     // Methods to enable/disable logging at runtime
//     fn enable(&self) {
//         self.enabled.store(true, Ordering::Relaxed);
//     }

//     fn disable(&self) {
//         self.enabled.store(false, Ordering::Relaxed);
//     }
// }

// impl log::Log for MyLogger {
//     fn enabled(&self, metadata: &Metadata) -> bool {
//         // Only log if logging is enabled
//         self.enabled.load(Ordering::Relaxed) && metadata.level() <= Level::Info
//     }

//     fn log(&self, record: &Record) {
//         if self.enabled(record.metadata()) {
//             // Print the log message to standard output
//             println!("{} - {}", record.level(), record.args());
//         }
//     }

//     fn flush(&self) {}
// }

// // Create a static instance of our logger
// static LOGGER: MyLogger = MyLogger::new(true);

// pub fn init_logger() {
//     log::set_logger(&LOGGER)
//         .map(|()| log::set_max_level(log::LevelFilter::Info))
//         .expect("Failed to initialize logger");
// }

// pub fn enable_logging() {
//     LOGGER.enable();
// }

// pub fn disable_logging() {
//     LOGGER.disable();
// }

// fn main() {
//     // Initialize the logger
//     init_logger();

//     // Log some messages
//     log::info!("Logging is enabled and this will be printed");

//     // Disable logging
//     disable_logging();
//     log::info!("This log will not be printed");

//     // Re-enable logging
//     enable_logging();
//     log::info!("Logging is enabled again!");
// }

// IT DOES NOT WORK

// fn _fun() {
//     if cfg!(debug) { println!("hello"); };
// }

// #[cfg(test)]
// mod tests{
//     use super::*;

//     #[tokio::test]
//     // run test by using: 'cargo test meta::debug::tests::test_debug -- --exact --nocapture'
//     async fn test_debug() {
//         #[cfg_attr(feature = "debug",true)]
//         _fun(); // Should print "hello"

//         _fun(); //Should not print anything

//         #[cfg(debug)]
//         _fun(); // Should print "hello"
//     }

// }
