use crate::app::App;
use crate::region::Region;
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

    let region = app.region;

    let canvas = Canvas::default()
        .block(
            Block::default()
                .title(format!(" {} ", region.name))
                .borders(Borders::ALL)
                .border_style(Style::default().fg(Color::DarkGray)),
        )
        .x_bounds([region.lon_min, region.lon_max])
        .y_bounds([region.lat_min, region.lat_max])
        .marker(Marker::Braille)
        .paint(move |ctx| {
            // Draw the region's water features and place names
            draw_region(ctx, region);

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

/// Draw a region's water features and place names.
///
/// Everything here is data-driven from `Region`, so adding a city is a data
/// edit in region.rs rather than a new drawing function.
fn draw_region(ctx: &mut ratatui::widgets::canvas::Context, region: &Region) {
    let water = Color::Blue;
    let water_dim = Color::Indexed(24); // dark blue
    let label_color = Color::DarkGray;
    let landmark_color = Color::Yellow;

    for feature in region.water {
        let color = if feature.major { water } else { water_dim };
        for pair in feature.points.windows(2) {
            ctx.draw(&CanvasLine::new(
                pair[0].0, pair[0].1, pair[1].0, pair[1].1, color,
            ));
        }
    }

    for label in region.labels {
        let color = if label.landmark {
            landmark_color
        } else {
            label_color
        };
        ctx.print(
            label.lon,
            label.lat,
            Line::from(Span::styled(label.text, Style::default().fg(color))),
        );
    }
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
