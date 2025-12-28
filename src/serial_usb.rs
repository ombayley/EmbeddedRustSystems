//! Transport Layer via USB Serial
//!
use embassy_executor::Spawner;
use embassy_futures::select::{Either, select};
use embassy_rp::Peri;
use embassy_rp::bind_interrupts;
use embassy_rp::peripherals::USB;
use embassy_rp::usb::{Driver, InterruptHandler};
use embassy_sync::blocking_mutex::raw::CriticalSectionRawMutex;
use embassy_sync::channel::Channel;
use embassy_usb::UsbDevice;
use embassy_usb::class::cdc_acm::{CdcAcmClass, State};
use heapless::Vec;
use static_cell::StaticCell;

// Interrupt handler
bind_interrupts!(struct Irqs {
    USBCTRL_IRQ => InterruptHandler<USB>;
});

// USB Device Types
type MyUsbDriver = Driver<'static, USB>;
type MyUsbDevice = UsbDevice<'static, MyUsbDriver>;

// Channels
static TX_TO_USB: Channel<CriticalSectionRawMutex, Vec<u8, 64>, 8> = Channel::new();
static RX_FROM_USB: Channel<CriticalSectionRawMutex, Vec<u8, 64>, 8> = Channel::new();

// API Struct
pub struct UsbSerialPort;

impl UsbSerialPort {
    /// Queue bytes to send to the host. Data is chunked into max 64-byte packets.
    pub async fn write(&self, data: &[u8]) {
        for chunk in data.chunks(64) {
            let mut v = Vec::<u8, 64>::new();
            let _ = v.extend_from_slice(chunk);
            TX_TO_USB.send(v).await;
        }
    }

    /// Receive next packet of bytes from the host (up to 64 bytes).
    pub async fn read(&self) -> Vec<u8, 64> {
        RX_FROM_USB.receive().await
    }
}

// USB Device Initialisation
pub fn init(spawner: &Spawner, usb_peripheral: Peri<'static, USB>) -> UsbSerialPort {
    // Create the driver, from the HAL.
    let driver = Driver::new(usb_peripheral, Irqs);

    // Create embassy-usb Config
    let config = {
        let mut config = embassy_usb::Config::new(0xc0de, 0xcafe);
        config.manufacturer = Some("Embassy");
        config.product = Some("USB-serial example");
        config.serial_number = Some("12345678");
        config.max_power = 100;
        config.max_packet_size_0 = 64;
        config
    };

    // Builder storage
    static CONFIG_DESCRIPTOR: StaticCell<[u8; 256]> = StaticCell::new();
    static BOS_DESCRIPTOR: StaticCell<[u8; 256]> = StaticCell::new();
    static CONTROL_BUF: StaticCell<[u8; 64]> = StaticCell::new();

    // Create embassy-usb DeviceBuilder using the driver and config.
    let mut builder = embassy_usb::Builder::new(
        driver,
        config,
        CONFIG_DESCRIPTOR.init([0; 256]),
        BOS_DESCRIPTOR.init([0; 256]),
        &mut [], // no msos descriptors
        CONTROL_BUF.init([0; 64]),
    );

    // CDC class storage
    static STATE: StaticCell<State> = StaticCell::new();
    let state = STATE.init(State::new());
    let class = CdcAcmClass::new(&mut builder, state, 64);

    // Build the builder.
    let usb = builder.build();

    // Spawn tasks: USB runner + CDC handler
    spawner.must_spawn(usb_task(usb));
    spawner.must_spawn(cdc_task(class));

    // Return API to user
    UsbSerialPort
}

#[embassy_executor::task]
async fn usb_task(mut usb: MyUsbDevice) -> ! {
    usb.run().await
}

#[embassy_executor::task]
async fn cdc_task(mut class: CdcAcmClass<'static, MyUsbDriver>) -> ! {
    let mut buf = [0u8; 64];

    loop {
        // Wait until host opens the port
        class.wait_connection().await;

        // While connected, service both RX and TX without blocking one on the other.
        loop {
            match select(class.read_packet(&mut buf), TX_TO_USB.receive()).await {
                Either::First(read_res) => match read_res {
                    Ok(n) => {
                        let mut v = Vec::<u8, 64>::new();
                        let _ = v.extend_from_slice(&buf[..n]);
                        RX_FROM_USB.send(v).await;
                    }
                    Err(_) => break, // disconnected
                },
                Either::Second(out) => {
                    // Best-effort send; if disconnected write_packet will error and we break.
                    if class.write_packet(&out).await.is_err() {
                        break;
                    }
                }
            }
        }
    }
}
