#![allow(clippy::cast_possible_truncation)]

use std::io::{Read, Write};

use snafu::prelude::*;

use crate::constants::{
    ANALOG_MAPPING_RESPONSE,
    ANALOG_MESSAGE,
    ANALOG_MESSAGE_BOUND,
    CAPABILITY_RESPONSE,
    DEFAULT_ANALOG_RESOLUTION,
    DIGITAL_MESSAGE,
    DIGITAL_MESSAGE_BOUND,
    END_SYSEX,
    I2C_REPLY,
    PIN_MODE_ANALOG,
    PIN_MODE_INPUT,
    PIN_STATE_RESPONSE,
    REPORT_FIRMWARE,
    REPORT_VERSION,
    START_SYSEX
};
use crate::types::{
    Error,
    I2CReply,
    Message,
    MessageTooShortSnafu,
    Pin,
    Result,
    StdIoSnafu,
    Utf8Snafu,
};

use super::Board;

// read incoming messages
impl<T: Read + Write + std::fmt::Debug> Board<T> {
	/// Read from the Firmata device, parse one Firmata message and return its type.
	#[tracing::instrument(skip(self), err, ret, level = "DEBUG")]
	pub fn poll(&mut self) -> Result<Message> {
		let mut buf = vec![0; 3];

		self.connection
			.read_exact(&mut buf)
			.with_context(|_| StdIoSnafu)?;

		tracing::info!("buf: {buf:?}");

		let Some(byte_0) = buf.get(0) else {
			return Err(Error::EmptyBufferMessage);
		};

		match *byte_0 {
			REPORT_VERSION => {
				tracing::info!("REPORT_VERSION");

				let Some(byte_1) = buf.get(1) else { return Err(Error::MessageTooShort); };
				let Some(byte_2) = buf.get(2) else { return Err(Error::MessageTooShort); };

				self.protocol_version = format!("{byte_1:o}.{byte_2:o}");
				Ok(Message::ProtocolVersion)
			}
			ANALOG_MESSAGE..=ANALOG_MESSAGE_BOUND => {
				tracing::info!("ANALOG_MESSAGE");

				let Some(byte_1) = buf.get(1) else { return Err(Error::MessageTooShort); };
				let Some(byte_2) = buf.get(2) else { return Err(Error::MessageTooShort); };

				// extract pin info
					let pin = (byte_0 & 0x0F) + 14;
					let value = byte_1 | (byte_2 << 7);

				// channel info into local data
					if let Some(pin) = self.pins.get_mut(pin as usize) {
						pin.value = value;
					} else {
						return Err(Error::PinOutOfBounds { pin, len: self.pins.len() })
					}

				Ok(Message::Analog)
			}
			DIGITAL_MESSAGE..=DIGITAL_MESSAGE_BOUND => {
				tracing::info!("DIGITAL_MESSAGE");

				let Some(byte_1) = buf.get(1) else { return Err(Error::MessageTooShort); };
				let Some(byte_2) = buf.get(2) else { return Err(Error::MessageTooShort); };

				// extract pin info
					let port = byte_0 & 0x0F;
					let value = byte_1 | (byte_2 << 7);
					tracing::info!("port: {port} value: {value}");

				// channel info into local data
					for i in 0..8 {
						let pin_index = (8 * (port as usize)) + i;
						let pins_length = self.pins.len();
						
						if let Some(pin) = self.pins.get_mut(pin_index) {
							if pins_length > pin_index && pin.mode == PIN_MODE_INPUT {
								pin.value = (value >> (i & 0x07)) & 0x01;
							}
						} else {
							break;
						}
					}
				
				Ok(Message::Digital)
			}
			START_SYSEX => {
				tracing::info!("START_SYSEX");

				let mut byte = [0];
				while byte[0] != END_SYSEX {
					self.connection
						.read_exact(&mut byte)
						.with_context(|_| StdIoSnafu)?;

					buf.push(byte[0]);
				}

				tracing::info!("buf: {buf:?}");

				let Some(byte_1) = buf.get(1) else { return Err(Error::MessageTooShort); };

				match *byte_1 {
					END_SYSEX => {
						tracing::info!("END_SYSEX");
						Ok(Message::EmptyResponse)
					},
					ANALOG_MAPPING_RESPONSE => {
						tracing::info!("ANALOG_MAPPING_RESPONSE");

						let mut i = 2;

						// Also break before pins indexing is out of bounds.
						let upper = (buf.len() - 1).min(self.pins.len() + 2);

						while i < upper {
							if buf.get(i) != Some(&127u8) {
								let pin = &mut self.pins[i - 2];
								pin.mode = PIN_MODE_ANALOG;
								pin.modes = vec![PIN_MODE_ANALOG];
								pin.resolution = DEFAULT_ANALOG_RESOLUTION;
							}
							i += 1;
						}

						Ok(Message::AnalogMappingResponse)
					}
					CAPABILITY_RESPONSE => {
						tracing::info!("CAPABILITY_RESPONSE");

						let mut index = 4;

						self.pins = vec![];
						self.pins.push(Pin::default()); // 0 is unused.
						self.pins.push(Pin::default()); // 1 is unused.

						let mut modes = vec![];
						let mut resolution = None;

						while index < buf.len() - 2 {
							// Completed a pin, push and continue.
							if buf[index] == 127u8 {
								self.pins.push(Pin {
									mode: *modes.first().expect("pin mode"),
									modes: std::mem::take(&mut modes),
									resolution: resolution.take().expect("pin resolution"),
									value: 0,
								});

								index += 1;
							} else {
								modes.push(buf[index]);
								if resolution.is_none() {
									// Only keep the first.
									resolution.replace(buf[index + 1]);
								}
								index += 2;
							}
						}

						Ok(Message::CapabilityResponse)
					}
					REPORT_FIRMWARE => {
						tracing::info!("REPORT_FIRMWARE");

						let major = buf.get(2).with_context(|| MessageTooShortSnafu)?;
						let minor = buf.get(3).with_context(|| MessageTooShortSnafu)?;

						self.firmware_version = format!("{major:o}.{minor:o}");

						if buf.len() - 1 > 4 {
							self.firmware_name = std::str::from_utf8(&buf[4..buf.len() - 1])
								.with_context(|_| Utf8Snafu)?
								.to_string();
						}

						Ok(Message::ReportFirmware)
					}
					I2C_REPLY => {
						tracing::info!("I2C_REPLY");

						let Some(byte_2) = buf.get(2) else { return Err(Error::MessageTooShort); };
						let Some(byte_3) = buf.get(3) else { return Err(Error::MessageTooShort); };
						let Some(byte_4) = buf.get(4) else { return Err(Error::MessageTooShort); };
						let Some(byte_5) = buf.get(5) else { return Err(Error::MessageTooShort); };
						let Some(byte_6) = buf.get(6) else { return Err(Error::MessageTooShort); };
						let Some(byte_7) = buf.get(7) else { return Err(Error::MessageTooShort); };

						let mut reply = I2CReply {
							address: byte_2 | (u16::from(*byte_3) << 7) as u8,
							register: byte_4 | (u16::from(*byte_5) << 7) as u8,
							data: vec![byte_6 | (u16::from(*byte_7) << 7) as u8],
						};

						let mut index = 8;

						while index < buf.len() - 2 {
							if buf[index] == 0xF7 {
								break;
							}

							if index + 2 > buf.len() {
								break;
							}

							reply.data.push(buf[index] | buf[index + 1] << 7);

							index += 2;
						}

						self.i2c_data.push(reply);

						Ok(Message::I2CReply)
					}
					PIN_STATE_RESPONSE => {
						tracing::info!("PIN_STATE_RESPONSE");

						let Some(byte_2) = buf.get(2) else { return Err(Error::MessageTooShort); };
						let Some(byte_3) = buf.get(3) else { return Err(Error::MessageTooShort); };

						if byte_3 == &END_SYSEX {
							return Ok(Message::PinStateResponse);
						}

						let pin = *byte_2;
						let Some(pin) = self.pins.get_mut(pin as usize) else {
						    return Err(Error::PinOutOfBounds { pin, len: self.pins.len() })
						};

						pin.modes = vec![*byte_3];

						// TODO: Extended values.
						let Some(byte_4) = buf.get(4) else { return Err(Error::MessageTooShort); };
						pin.value = *byte_4;

						Ok(Message::PinStateResponse)
					}
					_ => {
						tracing::info!("UnknownSysEx");

						let Some(byte_1) = buf.get(1) else { return Err(Error::MessageTooShort); };

						Err(Error::UnknownSysEx { code: *byte_1 })
					},
				}
			}
			_ => Err(Error::BadByte { byte: *byte_0 }),
		}
	}
}