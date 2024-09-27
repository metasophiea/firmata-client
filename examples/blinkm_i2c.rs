// use std::io::{Read, Write};
// use std::sync::{Arc, Mutex};
// use std::thread;
// use std::time::Duration;

// use serialport::*;

// use firmata_client_rs::Board;

// fn init<T: Read + Write + std::fmt::Debug>(board: &Arc<Mutex<Board>>) {
//     let mut b = board.lock().expect("lock");
//     b.i2c_config(0).expect("i2c delay set");
//     b.i2c_write(0x09, "o".as_bytes()).expect("i2c write");
//     thread::sleep(Duration::from_millis(10));
// }

// fn set_rgb<T: Read + Write + std::fmt::Debug>(board: &Arc<Mutex<Board>>, rgb: [u8; 3]) {
//     let mut b = board.lock().expect("lock");
//     b.i2c_write(0x09, "n".as_bytes()).expect("i2c write");
//     b.i2c_write(0x09, &rgb).expect("i2c write");
// }

// fn read_rgb<T: Read + Write + std::fmt::Debug>(board: &Arc<Mutex<Board>>) -> Vec<u8> {
// 	let mut b = board.lock().expect("lock");

// 	b.i2c_write(0x09, "g".as_bytes()).expect("i2c write");
// 	b.i2c_read(0x09, 3).expect("i2c read");

// 	loop {
// 		if let Some(i2c_reply) = b.get_i2c_data().last() {
// 			return i2c_reply.data.clone();
// 		}
//         thread::sleep(Duration::from_millis(10));
// 	}
// }

fn main() {
//     tracing_subscriber::fmt::init();

// 	let serial_port_builder = serialport::new("/dev/tty.usbmodem14201", 57_600)
// 		.data_bits(DataBits::Eight)
// 		.parity(Parity::None)
// 		.stop_bits(StopBits::One)
// 		.flow_control(FlowControl::None)
// 		.timeout(Duration::from_millis(10000));
//     let port = serial_port_builder.clone()
//         .open()
//         .expect("an opened serial port");

//     let board = Arc::new(Mutex::new(
//         firmata_client_rs::Board::new(serial_port_builder).expect("new board"),
//     ));

//     {
//         let b = board.clone();
//         thread::spawn(move || loop {
//             b.lock()
//                 .expect("lock")
//                 .poll()
//                 .expect("a message");
//             b.lock()
//                 .expect("lock")
//                 .report_firmware()
//                 .expect("firmware and protocol info");
//             thread::sleep(Duration::from_millis(10));
//         });
//     }

//     init(&board);

//     set_rgb(&board, [255, 0, 0]);
//     tracing::info!("rgb: {:?}", read_rgb(&board));
//     thread::sleep(Duration::from_millis(1000));

//     set_rgb(&board, [0, 255, 0]);
//     tracing::info!("rgb: {:?}", read_rgb(&board));
//     thread::sleep(Duration::from_millis(1000));

//     set_rgb(&board, [0, 0, 255]);
//     tracing::info!("rgb: {:?}", read_rgb(&board));
//     thread::sleep(Duration::from_millis(1000));
}