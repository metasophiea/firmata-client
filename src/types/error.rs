use snafu::prelude::*;

/// Firmata error type.
#[derive(Debug, Snafu)]
#[snafu(visibility(pub))]
pub enum Error {
    /// Unknown `SysEx` code: {code}
    UnknownSysEx { code: u8 },
    /// Received a bad byte: {byte}
    BadByte { byte: u8 },
    /// I/O error: {source}
    StdIo { source: std::io::Error },
    /// UTF8 error: {source}
    Utf8 { source: std::str::Utf8Error },
	/// Message was empty.
	EmptyBufferMessage,
    /// Message was too short.
    MessageTooShort,
    /// Pin out of bounds: {pin} ({len}).
    PinOutOfBounds { pin: u8, len: usize },
}