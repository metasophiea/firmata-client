mod constants;
mod types;

pub use types::Error;

pub use constants::{
	PIN_MODE_ANALOG,
	PIN_MODE_ENCODER,
	PIN_MODE_I2C,
	PIN_MODE_IGNORE,
	PIN_MODE_INPUT,
	PIN_MODE_ONEWIRE,
	PIN_MODE_OUTPUT,
	PIN_MODE_PULLUP,
	PIN_MODE_PWM,
	PIN_MODE_SERIAL,
	PIN_MODE_SERVO,
	PIN_MODE_SHIFT,
	PIN_MODE_STEPPER
};

mod connection_wrapper;
use connection_wrapper::ConnectionWrapper;

mod board;
pub use board::Board;