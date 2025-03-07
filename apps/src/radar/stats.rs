use std::time::SystemTime;

use adsb_deku::ICAO;
use ratatui::layout::{Constraint, Rect};
use ratatui::style::{Style, Stylize};
use ratatui::widgets::{Block, Row, Table};
use rsadsb_common::{Added, AirplaneCoor, Airplanes};
use tracing::info;

use crate::{Settings, BLUE, DEFAULT_PRECISION, WHITE};

#[derive(Debug, Default)]
pub struct Stats {
    most_distance: Option<(SystemTime, ICAO, AirplaneCoor)>,
    most_airplanes: Option<(SystemTime, u32)>,
    total_airplanes: u32,
}

impl Stats {
    pub fn update(&mut self, airplanes: &Airplanes, airplane_added: Added) {
        // Update most_distance
        let current_distance = self.most_distance.map_or(0.0, |most_distance| {
            most_distance.2.kilo_distance.map_or(0.0, |kilo_distance| kilo_distance)
        });
        for (key, state) in airplanes.iter() {
            if let Some(distance) = state.coords.kilo_distance {
                if distance > current_distance {
                    info!("new max distance: [{}]{:?}", key, state.coords);
                    self.most_distance = Some((SystemTime::now(), *key, state.coords));
                }
            }
        }

        // Update most airplanes
        let current_len = airplanes.len();
        let most_airplanes = self.most_airplanes.map_or(0, |most_airplanes| most_airplanes.1);

        if most_airplanes < current_len as u32 {
            info!("new most airplanes: {}", current_len);
            self.most_airplanes = Some((SystemTime::now(), current_len as u32));
        }

        // Update total airplanes
        if airplane_added == Added::Yes {
            self.total_airplanes += 1;
        }
    }
}

/// Render Help tab for tui display
pub fn build_tab_stats(
    f: &mut ratatui::Frame,
    chunks: &[Rect],
    stats: &Stats,
    settings: &Settings,
) {
    let format = time::format_description::parse("[month]/[day] [hour]:[minute]:[second]").unwrap();
    let mut rows: Vec<Row> = vec![];
    // Most distance
    let (time, value) = if let Some((time, key, value)) = stats.most_distance {
        let position = value.position.unwrap();
        let lat = format!("{:.DEFAULT_PRECISION$}", position.latitude);
        let lon = format!("{:.DEFAULT_PRECISION$}", position.longitude);
        let distance = format!("{:.DEFAULT_PRECISION$}", value.kilo_distance.unwrap());

        // display time
        let datetime = time::OffsetDateTime::from(time);
        (
            datetime.to_offset(settings.utc_offset).format(&format).unwrap(),
            format!("[{key}]: {distance}km {lat},{lon}"),
        )
    } else {
        ("None".to_string(), "".to_string())
    };
    rows.push(Row::new(vec!["Max Distance", &time, &value]));

    // Most airplanes tracked at one time
    let (time, value) = if let Some((time, most_airplanes)) = stats.most_airplanes {
        // display time
        let datetime = time::OffsetDateTime::from(time);
        (
            datetime.to_offset(settings.utc_offset).format(&format).unwrap(),
            most_airplanes.to_string(),
        )
    } else {
        ("None".to_string(), "".to_string())
    };
    rows.push(Row::new(vec!["Most Airplanes", &time, &value]));

    // Total Airplanes Tracked
    let total_airplanes_s = stats.total_airplanes.to_string();
    rows.push(Row::new(vec!["Total Airplanes", "All Time", &total_airplanes_s]));

    // draw table
    let widths = &[Constraint::Length(16), Constraint::Length(15), Constraint::Length(200)];
    let table = Table::new(rows, widths)
        .header(Row::new(vec!["Type", "DateTime", "Value"]).bottom_margin(1).fg(BLUE))
        .block(Block::bordered().style(Style::default().fg(WHITE)).title("Stats"))
        .column_spacing(1);
    f.render_widget(table, chunks[1]);
}
