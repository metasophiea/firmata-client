#![allow(clippy::cast_possible_truncation)]

use std::io::{Read, Write};

use snafu::prelude::*;

use crate::constants::{
    ANALOG_MAPPING_QUERY,
    ANALOG_MESSAGE,
    CAPABILITY_QUERY,
    DIGITAL_MESSAGE,
    END_SYSEX,
    I2C_CONFIG,
    I2C_READ,
    I2C_REQUEST,
    I2C_WRITE,
    REPORT_ANALOG,
    REPORT_DIGITAL,
    REPORT_FIRMWARE,
    SET_PIN_MODE,
    SYSEX_REALTIME,
    START_SYSEX
};
use crate::types::{
    Error,
    I2CReply,
    Pin,
    Result,
    StdIoSnafu
};

/// A Firmata board representation.
// definition
	#[derive(Debug)]
	pub struct Board<T: Read + Write + std::fmt::Debug> {
		pub connection: Box<T>,
		pub pins: Vec<Pin>,
		pub i2c_data: Vec<I2CReply>,
		pub protocol_version: String,
		pub firmware_name: String,
		pub firmware_version: String,
	}

// creation
	impl<T: Read + Write + std::fmt::Debug> Board<T> {
		#[tracing::instrument(err, ret(Display))]
		pub fn new(connection: Box<T>) -> Result<Board<T>> {
			let mut board = Board {
				connection,
				firmware_name: String::new(),
				firmware_version: String::new(),
				protocol_version: String::new(),
				pins: vec![],
				i2c_data: vec![],
			};

			board.report_firmware()?;
			board.poll()?;

			board.query_capabilities()?;
			board.poll()?;

			board.query_analog_mapping()?;
			board.poll()?;
			
			board.report_digital(0, 1)?;
			board.report_digital(1, 1)?;

			Ok(board)
		}
	}

// tools
	impl<T: Read + Write + std::fmt::Debug> Board<T> {
		/// Write on the internal connection.
		#[tracing::instrument(skip(self), err, ret, level = "DEBUG")]
		fn write_to_connection(&mut self, buf: &[u8]) -> Result<()> {
			tracing::info!("write_to_connection(buf{buf:?})");
			self.connection
				.write(buf)
				.map(|_| ())
				.with_context(|_| StdIoSnafu)
		}
	}

// printing
	impl<T: Read + Write + std::fmt::Debug> std::fmt::Display for Board<T> {
		fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
			write!(
				f,
				"Board {{ firmware={}, version={}, protocol={}, connection={:?}}}",
				self.firmware_name, self.firmware_version, self.protocol_version, self.connection
			)
		}
	}

// get
	impl<T: Read + Write + std::fmt::Debug> Board<T> {
		#[must_use]
		/// Get pins that the board has access to.
		pub fn get_pins(&self) -> &Vec<Pin> {
			&self.pins
		}

		#[must_use]
		/// Get the raw I2C replies that have been read from the board.
		pub fn get_i2c_data(&self) -> &Vec<I2CReply> {
			&self.i2c_data
		}

		#[must_use]
		/// Get the current Firmata protocol version.
		pub fn get_protocol_version(&self) -> &String {
			&self.protocol_version
		}

		#[must_use]
		/// Get the firmware name.
		pub fn get_firmware_name(&self) -> &String {
			&self.firmware_name
		}

		#[must_use]
		/// Get the firmware version.
		pub fn get_firmware_version(&self) -> &String {
			&self.firmware_version
		}
	}

// set
	impl<T: Read + Write + std::fmt::Debug> Board<T> {
		/// Set the `mode` of the specified `pin`.
		#[tracing::instrument(skip(self), err, ret, level = "DEBUG")]
		pub fn set_pin_mode(&mut self, pin: u8, mode: u8) -> Result<()> {
			if let Some(pin) = self.pins.get_mut(pin as usize) {
				pin.modes = vec![mode];
			} else {
				return Err(Error::PinOutOfBounds { pin, len: self.pins.len() })
			}

			self.write_to_connection(&[SET_PIN_MODE, pin, mode])
		}
	}

// query
	impl<T: Read + Write + std::fmt::Debug> Board<T> {
		/// Query the board for available analog pins.
		#[tracing::instrument(skip(self), err, ret, level = "DEBUG")]
		pub fn query_analog_mapping(&mut self) -> Result<()> {
			self.write_to_connection(&[START_SYSEX, ANALOG_MAPPING_QUERY, END_SYSEX])
		}
	
    	/// Query the board for all available capabilities.
		#[tracing::instrument(skip(self), err, ret, level = "DEBUG")]
		pub fn query_capabilities(&mut self) -> Result<()> {
			self.write_to_connection(&[START_SYSEX, CAPABILITY_QUERY, END_SYSEX])
		}
	}
	
// i2c
	impl<T: Read + Write + std::fmt::Debug> Board<T> {
		/// Configure the `delay` in microseconds for I2C devices that require a delay between when the
		/// register is written to and the data in that register can be read.
		#[tracing::instrument(skip(self), err, ret, level = "DEBUG")]
		pub fn i2c_config(&mut self, delay: u16) -> Result<()> {
			self.write_to_connection(&[
				START_SYSEX,
				I2C_CONFIG,
				(delay & 0xFF) as u8,
				(delay >> 8 & 0xFF) as u8,
				END_SYSEX,
			])
		}

    	/// Read `size` bytes from I2C device at the specified `address`.
		#[tracing::instrument(skip(self), err, ret, level = "DEBUG")]
		pub fn i2c_read(&mut self, address: u8, size: u8) -> Result<()> {
			self.write_to_connection(&[
				START_SYSEX,
				I2C_REQUEST,
				address,
				I2C_READ << 3,
				size & SYSEX_REALTIME,
				(u16::from(size) >> 7) as u8 & SYSEX_REALTIME,
				END_SYSEX,
			])
		}

    	/// Write `data` to the I2C device at the specified `address`.
		#[tracing::instrument(skip(self), err, ret, level = "DEBUG")]
		pub fn i2c_write(&mut self, address: u8, data: &[u8]) -> Result<()> {
			let mut buf = vec![
				START_SYSEX,
				I2C_REQUEST,
				address,
				I2C_WRITE << 3
			];

			for datum in data {
				buf.push(datum & SYSEX_REALTIME);
				buf.push((u16::from(*datum) >> 7) as u8 & SYSEX_REALTIME);
			}

			buf.push(END_SYSEX);

			self.write_to_connection(&buf)
		}
	}

// report
	impl<T: Read + Write + std::fmt::Debug> Board<T> {
		/// Query the board for current firmware and protocol information.
		#[tracing::instrument(skip(self), err, ret, level = "DEBUG")]
		pub fn report_firmware(&mut self) -> Result<()> {
			self.write_to_connection(&[START_SYSEX, REPORT_FIRMWARE, END_SYSEX])
		}

    	/// Set the analog reporting `state` of the specified `pin`.
		#[tracing::instrument(skip(self), err, ret, level = "DEBUG")]
		pub fn report_analog(&mut self, pin: u8, state: u8) -> Result<()> {
			self.write_to_connection(&[REPORT_ANALOG | pin, state])
		}

    	/// Set the digital reporting `state` of the specified `pin`.
		#[tracing::instrument(skip(self), err, ret, level = "DEBUG")]
		pub fn report_digital(&mut self, pin: u8, state: u8) -> Result<()> {
			let port = pin / 8;
			self.write_to_connection(&[REPORT_DIGITAL | port, state])
		}
	}

// write
	impl<T: Read + Write + std::fmt::Debug> Board<T> {
		/// Write `level` to the analog `pin`.
		#[tracing::instrument(skip(self), err, ret, level = "DEBUG")]
		pub fn analog_write(&mut self, pin: u8, level: u8) -> Result<()> {
			if let Some(pin) = self.pins.get_mut(pin as usize) {
				pin.value = level;
			} else {
				return Err(Error::PinOutOfBounds { pin, len: self.pins.len() })
			}

			self.write_to_connection(&[
				ANALOG_MESSAGE | pin,
				level & SYSEX_REALTIME,
				(u16::from(level) >> 7) as u8 & SYSEX_REALTIME,
			])
		}

    	/// Write `level` to the digital `pin`.
		#[tracing::instrument(skip(self), err, ret, level = "DEBUG")]
		pub fn digital_write(&mut self, pin: u8, level: u8) -> Result<()> {
			let port = (pin/8) as usize;
			let mut value = 0;

			if let Some(pin) = self.pins.get_mut(pin as usize) {
				pin.value = level;
			} else {
				return Err(Error::PinOutOfBounds { pin, len: self.pins.len() })
			}

			for index in 0..8 {
				if let Some(pin) = self.pins.get(8 * port + index) {
					if pin.value != 0 {
						value |= 1 << index;
					}
				} else {
					break;
				}
			}

			self.write_to_connection(&[
				DIGITAL_MESSAGE | port as u8,
				value & SYSEX_REALTIME,
				(u16::from(level) >> 7) as u8 & SYSEX_REALTIME,
			])
		}
	}

mod poll;