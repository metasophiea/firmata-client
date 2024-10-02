use std::sync::mpsc::SendError;

use serialport::Error as SerialPortError;

/// Firmata error type.
#[derive(Debug)]
pub enum Error {
	/// There is no connection to the board 
	Disconnected,
    /// Unknown `SysEx` code
    UnknownSysEx { code: u8 },
    /// Received a bad byte
    BadByte(u8),
    /// I/O error
    StdIo(std::io::Error),
    /// UTF8 error
    Utf8(std::str::Utf8Error),
	/// Mpsc `SendError`
	MpscSend(SendError<Vec<u8>>),
    /// Pin out of bounds
    PinOutOfBounds { pin: u8, len: usize, source: String },
    /// Serialport Error
	Serialport(SerialPortError)
}

impl std::fmt::Display for Error  {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		match self {
			Error::Disconnected => write!(f, "Disconnected"),
			Error::UnknownSysEx { code } => write!(f, "Unknown `SysEx` code: {code}"),
			Error::BadByte(byte) => write!(f, "Received a bad byte: {byte}"),
			Error::StdIo(error) => write!(f, "I/O error: {error}"),
			Error::Utf8(error) => write!(f, "UTF8 error: {error}"),
			Error::MpscSend(error) => write!(f, "Mpsc SendError error: {error}"),
			Error::PinOutOfBounds { pin, len, source } => write!(f, "Pin out of bounds: {pin} ({len}) source: {source}"),
			Error::Serialport(error) => write!(f, "Serialport Error: {error}"),
		}
	}
}

impl From<std::str::Utf8Error> for Error {
    fn from(error: std::str::Utf8Error) -> Self {
        Error::Utf8(error)
    }
}

impl From<SendError<Vec<u8>>> for Error {
    fn from(error: SendError<Vec<u8>>) -> Self {
        Error::MpscSend(error)
    }
}

impl From<SerialPortError> for Error {
    fn from(error: SerialPortError) -> Self {
        Error::Serialport(error)
    }
}