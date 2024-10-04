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
    /// Current resolution.
    pub resolution: u8,
    /// All pin modes.
    pub modes: Vec<u8>,
    /// Pin value.
    pub value: u8,
}

impl Default for Pin {
    fn default() -> Self {
        Self {
			analog: false,
            mode: PIN_MODE_ANALOG,
            modes: vec![PIN_MODE_ANALOG],
            resolution: DEFAULT_ANALOG_RESOLUTION,
            value: 0,
        }
    }
}
