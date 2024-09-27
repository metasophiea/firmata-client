use std::{thread, time::Duration};

use serialport::*;

fn main() {
	let serial_port_builder = serialport::new("/dev/tty.usbmodem14201", 57_600)
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

    let pin = 14; // A0

    board.set_pin_mode(pin, firmata_client_rs::PIN_MODE_ANALOG).expect("pin mode set");
    board.report_analog(pin, 1).expect("reporting state");

    loop {
        board.poll().expect("successful polling");
        println!("analog value: {}", board.get_pin(pin as usize).expect("pin").value);
        thread::sleep(Duration::from_millis(10));
    }
}
