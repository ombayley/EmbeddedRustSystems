# 📘 Bare Metal Raspberry Pi Pico 2W

> Bare-metal (no_std) Rust on a Raspberry Pi Pico 2W microcontroller with USB Serial Communication

---

## 🧩 Overview

This project is a bare-metal Rust implementation for the **Raspberry Pi Pico 2W** (RP2350 processor) featuring:
- **USB Serial Communication**: Custom Modbus-inspired framing protocol for reliable device communication
- **Async/Await Programming**: Using Embassy async runtime for efficient embedded systems
- **GPIO Control**: LED chase pattern demonstration with configurable timing
- **Zero Standard Library**: Complete `no_std` implementation optimized for embedded constraints

The project showcases advanced embedded Rust concepts including async executors, USB device communication, and hardware abstraction layers while maintaining memory safety without runtime overhead.

---

## 🎯 Objectives / Learning Goals

- 🔹 **Bare-metal Programming** — Master `no_std` Rust and embedded systems fundamentals
- 🔹 **Async Embedded Development** — Learn Embassy framework for async/await in embedded contexts
- 🔹 **Hardware Abstraction** — Understand GPIO, USB, and peripheral control using `embassy-rp` HAL
- 🔹 **Communication Protocols** — Implement custom framing protocol with CRC-16 error checking
- 🔹 **Memory Safety** — Leverage Rust's ownership system in resource-constrained environments

---

## ⚙️ Tech Stack

| Category    | Tools / Technologies                                   |
|-------------|--------------------------------------------------------|
| Language    | `Rust` (Edition 2024, `no_std`)                        |
| Frameworks  | `embassy` (async executor), `embassy-rp` (RP2350 HAL) |
| Tools       | `probe-rs`, `cargo-embed`, `defmt` (logging)           |
| Hardware    | Raspberry Pi Pico 2W (RP2350A), USB Serial             |
| Protocol    | Custom Modbus-inspired framing with CRC-16/Modbus      |

---

## 🏗️ Project Architecture

### Core Components

```
src/
├── main.rs         # Entry point with command loop and USB handling
├── protocol.rs     # Frame parser and builder (Modbus-inspired protocol)
├── serial_usb.rs   # USB Serial communication layer
├── chase.rs        # LED chase pattern demo
└── sys.rs          # System initialization helpers
```

### Communication Protocol

The project implements a **custom framing protocol** for reliable transport-agnostic communication:

**Frame Structure:**
```
[ STX | LEN | ADDR | CMD | <PAYLOAD...> | CRCL | CRCH ]
```

- **STX**: Start marker (`0xA5`) for frame synchronization
- **LEN**: Payload length (includes ADDR + CMD + data)
- **ADDR**: Device address (1 byte)
- **CMD**: Command/function code (1 byte)
- **PAYLOAD**: Variable data (0-253 bytes)
- **CRC**: CRC-16/Modbus checksum (little-endian)

**Supported Commands:**
- `0x01` — PING: Device health check
- `0x02` — CHASE: Trigger LED chase pattern
- `0x20` — GET_DEVICE_ID: Query unique device identifier

**Response Types:**
- **ACK**: Success response with status byte
- **ERROR**: Error response with error code
- **DATA**: Data response with byte count and payload

### Hardware Features

- **USB Serial**: Full-duplex communication over USB CDC-ACM
- **GPIO Control**: 5-pin LED chase sequence (pins 0-4)
- **Async Runtime**: Embassy executor enables concurrent tasks without blocking

---

## 🚀 Getting Started

### Prerequisites

1. **Rust Toolchain**  
   Install from [https://rustup.rs/](https://rustup.rs/)

2. **Target Architecture**  
   Add RP2350 ARM Cortex-M33 target:
   ```bash
   rustup target add thumbv8m.main-none-eabihf
   ```

3. **Debugging Tools** (Optional)  
   - Install `probe-rs`: `cargo install probe-rs-tools`
   - Install `cargo-embed`: Built into probe-rs suite

### Building the Firmware

The project uses a custom build script that automatically configures the target based on `.pico-rs` file:

```bash
# Build for RP2350 (Pico 2W default)
cargo build --release

# Build for RP2040 (legacy Pico)
echo "rp2040" > .pico-rs
cargo build --release

# Build for RP2350 RISC-V core
echo "rp2350-riscv" > .pico-rs
cargo build --release
```

**Build Profiles:**
- `dev` — Fast compilation, larger binaries, debug symbols
- `release` — Optimized for size and speed (recommended for deployment)

### Flashing to Device

#### Option 1: USB Bootloader (UF2)
1. Hold BOOTSEL button while connecting USB
2. Device mounts as mass storage
3. Copy `.uf2` binary to mounted drive:
   ```bash
   # Convert ELF to UF2 (if needed)
   elf2uf2-rs target/thumbv8m.main-none-eabihf/release/embedded-systems firmware.uf2
   ```

#### Option 2: Debug Probe (Recommended)
```bash
# Flash and attach debugger
cargo embed --release

# Or use probe-rs directly
probe-rs run --chip RP2350 target/thumbv8m.main-none-eabihf/release/embedded-systems
```

---

## 🧪 Usage

### Serial Communication

The device communicates over USB Serial (CDC-ACM). Use the Python client in `tools/serial_client/`:

#### Setup Python Environment
```bash
cd tools/serial_client
conda env create -f serial_client_env.yml
conda activate serial_client
```

#### Example Commands

**Send PING (0x01):**
```bash
python serial_client.py -c 0x01
```

**Trigger Chase Pattern (0x02):**
```bash
python serial_client.py -c 0x02
```

**Get Device ID (0x20):**
```bash
python serial_client.py -c 0x20
```

### Hardware Setup

For the LED chase demo, connect LEDs (with appropriate resistors) to:
- GPIO 0-4 → Anode (positive)
- Ground → Cathode (negative)

**Chase Pattern:** Each LED illuminates sequentially for 100ms with 100ms intervals.

---

## 📊 Implementation Highlights

### Memory Efficiency
- **Heapless Data Structures**: All buffers use compile-time fixed sizes (`heapless::Vec`)
- **Zero Dynamic Allocation**: No heap allocator required
- **Static Resources**: USB buffers and peripherals use `static_cell` for lifetime management

### Safety Features
- **CRC-16 Error Detection**: Modbus polynomial ensures data integrity
- **Frame Resynchronization**: Parser recovers from transmission errors by scanning for STX markers
- **Type-Safe Peripherals**: Rust ensures exclusive access to hardware resources at compile time

### Async Architecture
- **Non-blocking I/O**: USB reads/writes don't block the executor
- **Concurrent Tasks**: Spawner enables multiple async tasks (USB, timers, GPIO)
- **Zero-cost Abstractions**: Embassy compiles to efficient state machines

---

## 📂 Project Structure

```
EmbeddedRustSystems/
├── .cargo/             # Cargo configuration (target defaults, runner)
├── docs/               # Documentation (build guides, etc.)
├── embassy_examples/   # Example code from Embassy framework (66 files)
├── src/                # Main source code
│   ├── main.rs         # Application entry point
│   ├── protocol.rs     # Frame protocol implementation
│   ├── serial_usb.rs   # USB Serial abstraction
│   ├── chase.rs        # LED chase pattern
│   └── sys.rs          # System initialization
├── tools/              # Development tools
│   └── serial_client/  # Python USB Serial client
├── build.rs            # Build script for linker configuration
├── Cargo.toml          # Dependencies and build profiles
├── Embed.toml          # probe-rs configuration
├── rp2350.x            # Linker script for RP2350 ARM
├── rp2350_riscv.x      # Linker script for RP2350 RISC-V
└── rp2040.x            # Linker script for RP2040
```

---

## 🔮 Future Improvements

### Planned Features
- [ ] **Stepper Motor Control**: Implement PWM-based stepper motor driver
- [ ] **WiFi Integration**: Enable CYW43 driver for wireless communication
- [ ] **Advanced Protocols**: Add support for I2C/SPI peripheral communication
- [ ] **C++ Comparison**: Port implementation to C++ for performance benchmarking
- [ ] **Flash Storage**: Persistent configuration using RP2350 flash memory

### Potential Enhancements
- [ ] Multi-device addressing (use ADDR field for bus communication)
- [ ] Interrupt-driven GPIO with debouncing
- [ ] Watchdog timer for fault recovery
- [ ] Power management and sleep modes

---

## 📚 References / Resources

### Documentation
- [The Rust Book](https://doc.rust-lang.org/book/) — Core Rust programming concepts
- [The Embedded Rust Book](https://docs.rust-embedded.org/book/) — Embedded development guide
- [Embassy Documentation](https://embassy.dev/) — Async embedded framework

### RP2350 Resources
- [Embassy RP235x Examples](https://github.com/embassy-rs/embassy/tree/main/examples/rp235x)
- [RP235x Project Template](https://github.com/rp-rs/rp235x-project-template)
- [Raspberry Pi Pico 2W Datasheet](https://datasheets.raspberrypi.com/picow/pico-2-w-datasheet.pdf)
- [RP2350 SDK](https://github.com/raspberrypi/pico-sdk-tools/releases)

### Tools
- [probe-rs](https://probe.rs/) — Rust debugging and flashing tool
- [defmt](https://defmt.ferrous-systems.com/) — Efficient embedded logging

---

## 🧑‍💻 Author

**Olly Bayley**  
GitHub: [@ombayley](https://github.com/ombayley)

---

## 🪪 License

This project is licensed under the **GNU General Public License (GPL)** — See the [LICENSE](LICENSE) file for details.
The GPL License is a copyleft license, that requires any derivative work to also be released under the GPL License.
This means any derivative software that uses this code remains open-source and freely available to the public.
