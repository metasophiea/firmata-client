#![allow(clippy::cast_possible_truncation)]

use crate::constants::{
    ANALOG_MAPPING_RESPONSE,
    ANALOG_MESSAGE,
    ANALOG_MESSAGE_BOUND,
    CAPABILITY_RESPONSE,
    DIGITAL_MESSAGE,
    DIGITAL_MESSAGE_BOUND,
    END_SYSEX,
    I2C_REPLY,
	PIN_MODE_IGNORE,
    PIN_MODE_INPUT,
	PIN_MODE_PULLUP,
    PIN_STATE_RESPONSE,
	REPORT_DIGITAL,
    REPORT_FIRMWARE,
    REPORT_VERSION,
    START_SYSEX
};
use crate::types::{
    Error,
    I2CReply,
    Message,
    Pin,
    Result,
};

use super::Board;

// read incoming messages
impl Board {
	#[tracing::instrument(skip(self), err, ret, level = "DEBUG")]
	pub fn poll(&mut self) -> Result<Vec<Message>> {
		if self.firmware_name.is_some() && self.firmware_version.is_some() && !self.initial_messages_sent {
			self.initial_messages_sent = true;
			tracing::debug!("sending initial messages");
			self.query_capabilities()?;
			self.query_analog_mapping()?;
			self.write_to_connection(&[REPORT_DIGITAL, 1])?;
		}

		self.buffer.append(&mut self.connection_wrapper.poll()?);
		tracing::debug!("self.buffer: {:?}", self.buffer);

		let mut messages = vec![];
		while !self.buffer.is_empty() {
			let Some(byte_0) = self.buffer.get(0) else { break; };

			match *byte_0 {
				REPORT_VERSION => {
					tracing::debug!("REPORT_VERSION");
	
					let Some(byte_1) = self.buffer.get(1) else { break; };
					let Some(byte_2) = self.buffer.get(2) else { break; };
	
					self.protocol_version = Some(format!("{byte_1:o}.{byte_2:o}"));
					tracing::debug!("self.protocol_version: {}", format!("{byte_1:o}.{byte_2:o}"));
					messages.push(Message::ProtocolVersion(*byte_1, *byte_2));

					self.buffer.drain(0..=2);
				},
				ANALOG_MESSAGE..=ANALOG_MESSAGE_BOUND => {
					tracing::debug!("ANALOG_MESSAGE");
	
					let Some(byte_1) = self.buffer.get(1) else { break; };
					let Some(byte_2) = self.buffer.get(2) else { break; };
	
					// extract pin info
						let pin_index = (byte_0 & 0x0F) + 14;
						let value = byte_1 | (byte_2 << 7);
	
					// channel info into local data
						let mut pin_updates:Vec<(u8, u8)> = vec![];
						if let Some(pin) = self.pins.get_mut(pin_index as usize) {
							if pin.value != value {
								pin_updates.push((pin_index, value));
							}
							pin.value = value;
						}
	
					if !pin_updates.is_empty() {
						messages.push(Message::Analog(pin_updates));
					}

					self.buffer.drain(0..=2);
				},
				DIGITAL_MESSAGE..=DIGITAL_MESSAGE_BOUND => {
					tracing::debug!("DIGITAL_MESSAGE");
	
					let Some(byte_1) = self.buffer.get(1) else { tracing::debug!("byte_1 missing"); break; };
					let Some(byte_2) = self.buffer.get(2) else { tracing::debug!("byte_2 missing"); break; };
	
					// extract pin info
						let port = byte_0 & 0x0F;
						let value = byte_1 | (byte_2 << 7);
						tracing::debug!("port: {port} value: {value}");
	
					// channel info into local data
						let mut pin_updates:Vec<(u8, bool)> = vec![];

						for index in 0..8u8 {
							let pin_index = (8 * port) + index;
							tracing::debug!("pin_index: {pin_index}");
							
							if let Some(pin) = self.pins.get_mut(pin_index as usize) {
								tracing::debug!("pin.mode: {}", pin.mode);
								if pin.mode == PIN_MODE_INPUT || pin.mode == PIN_MODE_PULLUP {
									let new_value = (value >> (index & 0x07)) & 0x01;
									tracing::debug!("new_value: {new_value} pin.value: {}", pin.value);
									if new_value != pin.value {
										pin_updates.push((pin_index, new_value != 0));
									}
									pin.value = new_value;
								}
							}
						}
					
						if !pin_updates.is_empty() {
							messages.push(Message::Digital(pin_updates));
						}
						
						self.buffer.drain(0..=2);
				}
				START_SYSEX => {
					tracing::debug!("START_SYSEX");

					let Some(end_index) = self.buffer.iter().position(|byte| byte == &END_SYSEX) else { break; };
					let sysex_buffer = &self.buffer[0..=end_index];
					tracing::debug!("{sysex_buffer:?}");

					let Some(byte_1) = sysex_buffer.get(1) else { break; };

					match *byte_1 {
						END_SYSEX => {
							tracing::debug!("END_SYSEX");
							messages.push(Message::EmptyResponse);
						},
						ANALOG_MAPPING_RESPONSE => {
							tracing::debug!("ANALOG_MAPPING_RESPONSE");

							for index in 2..sysex_buffer.len() {
								if sysex_buffer[index] == PIN_MODE_IGNORE {
									continue;
								}

								if sysex_buffer[index] == END_SYSEX {
									break;
								}
								
								tracing::debug!("index: {index}, sysex_buffer[index]: {}, pin_index: {}", sysex_buffer[index], index-2);

								if let Some(pin) = self.pins.get_mut(index - 2) {
									pin.analog = true;
								}
							}

							messages.push(Message::AnalogMappingResponse);
						},
						CAPABILITY_RESPONSE => {
							tracing::debug!("CAPABILITY_RESPONSE");

							let mut index = 4;

							self.pins = vec![];
							self.pins.push(Pin::default_with_report_digital_active()); // 0 is unused.
							self.pins.push(Pin::default_with_report_digital_active()); // 1 is unused.

							let mut modes = vec![];
							let mut resolution = None;

							while index < sysex_buffer.len() - 1 {
								// Completed a pin, push and continue.
								if sysex_buffer[index] == 127u8 {
									self.pins.push(Pin {
										analog: false,
										mode: *modes.first().expect("pin mode"),
										modes: std::mem::take(&mut modes),
										report_analog_active: false,
										report_digital_active: false,
										resolution: resolution.take().expect("pin resolution"),
										value: 0,
									});
									tracing::debug!("pin: {} {:?}", self.pins.len()-1, self.pins[self.pins.len()-1]);

									index += 1;
								} else {
									modes.push(sysex_buffer[index]);
									if resolution.is_none() {
										// Only keep the first.
										resolution.replace(sysex_buffer[index + 1]);
									}
									index += 2;
								}
							}

							messages.push(Message::CapabilityResponse);
						},
						REPORT_FIRMWARE => {
							tracing::debug!("REPORT_FIRMWARE");

							let Some(major) = sysex_buffer.get(2) else { break; };
							let Some(minor) = sysex_buffer.get(3) else { break; };
							tracing::debug!("major: {major} minor: {minor}");

							let firmware_version = format!("{major:o}.{minor:o}");
							messages.push(Message::ReportFirmwareVersion(firmware_version.clone()));
							self.firmware_version = Some(firmware_version);

							if sysex_buffer.len() - 1 > 4 {
								let name = std::str::from_utf8(&sysex_buffer[4..sysex_buffer.len() - 1])?.to_string();
								tracing::debug!("firmware_name: {name}");
								messages.push(Message::ReportFirmwareName(name.clone()));
								self.firmware_name = Some(name);
							}
						},
						I2C_REPLY => {
							tracing::debug!("I2C_REPLY");

							let Some(byte_2) = sysex_buffer.get(2) else { break; };
							let Some(byte_3) = sysex_buffer.get(3) else { break; };
							let Some(byte_4) = sysex_buffer.get(4) else { break; };
							let Some(byte_5) = sysex_buffer.get(5) else { break; };
							let Some(byte_6) = sysex_buffer.get(6) else { break; };
							let Some(byte_7) = sysex_buffer.get(7) else { break; };

							let mut reply = I2CReply {
								address: byte_2 | (u16::from(*byte_3) << 7) as u8,
								register: byte_4 | (u16::from(*byte_5) << 7) as u8,
								data: vec![byte_6 | (u16::from(*byte_7) << 7) as u8],
							};

							let mut index = 8;

							while index < sysex_buffer.len() - 2 {
								if sysex_buffer[index] == 0xF7 {
									break;
								}

								if index + 2 > sysex_buffer.len() {
									break;
								}

								reply.data.push(sysex_buffer[index] | sysex_buffer[index + 1] << 7);

								index += 2;
							}

							self.i2c_data.push(reply);

							messages.push(Message::I2CReply);
						},
						PIN_STATE_RESPONSE => {
							tracing::debug!("PIN_STATE_RESPONSE");

							let Some(byte_2) = sysex_buffer.get(2) else { break; };
							let Some(byte_3) = sysex_buffer.get(3) else { break; };

							if byte_3 == &END_SYSEX {
								messages.push(Message::PinStateResponse);
								continue;
							}

							let pin = *byte_2;
							let Some(pin) = self.pins.get_mut(pin as usize) else {
								return Err(Error::PinOutOfBounds { pin, len: self.pins.len(), source: "poll : PIN_STATE_RESPONSE".to_string() })
							};

							pin.mode = *byte_3;

							// TODO: Extended values.
							let Some(byte_4) = sysex_buffer.get(4) else { break; };
							pin.value = *byte_4;

							messages.push(Message::PinStateResponse);
						},
						_ => {
							tracing::debug!("UnknownSysEx");
							return Err(Error::UnknownSysEx { code: *byte_1 });
						},
					}

					self.buffer.drain(0..sysex_buffer.len());
				},
				_ => { return Err(Error::BadByte( self.buffer.remove(0))); },
			}
		}

		Ok(messages)
	}
}