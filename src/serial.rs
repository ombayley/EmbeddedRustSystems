#![no_std]
#![no_main]

use embassy_executor::Spawner;
use embassy_rp::peripherals::UART0;
use embassy_rp::uart::{Config, DataBits, Parity, StopBits, Uart};
use embassy_time::Duration;
use {defmt_rtt as _, panic_probe as _};

#[embassy_executor::main]
async fn main(_spawner: Spawner) {
    let p = embassy_rp::init(Default::default());

    let mut cfg = Config::default();
    cfg.baudrate = 115_200;
    cfg.data_bits = DataBits::Eight;
    cfg.parity = Parity::None;
    cfg.stop_bits = StopBits::One;

    // Pick pins that match your board wiring
    let mut uart = Uart::new_blocking(p.UART0, p.PIN_0, p.PIN_1, cfg);

    let mut buf = [0u8; 128];

    loop {
        // Reads up to buf.len(); returns when at least 1 byte arrives.
        // For DMA/IRQ-driven, use `Uart::new` (async) and `.read(&mut buf).await`.
        match uart.read(&mut buf) {
            Ok(n) if n > 0 => {
                // process bytes [..n] — e.g., echo back:
                let _ = uart.write(&buf[..n]);
            }
            Ok(_) => { /* no data, continue */ }
            Err(_) => { /* handle/ignore errors as needed */ }
        }
    }
}
