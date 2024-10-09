use std::io::Read;

use serialport::{
	SerialPort,
	SerialPortBuilder
};

use crate::types::{Error, Result};

use super::Command;

pub struct Engine {
	//loop control
		halt: bool,

    //communication
		receiver: std::sync::mpsc::Receiver<Vec<u8>>,
		command_receiver: std::sync::mpsc::Receiver<Command>,
		sender: std::sync::mpsc::Sender<Vec<u8>>,
		error_sender: std::sync::mpsc::Sender<Error>,

	//connection
		connection: Box<dyn SerialPort>,
}

impl Engine {
	pub fn new(
		receiver: std::sync::mpsc::Receiver<Vec<u8>>,
		command_receiver: std::sync::mpsc::Receiver<Command>,
		sender: std::sync::mpsc::Sender<Vec<u8>>,
		error_sender: std::sync::mpsc::Sender<Error>,
		serial_port_builder: SerialPortBuilder,
	) -> Result<Engine> {
		let connection = serial_port_builder
			.timeout(std::time::Duration::from_millis(1))
			.open()?;

		Ok(
			Engine {
				halt: false,

				receiver,
				command_receiver,
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
		for command in self.command_receiver.try_iter() {
			match command {
				Command::Halt => {
					self.halt = true;
					return;
				}
			}
		}

		let buf = self.receiver.try_iter().flatten().collect::<Vec<u8>>();
		if let Err(write_all_error) = self.connection.write_all(&buf) {
			tracing::warn!("write_all error: {write_all_error}");
			if let Err(error) = self.error_sender.send(write_all_error.into()) {
				tracing::error!("mpsc send error: {error}");
			}
			self.halt = true;
			return;
		}
		if let Err(flush_error) = self.connection.flush() {
			tracing::warn!("flush error: {flush_error}");
			if let Err(error) = self.error_sender.send(flush_error.into()) {
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