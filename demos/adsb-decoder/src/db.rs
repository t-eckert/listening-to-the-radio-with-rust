use anyhow::{Context, Result};
use rusqlite::Connection;

pub struct AdsbDb {
    conn: Connection,
}

impl AdsbDb {
    pub fn open(path: &str) -> Result<Self> {
        let conn = Connection::open(path)
            .with_context(|| format!("opening database at {path}"))?;

        conn.execute_batch("PRAGMA journal_mode=WAL; PRAGMA synchronous=NORMAL;")?;

        conn.execute_batch(
            "
            CREATE TABLE IF NOT EXISTS aircraft (
                icao TEXT PRIMARY KEY,
                callsign TEXT,
                first_seen TEXT NOT NULL DEFAULT (datetime('now')),
                last_seen TEXT NOT NULL DEFAULT (datetime('now')),
                messages INTEGER NOT NULL DEFAULT 0
            );

            CREATE TABLE IF NOT EXISTS positions (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                icao TEXT NOT NULL,
                timestamp TEXT NOT NULL DEFAULT (datetime('now')),
                latitude REAL NOT NULL,
                longitude REAL NOT NULL,
                altitude_ft INTEGER,
                ground_speed_kt INTEGER,
                vertical_rate_fpm INTEGER,
                heading REAL,
                FOREIGN KEY (icao) REFERENCES aircraft(icao)
            );

            CREATE INDEX IF NOT EXISTS idx_positions_icao ON positions(icao);
            CREATE INDEX IF NOT EXISTS idx_positions_timestamp ON positions(timestamp);
            ",
        )
        .context("creating database tables")?;

        Ok(Self { conn })
    }

    pub fn update_aircraft(&self, icao: u32, callsign: Option<&str>, messages: u32) -> Result<()> {
        let icao_hex = format!("{icao:06X}");

        self.conn.execute(
            "INSERT INTO aircraft (icao, callsign, messages)
             VALUES (?1, ?2, ?3)
             ON CONFLICT(icao) DO UPDATE SET
                callsign = COALESCE(?2, aircraft.callsign),
                last_seen = datetime('now'),
                messages = ?3",
            rusqlite::params![icao_hex, callsign, messages],
        )?;

        Ok(())
    }

    pub fn insert_position(
        &self,
        icao: u32,
        lat: f64,
        lon: f64,
        altitude_ft: Option<i32>,
        ground_speed_kt: Option<u16>,
        vertical_rate_fpm: Option<i32>,
        heading: Option<f32>,
    ) -> Result<()> {
        let icao_hex = format!("{icao:06X}");

        self.conn.execute(
            "INSERT INTO positions (icao, latitude, longitude, altitude_ft, ground_speed_kt, vertical_rate_fpm, heading)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)",
            rusqlite::params![
                icao_hex,
                lat,
                lon,
                altitude_ft,
                ground_speed_kt.map(|v| v as i32),
                vertical_rate_fpm,
                heading,
            ],
        )?;

        Ok(())
    }
}
