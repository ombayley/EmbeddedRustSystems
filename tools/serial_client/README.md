# Serial Client Tool

This directory contains a small **host-side Python tool** used to send
and receive binary protocol frames to/from the embedded firmware over
**USB CDC or UART**.

The protocol is **binary, framed, length-based, and CRC-protected**,
inspired by industrial fieldbus designs (e.g. Modbus RTU), but adapted
to work reliably over packet-based transports like USB.

This tool is intended for: - bring-up and testing during firmware
development - manual command injection (setters / getters) - protocol
debugging and validation

It is **not** required for normal device operation.

------------------------------------------------------------------------

## Directory Structure

    tools/serial_client/
    ├── serial_client.py   # Python serial tool
    ├── README.md          # This file
    └── requirements.txt   # Python dependencies (optional)

------------------------------------------------------------------------

## Requirements

-   Python **3.8+**
-   `pyserial`

Install dependency:

``` bash
pip install pyserial
```

or, if using a virtual environment:

``` bash
pip install -r requirements.txt
```

------------------------------------------------------------------------

## Protocol Summary (High-Level)

Frames sent to the device use the following structure:

    [ STX, LEN, ADDR, CMD, <PAYLOAD...>, CRCL, CRCH ]

Where: - `STX` : start-of-frame marker (default `0xA5`) - `LEN` : number
of bytes from `ADDR` through end of `PAYLOAD` - `ADDR` : device
address - `CMD` : command / function code - `PAYLOAD` : optional data
bytes - `CRC` : CRC-16/Modbus (little-endian: low byte first)

CRC is calculated over all bytes from `STX` through the end of
`PAYLOAD`.

The Python tool automatically: - inserts `STX` - computes `LEN` -
calculates and appends CRC - converts small scalar data into the correct
byte representation

------------------------------------------------------------------------

## Basic Usage

### Example: call the `CHASE` command (setter)

``` python
from serial_client import CommandSender

comm = CommandSender("COM8", 115200)

ADDR = b"0x01"
CMD  = b"0x02"   # CHASE
DATA = b""       # no payload

tx = comm.send(ADDR, CMD, DATA)
print("TX:", tx.hex(" ").upper())

rx = comm.read_any()
print("RX:", rx.hex(" ").upper())

comm.close()
```

Expected response (ACK):

    A5 03 01 02 00 C1 A0

------------------------------------------------------------------------

## Data Handling Rules

The `send()` API accepts multiple input types for convenience:

### Address / Command

May be provided as: - `int` (e.g. `0x01`) - `bytes` (e.g. `b"0x01"`) -
`str` (e.g. `"0x01"` or `"01"`)

### DATA

-   `b""` or `None` → no payload
-   `int` in range `0..255` → encoded as **2-byte unsigned value**
-   `bytes` / `bytearray`
    -   length 1 → promoted to 2 bytes
    -   length 2 → used as-is
    -   longer → passed through (for future variable-length payloads)

------------------------------------------------------------------------

## USB CDC Notes (RP Pico / RP235x)

-   Opening the serial port resets USB CDC on the device.
-   The tool waits briefly after opening the port to allow enumeration.
-   First frame may be dropped if this delay is removed.

If communication appears unreliable, try: - closing and reopening the
serial port - increasing the read timeout

------------------------------------------------------------------------

## Debugging Tips

-   Use a serial monitor with **hex view** to compare frames.
-   Enable logging in firmware to print parsed frames.
-   Verify CRC byte order (CRC-Low first, CRC-High second).
-   Ensure no extra newline bytes are sent.

------------------------------------------------------------------------

## Why This Tool Lives in the Firmware Repo

This script is intentionally kept alongside the firmware because: - it
evolves with the protocol - it reduces mismatch between host and device
behavior - it provides a reproducible test interface for anyone cloning
the repo

For production systems, this tooling may be replaced by a higher-level
application.

------------------------------------------------------------------------

## Disclaimer

This tool is for **development and testing**. It performs minimal
validation on received data and assumes trusted input.

Do not expose it directly in production environments without additional
safeguards.
