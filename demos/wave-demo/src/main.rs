use std::{
    io::stdout,
    time::{Duration, Instant},
};

use crossterm::{
    event::{self, Event, KeyCode},
    execute,
    terminal::{EnterAlternateScreen, LeaveAlternateScreen, disable_raw_mode, enable_raw_mode},
};
use ratatui::{
    Frame, Terminal,
    backend::CrosstermBackend,
    layout::{Constraint, Layout},
    style::{Color, Style},
    symbols::Marker,
    text::{Line, Span},
    widgets::{
        Block, Borders, Paragraph,
        canvas::{Canvas, Line as CanvasLine, Points},
    },
};

const WAVE_SPEED: f64 = 25.0;
const FREQUENCY: f64 = 1.2;
const AMPLITUDE: f64 = 35.0;
const ANTENNA_X_LEFT: f64 = 5.0;
const ANTENNA_X_RIGHT: f64 = 95.0;
const ANTENNA_HEIGHT: f64 = 30.0;
const WAVE_Y_CENTER: f64 = 0.0;
const FPS: u64 = 60;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    enable_raw_mode()?;
    execute!(stdout(), EnterAlternateScreen)?;
    let mut terminal = Terminal::new(CrosstermBackend::new(stdout()))?;

    let start = Instant::now();
    let frame_duration = Duration::from_millis(1000 / FPS);

    loop {
        let t = start.elapsed().as_secs_f64();

        terminal.draw(|frame| render(frame, t))?;

        if event::poll(frame_duration)? {
            if let Event::Key(key) = event::read()? {
                match key.code {
                    KeyCode::Char('q') | KeyCode::Esc => break,
                    _ => {}
                }
            }
        }
    }

    disable_raw_mode()?;
    execute!(stdout(), LeaveAlternateScreen)?;
    Ok(())
}

fn render(frame: &mut Frame, t: f64) {
    let [title_area, canvas_area, footer_area] = Layout::vertical([
        Constraint::Length(2),
        Constraint::Min(10),
        Constraint::Length(2),
    ])
    .areas(frame.area());

    // Title
    let title = Paragraph::new(Line::from(vec![
        Span::styled("  Transmitter", Style::default().fg(Color::Cyan)),
        Span::raw("                                                    "),
        Span::styled("Receiver", Style::default().fg(Color::Green)),
    ]));
    frame.render_widget(title, title_area);

    // Footer
    let footer = Paragraph::new(Line::from(vec![Span::styled(
        "  Electromagnetic wave propagation",
        Style::default().fg(Color::DarkGray),
    )]));
    frame.render_widget(footer, footer_area);

    // Wave canvas
    let omega = 2.0 * std::f64::consts::PI * FREQUENCY;
    let k = omega / WAVE_SPEED;

    // How far the wave has propagated from the transmitter
    let wave_front = ANTENNA_X_LEFT + WAVE_SPEED * t;
    let wave_front_clamped = wave_front.min(ANTENNA_X_RIGHT);

    // Transmitter electron position
    let tx_y = AMPLITUDE * (omega * t).sin();

    // Receiver electron position — only moves once the wave arrives
    let rx_y = if wave_front >= ANTENNA_X_RIGHT {
        AMPLITUDE * (omega * t - k * (ANTENNA_X_RIGHT - ANTENNA_X_LEFT)).sin()
    } else {
        0.0
    };

    let canvas = Canvas::default()
        .block(
            Block::default()
                .borders(Borders::ALL)
                .border_style(Style::default().fg(Color::DarkGray)),
        )
        .x_bounds([0.0, 100.0])
        .y_bounds([-50.0, 50.0])
        .marker(Marker::Braille)
        .paint(move |ctx| {
            // Transmitter antenna (vertical line)
            ctx.draw(&CanvasLine::new(
                ANTENNA_X_LEFT,
                -ANTENNA_HEIGHT,
                ANTENNA_X_LEFT,
                ANTENNA_HEIGHT,
                Color::Cyan,
            ));

            // Transmitter electron (filled circle)
            let tx_pts = filled_circle(ANTENNA_X_LEFT, tx_y, 1.5);
            ctx.draw(&Points {
                coords: &tx_pts,
                color: Color::Cyan,
            });

            // Receiver antenna (vertical line)
            ctx.draw(&CanvasLine::new(
                ANTENNA_X_RIGHT,
                -ANTENNA_HEIGHT,
                ANTENNA_X_RIGHT,
                ANTENNA_HEIGHT,
                Color::Green,
            ));

            // Receiver electron (filled circle)
            let rx_pts = filled_circle(ANTENNA_X_RIGHT, rx_y, 1.5);
            ctx.draw(&Points {
                coords: &rx_pts,
                color: Color::Green,
            });

            // Wave between antennas
            let wave_start = ANTENNA_X_LEFT + 1.0;
            let wave_end = wave_front_clamped - 1.0;

            if wave_end > wave_start {
                // Draw wave as connected line segments
                let num_points = 300;
                let dx = (wave_end - wave_start) / num_points as f64;

                let mut prev_x = wave_start;
                let mut prev_y =
                    WAVE_Y_CENTER + AMPLITUDE * (omega * t - k * (wave_start - ANTENNA_X_LEFT)).sin();

                for i in 1..=num_points {
                    let x = wave_start + dx * i as f64;
                    let y = WAVE_Y_CENTER
                        + AMPLITUDE * (omega * t - k * (x - ANTENNA_X_LEFT)).sin();

                    // Fade color near the wave front for a natural look
                    let dist_from_front = wave_end - x;
                    let color = if dist_from_front < 5.0 && wave_front < ANTENNA_X_RIGHT {
                        Color::DarkGray
                    } else {
                        Color::Yellow
                    };

                    ctx.draw(&CanvasLine::new(prev_x, prev_y, x, y, color));

                    prev_x = x;
                    prev_y = y;
                }
            }
        });

    frame.render_widget(canvas, canvas_area);
}

/// Generate points filling a circle at (cx, cy) with the given radius.
/// The radius is in canvas coordinate units. Braille markers are dense
/// enough that stepping by 0.2 units gives a solid fill.
fn filled_circle(cx: f64, cy: f64, radius: f64) -> Vec<(f64, f64)> {
    let mut pts = Vec::new();
    let step = 0.2;
    let mut dy = -radius;
    while dy <= radius {
        let mut dx = -radius;
        while dx <= radius {
            if dx * dx + dy * dy <= radius * radius {
                pts.push((cx + dx, cy + dy));
            }
            dx += step;
        }
        dy += step;
    }
    pts
}
