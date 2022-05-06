use tui::layout::{Constraint, Rect};
use tui::style::{Color, Style};
use tui::widgets::{Block, Borders, Row, Table};

use crate::{Settings, Stats, DEFAULT_PRECISION};

/// Render Help tab for tui display
pub fn build_tab_stats<A: tui::backend::Backend>(
    f: &mut tui::Frame<A>,
    chunks: Vec<Rect>,
    stats: &Stats,
    settings: &Settings,
) {
    let format = time::format_description::parse("[month]/[day] [hour]:[minute]:[second]").unwrap();
    let mut rows: Vec<Row> = vec![];
    let (time, value) = if let Some((time, key, value)) = stats.most_distance {
        let position = value.position.unwrap();
        let lat = format!("{:.DEFAULT_PRECISION$}", position.latitude);
        let lon = format!("{:.DEFAULT_PRECISION$}", position.longitude);
        let distance = format!("{:.DEFAULT_PRECISION$}", value.kilo_distance.unwrap());

        // display time
        let datetime = time::OffsetDateTime::from(time);
        (
            datetime
                .to_offset(settings.utc_offset)
                .format(&format)
                .unwrap(),
            format!("[{key}]: {distance}km {lat},{lon}"),
        )
    } else {
        ("None".to_string(), "".to_string())
    };
    rows.push(Row::new(vec!["Max Distance", &time, &value]));

    let (time, value) = if let Some((time, most_airplanes)) = stats.most_airplanes {
        // display time
        let datetime = time::OffsetDateTime::from(time);
        (
            datetime
                .to_offset(settings.utc_offset)
                .format(&format)
                .unwrap(),
            most_airplanes.to_string(),
        )
    } else {
        ("None".to_string(), "".to_string())
    };
    rows.push(Row::new(vec!["Most Airplanes", &time, &value]));

    // draw table
    let table = Table::new(rows)
        .style(Style::default().fg(Color::White))
        .header(Row::new(vec!["Type", "DateTime", "Value"]).bottom_margin(1))
        .block(Block::default().title("Stats").borders(Borders::ALL))
        .widths(&[
            Constraint::Length(14),
            Constraint::Length(15),
            Constraint::Length(200),
        ])
        .column_spacing(1);
    f.render_widget(table, chunks[1]);
}
