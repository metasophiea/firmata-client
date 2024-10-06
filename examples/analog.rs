use std::{thread, time::Duration};

use serialport::*;

fn main() {
	let serial_port_builder = serialport::new("/dev/tty.usbmodem14301", 57_600)
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

    let led = 5;
    let pin = 14; // A0

    board.set_pin_mode(led, firmata_client::PIN_MODE_PWM).expect("pin mode set");
    board.set_pin_mode(pin, firmata_client::PIN_MODE_ANALOG).expect("pin mode set");
    board.report_analog(pin, true).expect("reporting state");

    loop {
        board.poll()
            .expect("successful polling")
            .iter()
            .filter_map(|message| message.try_as_analog())
            .flatten()
            .for_each(|(pin_index, value)|{
                println!("analog pin {pin_index} value: {value}");
                if pin_index == &pin {
                    board.analog_write(led, *value).expect("digital write");
                }
            });

        thread::sleep(Duration::from_millis(1));
    }
}
