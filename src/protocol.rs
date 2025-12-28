//! protocol.rs
//!
//! Framing (transport-agnostic, Modbus-inspired):
//!   [ STX, LEN, ADDR, CMD, <PAYLOAD...>, CRCL, CRCH ]
//!
//! Where:
//! - STX: 1 byte start marker (for resync)
//! - LEN: 1 byte = number of bytes from ADDR through end of PAYLOAD
//!        (so LEN >= 2 because it must include ADDR + CMD)
//! - ADDR: 1 byte address
//! - CMD:  1 byte command/function
//! - PAYLOAD: 0..253 bytes (because LEN is u8 and includes ADDR+CMD)
//! - CRC: CRC-16/Modbus over everything from STX through end of PAYLOAD
//!        appended little-endian as CRCL then CRCH (Modbus convention)
//!
//! Notes:
//! - Parser is stream-based (USB/UART chunks are arbitrary).
//! - Parser resyncs by scanning for STX.
//! - No timing dependence (unlike true Modbus RTU).
//!
//! Suggested semantics (optional, but handy):
//! - Setters respond with payload: [STATUS]
//! - Getters respond with payload: [STATUS, BYTECOUNT, <DATA...>]

#![allow(dead_code)]

use heapless::Vec;

pub const STX: u8 = 0xA5;

// LEN is u8 and includes ADDR+CMD, so payload max is 255 - 2 = 253.
pub const MAX_PAYLOAD: usize = 253;

// Total frame bytes = STX(1) + LEN(1) + (LEN bytes) + CRC(2) = LEN + 4
// Max total = 255 + 4 = 259
pub const MAX_FRAME: usize = 259;

// Internal stream buffer capacity (can be bigger than MAX_FRAME to hold multiple frames/chunks)
pub const STREAM_BUF_CAP: usize = 512;

#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub enum ParseError {
    LenTooSmall,
    LenTooBig,
    CrcMismatch,
}

fn drop_front<const N: usize>(buf: &mut heapless::Vec<u8, N>, count: usize) {
    let len = buf.len();
    if count >= len {
        buf.clear();
        return;
    }

    // Shift remaining bytes down
    for i in 0..(len - count) {
        buf[i] = buf[i + count];
    }
    buf.truncate(len - count);
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Frame {
    pub addr: u8,
    pub cmd: u8,
    pub payload: Vec<u8, MAX_PAYLOAD>,
}

impl Frame {
    /// Convenience if you use "status-first" responses:
    pub fn status(&self) -> Option<u8> {
        self.payload.first().copied()
    }
}

/// Stream parser for [STX, LEN, ...] frames.
pub struct Parser {
    buf: Vec<u8, STREAM_BUF_CAP>,
}

impl Parser {
    pub fn new() -> Self {
        Self { buf: Vec::new() }
    }

    /// Push raw bytes into the stream buffer. Returns number accepted.
    /// If buffer overflows, we clear it (simple, deterministic) and keep going.
    pub fn push_bytes(&mut self, bytes: &[u8]) -> usize {
        let mut accepted = 0;
        for &b in bytes {
            if self.buf.push(b).is_ok() {
                accepted += 1;
            } else {
                // Overflow policy: clear and keep latest byte.
                self.buf.clear();
                if self.buf.push(b).is_ok() {
                    accepted += 1;
                }
            }
        }
        accepted
    }

    /// Attempt to parse the next valid frame.
    ///
    /// - Ok(Some(frame)) on success (consumes that frame from internal buffer)
    /// - Ok(None) if not enough data yet
    /// - Err(e) only for "structural" issues of a candidate frame; parser also resyncs and continues scanning
    pub fn next_frame(&mut self) -> Result<Option<Frame>, ParseError> {
        loop {
            if self.buf.is_empty() {
                return Ok(None);
            }

            // Find STX
            let stx_pos = match self.buf.iter().position(|&b| b == STX) {
                Some(p) => p,
                None => {
                    self.buf.clear();
                    return Ok(None);
                }
            };

            // Drop anything before STX (resync)
            if stx_pos > 0 {
                drop_front(&mut self.buf, stx_pos);
            }

            // Need at least STX + LEN
            if self.buf.len() < 2 {
                return Ok(None);
            }

            let len = self.buf[1] as usize;

            // LEN must include ADDR+CMD
            if len < 2 {
                // Drop this STX and rescan
                drop_front(&mut self.buf, 1);
                return Err(ParseError::LenTooSmall);
            }

            if len > 255 {
                // impossible, but keep for completeness
                drop_front(&mut self.buf, 1);
                return Err(ParseError::LenTooBig);
            }

            let total_len = 1 + 1 + len + 2; // STX + LEN + body + CRC
            if total_len > MAX_FRAME {
                drop_front(&mut self.buf, 1);
                return Err(ParseError::LenTooBig);
            }

            // Wait for full candidate
            if self.buf.len() < total_len {
                return Ok(None);
            }

            let candidate = &self.buf[..total_len];

            // Verify CRC over candidate[0 .. total_len-2]
            let computed = crc16_modbus(&candidate[..total_len - 2]);
            let got = u16::from_le_bytes([candidate[total_len - 2], candidate[total_len - 1]]); // CRCL, CRCH

            if computed != got {
                // CRC mismatch: drop STX and keep scanning.
                drop_front(&mut self.buf, 1);
                return Err(ParseError::CrcMismatch);
            }

            // Extract fields
            let addr = candidate[2];
            let cmd = candidate[3];

            let payload_start = 4;
            let payload_end = 2 + len; // since LEN counts bytes from ADDR (index 2) through payload end
            let payload_slice = &candidate[payload_start..payload_end];

            let mut payload = Vec::<u8, MAX_PAYLOAD>::new();
            // payload length is <= 253 by construction (LEN <= 255 and includes ADDR+CMD)
            payload.extend_from_slice(payload_slice).ok();

            // Consume this frame from stream buffer
            drop_front(&mut self.buf, 1);

            return Ok(Some(Frame { addr, cmd, payload }));
        }
    }
}

// -----------------------------
// Frame builders
// -----------------------------

/// Build a frame: [STX, LEN, ADDR, CMD, payload..., CRCL, CRCH]
///
/// Returns a heapless Vec that you can pass directly to your transport write().
pub fn build_frame<const OUT_CAP: usize>(
    addr: u8,
    cmd: u8,
    payload: &[u8],
) -> Result<Vec<u8, OUT_CAP>, ()> {
    // LEN counts ADDR+CMD+payload
    if payload.len() > MAX_PAYLOAD {
        return Err(());
    }
    let len = 2 + payload.len();
    if len > 255 {
        return Err(());
    }

    // total = 1 + 1 + len + 2
    let total = 1 + 1 + len + 2;
    if total > OUT_CAP {
        return Err(());
    }

    let mut out = Vec::<u8, OUT_CAP>::new();

    out.push(STX).map_err(|_| ())?;
    out.push(len as u8).map_err(|_| ())?;
    out.push(addr).map_err(|_| ())?;
    out.push(cmd).map_err(|_| ())?;
    out.extend_from_slice(payload).map_err(|_| ())?;

    let crc = crc16_modbus(&out);
    let [crcl, crch] = crc.to_le_bytes(); // Modbus convention
    out.push(crcl).map_err(|_| ())?;
    out.push(crch).map_err(|_| ())?;

    Ok(out)
}

/// ACK for setters (payload: [STATUS=0])
pub fn build_ack<const OUT_CAP: usize>(addr: u8, cmd: u8) -> Result<Vec<u8, OUT_CAP>, ()> {
    build_frame::<OUT_CAP>(addr, cmd, &[0x00])
}

/// ERROR for setters (payload: [STATUS=err_code])
pub fn build_err<const OUT_CAP: usize>(
    addr: u8,
    cmd: u8,
    err_code: u8,
) -> Result<Vec<u8, OUT_CAP>, ()> {
    build_frame::<OUT_CAP>(addr, cmd, &[err_code])
}

/// Getter response (payload: [STATUS=0, BYTECOUNT, data...])
pub fn build_data<const OUT_CAP: usize>(
    addr: u8,
    cmd: u8,
    data: &[u8],
) -> Result<Vec<u8, OUT_CAP>, ()> {
    // payload will be 2 + data.len()
    if data.len() > (MAX_PAYLOAD - 2) {
        return Err(());
    }

    // BYTECOUNT is u8
    if data.len() > 255 {
        return Err(());
    }

    // Build payload into a small local buffer (stack) with heapless Vec
    let mut payload = Vec::<u8, MAX_PAYLOAD>::new();
    payload.push(0x00).map_err(|_| ())?; // STATUS OK
    payload.push(data.len() as u8).map_err(|_| ())?; // BYTECOUNT
    payload.extend_from_slice(data).map_err(|_| ())?;

    build_frame::<OUT_CAP>(addr, cmd, &payload)
}

// -----------------------------
// CRC-16/Modbus
// -----------------------------

/// CRC-16/Modbus: poly 0xA001 (reflected), init 0xFFFF
pub fn crc16_modbus(data: &[u8]) -> u16 {
    let mut crc: u16 = 0xFFFF;
    for &b in data {
        crc ^= b as u16;
        for _ in 0..8 {
            let lsb = (crc & 0x0001) != 0;
            crc >>= 1;
            if lsb {
                crc ^= 0xA001;
            }
        }
    }
    crc
}
