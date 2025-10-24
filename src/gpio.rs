//! Basic GPIO calling

#![no_std]
#![no_main]

use embassy_rp::{
    Peripherals,
    gpio::{Level, Output},
};
use embassy_time::{Duration, Timer};
use {defmt_rtt as _, panic_probe as _};

async fn main() {
    let p: Peripherals = embassy_rp::init(Default::default());
    let mut led: Output<'_> = Output::new(p.PIN_2, Level::Low);
    let delay: Duration = Duration::from_millis(250);

    loop {
        led.set_high();
        Timer::after(delay).await;

        led.set_low();
        Timer::after(delay).await;
    }
}
