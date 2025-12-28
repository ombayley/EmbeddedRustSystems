use embassy_rp::Peri;
use embassy_rp::gpio::{AnyPin, Level, Output};
use embassy_time::Timer;

pub struct Chase {
    pins: [Output<'static>; 5],
    delay_ms: u64,
}

pub fn init(pins: [Peri<'static, AnyPin>; 5], delay_ms: u64) -> Chase {
    let pins = pins.map(|p| Output::new(p, Level::Low));
    Chase { pins, delay_ms }
}

impl Chase {
    pub async fn run(&mut self) {
        for pin in self.pins.iter_mut() {
            pin.set_high();
            Timer::after_millis(self.delay_ms).await;
            pin.set_low();
            Timer::after_millis(self.delay_ms).await;
        }
    }
}
