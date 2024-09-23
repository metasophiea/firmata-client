use std::{thread, time::Duration};

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

    let mut b = firmata_client_rs::Board::new(Box::new(port)).expect("new board");

    b.set_pin_mode(13, firmata_client_rs::PIN_MODE_OUTPUT)
        .expect("pin mode set");

    let mut i = 0;

    loop {
        thread::sleep(Duration::from_millis(400));
        b.digital_write(13, i).expect("digital write");
        i ^= 1;
    }
}
