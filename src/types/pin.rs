use crate::constants::{
    DEFAULT_ANALOG_RESOLUTION,
    PIN_MODE_ANALOG
};

/// The current state and configuration of a pin.
#[derive(Debug)]
pub struct Pin {
	/// Whether this is an analog pin or not.
	pub analog: bool,
    /// Currently configured mode.
    pub mode: u8,
    /// All pin modes.
    pub modes: Vec<u8>,
	/// The report analog state.
	pub report_analog_active: bool,
	/// The report digital state.
	pub report_digital_active: bool,
    /// Current resolution.
    pub resolution: u8,
    /// Pin value.
    pub value: u8,
}

impl Pin {
    pub fn default_with_report_digital_active() -> Self {
        Self {
			analog: false,
            mode: PIN_MODE_ANALOG,
            modes: vec![PIN_MODE_ANALOG],
			report_analog_active: false,
			report_digital_active: true,
            resolution: DEFAULT_ANALOG_RESOLUTION,
            value: 0,
        }
    }
}