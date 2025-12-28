# Raspberry Pi Pico 2 Rust Project --- Build & Flash Guide

This document describes how to build, run, and flash the Rust programs
in this repository for the **Raspberry Pi Pico 2 (RP235x)**.

------------------------------------------------------------------------

## 0. Quickview Cheatsheet

``` bash
# Debug build + flash
cargo run

# Release build + flash
cargo run --release
cargo runr  # alias

# Debug build without flashing:
cargo build

# Release build without flashing:
cargo build --release
cargo buildr  # alias

# Convert to UF2 for simple flashing
picotool uf2 convert -t elf target/thumbv8m.main-none-eabihf/release/embedded-systems dist/embedded-systems.uf2
```

------------------------------------------------------------------------

## 1. Project Cargo Configuration

The `.cargo/config.toml` gives preset build flags  as well as environemnt and aliases for the cargo commands. In this case it sets the build flags for the **target** and **runner** meaning you do **not** need to pass `--target` and `cargo run` **automatically builds & flashes** the ELF using `picotool`

- The **target** describes the processor to build for. This defaults to `thumbv8m.main-none-eabihf` which corresponds to the ARM Cortex M-33 processor used by the Pico 2. See: [Cross-compilation](https://pico.implrust.com/std-to-no-std/cross-compilation/index.html)

-The **runner** is an external tool we call to flash the ELF onto the Pico. In the case of the rp2040 both `picotool` and `elf2uf2-rs` are available. However `elf2uf2-rs` is known to have issues with the rp2350 processor. See: [GitHub Issue](https://github.com/JoNil/elf2uf2-rs/issues/38) so we use `picotool` instead.

- The **aliases** are shortcuts for common commands allowing custom comands to be defined.

------------------------------------------------------------------------

## 2. Flashing the Pico 2

To flash to the pico, two routes are available.

The first option uses the `probe-rs` flashing tool and a [Raspberry Pi Debug Probe](https://www.raspberrypi.com/products/debug-probe/) to directly flash the elf contnents to the MCU over the debug port. However this requires extra equipment.

The second option (recomended) allows the Pico 2 to be flashed from BOOTSEL mode (via comand line or drag-and-drop). Once in BOOTSEL mode the Pico 2 will mount as a USB drive and allow a uf2 file to be loaded onto it. Loading the uf2 file can be done maually (via drag-and-drop) or automatically during the cargo run command as described below.

### BOOTSEL Mode

#### Option 1 - Manual

The Pico can be put into BOOTSEL mode by holding the BOOTSEL button while plugging it in.

#### Option 2 - 1200-Baud Touch [TBD]

Using the `rp235x_hal::reboot` module a 1200-Baud Touch protocol is to be built into the
`serial_usb.rs` script to allow the Pico 2 to be put into BOOTSEL when a client connects to the serial port at 1200-baud with the DTR line held low. This is a standard protocol and has been used
in many MCUs.

### Flashing

#### Automatically via `cargo run` (recomended)

Due to the `runner` field in the `.cargo/config.toml` file the `cargo run` command will automatically flash the ELF file to the Pico via `picotool` **ONLY IF** the Pico is in BOOTSEL mode.

``` bash
cargo run --release
```

#### Manually via drag‑and‑drop

1. Once in BOOTSEL mode the Pico 2 will mount as a USB drive
2. Compile the program to a `*.uf2` file [see below]
3. Drag-and-drop the `*.uf2` file onto the Pico in the windows explorer
4. Wait for the Pico to reboot automatically

------------------------------------------------------------------------

## 3. Manual Generation of UF2 file (optional)

Sometimes a UF2 file to drag‑and‑drop onto the Pico is desirable for portability and distribution.
To prepare a UF2 file:

### Step 1 --- Build the ELF

``` bash
cargo build --release
```

### Step 2 --- Convert to UF2

``` bash
picotool uf2 convert -t elf target/thumbv8m.main-none-eabihf/release/embedded-systems dist/embedded-systems.uf2
```

note: the 'thumbv8m.main-none-eabihf' (target) and 'embedded-systems' (package name) may be subject to change

------------------------------------------------------------------------

## 4. Selecting a program to compile

If multiple binaries exist under `src/bin/` they can be selected by name using `--bin <NAME>`:

``` bash
cargo run --bin <name>
cargo build --bin <name>
```

Examples:

``` bash
cargo run --bin blinky
cargo run --release --bin gpio
cargo build --release --bin serial
```

------------------------------------------------------------------------

## 5. Important takeaways

- Currently all builds are automatically target Cortex‑M33 due to the default target being `thumbv8m.main-none-eabihf` in the `.cargo/config.toml`
- All `cargo run` invocations automatically flash using `picotool`
