//! Map regions.
//!
//! The tracker used to hard-code Ottawa's viewport, database filter, and river
//! geometry. That silently breaks at any other venue: aircraft outside the
//! longitude window are dropped by the SQL query, so the map renders empty with
//! no error. A region bundles all three so moving cities is one flag.

/// A water feature: a polyline in (lon, lat) order.
pub struct Water {
    pub points: &'static [(f64, f64)],
    /// Major channels draw bright; tributaries draw dim.
    pub major: bool,
}

/// A place name printed on the map.
pub struct Label {
    pub lon: f64,
    pub lat: f64,
    pub text: &'static str,
    /// Landmarks (airports) draw highlighted.
    pub landmark: bool,
}

pub struct Region {
    pub name: &'static str,
    pub lat_min: f64,
    pub lat_max: f64,
    pub lon_min: f64,
    pub lon_max: f64,
    pub water: &'static [Water],
    pub labels: &'static [Label],
}

/// Extra degrees allowed past the viewport in the SQL filter. The query is a
/// sanity net against CPR decode glitches, not the viewport clip — `App::tick`
/// does the tight clipping. Keeping it a little loose avoids dropping an
/// aircraft that is only just off-screen.
///
/// Kept at 0.5 so neighbouring regions do not overlap: at 1.0, Ottawa's window
/// reached east to -73.5 and swallowed downtown Montreal at -73.57.
const DB_MARGIN_DEG: f64 = 0.5;

impl Region {
    /// Bounds for the database query: the viewport plus a margin.
    pub fn db_bounds(&self) -> (f64, f64, f64, f64) {
        (
            self.lat_min - DB_MARGIN_DEG,
            self.lat_max + DB_MARGIN_DEG,
            self.lon_min - DB_MARGIN_DEG,
            self.lon_max + DB_MARGIN_DEG,
        )
    }

    /// Look up a built-in region by name.
    pub fn by_name(name: &str) -> Option<&'static Region> {
        match name.to_ascii_lowercase().as_str() {
            "ottawa" => Some(&OTTAWA),
            "montreal" | "montréal" => Some(&MONTREAL),
            _ => None,
        }
    }

    pub fn names() -> &'static [&'static str] {
        &["ottawa", "montreal"]
    }

    /// An arbitrary viewport with no built-in geography — aircraft and trails
    /// only. Leaked deliberately: it lives for the whole process, and this
    /// keeps `App` and the canvas closure free of lifetime plumbing.
    pub fn custom(lat_min: f64, lat_max: f64, lon_min: f64, lon_max: f64) -> &'static Region {
        Box::leak(Box::new(Region {
            name: Box::leak(
                format!("{lat_min:.2},{lon_min:.2} to {lat_max:.2},{lon_max:.2}")
                    .into_boxed_str(),
            ),
            lat_min,
            lat_max,
            lon_min,
            lon_max,
            water: &[],
            labels: &[],
        }))
    }
}

// ============================================================================
// Ottawa — traced against a map, landmarks verified.
// ============================================================================

pub static OTTAWA: Region = Region {
    name: "Ottawa Area",
    lat_min: 44.8,
    lat_max: 45.8,
    lon_min: -76.5,
    lon_max: -74.5,
    water: &[
        Water {
            major: true,
            // Ottawa River, main channel
            points: &[
                (-76.35, 45.49),
                (-76.20, 45.49),
                (-76.05, 45.47),
                (-75.95, 45.47), // Fitzroy Harbour
                (-75.90, 45.43), // Lac Deschênes west
                (-75.87, 45.42),
                (-75.84, 45.41),
                (-75.82, 45.40),
                (-75.80, 45.39),
                (-75.78, 45.39), // Lac Deschênes east
                (-75.76, 45.39),
                (-75.74, 45.39), // Aylmer
                (-75.72, 45.39),
                (-75.70, 45.40), // Champlain Bridge
                (-75.68, 45.40),
                (-75.66, 45.41),
                (-75.64, 45.42), // Remic Rapids
                (-75.62, 45.42),
                (-75.60, 45.42), // Lemieux Island
                (-75.58, 45.42), // Chaudière Falls
                (-75.56, 45.43),
                (-75.54, 45.43), // Parliament
                (-75.52, 45.43),
                (-75.50, 45.44), // Rideau Falls
                (-75.48, 45.44),
                (-75.46, 45.45), // Rockcliffe
                (-75.44, 45.46),
                (-75.42, 45.47),
                (-75.38, 45.48),
                (-75.34, 45.49),
                (-75.30, 45.50),
                (-75.25, 45.51), // Petrie Island
                (-75.20, 45.52),
                (-75.10, 45.53), // Cumberland
                (-75.00, 45.54),
                (-74.90, 45.55),
                (-74.80, 45.56),
                (-74.70, 45.57),
                (-74.60, 45.58),
                (-74.50, 45.58),
            ],
        },
        Water {
            major: false,
            // Lac Deschênes north shore
            points: &[
                (-75.92, 45.45),
                (-75.89, 45.44),
                (-75.86, 45.43),
                (-75.84, 45.43),
                (-75.82, 45.42),
                (-75.80, 45.42),
                (-75.78, 45.41),
                (-75.76, 45.41),
                (-75.74, 45.41),
            ],
        },
        Water {
            major: false,
            // Lac Deschênes south shore
            points: &[
                (-75.92, 45.40),
                (-75.89, 45.39),
                (-75.86, 45.38),
                (-75.84, 45.37),
                (-75.82, 45.37),
                (-75.80, 45.36),
                (-75.78, 45.37),
                (-75.76, 45.37),
                (-75.74, 45.38),
            ],
        },
        Water {
            major: false,
            // Rideau River
            points: &[
                (-75.73, 44.80),
                (-75.72, 44.88),
                (-75.71, 44.95),
                (-75.69, 45.02), // Manotick
                (-75.68, 45.08),
                (-75.68, 45.14),
                (-75.69, 45.20), // Hunt Club
                (-75.69, 45.25), // Mooney's Bay
                (-75.69, 45.28), // Hog's Back Falls
                (-75.68, 45.30), // Carleton U
                (-75.68, 45.33),
                (-75.69, 45.36), // Lansdowne
                (-75.69, 45.38),
                (-75.70, 45.41),
                (-75.70, 45.43), // Rideau Falls -> Ottawa River
            ],
        },
        Water {
            major: false,
            // Rideau Canal, downtown section
            points: &[
                (-75.69, 45.39), // Dow's Lake
                (-75.69, 45.40),
                (-75.69, 45.41), // Glebe
                (-75.69, 45.42),
                (-75.70, 45.43),
                (-75.70, 45.44), // Parliament locks
            ],
        },
        Water {
            major: false,
            // Gatineau River
            points: &[
                (-75.75, 45.80),
                (-75.74, 45.72),
                (-75.73, 45.65),
                (-75.72, 45.60),
                (-75.71, 45.56),
                (-75.70, 45.52),
                (-75.68, 45.50),
                (-75.64, 45.48),
                (-75.61, 45.46),
                (-75.58, 45.44), // Joins Ottawa River
            ],
        },
    ],
    labels: &[
        Label { lon: -75.67, lat: 45.32, text: "✦ YOW", landmark: true },
        Label { lon: -75.70, lat: 45.43, text: "▪ Parliament", landmark: false },
        Label { lon: -75.70, lat: 45.48, text: "Gatineau", landmark: false },
        Label { lon: -75.90, lat: 45.35, text: "Kanata", landmark: false },
        Label { lon: -75.52, lat: 45.47, text: "Orléans", landmark: false },
        Label { lon: -75.73, lat: 45.28, text: "Barrhaven", landmark: false },
    ],
};

// ============================================================================
// Montreal
//
// NOTE: this geometry is APPROXIMATE. Unlike the Ottawa outline it was not
// traced point-by-point against a map — it is a recognisable sketch of the
// St. Lawrence and Rivière des Prairies around the island. Aircraft positions
// are unaffected (those are real ADS-B), but check the rivers against a map
// before showing this on stage if the shape matters to you.
// ============================================================================

pub static MONTREAL: Region = Region {
    name: "Montreal Area",
    lat_min: 45.0,
    lat_max: 46.0,
    lon_min: -74.6,
    lon_max: -72.6,
    water: &[
        Water {
            major: true,
            // St. Lawrence River, main channel, flowing NE
            points: &[
                (-74.20, 45.30),
                (-74.10, 45.32),
                (-74.00, 45.35), // Lac Saint-Louis west
                (-73.92, 45.38),
                (-73.85, 45.40),
                (-73.80, 45.41),
                (-73.75, 45.42), // Lachine
                (-73.70, 45.43),
                (-73.65, 45.44), // Lachine Rapids
                (-73.60, 45.46),
                (-73.57, 45.48), // Victoria Bridge
                (-73.54, 45.50), // Old Port / downtown
                (-73.52, 45.52), // Île Sainte-Hélène
                (-73.50, 45.55),
                (-73.47, 45.58),
                (-73.44, 45.61),
                (-73.40, 45.64),
                (-73.35, 45.67), // Repentigny
                (-73.28, 45.70),
                (-73.20, 45.73),
            ],
        },
        Water {
            major: false,
            // Rivière des Prairies, north of the island
            points: &[
                (-74.00, 45.44),
                (-73.95, 45.46),
                (-73.90, 45.48),
                (-73.85, 45.50),
                (-73.80, 45.53),
                (-73.75, 45.56),
                (-73.70, 45.58),
                (-73.65, 45.61),
                (-73.60, 45.63),
                (-73.55, 45.66),
                (-73.50, 45.68),
                (-73.45, 45.70),
            ],
        },
        Water {
            major: false,
            // Lac des Deux Montagnes / Rivière des Outaouais approach
            points: &[
                (-74.40, 45.50),
                (-74.30, 45.49),
                (-74.20, 45.48),
                (-74.12, 45.46),
                (-74.05, 45.45),
                (-74.00, 45.44),
            ],
        },
        Water {
            major: false,
            // Richelieu River, joining from the south
            points: &[
                (-73.28, 45.10),
                (-73.27, 45.20),
                (-73.26, 45.30),
                (-73.25, 45.40),
                (-73.24, 45.50),
                (-73.25, 45.60),
                (-73.28, 45.70),
            ],
        },
    ],
    labels: &[
        Label { lon: -73.74, lat: 45.47, text: "✦ YUL", landmark: true },
        Label { lon: -73.57, lat: 45.50, text: "▪ Downtown", landmark: false },
        Label { lon: -73.55, lat: 45.63, text: "Montréal-Nord", landmark: false },
        Label { lon: -73.95, lat: 45.42, text: "Lachine", landmark: false },
        Label { lon: -73.47, lat: 45.53, text: "Longueuil", landmark: false },
        Label { lon: -73.85, lat: 45.62, text: "Laval", landmark: false },
    ],
};
