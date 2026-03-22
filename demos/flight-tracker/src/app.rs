use crate::db::{AircraftRow, Position, TrackerDb};
use anyhow::Result;

pub struct App {
    pub aircraft: Vec<AircraftRow>,
    pub trails: Vec<Vec<Position>>,
    pub selected: usize,
    pub running: bool,
    pub db: TrackerDb,
    // Map viewport (Ottawa area)
    pub lon_min: f64,
    pub lon_max: f64,
    pub lat_min: f64,
    pub lat_max: f64,
}

impl App {
    pub fn new(db: TrackerDb) -> Self {
        Self {
            aircraft: Vec::new(),
            trails: Vec::new(),
            selected: 0,
            running: true,
            db,
            lon_min: -76.5,
            lon_max: -74.5,
            lat_min: 44.8,
            lat_max: 45.8,
        }
    }

    /// Refresh data from the database.
    pub fn tick(&mut self) -> Result<()> {
        self.aircraft = self.db.load_aircraft(1800)?; // last 30 minutes

        // Clear positions that are outside the map viewport (CPR glitches)
        for ac in &mut self.aircraft {
            if let (Some(lat), Some(lon)) = (ac.lat, ac.lon) {
                if lat < self.lat_min || lat > self.lat_max
                    || lon < self.lon_min || lon > self.lon_max
                {
                    ac.lat = None;
                    ac.lon = None;
                }
            }
        }

        // Clamp selection
        if !self.aircraft.is_empty() && self.selected >= self.aircraft.len() {
            self.selected = self.aircraft.len() - 1;
        }

        // Load trails for all aircraft with positions
        self.trails.clear();
        for (i, ac) in self.aircraft.iter().enumerate() {
            if ac.lat.is_some() && ac.lon.is_some() {
                let limit = if i == self.selected { 20 } else { 5 };
                let trail = self.db.load_trail(&ac.icao, limit)?;
                self.trails.push(trail);
            } else {
                self.trails.push(Vec::new());
            }
        }

        Ok(())
    }

    pub fn select_next(&mut self) {
        if !self.aircraft.is_empty() {
            self.selected = (self.selected + 1).min(self.aircraft.len() - 1);
        }
    }

    pub fn select_prev(&mut self) {
        self.selected = self.selected.saturating_sub(1);
    }

    pub fn quit(&mut self) {
        self.running = false;
    }
}
