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
mod ui;

#[derive(Parser)]
#[command(name = "flight-tracker", about = "Live ADS-B flight tracker map")]
struct Args {
    /// Path to the ADS-B SQLite database
    #[arg(long, default_value = "adsb.db")]
    db: String,

    /// Database poll interval in milliseconds
    #[arg(long, default_value_t = 1000)]
    refresh: u64,
}

fn main() -> Result<()> {
    let args = Args::parse();

    let db = db::TrackerDb::open(&args.db)?;
    let mut app = app::App::new(db);

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
