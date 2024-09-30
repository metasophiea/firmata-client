use std::{thread, time::Duration};

use serialport::*;

fn main() {
    tracing_subscriber::fmt::init();

	let serial_port_builder = serialport::new("/dev/tty.usbmodem14301", 57_600)
		.data_bits(DataBits::Eight)
		.parity(Parity::None)
		.stop_bits(StopBits::One)
		.flow_control(FlowControl::None);

    let mut board = firmata_client_rs::Board::new(serial_port_builder);
	while !board.is_ready() {
		board.poll().expect("successful polling");
		println!("waiting...");
        thread::sleep(Duration::from_millis(100));
	}
	println!("setup complete");

	let pin = 15;
    board.set_pin_mode(pin, firmata_client_rs::PIN_MODE_OUTPUT).expect("pin mode set");

    let mut state = false;

    loop {
        thread::sleep(Duration::from_millis(1000));
		println!(">> {state}");
        board.digital_write(pin, state).expect("digital write");
        state = !state;
    }
}
