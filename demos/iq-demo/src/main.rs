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
        canvas::{Canvas, Circle, Line as CanvasLine, Points},
    },
};

const FPS: u64 = 60;
const HISTORY_SECONDS: f64 = 5.0;

const FREQ_MIN: f64 = 0.1;
const FREQ_MAX: f64 = 2.0;
const FREQ_STEP: f64 = 0.1;

const AMP_MIN: f64 = 0.1;
const AMP_MAX: f64 = 1.0;
const AMP_STEP: f64 = 0.1;

struct State {
    frequency: f64,
    amplitude: f64,
    /// Accumulated phase in radians — ensures continuity when frequency changes.
    phase: f64,
    last_t: f64,
}

impl State {
    fn new() -> Self {
        Self {
            frequency: 0.4,
            amplitude: 0.8,
            phase: 0.0,
            last_t: 0.0,
        }
    }

    fn update(&mut self, t: f64) {
        let dt = t - self.last_t;
        self.phase += 2.0 * std::f64::consts::PI * self.frequency * dt;
        self.last_t = t;
    }

    fn adjust_frequency(&mut self, delta: f64) {
        self.frequency = (self.frequency + delta).clamp(FREQ_MIN, FREQ_MAX);
    }

    fn adjust_amplitude(&mut self, delta: f64) {
        self.amplitude = (self.amplitude + delta).clamp(AMP_MIN, AMP_MAX);
    }

}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    enable_raw_mode()?;
    execute!(stdout(), EnterAlternateScreen)?;
    let mut terminal = Terminal::new(CrosstermBackend::new(stdout()))?;

    let start = Instant::now();
    let frame_duration = Duration::from_millis(1000 / FPS);
    let mut state = State::new();

    loop {
        let t = start.elapsed().as_secs_f64();
        state.update(t);

        terminal.draw(|frame| render(frame, &state))?;

        if event::poll(frame_duration)? {
            if let Event::Key(key) = event::read()? {
                match key.code {
                    KeyCode::Char('q') | KeyCode::Esc => break,
                    KeyCode::Right => state.adjust_frequency(FREQ_STEP),
                    KeyCode::Left => state.adjust_frequency(-FREQ_STEP),
                    KeyCode::Up => state.adjust_amplitude(AMP_STEP),
                    KeyCode::Down => state.adjust_amplitude(-AMP_STEP),
                    _ => {}
                }
            }
        }
    }

    disable_raw_mode()?;
    execute!(stdout(), LeaveAlternateScreen)?;
    Ok(())
}

fn render(frame: &mut Frame, state: &State) {
    let [title_area, main_area, footer_area] = Layout::vertical([
        Constraint::Length(2),
        Constraint::Min(10),
        Constraint::Length(2),
    ])
    .areas(frame.area());

    // Title
    let title = Paragraph::new(Line::from(vec![
        Span::styled("  Signal (time domain)", Style::default().fg(Color::Yellow)),
        Span::raw("                              "),
        Span::styled("IQ Point (complex plane)", Style::default().fg(Color::Cyan)),
    ]));
    frame.render_widget(title, title_area);

    // Footer with controls and current values
    let footer = Paragraph::new(Line::from(vec![
        Span::styled(
            format!(
                "  ←→ frequency: {:.1} Hz   ↑↓ amplitude: {:.1}",
                state.frequency, state.amplitude
            ),
            Style::default().fg(Color::DarkGray),
        ),
    ]));
    frame.render_widget(footer, footer_area);

    let [wave_area, iq_area] =
        Layout::horizontal([Constraint::Percentage(55), Constraint::Percentage(45)])
            .areas(main_area);

    render_oscilloscope(frame, state, wave_area);
    render_complex_plane(frame, state, iq_area);
}

fn render_oscilloscope(frame: &mut Frame, state: &State, area: ratatui::layout::Rect) {
    let amplitude = state.amplitude;
    let phase = state.phase;
    let frequency = state.frequency;
    let omega = 2.0 * std::f64::consts::PI * frequency;

    let current_y = amplitude * phase.sin();

    let canvas = Canvas::default()
        .block(
            Block::default()
                .borders(Borders::ALL)
                .border_style(Style::default().fg(Color::DarkGray)),
        )
        .x_bounds([0.0, HISTORY_SECONDS])
        .y_bounds([-1.2, 1.2])
        .marker(Marker::Braille)
        .paint(move |ctx| {
            // Zero line
            ctx.draw(&CanvasLine::new(
                0.0,
                0.0,
                HISTORY_SECONDS,
                0.0,
                Color::Indexed(236),
            ));

            // Waveform — draw using current frequency/amplitude
            // (This shows what the wave looks like at the current settings,
            // not a true recording of past values. Good enough for illustration.)
            let num_points = 400;
            let dt = HISTORY_SECONDS / num_points as f64;

            let mut prev_x = 0.0;
            let mut prev_y = amplitude * (phase - omega * HISTORY_SECONDS).sin();

            for i in 1..=num_points {
                let seconds_ago = HISTORY_SECONDS - dt * i as f64;
                let x = dt * i as f64;
                let y = amplitude * (phase - omega * seconds_ago).sin();

                let color = if seconds_ago > HISTORY_SECONDS * 0.7 {
                    Color::Indexed(238)
                } else if seconds_ago > HISTORY_SECONDS * 0.4 {
                    Color::Indexed(178)
                } else {
                    Color::Yellow
                };

                ctx.draw(&CanvasLine::new(prev_x, prev_y, x, y, color));
                prev_x = x;
                prev_y = y;
            }

            // Current point
            let dot = filled_circle(HISTORY_SECONDS, current_y, 0.06);
            ctx.draw(&Points {
                coords: &dot,
                color: Color::White,
            });

            // Amplitude labels
            ctx.print(
                0.1,
                1.05,
                Line::from(Span::styled("+A", Style::default().fg(Color::DarkGray))),
            );
            ctx.print(
                0.1,
                -1.05,
                Line::from(Span::styled("-A", Style::default().fg(Color::DarkGray))),
            );
        });

    frame.render_widget(canvas, area);
}

fn render_complex_plane(frame: &mut Frame, state: &State, area: ratatui::layout::Rect) {
    let amplitude = state.amplitude;
    let phase = state.phase;
    let frequency = state.frequency;

    let i_val = amplitude * phase.cos();
    let q_val = amplitude * phase.sin();

    let canvas = Canvas::default()
        .block(
            Block::default()
                .borders(Borders::ALL)
                .border_style(Style::default().fg(Color::DarkGray)),
        )
        .x_bounds([-1.3, 1.3])
        .y_bounds([-1.3, 1.3])
        .marker(Marker::Braille)
        .paint(move |ctx| {
            // Axes
            ctx.draw(&CanvasLine::new(-1.2, 0.0, 1.2, 0.0, Color::Indexed(236)));
            ctx.draw(&CanvasLine::new(0.0, -1.2, 0.0, 1.2, Color::Indexed(236)));

            // Amplitude circle
            ctx.draw(&Circle {
                x: 0.0,
                y: 0.0,
                radius: amplitude,
                color: Color::DarkGray,
            });

            // Trail — one full cycle
            let trail_points = 80;
            let trail_duration = 1.0 / frequency;
            let omega = 2.0 * std::f64::consts::PI * frequency;
            let dt = trail_duration / trail_points as f64;

            for j in 1..=trail_points {
                let age = dt * j as f64;
                let p1 = phase - omega * age;
                let x1 = amplitude * p1.cos();
                let y1 = amplitude * p1.sin();
                let p0 = phase - omega * (age + dt);
                let x0 = amplitude * p0.cos();
                let y0 = amplitude * p0.sin();

                let frac = j as f64 / trail_points as f64;
                let color = if frac > 0.7 {
                    Color::Indexed(236)
                } else if frac > 0.3 {
                    Color::Indexed(30)
                } else {
                    Color::Indexed(37)
                };

                ctx.draw(&CanvasLine::new(x0, y0, x1, y1, color));
            }

            // Line from origin to IQ point
            ctx.draw(&CanvasLine::new(0.0, 0.0, i_val, q_val, Color::Cyan));

            // IQ point
            let dot = filled_circle(i_val, q_val, 0.06);
            ctx.draw(&Points {
                coords: &dot,
                color: Color::White,
            });

            // Axis labels
            ctx.print(
                1.05,
                -0.12,
                Line::from(Span::styled("I", Style::default().fg(Color::DarkGray))),
            );
            ctx.print(
                0.05,
                1.1,
                Line::from(Span::styled("Q", Style::default().fg(Color::DarkGray))),
            );
        });

    frame.render_widget(canvas, area);
}

/// Generate points filling a circle at (cx, cy) with the given radius.
fn filled_circle(cx: f64, cy: f64, radius: f64) -> Vec<(f64, f64)> {
    let mut pts = Vec::new();
    let step = radius / 8.0;
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
