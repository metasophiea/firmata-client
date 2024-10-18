# Firmata client library in Rust

Control your [Firmata](https://github.com/firmata/protocol) devices from Rust

The library comes with a Board struct, which you can initialize with a SerialPortBuilder object. The actual serial port is held in a separate thread, thus removing the blocking nature of reading and writing to a serial port.

The crate has been set up to utilize `tracing`, which helps in seeing the signals flowing to and from the arduino. If you set the environment variable `RUST_LOG=DEBUG` you can capture the most noise.

## Acknowledgements

This library is very based on earlier work by Tiemen Schuijbroek which can be found at https://gitlab.com/Tiemen/firmata-rs, which itself was largely based on even earlier work by Adrian Zankich which can be found at https://github.com/zankich/rust-firmata. To both should go many thanks!
