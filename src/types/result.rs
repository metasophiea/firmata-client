use super::Error;

/// Result type with Firmata Error.
pub type Result<T> = std::result::Result<T, Error>;
