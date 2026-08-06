use anyhow::{Context, Result};
use rusqlite::Connection;
use serde::Serialize;

/// Geographic filter applied to position rows.
///
/// `adsb-decoder` stores every position it decodes, including ones mangled by a
/// bad CPR solve. Clamping to a plausible box drops those. The default is the
/// whole planet so the API works wherever the receiver is; narrow it with
/// `--bbox` when you know the receiver's range.
#[derive(Debug, Clone, Copy)]
pub struct BoundingBox {
    pub lat_min: f64,
    pub lat_max: f64,
    pub lon_min: f64,
    pub lon_max: f64,
}

impl BoundingBox {
    pub const WORLD: Self = Self {
        lat_min: -90.0,
        lat_max: 90.0,
        lon_min: -180.0,
        lon_max: 180.0,
    };
}

impl Default for BoundingBox {
    fn default() -> Self {
        Self::WORLD
    }
}

impl std::str::FromStr for BoundingBox {
    type Err = anyhow::Error;

    /// Parses `lat_min,lat_max,lon_min,lon_max` — e.g. Ottawa is
    /// `44.0,46.5,-77.5,-74.0`.
    fn from_str(s: &str) -> Result<Self> {
        let parts: Vec<f64> = s
            .split(',')
            .map(|p| p.trim().parse::<f64>())
            .collect::<std::result::Result<_, _>>()
            .context("bbox values must be numbers")?;

        let [lat_min, lat_max, lon_min, lon_max] = parts.as_slice() else {
            anyhow::bail!("expected 4 comma-separated values: lat_min,lat_max,lon_min,lon_max");
        };

        anyhow::ensure!(lat_min < lat_max, "lat_min must be less than lat_max");
        anyhow::ensure!(lon_min < lon_max, "lon_min must be less than lon_max");

        Ok(Self {
            lat_min: *lat_min,
            lat_max: *lat_max,
            lon_min: *lon_min,
            lon_max: *lon_max,
        })
    }
}

#[derive(Debug, Clone, Serialize)]
pub struct Aircraft {
    pub icao: String,
    pub callsign: Option<String>,
    pub altitude_ft: Option<i32>,
    pub ground_speed_kt: Option<i32>,
    pub vertical_rate_fpm: Option<i32>,
    pub heading: Option<f64>,
    pub lat: Option<f64>,
    pub lon: Option<f64>,
    pub messages: i32,
    pub last_seen: String,
}

#[derive(Debug, Clone, Copy, Serialize)]
pub struct Position {
    pub lat: f64,
    pub lon: f64,
}

pub struct ApiDb {
    conn: Connection,
    bbox: BoundingBox,
}

impl ApiDb {
    /// Opens `adsb-decoder`'s database read-only.
    ///
    /// The decoder is the only writer; this process must never take a write
    /// lock on it. Mirrors `flight-tracker`'s `TrackerDb::open`.
    pub fn open(path: &str, bbox: BoundingBox) -> Result<Self> {
        let conn = Connection::open_with_flags(
            path,
            rusqlite::OpenFlags::SQLITE_OPEN_READ_ONLY | rusqlite::OpenFlags::SQLITE_OPEN_NO_MUTEX,
        )
        .with_context(|| format!("opening database at {path}"))?;

        conn.execute_batch("PRAGMA query_only = true;")
            .context("setting read-only pragma")?;

        Ok(Self { conn, bbox })
    }

    /// Cheap liveness probe that proves the database is still readable.
    pub fn ping(&self) -> Result<i64> {
        let count = self
            .conn
            .query_row("SELECT COUNT(*) FROM aircraft", [], |row| row.get(0))
            .context("counting aircraft")?;
        Ok(count)
    }

    /// Every aircraft seen in the last `max_age_secs` seconds, joined with its
    /// most recent in-bounds position.
    pub fn load_aircraft(&self, max_age_secs: i64) -> Result<Vec<Aircraft>> {
        let mut stmt = self.conn.prepare_cached(
            "SELECT
                a.icao,
                a.callsign,
                a.messages,
                a.last_seen,
                p.latitude,
                p.longitude,
                p.altitude_ft,
                p.ground_speed_kt,
                p.vertical_rate_fpm,
                p.heading
            FROM aircraft a
            LEFT JOIN positions p ON p.icao = a.icao
                AND p.id = (
                    SELECT MAX(id) FROM positions
                    WHERE icao = a.icao
                      AND latitude BETWEEN ?2 AND ?3
                      AND longitude BETWEEN ?4 AND ?5
                )
            WHERE a.last_seen >= datetime('now', ?1)
            ORDER BY
                CASE WHEN p.latitude IS NOT NULL THEN 0 ELSE 1 END,
                a.messages DESC",
        )?;

        let age_param = format!("-{max_age_secs} seconds");
        let rows = stmt
            .query_map(
                rusqlite::params![
                    age_param,
                    self.bbox.lat_min,
                    self.bbox.lat_max,
                    self.bbox.lon_min,
                    self.bbox.lon_max,
                ],
                |row| {
                    Ok(Aircraft {
                        icao: row.get(0)?,
                        callsign: row.get(1)?,
                        messages: row.get(2)?,
                        last_seen: row.get(3)?,
                        lat: row.get(4)?,
                        lon: row.get(5)?,
                        altitude_ft: row.get(6)?,
                        ground_speed_kt: row.get(7)?,
                        vertical_rate_fpm: row.get(8)?,
                        heading: row.get(9)?,
                    })
                },
            )?
            .collect::<std::result::Result<Vec<_>, _>>()?;

        Ok(rows)
    }

    /// The last `limit` in-bounds positions for one aircraft, oldest first so
    /// the client can draw the trail without reversing it.
    pub fn load_trail(&self, icao: &str, limit: usize) -> Result<Vec<Position>> {
        let mut stmt = self.conn.prepare_cached(
            "SELECT latitude, longitude FROM (
                 SELECT id, latitude, longitude
                 FROM positions
                 WHERE icao = ?1
                   AND latitude BETWEEN ?3 AND ?4
                   AND longitude BETWEEN ?5 AND ?6
                 ORDER BY id DESC
                 LIMIT ?2
             ) ORDER BY id ASC",
        )?;

        let rows = stmt
            .query_map(
                rusqlite::params![
                    icao,
                    limit as i64,
                    self.bbox.lat_min,
                    self.bbox.lat_max,
                    self.bbox.lon_min,
                    self.bbox.lon_max,
                ],
                |row| {
                    Ok(Position {
                        lat: row.get(0)?,
                        lon: row.get(1)?,
                    })
                },
            )?
            .collect::<std::result::Result<Vec<_>, _>>()?;

        Ok(rows)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_a_bbox() {
        let bbox: BoundingBox = "44.0,46.5,-77.5,-74.0".parse().unwrap();
        assert_eq!(bbox.lat_min, 44.0);
        assert_eq!(bbox.lat_max, 46.5);
        assert_eq!(bbox.lon_min, -77.5);
        assert_eq!(bbox.lon_max, -74.0);
    }

    #[test]
    fn rejects_inverted_bounds() {
        assert!("46.5,44.0,-77.5,-74.0".parse::<BoundingBox>().is_err());
    }

    #[test]
    fn rejects_wrong_arity() {
        assert!("44.0,46.5,-77.5".parse::<BoundingBox>().is_err());
    }

    #[test]
    fn default_spans_the_world() {
        let bbox = BoundingBox::default();
        assert_eq!(bbox.lat_min, -90.0);
        assert_eq!(bbox.lon_max, 180.0);
    }
}
