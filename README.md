# Firmata client library in Rust

Control your [Firmata](https://github.com/firmata/protocol) devices from Rust

The library comes with a Board struct, which you can initialize with any object that implements `std:io::{Read, Write}` and `Debug` for formatting purposes. This avoids being locked in to a specific interface library. I highly recommend `serialport` for your USB connections (used in examples), but feel free to use `serial` or any other.

The crate has been set up to utilize `tracing`, which helps in finding where your messages went. If you set the environment variable `CARGO_LOG=DEBUG` you can capture the most noise.

## Acknowledgements

This library is very based on earlier work by Tiemen Schuijbroek which can be found at https://gitlab.com/Tiemen/firmata-rs, which itself was largely based on even earlier work by Adrian Zankich which can be found at https://github.com/zankich/rust-firmata. To both should go many thanks!
