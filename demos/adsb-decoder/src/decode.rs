/// ADS-B Mode S message decoder.
///
/// Parses raw bit arrays extracted from 1090 MHz PPM into structured aircraft data.

/// A decoded ADS-B message.
#[derive(Debug, Clone)]
pub struct AdsbMessage {
    /// Downlink format (5 bits)
    pub df: u8,
    /// ICAO 24-bit aircraft address
    pub icao: u32,
    /// Decoded payload
    pub payload: Payload,
}

#[derive(Debug, Clone)]
pub enum Payload {
    /// Aircraft identification (callsign)
    Identification { callsign: String },
    /// Airborne position
    Position {
        altitude_ft: Option<i32>,
        cpr_lat: u32,
        cpr_lon: u32,
        cpr_odd: bool,
    },
    /// Airborne velocity
    Velocity {
        ground_speed_kt: Option<u16>,
        vertical_rate_fpm: Option<i32>,
        heading: Option<f32>,
    },
    /// Other/unhandled type code
    Other { type_code: u8 },
}

/// Try to decode a message from 112 extracted bits.
/// Handles both 56-bit short (DF 0,4,5,11) and 112-bit long (DF 17,18) messages.
pub fn decode_message(bits: &[u8; 112]) -> Option<AdsbMessage> {
    let df = bits_to_u32(bits, 0, 5) as u8;

    match df {
        17 | 18 => decode_extended_squitter(bits, df),
        0 | 4 | 5 | 11 => decode_short_message(bits, df),
        _ => None,
    }
}

/// Decode a 112-bit extended squitter (DF 17/18).
fn decode_extended_squitter(bits: &[u8; 112], df: u8) -> Option<AdsbMessage> {
    let icao = bits_to_u32(bits, 8, 24);

    // CRC check: processing all 112 bits should yield 0
    let computed = crc24(bits, 112);
    if computed != 0 {
        return None;
    }

    let type_code = bits_to_u32(bits, 32, 5) as u8;

    let payload = match type_code {
        1..=4 => decode_identification(bits),
        9..=18 => decode_airborne_position(bits, type_code),
        19 => decode_airborne_velocity(bits),
        _ => Payload::Other { type_code },
    };

    Some(AdsbMessage { df, icao, payload })
}

/// Decode a 56-bit short message (DF 0, 4, 5, 11).
/// For short messages, the last 24 bits of the 56-bit message are
/// the CRC XORed with the ICAO address — so computing CRC over the
/// first 56 bits gives us the ICAO address directly.
fn decode_short_message(bits: &[u8; 112], df: u8) -> Option<AdsbMessage> {
    // Only use the first 56 bits
    let mut short_bits = [0u8; 112];
    short_bits[..56].copy_from_slice(&bits[..56]);

    let remainder = crc24(&short_bits, 56);
    // The remainder IS the ICAO address for short messages
    let icao = remainder;

    // Sanity: ICAO address shouldn't be 0
    if icao == 0 {
        return None;
    }

    let payload = match df {
        4 | 20 => {
            // Altitude reply — extract altitude from bits 19..=31 (13 bits)
            // We can reuse altitude decoding logic
            Payload::Other { type_code: df }
        }
        5 | 21 => {
            // Identity reply (squawk code)
            Payload::Other { type_code: df }
        }
        _ => Payload::Other { type_code: df },
    };

    Some(AdsbMessage { df, icao, payload })
}

fn decode_identification(bits: &[u8; 112]) -> Payload {
    const CHARSET: &[u8] = b"#ABCDEFGHIJKLMNOPQRSTUVWXYZ##### ###############0123456789######";

    let mut callsign = String::with_capacity(8);
    for i in 0..8 {
        let idx = bits_to_u32(bits, 40 + i * 6, 6) as usize;
        if idx < CHARSET.len() {
            let ch = CHARSET[idx] as char;
            if ch != '#' {
                callsign.push(ch);
            }
        }
    }

    Payload::Identification {
        callsign: callsign.trim().to_string(),
    }
}

fn decode_airborne_position(bits: &[u8; 112], type_code: u8) -> Payload {
    // Altitude encoding depends on type code
    let altitude_ft = decode_altitude(bits, type_code);

    let cpr_odd = bits[53] == 1;
    let cpr_lat = bits_to_u32(bits, 54, 17);
    let cpr_lon = bits_to_u32(bits, 71, 17);

    Payload::Position {
        altitude_ft,
        cpr_lat,
        cpr_lon,
        cpr_odd,
    }
}

fn decode_altitude(bits: &[u8; 112], _type_code: u8) -> Option<i32> {
    // Extract the 12-bit altitude field (bits 40-51)
    // Bit 47 (Q bit) determines encoding:
    //   Q=1: 25-foot increments
    //   Q=0: 100-foot (Gillham) increments
    let q_bit = bits[47];

    if q_bit == 1 {
        // Remove the Q bit to get an 11-bit value
        // Bits: 40-46, then 48-51 (skip bit 47)
        let high = bits_to_u32(bits, 40, 7) as i32;
        let low = bits_to_u32(bits, 48, 4) as i32;
        let n = (high << 4) | low;
        let alt = n * 25 - 1000;
        Some(alt)
    } else {
        // Gillham coding — skip for now, less common at high altitudes
        None
    }
}

fn decode_airborne_velocity(bits: &[u8; 112]) -> Payload {
    let subtype = bits_to_u32(bits, 37, 3) as u8;

    match subtype {
        1 | 2 => {
            // Ground speed, normal (1) or supersonic (2)
            let ew_sign = bits[45];
            let ew_vel = bits_to_u32(bits, 46, 10) as i32;
            let ns_sign = bits[56];
            let ns_vel = bits_to_u32(bits, 57, 10) as i32;

            let ew = if ew_sign == 1 { -(ew_vel - 1) } else { ew_vel - 1 };
            let ns = if ns_sign == 1 { -(ns_vel - 1) } else { ns_vel - 1 };

            let speed = ((ew * ew + ns * ns) as f64).sqrt() as u16;
            let heading = (ew as f64).atan2(ns as f64).to_degrees();
            let heading = if heading < 0.0 { heading + 360.0 } else { heading } as f32;

            // Vertical rate
            let vr_sign = bits[68];
            let vr_val = bits_to_u32(bits, 69, 9) as i32;
            let vr = if vr_val == 0 {
                None
            } else {
                let rate = (vr_val - 1) * 64;
                Some(if vr_sign == 1 { -rate } else { rate })
            };

            Payload::Velocity {
                ground_speed_kt: if ew_vel == 0 && ns_vel == 0 { None } else { Some(speed) },
                vertical_rate_fpm: vr,
                heading: Some(heading),
            }
        }
        _ => Payload::Velocity {
            ground_speed_kt: None,
            vertical_rate_fpm: None,
            heading: None,
        },
    }
}

/// Compute the Mode S CRC-24 over a message.
/// For a valid message, processing all 112 bits (including the CRC) should return 0.
///
/// Generator polynomial: x^24 + x^23 + x^10 + x^3 + x^0 = 0xFFF409
fn crc24(bits: &[u8; 112], len: usize) -> u32 {
    const POLY: u32 = 0xFFF409;
    let mut crc: u32 = 0;

    for i in 0..len {
        let bit = bits[i] as u32;
        let msb = (crc >> 23) & 1;
        crc = ((crc << 1) & 0xFFFFFF) | bit;
        if msb != 0 {
            crc ^= POLY;
        }
    }

    crc & 0xFFFFFF
}

/// Extract `count` bits starting at `offset` from a bit array, returned as u32.
pub fn bits_to_u32_pub(bits: &[u8], offset: usize, count: usize) -> u32 {
    bits_to_u32(bits, offset, count)
}

/// Public wrapper for CRC testing.
pub fn crc24_pub(bits: &[u8; 112], len: usize) -> u32 {
    crc24(bits, len)
}

fn bits_to_u32(bits: &[u8], offset: usize, count: usize) -> u32 {
    let mut val: u32 = 0;
    for i in 0..count {
        val = (val << 1) | bits[offset + i] as u32;
    }
    val
}

/// Resolve a global position from an even/odd CPR pair.
/// Returns (latitude, longitude) in degrees, or None if invalid.
pub fn decode_cpr_global(
    cpr_lat_even: u32,
    cpr_lon_even: u32,
    cpr_lat_odd: u32,
    cpr_lon_odd: u32,
    most_recent_odd: bool,
) -> Option<(f64, f64)> {
    let cpr_lat_even = cpr_lat_even as f64 / 131072.0;
    let cpr_lon_even = cpr_lon_even as f64 / 131072.0;
    let cpr_lat_odd = cpr_lat_odd as f64 / 131072.0;
    let cpr_lon_odd = cpr_lon_odd as f64 / 131072.0;

    // Latitude zone sizes
    const NZ: f64 = 15.0; // Number of latitude zones
    let d_lat_even = 360.0 / (4.0 * NZ);
    let d_lat_odd = 360.0 / (4.0 * NZ - 1.0);

    // Latitude index
    let j = (59.0 * cpr_lat_even - 60.0 * cpr_lat_odd + 0.5).floor();

    // The latitude step yields a value in 0..360. Wrap both parities into
    // -90..90 straight away: nl() below treats anything at or beyond 87 as a
    // polar zone, so feeding it an unwrapped southern latitude (which arrives
    // here as roughly 300) would silently collapse it to a single zone.
    let wrap = |lat: f64| if lat >= 270.0 { lat - 360.0 } else { lat };
    let lat_even = wrap(d_lat_even * (j.rem_euclid(60.0) + cpr_lat_even));
    let lat_odd = wrap(d_lat_odd * (j.rem_euclid(59.0) + cpr_lat_odd));

    // Pick latitude based on which message was most recent
    let lat = if most_recent_odd { lat_odd } else { lat_even };

    if !(-90.0..=90.0).contains(&lat) {
        return None;
    }

    // Longitude zone count at this latitude
    let nl_even = nl(lat_even);
    let nl_odd = nl(lat_odd);

    // Both frames must be in the same longitude zone
    if nl_even != nl_odd {
        return None;
    }

    let nl = if most_recent_odd { nl_odd } else { nl_even };

    // The two parities divide the globe into a different number of longitude
    // zones: the even frame uses NL, the odd frame NL-1. Using NL for both
    // scales every odd-frame longitude by (NL-1)/NL, which near the antimeridian
    // of the 0..360 frame is an error of several degrees.
    let ni = if most_recent_odd {
        (nl.saturating_sub(1)).max(1)
    } else {
        nl.max(1)
    };
    let d_lon = 360.0 / ni as f64;

    let m = (cpr_lon_even * (nl as f64 - 1.0) - cpr_lon_odd * nl as f64 + 0.5).floor();
    let lon = if most_recent_odd {
        d_lon * (m.rem_euclid(ni as f64) + cpr_lon_odd)
    } else {
        d_lon * (m.rem_euclid(ni as f64) + cpr_lon_even)
    };

    let lon = if lon >= 180.0 { lon - 360.0 } else { lon };

    Some((lat, lon))
}

/// Number of longitude zones at a given latitude.
fn nl(lat: f64) -> u32 {
    if lat.abs() >= 87.0 {
        return 1;
    }
    let nz = 15.0_f64;
    let a = 1.0 - (std::f64::consts::PI / (2.0 * nz)).cos();
    let b = (std::f64::consts::PI / 180.0 * lat).cos().powi(2);
    let nl = (2.0 * std::f64::consts::PI / (1.0 - (a / b)).acos()).floor();
    nl as u32
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_bits_to_u32() {
        let bits = [1, 0, 1, 0, 1, 1, 0, 0];
        assert_eq!(bits_to_u32(&bits, 0, 8), 0xAC);
        assert_eq!(bits_to_u32(&bits, 0, 4), 0x0A);
    }

    #[test]
    fn test_nl_equator() {
        assert_eq!(nl(0.0), 59);
    }

    #[test]
    fn test_nl_poles() {
        assert_eq!(nl(87.0), 1);
        assert_eq!(nl(-87.0), 1);
    }

    /// Assert a decoded position is within ~1 metre of the expected one.
    fn assert_pos(got: Option<(f64, f64)>, want_lat: f64, want_lon: f64, what: &str) {
        let (lat, lon) = got.unwrap_or_else(|| panic!("{what}: decode returned None"));
        assert!(
            (lat - want_lat).abs() < 1e-4 && (lon - want_lon).abs() < 1e-4,
            "{what}: got ({lat:.5}, {lon:.5}), want ({want_lat:.5}, {want_lon:.5})"
        );
    }

    /// The published worked example from the Mode S / ADS-B literature:
    /// even frame 8D40621D58C382D690C8AC2863A7,
    /// odd  frame 8D40621D58C386435CC412692AD6.
    #[test]
    fn cpr_matches_the_published_worked_example() {
        let (lat_e, lon_e) = (93000u32, 51372u32);
        let (lat_o, lon_o) = (74158u32, 50194u32);

        assert_pos(
            decode_cpr_global(lat_e, lon_e, lat_o, lon_o, false),
            52.25720,
            3.91937,
            "even-latest",
        );
        // The odd frame resolves to its own slightly later latitude band. The
        // longitude zone width here is 360/(NL-1), not 360/NL.
        assert_pos(
            decode_cpr_global(lat_e, lon_e, lat_o, lon_o, true),
            52.26578,
            3.93891,
            "odd-latest",
        );
    }

    /// Both parities must land on the same place, whichever arrived last.
    #[test]
    fn cpr_agrees_between_parities() {
        // Encoded from 41.79 N, 76.79 W (Troy, Pennsylvania).
        let (lat_e, lon_e) = (126484u32, 80551u32);
        let (lat_o, lon_o) = (111269u32, 108509u32);

        assert_pos(
            decode_cpr_global(lat_e, lon_e, lat_o, lon_o, false),
            41.78998,
            -76.79000,
            "troy even-latest",
        );
        assert_pos(
            decode_cpr_global(lat_e, lon_e, lat_o, lon_o, true),
            41.78999,
            -76.79003,
            "troy odd-latest",
        );
    }

    /// Southern latitudes come out of the latitude step as values near 300,
    /// which must be wrapped into -90..90 before nl() sees them -- nl() treats
    /// anything at or beyond 87 as a polar zone and returns 1.
    #[test]
    fn cpr_works_in_the_southern_hemisphere() {
        // Encoded from 33.87 S, 151.21 E (Sydney).
        let (lat_e, lon_e) = (46531u32, 76200u32);
        let (lat_o, lon_o) = (58862u32, 21146u32);

        assert_pos(
            decode_cpr_global(lat_e, lon_e, lat_o, lon_o, false),
            -33.86998,
            151.20999,
            "sydney even-latest",
        );
        assert_pos(
            decode_cpr_global(lat_e, lon_e, lat_o, lon_o, true),
            -33.87001,
            151.20998,
            "sydney odd-latest",
        );
    }

    #[test]
    fn test_crc24_known_message() {
        // Known good ADS-B message: 8D4840D6202CC371C32CE0576098
        // As bits, this 112-bit message should have CRC = 0 when valid
        let hex = "8D4840D6202CC371C32CE0576098";
        let bytes: Vec<u8> = (0..hex.len())
            .step_by(2)
            .map(|i| u8::from_str_radix(&hex[i..i + 2], 16).unwrap())
            .collect();

        let mut bits = [0u8; 112];
        for (i, &byte) in bytes.iter().enumerate() {
            for bit in 0..8 {
                bits[i * 8 + bit] = (byte >> (7 - bit)) & 1;
            }
        }

        let result = crc24(&bits, 112);
        assert_eq!(result, 0, "CRC should be 0 for a valid message, got {result:#X}");
    }
}
