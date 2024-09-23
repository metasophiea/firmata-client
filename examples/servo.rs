use std::{thread, time::Duration};

use serialport::*;

fn main() {
    tracing_subscriber::fmt::init();

    let port = serialport::new("/dev/ttyACM0", 57_600)
        .data_bits(DataBits::Eight)
        .parity(Parity::None)
        .stop_bits(StopBits::One)
        .flow_control(FlowControl::None)
        .timeout(Duration::from_millis(1000))
        .open()
        .expect("an opened serial port");

    let mut b = firmata_client_rs::Board::new(Box::new(port)).expect("new board");

    let pin = 3;

    b.set_pin_mode(pin, firmata_client_rs::PIN_MODE_SERVO)
        .expect("pin mode set");

    tracing::info!("Starting loop...");

    loop {
        for value in 0..180 {
            b.analog_write(pin, value).expect("analog write");
            tracing::info!("{}", value);
            thread::sleep(Duration::from_millis(10));
        }
    }
}
