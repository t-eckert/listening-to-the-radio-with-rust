use anyhow::Result;
use clap::Parser;
use crossterm::{
    event::{self, Event, KeyCode, KeyEventKind},
    terminal::{EnterAlternateScreen, LeaveAlternateScreen, disable_raw_mode, enable_raw_mode},
};
use ratatui::prelude::CrosstermBackend;
use std::io;
use std::time::{Duration, Instant};

mod app;
mod db;
mod region;
mod ui;

use region::Region;

#[derive(Parser)]
#[command(name = "flight-tracker", about = "Live ADS-B flight tracker map")]
struct Args {
    /// Path to the ADS-B SQLite database
    #[arg(long, default_value = "adsb.db")]
    db: String,

    /// Database poll interval in milliseconds
    #[arg(long, default_value_t = 1000)]
    refresh: u64,

    /// Map region: ottawa or montreal
    #[arg(long, default_value = "ottawa")]
    region: String,

    /// Custom viewport as LAT_MIN,LAT_MAX,LON_MIN,LON_MAX. Overrides --region
    /// and draws no coastline — use it at a venue with no built-in region.
    #[arg(long)]
    bounds: Option<String>,
}

/// Parse `LAT_MIN,LAT_MAX,LON_MIN,LON_MAX`.
fn parse_bounds(s: &str) -> Result<(f64, f64, f64, f64)> {
    let parts: Vec<&str> = s.split(',').map(str::trim).collect();
    if parts.len() != 4 {
        anyhow::bail!("expected 4 comma-separated values, got {}", parts.len());
    }
    let v: Vec<f64> = parts
        .iter()
        .map(|p| p.parse::<f64>().map_err(|e| anyhow::anyhow!("{p:?}: {e}")))
        .collect::<Result<_>>()?;
    let (lat_min, lat_max, lon_min, lon_max) = (v[0], v[1], v[2], v[3]);
    if lat_min >= lat_max {
        anyhow::bail!("LAT_MIN ({lat_min}) must be less than LAT_MAX ({lat_max})");
    }
    if lon_min >= lon_max {
        anyhow::bail!("LON_MIN ({lon_min}) must be less than LON_MAX ({lon_max})");
    }
    Ok((lat_min, lat_max, lon_min, lon_max))
}

fn main() -> Result<()> {
    let args = Args::parse();

    // Resolve the region before touching the terminal, so a bad argument prints
    // a normal error instead of a garbled one from inside the alternate screen.
    let region: &'static Region = match &args.bounds {
        Some(s) => {
            let (lat_min, lat_max, lon_min, lon_max) = parse_bounds(s)
                .map_err(|e| anyhow::anyhow!("--bounds {s:?}: {e}"))?;
            Region::custom(lat_min, lat_max, lon_min, lon_max)
        }
        None => Region::by_name(&args.region).ok_or_else(|| {
            anyhow::anyhow!(
                "unknown region {:?}; known regions: {}. Use --bounds for anywhere else.",
                args.region,
                Region::names().join(", ")
            )
        })?,
    };

    let db = db::TrackerDb::open(&args.db)?;
    let mut app = app::App::new(db, region);

    // Initial data load
    app.tick()?;

    // Terminal setup
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    crossterm::execute!(stdout, EnterAlternateScreen)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = ratatui::Terminal::new(backend)?;

    let refresh = Duration::from_millis(args.refresh);
    let mut last_tick = Instant::now();

    while app.running {
        terminal.draw(|frame| ui::render(frame, &app))?;

        // Wait for events up to the remaining tick time
        let timeout = refresh.saturating_sub(last_tick.elapsed());
        if event::poll(timeout)? {
            if let Event::Key(key) = event::read()? {
                if key.kind == KeyEventKind::Press {
                    match key.code {
                        KeyCode::Char('q') | KeyCode::Esc => app.quit(),
                        KeyCode::Down | KeyCode::Char('j') => app.select_next(),
                        KeyCode::Up | KeyCode::Char('k') => app.select_prev(),
                        _ => {}
                    }
                }
            }
        }

        // Poll database on tick
        if last_tick.elapsed() >= refresh {
            app.tick()?;
            last_tick = Instant::now();
        }
    }

    // Restore terminal
    disable_raw_mode()?;
    crossterm::execute!(terminal.backend_mut(), LeaveAlternateScreen)?;

    Ok(())
}
