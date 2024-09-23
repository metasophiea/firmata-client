use std::time::Duration;

use serialport::*;

fn main() {
    tracing_subscriber::fmt::init();

    let port = serialport::new("/dev/tty.usbmodem14201", 57_600)
        .data_bits(DataBits::Eight)
        .parity(Parity::None)
        .stop_bits(StopBits::One)
        .flow_control(FlowControl::None)
        .timeout(Duration::from_millis(10000))
        .open()
        .expect("an opened serial port");

    let mut board = firmata_client_rs::Board::new(Box::new(port)).expect("new board");

    let led = 13;
    let button = 2;
	
    board.report_digital(button, 1).expect("digital reporting mode");
    board.set_pin_mode(led, firmata_client_rs::PIN_MODE_OUTPUT).expect("pin mode set");
    board.set_pin_mode(button, firmata_client_rs::PIN_MODE_INPUT | firmata_client_rs::PIN_MODE_PULLUP).expect("pin mode set");

    tracing::info!("Starting loop...");

    loop {
        board.poll().expect("a message");
        if board.get_pins()[button as usize].value == 0 {
            tracing::info!("off");
        } else {
            tracing::info!("on");
        }
    }
}