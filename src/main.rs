#![no_std]
#![no_main]

// --- Required runtime stuff for embedded, logging, and panics ---
use defmt::*;
use embassy_executor::Spawner;
use embassy_time::{Duration, Timer};
use {defmt_rtt as _, panic_probe as _};

// --- RP235x (Pico 2) HAL + PIO-SPI driver to talk to the CYW43 chip ---
use cyw43_pio::{PioSpi, RM2_CLOCK_DIVIDER};
use embassy_rp::bind_interrupts;
use embassy_rp::gpio::{Level, Output};
use embassy_rp::peripherals::{DMA_CH0, PIO0};
use embassy_rp::pio::{InterruptHandler, Pio};

// --- WiFi/LED driver state + firmware blobs ---
use static_cell::StaticCell;

bind_interrupts!(struct Irqs {
    PIO0_IRQ_0 => InterruptHandler<PIO0>;
});

// This async task is required by the cyw43 driver (it handles the chip in the background)
#[embassy_executor::task]
async fn cyw43_task(
    runner: cyw43::Runner<'static, Output<'static>, PioSpi<'static, PIO0, 0, DMA_CH0>>,
) -> ! {
    runner.run().await
}

#[embassy_executor::main]
async fn main(spawner: Spawner) {
    // 1) Set up the chip/board peripherals.
    let p = embassy_rp::init(Default::default());

    // 2) Get firmware bytes (provided by the `cyw43_firmware` crate).
    let fw: &[u8] = cyw43_firmware::CYW43_43439A0;
    let clm: &[u8] = cyw43_firmware::CYW43_43439A0_CLM;

    // 3) Set up the pins the CYW43 chip needs:
    //    - pwr: power control pin to the WiFi module
    //    - cs:  chip select for SPI
    let pwr = Output::new(p.PIN_23, Level::Low);
    let cs = Output::new(p.PIN_25, Level::High);

    // 4) Create a PIO instance and then a SPI interface on top of it for the CYW43.
    let mut pio = Pio::new(p.PIO0, Irqs);
    let spi = PioSpi::new(
        &mut pio.common,
        pio.sm0,
        RM2_CLOCK_DIVIDER, // safe clock speed for this board/chip
        pio.irq0,
        cs,
        p.PIN_24, // MOSI
        p.PIN_29, // SCK
        p.DMA_CH0,
    );

    // 5) Make a global driver state, create the CYW43 driver, and spawn its background task.
    static STATE: StaticCell<cyw43::State> = StaticCell::new();
    let state = STATE.init(cyw43::State::new());
    let (_net_device, mut control, runner) = cyw43::new(state, pwr, spi, fw).await;
    unwrap!(spawner.spawn(cyw43_task(runner)));

    // 6) Send the regulatory database to the chip (CLM), and choose a power mode.
    control.init(clm).await;
    control
        .set_power_management(cyw43::PowerManagementMode::PowerSave)
        .await;

    // 7) Blink the LED that lives **inside** the CYW43 chip (GPIO 0).
    let delay = Duration::from_millis(250);
    loop {
        info!("led on");
        control.gpio_set(0, true).await; // turn on LED
        Timer::after(delay).await;

        info!("led off");
        control.gpio_set(0, false).await; // turn off LED
        Timer::after(delay).await;
    }
}
