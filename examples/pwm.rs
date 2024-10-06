use std::{thread, time::Duration};

use serialport::*;

fn main() {
	let serial_port_builder = serialport::new("/dev/tty.usbmodem14201", 57_600)
		.data_bits(DataBits::Eight)
		.parity(Parity::None)
		.stop_bits(StopBits::One)
		.flow_control(FlowControl::None);

    let mut board = firmata_client::Board::new(serial_port_builder);

	while !board.is_ready() {
		board.poll().expect("successful polling");
		println!("waiting...");
        thread::sleep(Duration::from_millis(100));
	}
	println!("setup complete");

    let pin = 5;

    board.set_pin_mode(pin, firmata_client::PIN_MODE_PWM).expect("pin set");
    board.analog_write(pin, 0).expect("pin write");

    println!("Starting loop...");

    loop {
        for value in (0..255).step_by(5) {
            board.analog_write(pin, value).expect("pin write");
            println!(">> {}", value);
            thread::sleep(Duration::from_millis(100));
        }
    }
}
