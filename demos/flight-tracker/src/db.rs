use anyhow::{Context, Result};
use rusqlite::Connection;

pub struct TrackerDb {
    conn: Connection,
}

#[derive(Debug, Clone)]
pub struct AircraftRow {
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

#[derive(Debug, Clone, Copy)]
pub struct Position {
    pub lat: f64,
    pub lon: f64,
}

impl TrackerDb {
    pub fn open(path: &str) -> Result<Self> {
        let conn = Connection::open_with_flags(
            path,
            rusqlite::OpenFlags::SQLITE_OPEN_READ_ONLY | rusqlite::OpenFlags::SQLITE_OPEN_NO_MUTEX,
        )
        .with_context(|| format!("opening database at {path}"))?;

        conn.execute_batch("PRAGMA query_only = true;")
            .context("setting read-only pragma")?;

        Ok(Self { conn })
    }

    /// Load all aircraft seen in the last `max_age_secs` seconds,
    /// joined with their most recent position.
    pub fn load_aircraft(&self, max_age_secs: i64) -> Result<Vec<AircraftRow>> {
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
                      AND latitude BETWEEN 44.0 AND 46.5
                      AND longitude BETWEEN -77.5 AND -74.0
                )
            WHERE a.last_seen >= datetime('now', ?1)
            ORDER BY
                CASE WHEN p.latitude IS NOT NULL THEN 0 ELSE 1 END,
                a.messages DESC",
        )?;

        let age_param = format!("-{max_age_secs} seconds");
        let rows = stmt
            .query_map([&age_param], |row| {
                Ok(AircraftRow {
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
            })?
            .collect::<std::result::Result<Vec<_>, _>>()?;

        Ok(rows)
    }

    /// Load the last `limit` positions for a given aircraft ICAO.
    pub fn load_trail(&self, icao: &str, limit: usize) -> Result<Vec<Position>> {
        let mut stmt = self.conn.prepare_cached(
            "SELECT latitude, longitude
             FROM positions
             WHERE icao = ?1
               AND latitude BETWEEN 44.0 AND 46.5
               AND longitude BETWEEN -77.5 AND -74.0
             ORDER BY id DESC
             LIMIT ?2",
        )?;

        let rows = stmt
            .query_map(rusqlite::params![icao, limit as i64], |row| {
                Ok(Position {
                    lat: row.get(0)?,
                    lon: row.get(1)?,
                })
            })?
            .collect::<std::result::Result<Vec<_>, _>>()?;

        Ok(rows)
    }
}
