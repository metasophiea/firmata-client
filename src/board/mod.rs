#![allow(clippy::cast_possible_truncation)]

use serialport::SerialPortBuilder;

use crate::ConnectionWrapper;
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
};

/// A Firmata board representation.
// definition
	#[derive(Debug)]
	pub struct Board {
		connection_wrapper: ConnectionWrapper,
		buffer: Vec<u8>,
		initial_messages_sent: bool,

		firmware_name: Option<String>,
		firmware_version: Option<String>,
		protocol_version: Option<String>,
		pins: Vec<Pin>,
		i2c_data: Vec<I2CReply>,
	}

// creation
	impl Board {
		#[must_use]
		pub fn new(serial_port_builder: SerialPortBuilder) -> Board {
			Board {
				connection_wrapper: ConnectionWrapper::new(serial_port_builder),
				buffer: vec![],
				initial_messages_sent: false,

				firmware_name: None,
				firmware_version: None,
				protocol_version: None,
				pins: vec![],
				i2c_data: vec![],
			}
		}
	}

// disconnect
	impl Board {
		pub fn disconnect(&mut self) -> Result<()> {
			self.connection_wrapper.disconnect()
		}
	}

// tools
	impl Board {
		/// Write on the internal connection.
		#[tracing::instrument(skip(self), err, ret, level = "DEBUG")]
		fn write_to_connection(&mut self, buf: &[u8]) -> Result<()> {
			self.connection_wrapper.write(buf.to_vec())?;
			Ok(())
		}
	}

// printing
	impl std::fmt::Display for Board  {
		fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
			write!(
				f,
				"Board {{ firmware={:?}, version={:?}, protocol={:?}, connection_wrapper={:?}}}",
				self.firmware_name, self.firmware_version, self.protocol_version, self.connection_wrapper
			)
		}
	}

// get
	impl Board {
		#[must_use]
		/// Check if the connection to the board has been successfully established
		pub fn is_ready(&self) -> bool {
			self.initial_messages_sent && !self.pins.is_empty() && self.connection_wrapper.is_active()
		}

		#[must_use]
		/// Get all pins that the board has access to.
		pub fn get_all_pins(&self) -> &Vec<Pin> {
			&self.pins
		}

		#[must_use]
		/// Get a pin from the board.
		pub fn get_pin(&self, pin:usize) -> Option<&Pin> {
			self.pins.get(pin)
		}

		#[must_use]
		/// Get the raw I2C replies that have been read from the board.
		pub fn get_i2c_data(&self) -> &Vec<I2CReply> {
			&self.i2c_data
		}

		#[must_use]
		/// Get the current Firmata protocol version.
		pub fn get_protocol_version(&self) -> Option<&String> {
			self.protocol_version.as_ref()
		}

		#[must_use]
		/// Get the firmware name.
		pub fn get_firmware_name(&self) -> Option<&String> {
			self.firmware_name.as_ref()
		}

		#[must_use]
		/// Get the firmware version.
		pub fn get_firmware_version(&self) -> Option<&String> {
			self.firmware_version.as_ref()
		}
	}

// set
	impl Board {
		/// Set the `mode` of the specified `pin`.
		#[tracing::instrument(skip(self), err, ret, level = "DEBUG")]
		pub fn set_pin_mode(&mut self, pin_index: u8, mode: u8) -> Result<()> {
			if let Some(pin) = self.pins.get_mut(pin_index as usize) {

				if !pin.modes.contains(&mode) {
					return Err(Error::InvalidPinMode { pin:pin_index, modes:pin.modes.clone() });
				}

				pin.mode = mode;
			} else {
				return Err(Error::PinOutOfBounds { pin:pin_index, len: self.pins.len(), source: "set_pin_mode".to_string() })
			}

			self.write_to_connection(&[SET_PIN_MODE, pin_index, mode])
		}
	}

// query
	impl Board {
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
	impl Board {
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
	impl Board {
		/// Query the board for current firmware and protocol information.
		#[tracing::instrument(skip(self), err, ret, level = "DEBUG")]
		pub fn report_firmware(&mut self) -> Result<()> {
			self.write_to_connection(&[START_SYSEX, REPORT_FIRMWARE, END_SYSEX])
		}

    	/// Set the analog reporting `state` of the specified `pin`.
		#[tracing::instrument(skip(self), err, ret, level = "DEBUG")]
		pub fn report_analog(&mut self, pin: u8, state: bool) -> Result<()> {
			// get pin
				let p = if let Some(pin) = self.pins.get_mut(pin as usize) {
					pin
				} else {
					return Err(Error::PinOutOfBounds { pin, len: self.pins.len(), source: "report_analog".to_string() })
				};

			// check this is an actual change
				if p.report_analog_active == state {
					return Ok(());
				}

			// update state
				p.report_analog_active = state;

			// get port
				let port = pin / 8;

			// check other pins in this port and generate OR'd state for port
				let mut new_state = false;
				for pin_index in 0..8 {
					if let Some(pin) = self.pins.get(pin_index) {
						new_state ^= pin.report_analog_active;
					}
				}

			// send message to board
				self.write_to_connection(&[REPORT_ANALOG | port, u8::from(new_state)])
		}

    	/// Set the digital reporting `state` of the specified `pin`.
		#[tracing::instrument(skip(self), err, ret, level = "DEBUG")]
		pub fn report_digital(&mut self, pin: u8, state: bool) -> Result<()> {
			// get pin
				let p = if let Some(pin) = self.pins.get_mut(pin as usize) {
					pin
				} else {
					return Err(Error::PinOutOfBounds { pin, len: self.pins.len(), source: "report_digital".to_string() })
				};

			// check this is an actual change
				if p.report_digital_active == state {
					return Ok(());
				}

			// update state
				p.report_digital_active = state;

			// get port
				let port = pin / 8;

			// check other pins in this port and generate OR'd state for port
				let mut new_state = false;
				for pin_index in 0..8 {
					if let Some(pin) = self.pins.get(port as usize + pin_index) {
						new_state |= pin.report_digital_active;
					}
				}

			// send message to board
				self.write_to_connection(&[REPORT_DIGITAL | port, u8::from(new_state)])
		}
	}

// write
	impl Board {
		/// Write `level` to the analog `pin`.
		#[tracing::instrument(skip(self), err, ret, level = "DEBUG")]
		pub fn analog_write(&mut self, pin: u8, level: u8) -> Result<()> {
			if let Some(pin) = self.pins.get_mut(pin as usize) {
				pin.value = level;
			} else {
				return Err(Error::PinOutOfBounds { pin, len: self.pins.len(), source: "analog_write".to_string() })
			}

			self.write_to_connection(&[
				ANALOG_MESSAGE | pin,
				level & SYSEX_REALTIME,
				(u16::from(level) >> 7) as u8 & SYSEX_REALTIME,
			])
		}

    	/// Write `level` to the digital `pin`.
		#[tracing::instrument(skip(self), err, ret, level = "DEBUG")]
		pub fn digital_write(&mut self, pin: u8, level: bool) -> Result<()> {
			let port = (pin/8) as usize;
			
			if let Some(pin) = self.pins.get_mut(pin as usize) {
				pin.value = u8::from(level);
			} else {
				return Err(Error::PinOutOfBounds { pin, len: self.pins.len(), source: "digital_write".to_string() })
			}

			let mut value = 0;
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
				(u16::from(value) >> 7) as u8 & SYSEX_REALTIME,
			])
		}
	}

mod poll;