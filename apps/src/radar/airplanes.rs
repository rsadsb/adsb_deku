use ratatui::layout::{Constraint, Layout, Rect};
use ratatui::style::{Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Paragraph, Row, Table, TableState};
use rsadsb_common::{AirplaneDetails, Airplanes};

use crate::{DEFAULT_PRECISION, Settings};

/// Render Airplanes tab for tui display
pub fn build_tab_airplanes(
    f: &mut ratatui::Frame,
    chunks: &[Rect],
    adsb_airplanes: &Airplanes,
    airplanes_state: &mut TableState,
    settings: &Settings,
) {
    let vertical_chunks = Layout::default()
        .direction(ratatui::layout::Direction::Vertical)
        .constraints([Constraint::Min(10), Constraint::Length(5)])
        .split(chunks[1]);

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
            Span::styled(format!("{key}"), Style::default().fg(settings.theme.accent_secondary)),
            Span::styled(
                state.callsign.as_ref().unwrap_or(&empty).clone(),
                Style::default().fg(settings.theme.accent),
            ),
            Span::styled(lat, Style::default().fg(settings.theme.location)),
            Span::styled(lon, Style::default().fg(settings.theme.location)),
            Span::styled(heading, Style::default().fg(settings.theme.heading)),
            Span::styled(format!("{alt:>8}"), Style::default().fg(settings.theme.text)),
            Span::styled(
                state.vert_speed.map_or_else(|| "".into(), |v| format!("{v:>6}")),
                Style::default().fg(settings.theme.aircraft),
            ),
            Span::styled(
                state.speed.map_or_else(|| "".into(), |v| format!("{v:>5.0}")),
                Style::default().fg(settings.theme.track),
            ),
            Span::styled(
                format!("{s_kilo_distance:>8}"),
                Style::default().fg(settings.theme.range_labels),
            ),
            Span::styled(
                format!("{:>4}", state.num_messages),
                Style::default().fg(settings.theme.text_dim),
            ),
        ]));
    }

    let rows_len = rows.len();

    // check the length of selected airplanes
    if let Some(selected) = airplanes_state.selected() {
        if selected > rows_len - 1 {
            airplanes_state.select(Some(rows_len - 1));
        }
    }

    let widths = &[
        Constraint::Length(6),
        Constraint::Length(9),
        Constraint::Length(7),
        Constraint::Length(7),
        Constraint::Length(7),
        Constraint::Length(8),
        Constraint::Length(8),
        Constraint::Length(7),
        Constraint::Length(8),
        Constraint::Length(6),
    ];
    let table = Table::new(rows, widths)
        .style(Style::default().fg(settings.theme.text))
        .header(
            Row::new(vec![
                "ICAO",
                "Call sign",
                "Lat",
                "Long",
                "Heading°",
                " Alt(ft)",
                " FPM(ft)",
                "Spd(kt)",
                "Dist(km)",
                "Msgs",
            ])
            .style(Style::default().fg(settings.theme.table_header))
            .bottom_margin(1),
        )
        .block(
            Block::bordered()
                .title(format!("Airplanes({rows_len})"))
                .title_style(Style::default().fg(settings.theme.title))
                .border_style(Style::default().fg(settings.theme.border)),
        )
        .column_spacing(1)
        .row_highlight_style(
            Style::default().fg(settings.theme.accent).add_modifier(Modifier::BOLD),
        )
        .highlight_symbol(">> ");
    f.render_stateful_widget(table, vertical_chunks[0], &mut airplanes_state.clone());

    // Render description panel
    let description = Paragraph::new(vec![
        Line::from(vec![
            Span::styled("FPM", Style::default().fg(settings.theme.aircraft)),
            Span::styled(
                " = Feet Per Minute (vertical speed)",
                Style::default().fg(settings.theme.text),
            ),
        ]),
        Line::from(vec![
            Span::styled("Spd", Style::default().fg(settings.theme.track)),
            Span::styled(" = Speed in Knots", Style::default().fg(settings.theme.text)),
        ]),
        Line::from(vec![
            Span::styled("Dist", Style::default().fg(settings.theme.range_labels)),
            Span::styled(" = Distance in Kilometers", Style::default().fg(settings.theme.text)),
        ]),
    ])
    .block(
        Block::bordered()
            .title("Units")
            .title_style(Style::default().fg(settings.theme.title))
            .border_style(Style::default().fg(settings.theme.border)),
    );
    f.render_widget(description, vertical_chunks[1]);
}
