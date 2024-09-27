use std::{thread, time::Duration};

use serialport::*;

fn main() {
    tracing_subscriber::fmt::init();

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

    board.set_pin_mode(13, firmata_client_rs::PIN_MODE_OUTPUT).expect("pin mode set");

    let mut i = 0;

    loop {
        thread::sleep(Duration::from_millis(400));
		println!(">> {i}");
        board.digital_write(13, i).expect("digital write");
        i ^= 1;
    }
}
