use ::std::fmt;

#[derive(Debug, Clone)]
pub struct TimeoutError;

impl fmt::Display for TimeoutError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "Function execution timed out")
    }
}

impl std::error::Error for TimeoutError {}
