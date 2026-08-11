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
    pub fn load_aircraft(
        &self,
        max_age_secs: i64,
        lat_min: f64,
        lat_max: f64,
        lon_min: f64,
        lon_max: f64,
    ) -> Result<Vec<AircraftRow>> {
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
                rusqlite::params![age_param, lat_min, lat_max, lon_min, lon_max],
                |row| {
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
                },
            )?
            .collect::<std::result::Result<Vec<_>, _>>()?;

        Ok(rows)
    }

    /// Load the last `limit` positions for a given aircraft ICAO.
    pub fn load_trail(
        &self,
        icao: &str,
        limit: usize,
        lat_min: f64,
        lat_max: f64,
        lon_min: f64,
        lon_max: f64,
    ) -> Result<Vec<Position>> {
        let mut stmt = self.conn.prepare_cached(
            "SELECT latitude, longitude
             FROM positions
             WHERE icao = ?1
               AND latitude BETWEEN ?3 AND ?4
               AND longitude BETWEEN ?5 AND ?6
             ORDER BY id DESC
             LIMIT ?2",
        )?;

        let rows = stmt
            .query_map(
                rusqlite::params![icao, limit as i64, lat_min, lat_max, lon_min, lon_max],
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
    use crate::region::{MONTREAL, OTTAWA};

    /// Build a throwaway database with one aircraft over Ottawa and one over
    /// Montreal, each with a position.
    fn fixture(path: &std::path::Path) {
        let conn = Connection::open(path).unwrap();
        conn.execute_batch(
            "CREATE TABLE aircraft (
                icao TEXT PRIMARY KEY,
                callsign TEXT,
                first_seen TEXT NOT NULL DEFAULT (datetime('now')),
                last_seen TEXT NOT NULL DEFAULT (datetime('now')),
                messages INTEGER NOT NULL DEFAULT 0
             );
             CREATE TABLE positions (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                icao TEXT NOT NULL,
                timestamp TEXT NOT NULL DEFAULT (datetime('now')),
                latitude REAL NOT NULL,
                longitude REAL NOT NULL,
                altitude_ft INTEGER,
                ground_speed_kt INTEGER,
                vertical_rate_fpm INTEGER,
                heading REAL
             );
             INSERT INTO aircraft (icao, callsign, messages) VALUES ('AAA111', 'OTT001', 10);
             INSERT INTO aircraft (icao, callsign, messages) VALUES ('BBB222', 'MTL001', 10);
             -- Over Ottawa, and over Montreal.
             INSERT INTO positions (icao, latitude, longitude) VALUES ('AAA111', 45.35, -75.70);
             INSERT INTO positions (icao, latitude, longitude) VALUES ('BBB222', 45.50, -73.57);",
        )
        .unwrap();
    }

    /// `name` keeps concurrently-running tests off each other's database file.
    fn with_db<T>(name: &str, f: impl FnOnce(&TrackerDb) -> T) -> T {
        let dir = std::env::temp_dir().join(format!("ft-test-{}-{name}", std::process::id()));
        let _ = std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(&dir).unwrap();
        let path = dir.join("test.db");
        fixture(&path);
        let db = TrackerDb::open(path.to_str().unwrap()).unwrap();
        let out = f(&db);
        let _ = std::fs::remove_dir_all(&dir);
        out
    }

    /// The bug this guards: bounds used to be hard-coded to Ottawa, so an
    /// aircraft over Montreal silently lost its position and vanished from the
    /// map with no error.
    #[test]
    fn bounds_select_the_right_aircraft() {
        with_db("bounds", |db| {
            let (a, b, c, d) = OTTAWA.db_bounds();
            let ottawa = db.load_aircraft(86_400, a, b, c, d).unwrap();
            let positioned: Vec<_> = ottawa
                .iter()
                .filter(|r| r.lat.is_some())
                .map(|r| r.callsign.clone().unwrap())
                .collect();
            assert_eq!(positioned, vec!["OTT001"], "Ottawa bounds");

            let (a, b, c, d) = MONTREAL.db_bounds();
            let montreal = db.load_aircraft(86_400, a, b, c, d).unwrap();
            let positioned: Vec<_> = montreal
                .iter()
                .filter(|r| r.lat.is_some())
                .map(|r| r.callsign.clone().unwrap())
                .collect();
            assert_eq!(positioned, vec!["MTL001"], "Montreal bounds");
        });
    }

    #[test]
    fn trail_respects_bounds() {
        with_db("trail", |db| {
            let (a, b, c, d) = MONTREAL.db_bounds();
            assert_eq!(db.load_trail("BBB222", 10, a, b, c, d).unwrap().len(), 1);
            // The Ottawa aircraft has no position inside Montreal's window.
            assert_eq!(db.load_trail("AAA111", 10, a, b, c, d).unwrap().len(), 0);
        });
    }
}
