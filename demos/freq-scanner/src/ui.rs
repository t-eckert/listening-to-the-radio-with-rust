use ratatui::{
    prelude::*,
    widgets::{Bar, BarChart, BarGroup, Block, Borders, Gauge, List, ListItem},
};

use crate::scanner::FrequencyScanner;

pub fn render(frame: &mut Frame, scanner: &FrequencyScanner) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3), // Status bar
            Constraint::Min(10),   // Spectrum
            Constraint::Length(8), // Signal list
        ])
        .split(frame.area());

    render_status(frame, chunks[0], scanner);
    render_spectrum(frame, chunks[1], scanner);
    render_signals(frame, chunks[2], scanner);
}

fn render_status(frame: &mut Frame, area: Rect, scanner: &FrequencyScanner) {
    let freq_mhz = scanner.current_freq() as f64 / 1e6;
    let text = format!(
        " Scanning: {:.3} MHz | Sweep #{} | Floor: {:.0} dB | [q]uit [r]eset",
        freq_mhz,
        scanner.sweep_count(),
        scanner.noise_floor(),
    );

    let gauge = Gauge::default()
        .block(Block::default().borders(Borders::ALL).title("Freq Scanner"))
        .gauge_style(Style::default().fg(Color::Cyan))
        .percent((scanner.progress() * 100.0) as u16)
        .label(text);

    frame.render_widget(gauge, area);
}

fn render_spectrum(frame: &mut Frame, area: Rect, scanner: &FrequencyScanner) {
    let spectrum = scanner.spectrum();
    let available_width = area.width.saturating_sub(2) as usize; // account for borders

    if spectrum.is_empty() || available_width == 0 {
        return;
    }

    // Find min/max for dynamic range scaling
    let min_db = spectrum.iter().cloned().fold(f32::INFINITY, f32::min);
    let max_db = spectrum.iter().cloned().fold(f32::NEG_INFINITY, f32::max);
    let range = (max_db - min_db).max(1.0);

    let signal_threshold = scanner.noise_floor() + scanner.threshold_db();

    // Downsample spectrum to fit display width
    let step = spectrum.len().max(1) / available_width.max(1);
    let step = step.max(1);

    let bars: Vec<Bar> = spectrum
        .chunks(step)
        .map(|chunk| {
            let peak_db = chunk.iter().cloned().fold(f32::NEG_INFINITY, f32::max);
            // Scale relative to the observed range
            let height = ((peak_db - min_db) / range * 100.0).clamp(0.0, 100.0) as u64;

            let color = if peak_db > signal_threshold {
                Color::Green
            } else {
                Color::DarkGray
            };

            Bar::default().value(height).style(Style::default().fg(color))
        })
        .collect();

    let start_mhz = scanner.start_hz() as f64 / 1e6;
    let end_mhz = scanner.end_hz() as f64 / 1e6;

    let chart = BarChart::default()
        .block(
            Block::default()
                .borders(Borders::ALL)
                .title(format!(" Spectrum: {start_mhz:.1} - {end_mhz:.1} MHz ")),
        )
        .data(BarGroup::default().bars(&bars))
        .bar_width(1)
        .bar_gap(0);

    frame.render_widget(chart, area);
}

fn render_signals(frame: &mut Frame, area: Rect, scanner: &FrequencyScanner) {
    let signals = scanner.signals();

    let items: Vec<ListItem> = signals
        .iter()
        .take(6)
        .map(|s| {
            let freq_mhz = s.freq_hz as f64 / 1e6;
            ListItem::new(format!("  {:.1} MHz  {:>6.1} dB", freq_mhz, s.power_db))
                .style(Style::default().fg(Color::Yellow))
        })
        .collect();

    let list = List::new(items).block(
        Block::default()
            .borders(Borders::ALL)
            .title(format!(" Detected Signals ({}) ", signals.len())),
    );

    frame.render_widget(list, area);
}
