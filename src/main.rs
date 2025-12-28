//! A Universal entrypoint for using rp2350 microcontrollers.
//!
#![no_std]
#![no_main] // default 'main' call must be overwriten by embassy: #[embassy_executor::main] or by hal: #[hal::entry] global variable initialisation 
use embassy_executor::Spawner;
use embassy_rp as hal;
use embassy_rp::Peri;
use embassy_rp::gpio::AnyPin;
mod chase;
mod protocol;
mod serial_usb;
mod sys;

/// Entry point.
#[embassy_executor::main]
async fn main(spawner: Spawner) {
    // Prepare system
    sys::init();

    // Get peripherals
    let peripherals: embassy_rp::Peripherals = hal::init(Default::default());

    // Start USB communication
    let port: serial_usb::UsbSerialPort = serial_usb::init(&spawner, peripherals.USB);

    // Create parser to read comands
    let mut parser = protocol::Parser::new();

    // Prepare chase pins
    let pins: [Peri<'static, AnyPin>; 5] = [
        peripherals.PIN_0.into(),
        peripherals.PIN_1.into(),
        peripherals.PIN_2.into(),
        peripherals.PIN_3.into(),
        peripherals.PIN_4.into(),
    ];
    let mut chase: chase::Chase = chase::init(pins, 100);

    // Action
    loop {
        let data: heapless::Vec<u8, 64> = port.read().await;
        parser.push_bytes(&data);

        match parser.next_frame() {
            Ok(Some(frame)) => {
                match frame.cmd {
                    0x01 => {
                        // PING
                        let resp = protocol::build_ack::<64>(frame.addr, frame.cmd).unwrap();
                        port.write(&resp).await;
                    }
                    0x02 => {
                        // CHASE (setter)
                        let resp = protocol::build_ack::<64>(frame.addr, frame.cmd).unwrap();
                        port.write(&resp).await;
                        chase.run().await;
                    }
                    0x20 => {
                        // GET_DEVICE_ID (getter)
                        let id: [u8; 8] = [1, 2, 3, 4, 5, 6, 7, 8]; // replace with real ID bytes
                        let resp = protocol::build_data::<64>(frame.addr, frame.cmd, &id).unwrap();
                        port.write(&resp).await;
                    }
                    _ => {
                        let resp = protocol::build_err::<64>(
                            frame.addr, frame.cmd, 0x02, /* BAD_CMD */
                        )
                        .unwrap();
                        port.write(&resp).await;
                    }
                }
            }
            Ok(None) => break,  // need more bytes
            Err(_) => continue, // resync + keep scanning
        }
    }
}
// End of file
