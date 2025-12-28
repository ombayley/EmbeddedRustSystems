# Serial Client Tool

This directory contains a small **client-side Python tool** used to send
and receive protocol frames to/from the the pico over **USB(CDC) or UART**.

The protocol implemented on the pico is based on industrial fieldbus protocols (e.g. Modbus RTU), but adapted to work via USB which uses packets rather than the UART timing system (as is used by Modbus RTU).

As the communication protocol requires byte frames with a CRC, this tool is used to send and receive these frames rather than the serial monitor. This tool is intended for testing/debugging during firmware development and is **not** required for normal device operation.

------------------------------------------------------------------------

## Sub-Directory Structure

    tools/serial_client/
    ├── serial_client.py       # Python serial tool
    ├── README.md              # This file
    ├── serial_client_env.yml  # Conda environment
    └── pyproject.toml         # Project configuration 

------------------------------------------------------------------------

## Requirements

- Python **3.10+**
- `pyserial`

If this is your **First Time** using conda, it must be initialised using:

    ```bash
    conda init
    ```

To set up the Conda environment, navigate to the root directory and run:

    ```bash
    conda env create -f tools/serial_client/serial_client_env.yml
    ```

To activate the environment, use:

    ```bash
    conda activate SerialClient_env
    ```

You may also need to set the interpreter in your IDE (e.g. VSCode or PyCharm).

VSCode:

    Ctrl+Shift+P > Python: Select Interpreter > SerialClient_env

PyCharm:
<https://www.jetbrains.com/help/pycharm/configuring-python-interpreter.html>

------------------------------------------------------------------------

## Protocol Summary (High-Level)

Frames sent to the device use the following structure:

    [ STX, LEN, ADDR, CMD, <PAYLOAD...>, CRCL, CRCH ]

Where:

- `STX` : start-of-frame marker (default `0xA5`)
- `LEN` : number of bytes from `ADDR` through end of `PAYLOAD`
- `ADDR` : device address
- `CMD` : command / function code
- `PAYLOAD` : optional data bytes
- `CRC` : CRC-16/Modbus (little-endian: low byte first)

CRC is calculated over all bytes from `STX` through the end of `PAYLOAD`.

The Python tool automatically:

- inserts `STX`
- computes `LEN`
- calculates and appends CRC
- converts small scalar data into the correct byte representation

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

Expected Send frame (CHASE with no payload):

    A5 04 01 02 00 00 49 12

Expected response (ACK):

    A5 03 01 02 00 38 FD

------------------------------------------------------------------------

## Data Handling Rules

The `send()` API accepts multiple input types for convenience:

### Address / Command

May be provided as:

- `int` (e.g. `0x01`)
- `bytes` (e.g. `b"0x01"`)
- `str` (e.g. `"0x01"` or `"01"`)

### DATA

- `b""` or `None` → no payload
- `int` in range `0..255` → encoded as **2-byte unsigned value**
- `bytes` / `bytearray`
  - length 1 → promoted to 2 bytes
  - length 2 → used as-is
  - longer → passed through (for future variable-length payloads)

------------------------------------------------------------------------

## USB CDC Notes (RP Pico / RP235x)

- Opening the serial port resets USB CDC on the device.
- The tool waits briefly after opening the port to allow enumeration.
- First frame may be dropped if this delay is removed.

------------------------------------------------------------------------

## Disclaimer

This tool is for **development and testing**. It performs minimal
validation on received data and assumes trusted input.

Do not expose it directly in production environments without additional
safeguards.
