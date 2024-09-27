use std::io::{Error as IOError, Read};

use serialport::{
	SerialPort,
	SerialPortBuilder
};

use crate::types::Result;

pub struct Engine {
	//loop control
		halt: bool,

    //communication
		receiver: std::sync::mpsc::Receiver<Vec<u8>>,
		sender: std::sync::mpsc::Sender<Vec<u8>>,
		error_sender: std::sync::mpsc::Sender<IOError>,

	//connection
		connection: Box<dyn SerialPort>,
}

impl Engine {
	pub fn new(
		receiver: std::sync::mpsc::Receiver<Vec<u8>>,
		sender: std::sync::mpsc::Sender<Vec<u8>>,
		error_sender: std::sync::mpsc::Sender<IOError>,
		serial_port_builder: SerialPortBuilder,
	) -> Result<Engine> {
		let connection = serial_port_builder
			.timeout(std::time::Duration::from_millis(1))
			.open()?;

		Ok(
			Engine {
				halt: false,

				receiver,
				sender,
				error_sender,

				connection,
			}
		)
	}
	pub fn ignition(&mut self) {
		while !self.halt {
			self.revolution();
		}
	}
}

impl Engine {
	#[tracing::instrument(skip(self), level = "DEBUG")]
	fn revolution(&mut self) {
		let buf = self.receiver.try_iter().flatten().collect::<Vec<u8>>();
		if let Err(write_all_error) = self.connection.write_all(&buf) {
			tracing::warn!("write_all error: {write_all_error}");
			if let Err(error) = self.error_sender.send(write_all_error) {
				tracing::error!("mpsc send error: {error}");
			}
			self.halt = true;
			return;
		}
		if let Err(flush_error) = self.connection.flush() {
			tracing::warn!("flush error: {flush_error}");
			if let Err(error) = self.error_sender.send(flush_error) {
				tracing::error!("mpsc send error: {error}");
			}
			self.halt = true;
			return;
		}

		let mut buffer:Vec<u8> = vec![];
		let mut byte = [0];
		while let Ok(()) = self.connection.read_exact(&mut byte) {
			buffer.push(byte[0]);
		}
		if let Err(error) = self.sender.send(buffer) {
			tracing::warn!("{error}");
			self.halt = true;
			return;
		}
	}
}