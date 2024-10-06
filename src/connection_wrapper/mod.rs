#![allow(dead_code)]

use serialport::SerialPortBuilder;

use crate::types::{Error, Result};

mod engine;
use engine::Engine;

#[derive(Debug)]
pub struct ConnectionWrapper {
	thread_handle: std::thread::JoinHandle<()>,
	receiver: std::sync::mpsc::Receiver<Vec<u8>>,
	sender: std::sync::mpsc::Sender<Vec<u8>>,
	error_receiver: std::sync::mpsc::Receiver<Error>,
}

impl ConnectionWrapper {
	pub fn new(serial_port_builder: SerialPortBuilder) -> ConnectionWrapper {
		let (to_engine_sender, to_engine_receiver) = std::sync::mpsc::channel::<Vec<u8>>();
		let (from_engine_sender, from_engine_receiver) = std::sync::mpsc::channel::<Vec<u8>>();
		let (from_engine_error_sender, from_engine_error_receiver) = std::sync::mpsc::channel::<Error>();

		let thread_handle = std::thread::spawn(move || {
			match Engine::new(
				to_engine_receiver,
				from_engine_sender,
				from_engine_error_sender.clone(),
				serial_port_builder,
			) {
				Ok(mut engine) => engine.ignition(), 
				Err(err) => {
					tracing::debug!("Engine failed to start: {err}");
					if let Err(error) = from_engine_error_sender.send(err) {
						tracing::error!("mpsc send error: {error}");
					}
				}
			}
		});

		ConnectionWrapper {
			thread_handle,
			receiver: from_engine_receiver,
			sender: to_engine_sender,
			error_receiver: from_engine_error_receiver
		}
	}
}

impl ConnectionWrapper {
	pub fn is_active(&self) -> bool {
		!self.thread_handle.is_finished()
	}
}

impl ConnectionWrapper {
	#[tracing::instrument(skip(self), err, ret, level = "DEBUG")]
	pub fn write(&mut self, buf:Vec<u8>) -> Result<()> {
		let mut errors = self.error_receiver.try_iter().collect::<Vec<Error>>();
		if !errors.is_empty() {
			return Err( errors.swap_remove(0) );
		}

		if self.thread_handle.is_finished() {
			return Err(Error::Disconnected);
		}

		self.sender.send(buf)?;

		Ok(())
	}
	#[tracing::instrument(skip(self), err, ret, level = "DEBUG")]
	pub fn poll(&self) -> Result<Vec<u8>> {
		let mut errors = self.error_receiver.try_iter().collect::<Vec<Error>>();
		if !errors.is_empty() {
			return Err( errors.swap_remove(0) );
		}

		if self.thread_handle.is_finished() {
			return Err(Error::Disconnected);
		}
		
		Ok(self.receiver.try_iter().flatten().collect::<Vec<u8>>())
	}
}