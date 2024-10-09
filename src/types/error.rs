use std::sync::mpsc::SendError;

use serialport::Error as SerialPortError;

use super::super::connection_wrapper::Command;

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
	/// Mpsc Buf `SendError`
	MpscBufSend(SendError<Vec<u8>>),
	/// Mpsc Command `SendError`
	MpscCommandSend(SendError<Command>),
	/// Invalid Pin Mode
	InvalidPinMode { pin: u8, modes: Vec<u8> },
    /// Pin out of bounds
    PinOutOfBounds { pin: u8, len: usize, source: String },
    /// Serialport Error
	Serialport(SerialPortError)
}

impl Error {
	pub fn is_disconnected(&self) -> bool {
		if let Error::Disconnected = self {
			true
		} else {
			false
		}
	}
}

impl std::fmt::Display for Error {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		match self {
			Error::Disconnected => write!(f, "Disconnected"),
			Error::UnknownSysEx { code } => write!(f, "Unknown `SysEx` code: {code}"),
			Error::BadByte(byte) => write!(f, "Received a bad byte: {byte}"),
			Error::StdIo(error) => write!(f, "I/O error: {error}"),
			Error::Utf8(error) => write!(f, "UTF8 error: {error}"),
			Error::MpscBufSend(error) => write!(f, "Mpsc Buf SendError error: {error}"),
			Error::MpscCommandSend(error) => write!(f, "Mpsc Command SendError error: {error}"),
			Error::InvalidPinMode { pin, modes } => write!(f, "Invalid Pin Mode: {pin} modes: {modes:?}"),
			Error::PinOutOfBounds { pin, len, source } => write!(f, "Pin out of bounds: {pin} ({len}) source: {source}"),
			Error::Serialport(error) => write!(f, "Serialport Error: {error}"),
		}
	}
}

impl From<std::io::Error> for Error {
    fn from(error: std::io::Error) -> Self {
        Error::StdIo(error)
    }
}

impl From<std::str::Utf8Error> for Error {
    fn from(error: std::str::Utf8Error) -> Self {
        Error::Utf8(error)
    }
}

impl From<SendError<Vec<u8>>> for Error {
    fn from(error: SendError<Vec<u8>>) -> Self {
        Error::MpscBufSend(error)
    }
}

impl From<SendError<Command>> for Error {
    fn from(error: SendError<Command>) -> Self {
        Error::MpscCommandSend(error)
    }
}

impl From<SerialPortError> for Error {
    fn from(error: SerialPortError) -> Self {
        Error::Serialport(error)
    }
}