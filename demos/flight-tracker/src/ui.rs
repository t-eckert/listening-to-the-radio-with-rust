use crate::app::App;
use ratatui::{
    Frame,
    layout::{Constraint, Layout},
    style::{Color, Modifier, Style},
    symbols::Marker,
    text::{Line, Span},
    widgets::{
        Block, Borders, Cell, Row, Table, TableState,
        canvas::{Canvas, Line as CanvasLine, Points},
    },
};

pub fn render(frame: &mut Frame, app: &App) {
    let [map_area, list_area] =
        Layout::horizontal([Constraint::Percentage(60), Constraint::Percentage(40)])
            .areas(frame.area());

    render_map(frame, app, map_area);
    render_list(frame, app, list_area);
}

fn render_map(frame: &mut Frame, app: &App, area: ratatui::layout::Rect) {
    // Pre-extract data for the paint closure (needs 'static)
    let selected = app.selected;

    // Build trail segments as owned data
    let trail_segments: Vec<(Vec<(f64, f64)>, bool)> = app
        .trails
        .iter()
        .enumerate()
        .map(|(i, trail)| {
            let points: Vec<(f64, f64)> = trail.iter().map(|p| (p.lon, p.lat)).collect();
            (points, i == selected)
        })
        .collect();

    // Build aircraft display data as owned
    let aircraft_dots: Vec<(f64, f64, String, bool)> = app
        .aircraft
        .iter()
        .enumerate()
        .filter_map(|(i, ac)| {
            let lat = ac.lat?;
            let lon = ac.lon?;
            let label = ac.callsign.clone().unwrap_or_else(|| ac.icao.clone());
            Some((lon, lat, label, i == selected))
        })
        .collect();

    let canvas = Canvas::default()
        .block(
            Block::default()
                .title(" Ottawa Area ")
                .borders(Borders::ALL)
                .border_style(Style::default().fg(Color::DarkGray)),
        )
        .x_bounds([app.lon_min, app.lon_max])
        .y_bounds([app.lat_min, app.lat_max])
        .marker(Marker::Braille)
        .paint(move |ctx| {
            // Draw Ottawa River
            draw_ottawa_river(ctx);

            // Draw trails
            for (points, is_selected) in &trail_segments {
                if points.len() < 2 {
                    continue;
                }
                let color = if *is_selected {
                    Color::Yellow
                } else {
                    Color::DarkGray
                };
                for pair in points.windows(2) {
                    ctx.draw(&CanvasLine::new(
                        pair[0].0, pair[0].1, pair[1].0, pair[1].1, color,
                    ));
                }
            }

            // Draw aircraft dots and labels
            for (lon, lat, label, is_selected) in &aircraft_dots {
                let dot_color = if *is_selected {
                    Color::Cyan
                } else {
                    Color::Green
                };
                ctx.draw(&Points {
                    coords: &[(*lon, *lat)],
                    color: dot_color,
                });

                let label_color = if *is_selected {
                    Color::Cyan
                } else {
                    Color::White
                };
                ctx.print(
                    lon + 0.02,
                    lat + 0.02,
                    Line::from(Span::styled(
                        label.clone(),
                        Style::default().fg(label_color),
                    )),
                );
            }
        });

    frame.render_widget(canvas, area);
}

/// Draw Ottawa-area water features and landmarks.
fn draw_ottawa_river(ctx: &mut ratatui::widgets::canvas::Context) {
    let water = Color::Blue;
    let water_dim = Color::Indexed(24); // dark blue
    let label_color = Color::DarkGray;
    let landmark = Color::Yellow;

    // Helper to draw a polyline
    let draw_line = |ctx: &mut ratatui::widgets::canvas::Context, points: &[(f64, f64)], color: Color| {
        for pair in points.windows(2) {
            ctx.draw(&CanvasLine::new(pair[0].0, pair[0].1, pair[1].0, pair[1].1, color));
        }
    };

    // === OTTAWA RIVER — main channel ===
    let ottawa_river: &[(f64, f64)] = &[
        (-76.35, 45.49),
        (-76.20, 45.49),
        (-76.05, 45.47),
        (-75.95, 45.47),  // Fitzroy Harbour
        (-75.90, 45.43),  // Lac Deschênes west
        (-75.87, 45.42),
        (-75.84, 45.41),
        (-75.82, 45.40),
        (-75.80, 45.39),
        (-75.78, 45.39),  // Lac Deschênes east
        (-75.76, 45.39),
        (-75.74, 45.39),  // Aylmer
        (-75.72, 45.39),
        (-75.70, 45.40),  // Champlain Bridge
        (-75.68, 45.40),
        (-75.66, 45.41),
        (-75.64, 45.42),  // Remic Rapids
        (-75.62, 45.42),
        (-75.60, 45.42),  // Lemieux Island
        (-75.58, 45.42),  // Chaudière Falls
        (-75.56, 45.43),
        (-75.54, 45.43),  // Parliament
        (-75.52, 45.43),
        (-75.50, 45.44),  // Rideau Falls
        (-75.48, 45.44),
        (-75.46, 45.45),  // Rockcliffe
        (-75.44, 45.46),
        (-75.42, 45.47),
        (-75.38, 45.48),
        (-75.34, 45.49),
        (-75.30, 45.50),
        (-75.25, 45.51),  // Petrie Island
        (-75.20, 45.52),
        (-75.10, 45.53),  // Cumberland
        (-75.00, 45.54),
        (-74.90, 45.55),
        (-74.80, 45.56),
        (-74.70, 45.57),
        (-74.60, 45.58),
        (-74.50, 45.58),
    ];
    draw_line(ctx, ottawa_river, water);

    // === LAC DESCHÊNES — north and south shores for width ===
    let lac_north: &[(f64, f64)] = &[
        (-75.92, 45.45),
        (-75.89, 45.44),
        (-75.86, 45.43),
        (-75.84, 45.43),
        (-75.82, 45.42),
        (-75.80, 45.42),
        (-75.78, 45.41),
        (-75.76, 45.41),
        (-75.74, 45.41),
    ];
    let lac_south: &[(f64, f64)] = &[
        (-75.92, 45.40),
        (-75.89, 45.39),
        (-75.86, 45.38),
        (-75.84, 45.37),
        (-75.82, 45.37),
        (-75.80, 45.36),
        (-75.78, 45.37),
        (-75.76, 45.37),
        (-75.74, 45.38),
    ];
    draw_line(ctx, lac_north, water_dim);
    draw_line(ctx, lac_south, water_dim);

    // === RIDEAU RIVER ===
    let rideau: &[(f64, f64)] = &[
        (-75.73, 44.80),
        (-75.72, 44.88),
        (-75.71, 44.95),
        (-75.69, 45.02),  // Manotick
        (-75.68, 45.08),
        (-75.68, 45.14),
        (-75.69, 45.20),  // Hunt Club
        (-75.69, 45.25),  // Mooney's Bay
        (-75.69, 45.28),  // Hog's Back Falls
        (-75.68, 45.30),  // Carleton U
        (-75.68, 45.33),
        (-75.69, 45.36),  // Lansdowne
        (-75.69, 45.38),
        (-75.70, 45.41),
        (-75.70, 45.43),  // Rideau Falls → Ottawa River
    ];
    draw_line(ctx, rideau, water_dim);

    // === RIDEAU CANAL — downtown section ===
    let canal: &[(f64, f64)] = &[
        (-75.69, 45.39),  // Dow's Lake
        (-75.69, 45.40),
        (-75.69, 45.41),  // Glebe
        (-75.69, 45.42),
        (-75.70, 45.43),
        (-75.70, 45.44),  // Parliament locks
    ];
    draw_line(ctx, canal, water_dim);

    // === GATINEAU RIVER ===
    let gatineau: &[(f64, f64)] = &[
        (-75.75, 45.80),
        (-75.74, 45.72),
        (-75.73, 45.65),
        (-75.72, 45.60),
        (-75.71, 45.56),
        (-75.70, 45.52),
        (-75.68, 45.50),
        (-75.64, 45.48),
        (-75.61, 45.46),
        (-75.58, 45.44),  // Joins Ottawa River
    ];
    draw_line(ctx, gatineau, water_dim);

    // === LANDMARKS ===
    ctx.print(-75.67, 45.32, Line::from(
        Span::styled("✦ YOW", Style::default().fg(landmark)),
    ));
    ctx.print(-75.70, 45.43, Line::from(
        Span::styled("▪ Parliament", Style::default().fg(label_color)),
    ));
    ctx.print(-75.70, 45.48, Line::from(
        Span::styled("Gatineau", Style::default().fg(label_color)),
    ));
    ctx.print(-75.90, 45.35, Line::from(
        Span::styled("Kanata", Style::default().fg(label_color)),
    ));
    ctx.print(-75.52, 45.47, Line::from(
        Span::styled("Orléans", Style::default().fg(label_color)),
    ));
    ctx.print(-75.73, 45.28, Line::from(
        Span::styled("Barrhaven", Style::default().fg(label_color)),
    ));
}

fn render_list(frame: &mut Frame, app: &App, area: ratatui::layout::Rect) {
    let header = Row::new(vec![
        Cell::from("ICAO"),
        Cell::from("Call"),
        Cell::from("Alt"),
        Cell::from("Spd"),
        Cell::from("Hdg"),
        Cell::from("Msgs"),
    ])
    .style(Style::default().fg(Color::DarkGray))
    .bottom_margin(1);

    let rows: Vec<Row> = app
        .aircraft
        .iter()
        .enumerate()
        .map(|(i, ac)| {
            let style = if i == app.selected {
                Style::default()
                    .fg(Color::Cyan)
                    .add_modifier(Modifier::BOLD)
            } else if ac.lat.is_some() {
                Style::default().fg(Color::White)
            } else {
                Style::default().fg(Color::DarkGray)
            };

            let callsign = ac.callsign.as_deref().unwrap_or("------");
            let alt = ac
                .altitude_ft
                .map(|a| format!("{a}"))
                .unwrap_or_else(|| "-".into());
            let spd = ac
                .ground_speed_kt
                .map(|s| format!("{s}"))
                .unwrap_or_else(|| "-".into());
            let hdg = ac
                .heading
                .map(|h| format!("{h:.0}°"))
                .unwrap_or_else(|| "-".into());

            Row::new(vec![
                Cell::from(ac.icao.clone()),
                Cell::from(callsign.to_string()),
                Cell::from(alt),
                Cell::from(spd),
                Cell::from(hdg),
                Cell::from(format!("{}", ac.messages)),
            ])
            .style(style)
        })
        .collect();

    let aircraft_count = app.aircraft.len();
    let with_pos = app.aircraft.iter().filter(|a| a.lat.is_some()).count();

    let table = Table::new(
        rows,
        [
            Constraint::Length(7),
            Constraint::Length(8),
            Constraint::Length(6),
            Constraint::Length(4),
            Constraint::Length(5),
            Constraint::Length(4),
        ],
    )
    .header(header)
    .block(
        Block::default()
            .title(format!(
                " Aircraft: {aircraft_count} ({with_pos} with position) "
            ))
            .borders(Borders::ALL)
            .border_style(Style::default().fg(Color::DarkGray)),
    )
    .row_highlight_style(Style::default().bg(Color::DarkGray));

    let mut state = TableState::default();
    if !app.aircraft.is_empty() {
        state.select(Some(app.selected));
    }
    frame.render_stateful_widget(table, area, &mut state);
}
