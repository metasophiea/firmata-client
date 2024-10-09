use std::{thread, time::Duration};

use serialport::*;

fn main() {
    tracing_subscriber::fmt::init();

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

    let led = 13;
    let button = 2;
	
    board.report_digital(button, true).expect("digital reporting mode");
    board.set_pin_mode(led, firmata_client::PIN_MODE_OUTPUT).expect("pin mode set");
    board.set_pin_mode(button, firmata_client::PIN_MODE_INPUT | firmata_client::PIN_MODE_PULLUP).expect("pin mode set");

    println!("Starting loop...");

    loop {
        board.poll()
            .expect("successful polling")
            .iter()
            .filter_map(|message| message.try_as_digital())
            .flatten()
            .for_each(|(pin_index, value)|{
                if pin_index == &button {
                    if *value {
                        println!("on");
                        board.digital_write(led, true).expect("digital write");
                    } else {
                        println!("off");
                        board.digital_write(led, false).expect("digital write");
                    }
                }
            });

        thread::sleep(Duration::from_millis(1));
    }
}