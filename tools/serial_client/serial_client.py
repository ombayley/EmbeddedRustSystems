import serial
import time
from typing import Union, Optional


def crc16_modbus(data: bytes) -> int:
    """CRC-16/Modbus: poly=0xA001, init=0xFFFF, output u16"""
    crc = 0xFFFF
    for b in data:
        crc ^= b
        for _ in range(8):
            lsb = crc & 0x0001
            crc >>= 1
            if lsb:
                crc ^= 0xA001
    return crc & 0xFFFF


def parse_u8(x: Union[int, bytes, str]) -> int:
    """
    Accepts:
      - int 0..255
      - bytes like b'0x01' or b'01' or b'\x01'
      - str like '0x01' or '01'
    Returns int 0..255.
    """
    if isinstance(x, int):
        if not (0 <= x <= 255):
            raise ValueError("Value out of range for u8 (0..255)")
        return x

    if isinstance(x, bytes):
        # If it's a single raw byte, interpret directly.
        if len(x) == 1:
            return x[0]
        # Otherwise treat as ASCII text e.g. b'0x01'
        s = x.decode(errors="strict").strip()
        return parse_u8(s)

    if isinstance(x, str):
        s = x.strip().lower()
        # Allow "0x01" or "01"
        if s.startswith("0x"):
            v = int(s, 16)
        else:
            v = int(s, 16) if all(c in "0123456789abcdef" for c in s) else int(s)
        if not (0 <= v <= 255):
            raise ValueError("Value out of range for u8 (0..255)")
        return v

    raise TypeError("Unsupported type for u8")


def data_to_2_bytes(data: Union[bytes, bytearray, int, str, None]) -> bytes:
    """
    Convert DATA into exactly 2 bytes (u16 big-endian) if DATA is provided as a scalar.
    Rules:
      - None or b'' -> empty payload
      - int 1..255  -> two bytes: [0x00, value] (big-endian u16)
      - str like '25' or '0x19' -> same as int
      - bytes/bytearray:
          * length 0 -> empty payload
          * length 2 -> used as-is
          * length 1 -> promoted to 2 bytes [0x00, b0]
          * other lengths -> used as-is (lets you send variable payloads later)
    """
    if data is None:
        return b""

    if isinstance(data, (bytes, bytearray)):
        bts = bytes(data)
        if len(bts) == 0:
            return b""
        if len(bts) == 1:
            return bytes([0x00, bts[0]])
        if len(bts) == 2:
            return bts
        # If you later want variable payloads, allow it:
        return bts

    if isinstance(data, str):
        data = data.strip()
        if data == "":
            return b""
        # accept '0x19' or '25'
        v = int(data, 16) if data.lower().startswith("0x") else int(data)
        return data_to_2_bytes(v)

    if isinstance(data, int):
        if data == 0:
            # treat 0 as valid if you want; you said 1-255 but allowing 0 is often useful
            return bytes([0x00, 0x00])
        if not (0 <= data <= 255):
            raise ValueError("DATA int must be in 0..255")
        return bytes([0x00, data])  # u16 big-endian

    raise TypeError("Unsupported DATA type")


class CommandSender:
    def __init__(self, port: str, baudrate: int = 115200, timeout: float = 1.0, stx: int = 0xA5):
        self.ser = serial.Serial(port=port, baudrate=baudrate, timeout=timeout)
        self.stx = stx & 0xFF

        # Pico USB CDC often benefits from a short settle time after opening
        time.sleep(2)

    def build_frame(self, addr: Union[int, bytes, str], cmd: Union[int, bytes, str], data: Union[bytes, bytearray, int, str, None]) -> bytes:
        addr_u8 = parse_u8(addr)
        cmd_u8 = parse_u8(cmd)

        payload = data_to_2_bytes(data)

        # LEN counts bytes from ADDR through end of PAYLOAD
        length = 2 + len(payload)  # ADDR + CMD + payload
        if not (2 <= length <= 255):
            raise ValueError("Frame LEN out of range (2..255). Payload too large?")

        head = bytes([self.stx, length, addr_u8, cmd_u8]) + payload
        crc = crc16_modbus(head)
        crcl, crch = (crc & 0xFF), ((crc >> 8) & 0xFF)  # Modbus CRC order: low byte then high byte
        return head + bytes([crcl, crch])

    def send(self, addr: Union[int, bytes, str], cmd: Union[int, bytes, str], data: Union[bytes, bytearray, int, str, None] = b"") -> bytes:
        frame = self.build_frame(addr, cmd, data)
        self.ser.write(frame)
        return frame  # return what we sent (useful for logging)

    def read_any(self, max_bytes: int = 256) -> bytes:
        """Read up to max_bytes (whatever is available until timeout)."""
        return self.ser.read(max_bytes)

    def close(self):
        self.ser.close()


if __name__ == "__main__":
    comm = CommandSender("COM8", 115200)

    # Example 1: CHASE with empty payload (no DATA)
    ADDR = 0x01
    CMD = 0x02
    DATA = 0x00
    for _ in range(2):
        tx = comm.send(ADDR, CMD, DATA)
        print("TX:", tx.hex(" ").upper())
        rx = comm.read_any()
        print("RX:", rx.hex(" ").upper() if rx else "<no response>")
        time.sleep(1)

    # Example 2: send a DATA value (1..255) as 2 bytes
    # tx = comm.send(b"0x01", b"0x10", 25)  # payload becomes 00 19
    # print("TX:", tx.hex(" ").upper())

    comm.close()
