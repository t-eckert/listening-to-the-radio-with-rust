/// CHU time code decoder.
///
/// CHU transmits time information using Bell 103-style FSK at 300 baud.
/// Data bursts occur at seconds 31-39 of each minute.
///
/// Each burst contains 10 bytes (8N2 framing = 11 bits per byte):
/// - Bytes 0-4: data block
/// - Bytes 5-9: redundancy block (identical for Format A, complemented for Format B)
///
/// Format A (seconds 32-39):
///   After nibble swap: [6|d1] [d2|d3] [h1|h2] [m1|m2] [s1|s2]
///   Where d=day-of-year, h=hour, m=minute, s=second (all BCD)
///
/// Format B (second 31):
///   After nibble swap: [x|z] [y1|y2] [y3|y4] [t1|t2] [a1|a2]
///   Where x=control, z=|DUT1|, y=year, t=TAI-UTC, a=DST code

#[derive(Debug, Clone)]
pub struct ChuTime {
    pub year: Option<u16>,
    pub day_of_year: u16,
    pub hours: u8,
    pub minutes: u8,
    pub seconds: u8,
    pub dut1: Option<f32>,
}

pub struct ChuDecoder {
    state: DecoderState,
    byte_buffer: Vec<u8>,
    current_byte: u8,
    bit_count: u8,
    stop_bits_seen: u8,
    last_year: Option<u16>,
    last_dut1: Option<f32>,
}

#[derive(Debug, PartialEq)]
enum DecoderState {
    WaitingForStart,
    ReceivingBits,
    WaitingForStop,
}

impl ChuDecoder {
    pub fn new() -> Self {
        Self {
            state: DecoderState::WaitingForStart,
            byte_buffer: Vec::new(),
            current_byte: 0,
            bit_count: 0,
            stop_bits_seen: 0,
            last_year: None,
            last_dut1: None,
        }
    }

    /// Feed a single demodulated bit. Returns decoded time when a valid frame is received.
    pub fn feed_bit(&mut self, bit: u8) -> Option<ChuTime> {
        match self.state {
            DecoderState::WaitingForStart => {
                if bit == 0 {
                    self.state = DecoderState::ReceivingBits;
                    self.current_byte = 0;
                    self.bit_count = 0;
                }
                None
            }
            DecoderState::ReceivingBits => {
                // 8 data bits, LSB first
                self.current_byte |= bit << self.bit_count;
                self.bit_count += 1;

                if self.bit_count >= 8 {
                    self.state = DecoderState::WaitingForStop;
                    self.stop_bits_seen = 0;
                }
                None
            }
            DecoderState::WaitingForStop => {
                if bit != 1 {
                    // Framing error — reset
                    self.state = DecoderState::WaitingForStart;
                    self.byte_buffer.clear();
                    return None;
                }

                self.stop_bits_seen += 1;
                if self.stop_bits_seen >= 2 {
                    // Byte complete
                    self.byte_buffer.push(self.current_byte);
                    self.state = DecoderState::WaitingForStart;

                    // CHU sends 10 bytes per burst
                    if self.byte_buffer.len() >= 10 {
                        let result = self.try_decode_burst();
                        self.byte_buffer.clear();
                        return result;
                    }
                }
                None
            }
        }
    }

    fn try_decode_burst(&mut self) -> Option<ChuTime> {
        if self.byte_buffer.len() < 10 {
            return None;
        }

        // Nibble-swap all bytes
        let swapped: Vec<u8> = self.byte_buffer.iter().map(|&b| nibble_swap(b)).collect();

        // Data block is bytes 0-4, redundancy is bytes 5-9
        let data = &swapped[0..5];
        let redundancy = &swapped[5..10];

        // Try Format A first: redundancy should be identical to data
        if data == redundancy {
            return self.decode_format_a(data);
        }

        // Try Format B: redundancy should be one's complement of data
        let complemented: Vec<u8> = redundancy.iter().map(|&b| !b).collect();
        if data == complemented.as_slice() {
            self.decode_format_b(data);
            // Format B doesn't produce a time directly, but updates year/DUT1
            return None;
        }

        None
    }

    fn decode_format_a(&self, data: &[u8]) -> Option<ChuTime> {
        // Byte 0: [6][d1] — framing code must be 6
        let framing = (data[0] >> 4) & 0x0F;
        if framing != 6 {
            return None;
        }
        let d1 = data[0] & 0x0F;

        // Byte 1: [d2][d3]
        let d2 = (data[1] >> 4) & 0x0F;
        let d3 = data[1] & 0x0F;
        let day_of_year = d1 as u16 * 100 + d2 as u16 * 10 + d3 as u16;

        // Byte 2: [h1][h2]
        let h1 = (data[2] >> 4) & 0x0F;
        let h2 = data[2] & 0x0F;
        let hours = h1 * 10 + h2;

        // Byte 3: [m1][m2]
        let m1 = (data[3] >> 4) & 0x0F;
        let m2 = data[3] & 0x0F;
        let minutes = m1 * 10 + m2;

        // Byte 4: [s1][s2]
        let s1 = (data[4] >> 4) & 0x0F;
        let s2 = data[4] & 0x0F;
        let seconds = s1 * 10 + s2;

        // Sanity checks
        if day_of_year == 0 || day_of_year > 366 {
            return None;
        }
        if hours > 23 || minutes > 59 || seconds > 60 {
            return None;
        }
        // Format A appears at seconds 32-39
        if seconds < 32 || seconds > 39 {
            return None;
        }

        Some(ChuTime {
            year: self.last_year,
            day_of_year,
            hours,
            minutes,
            seconds,
            dut1: self.last_dut1,
        })
    }

    fn decode_format_b(&mut self, data: &[u8]) {
        // Byte 0: [x][z] — control flags and |DUT1|
        let x = (data[0] >> 4) & 0x0F;
        let z = data[0] & 0x0F;

        // x bit 0: DUT1 sign (0=positive, 1=negative)
        let dut1_negative = (x & 0x01) != 0;
        let dut1_magnitude = z as f32 / 10.0;
        self.last_dut1 = Some(if dut1_negative {
            -dut1_magnitude
        } else {
            dut1_magnitude
        });

        // Bytes 1-2: year [y1][y2] [y3][y4]
        let y1 = (data[1] >> 4) & 0x0F;
        let y2 = data[1] & 0x0F;
        let y3 = (data[2] >> 4) & 0x0F;
        let y4 = data[2] & 0x0F;
        let year = y1 as u16 * 1000 + y2 as u16 * 100 + y3 as u16 * 10 + y4 as u16;

        if year >= 2020 && year <= 2040 {
            self.last_year = Some(year);
        }
    }
}

/// Swap the high and low nibbles of a byte.
fn nibble_swap(byte: u8) -> u8 {
    (byte >> 4) | (byte << 4)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn nibble_swap_works() {
        assert_eq!(nibble_swap(0xAB), 0xBA);
        assert_eq!(nibble_swap(0x12), 0x21);
        assert_eq!(nibble_swap(0x60), 0x06);
    }

    #[test]
    fn format_a_decode() {
        // Simulate a Format A burst for Day 083, 14:30:35 UTC
        // After nibble swap: 60 83 14 30 35
        // Before nibble swap: 06 38 41 03 53
        let mut decoder = ChuDecoder::new();

        // Data bytes (before nibble swap) + identical redundancy
        let raw_data: [u8; 10] = [0x06, 0x38, 0x41, 0x03, 0x53, 0x06, 0x38, 0x41, 0x03, 0x53];

        // Feed as 8N2 frames: start(0) + 8 data bits LSB first + stop(1) + stop(1)
        for &byte in &raw_data {
            // Start bit
            decoder.feed_bit(0);
            // 8 data bits LSB first
            for i in 0..8 {
                decoder.feed_bit((byte >> i) & 1);
            }
            // 2 stop bits
            decoder.feed_bit(1);
            decoder.feed_bit(1);
        }
        // The 10th byte's stop bits trigger decoding — check the result from feed_bit
        // Re-run to actually capture the result
        let mut decoder = ChuDecoder::new();
        let mut result = None;
        for &byte in &raw_data {
            decoder.feed_bit(0);
            for i in 0..8 {
                let r = decoder.feed_bit((byte >> i) & 1);
                if r.is_some() {
                    result = r;
                }
            }
            let r = decoder.feed_bit(1);
            if r.is_some() {
                result = r;
            }
            let r = decoder.feed_bit(1);
            if r.is_some() {
                result = r;
            }
        }

        let time = result.expect("should decode a valid Format A frame");
        assert_eq!(time.day_of_year, 83);
        assert_eq!(time.hours, 14);
        assert_eq!(time.minutes, 30);
        assert_eq!(time.seconds, 35);
    }
}
