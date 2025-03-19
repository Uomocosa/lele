// Automatic imports (from build.rs)
mod timeout_error;
pub use timeout_error::TimeoutError;
mod timeout_wrapper;
pub use timeout_wrapper::timeout_wrapper;
