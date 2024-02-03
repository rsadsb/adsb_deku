use ratatui::layout::{Constraint, Rect};
use ratatui::style::{Color, Modifier, Style};
use ratatui::widgets::{Block, Row, Table, TableState};
use rsadsb_common::{AirplaneDetails, Airplanes};

use crate::DEFAULT_PRECISION;

/// Render Airplanes tab for tui display
pub fn build_tab_airplanes(
    f: &mut ratatui::Frame,
    chunks: &[Rect],
    adsb_airplanes: &Airplanes,
    airplanes_state: &mut TableState,
) {
    let mut rows = vec![];
    // make a vec of all strings to get a total amount of airplanes with
    // position information
    let empty = "".to_string();
    for key in adsb_airplanes.keys() {
        let state = adsb_airplanes.get(*key).unwrap();
        let aircraft_details = adsb_airplanes.aircraft_details(*key);
        let mut lat = empty.clone();
        let mut lon = empty.clone();
        let mut alt = empty.clone();
        let mut s_kilo_distance = empty.clone();
        if let Some(AirplaneDetails { position, altitude, kilo_distance, .. }) = aircraft_details {
            lat = format!("{:.DEFAULT_PRECISION$}", position.latitude);
            lon = format!("{:.DEFAULT_PRECISION$}", position.longitude);
            s_kilo_distance = format!("{kilo_distance:.DEFAULT_PRECISION$}");
            alt = altitude.to_string();
        }

        let heading =
            state.heading.map_or_else(|| "".to_string(), |heading| format!("{heading:>7.1}"));

        rows.push(Row::new(vec![
            format!("{key}"),
            state.callsign.as_ref().unwrap_or(&empty).clone(),
            lat,
            lon,
            heading,
            format!("{alt:>8}"),
            state.vert_speed.map_or_else(|| "".into(), |v| format!("{v:>6}")),
            state.speed.map_or_else(|| "".into(), |v| format!("{v:>5.0}")),
            format!("{s_kilo_distance:>8}"),
            format!("{:>4}", state.num_messages),
        ]));
    }

    let rows_len = rows.len();

    // check the length of selected airplanes
    if let Some(selected) = airplanes_state.selected() {
        if selected > rows_len - 1 {
            airplanes_state.select(Some(rows_len - 1));
        }
    }

    // draw table
    let widths = &[
        Constraint::Length(6),
        Constraint::Length(9),
        Constraint::Length(7),
        Constraint::Length(7),
        Constraint::Length(7),
        Constraint::Length(8),
        Constraint::Length(6),
        Constraint::Length(5),
        Constraint::Length(8),
        Constraint::Length(6),
    ];
    let table = Table::new(rows, widths)
        .style(Style::default().fg(Color::White))
        .header(
            Row::new(vec![
                "ICAO",
                "Call sign",
                "Lat",
                "Long",
                "Heading",
                "Altitude",
                "   FPM",
                "Speed",
                "Distance",
                "Msgs",
            ])
            .bottom_margin(1),
        )
        .block(Block::bordered().title(format!("Airplanes({rows_len})")))
        .column_spacing(1)
        .highlight_style(Style::default().add_modifier(Modifier::BOLD))
        .highlight_symbol(">> ");
    f.render_stateful_widget(table, chunks[1], &mut airplanes_state.clone());
}
