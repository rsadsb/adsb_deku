use ratatui::layout::{Constraint, Direction, Layout, Rect};
use ratatui::style::Style;
use ratatui::text::Span;
use ratatui::widgets::{Block, Row, Table};

use crate::Settings;

/// Render Help tab for tui display
pub fn build_tab_help(f: &mut ratatui::Frame, chunks: &[Rect], settings: &Settings) {
    let horizontal_chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage(2),
            Constraint::Percentage(96),
            Constraint::Percentage(2),
        ])
        .split(chunks[1]);

    let vertical_chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Percentage(2),
            Constraint::Percentage(35),
            Constraint::Percentage(35),
            Constraint::Percentage(10),
            Constraint::Percentage(10),
            Constraint::Percentage(2),
        ])
        .split(horizontal_chunks[1]);

    // First help section
    let rows = vec![
        Row::new(vec![
            Span::styled("F1", Style::default().fg(settings.theme.accent)),
            Span::styled("Move to Radar screen", Style::default().fg(settings.theme.text)),
        ]),
        Row::new(vec![
            Span::styled("F2", Style::default().fg(settings.theme.accent)),
            Span::styled("Move to Coverage screen", Style::default().fg(settings.theme.text)),
        ]),
        Row::new(vec![
            Span::styled("F3", Style::default().fg(settings.theme.accent)),
            Span::styled("Move to Airplanes screen", Style::default().fg(settings.theme.text)),
        ]),
        Row::new(vec![
            Span::styled("F4", Style::default().fg(settings.theme.accent)),
            Span::styled("Move to Stats screen", Style::default().fg(settings.theme.text)),
        ]),
        Row::new(vec![
            Span::styled("F5", Style::default().fg(settings.theme.accent)),
            Span::styled("Move to Help screen", Style::default().fg(settings.theme.text)),
        ]),
        Row::new(vec![
            Span::styled("l", Style::default().fg(settings.theme.location)),
            Span::styled("control --disable-lat-long", Style::default().fg(settings.theme.text)),
        ]),
        Row::new(vec![
            Span::styled("i", Style::default().fg(settings.theme.accent_secondary)),
            Span::styled("control --disable-icao", Style::default().fg(settings.theme.text)),
        ]),
        Row::new(vec![
            Span::styled("h", Style::default().fg(settings.theme.heading)),
            Span::styled("control --disable-heading", Style::default().fg(settings.theme.text)),
        ]),
        Row::new(vec![
            Span::styled("t", Style::default().fg(settings.theme.track)),
            Span::styled("control --disable-track", Style::default().fg(settings.theme.text)),
        ]),
        Row::new(vec![
            Span::styled("n", Style::default().fg(settings.theme.accent)),
            Span::styled("toggle --disable-callsign", Style::default().fg(settings.theme.text)),
        ]),
        Row::new(vec![
            Span::styled("r", Style::default().fg(settings.theme.range_labels)),
            Span::styled(
                "toggle --disable-range-circles",
                Style::default().fg(settings.theme.text),
            ),
        ]),
        Row::new(vec![
            Span::styled("TAB", Style::default().fg(settings.theme.accent)),
            Span::styled("Move to Next screen", Style::default().fg(settings.theme.text)),
        ]),
        Row::new(vec![
            Span::styled("q", Style::default().fg(settings.theme.airport)),
            Span::styled("Quit this app", Style::default().fg(settings.theme.text)),
        ]),
        Row::new(vec![
            Span::styled("ctrl+c", Style::default().fg(settings.theme.airport)),
            Span::styled("Quit this app", Style::default().fg(settings.theme.text)),
        ]),
    ];
    let widths = &[Constraint::Percentage(10), Constraint::Percentage(90)];
    let table = Table::new(rows, widths)
        .style(Style::default().fg(settings.theme.text))
        .header(
            Row::new(vec!["Key", "Action"])
                .style(Style::default().fg(settings.theme.table_header))
                .bottom_margin(1),
        )
        .column_spacing(1)
        .block(
            Block::bordered()
                .title("Key Bindings - Any Tab")
                .title_style(Style::default().fg(settings.theme.title))
                .border_style(Style::default().fg(settings.theme.border)),
        );
    f.render_widget(table, vertical_chunks[1]);

    // Second help section
    let rows = vec![
        Row::new(vec![
            Span::styled("-", Style::default().fg(settings.theme.accent)),
            Span::styled("Zoom out", Style::default().fg(settings.theme.text)),
        ]),
        Row::new(vec![
            Span::styled("+", Style::default().fg(settings.theme.location)),
            Span::styled("Zoom in", Style::default().fg(settings.theme.text)),
        ]),
        Row::new(vec![
            Span::styled("Up", Style::default().fg(settings.theme.aircraft)),
            Span::styled("Move map up", Style::default().fg(settings.theme.text)),
        ]),
        Row::new(vec![
            Span::styled("Down", Style::default().fg(settings.theme.aircraft)),
            Span::styled("Move map down", Style::default().fg(settings.theme.text)),
        ]),
        Row::new(vec![
            Span::styled("Left", Style::default().fg(settings.theme.track)),
            Span::styled("Move map left", Style::default().fg(settings.theme.text)),
        ]),
        Row::new(vec![
            Span::styled("Right", Style::default().fg(settings.theme.track)),
            Span::styled("Move map right", Style::default().fg(settings.theme.text)),
        ]),
        Row::new(vec![
            Span::styled("Enter", Style::default().fg(settings.theme.heading)),
            Span::styled("Map position reset", Style::default().fg(settings.theme.text)),
        ]),
    ];
    let table = Table::new(rows, widths)
        .style(Style::default().fg(settings.theme.text))
        .header(
            Row::new(vec!["Key", "Action"])
                .style(Style::default().fg(settings.theme.table_header))
                .bottom_margin(1),
        )
        .column_spacing(1)
        .block(
            Block::bordered()
                .title("Key Bindings - Map or Coverage")
                .title_style(Style::default().fg(settings.theme.title))
                .border_style(Style::default().fg(settings.theme.border)),
        );
    f.render_widget(table, vertical_chunks[2]);

    // Third help section
    let rows = [
        Row::new(vec![
            Span::styled("Up", Style::default().fg(settings.theme.aircraft)),
            Span::styled("Move selection upward", Style::default().fg(settings.theme.text)),
        ]),
        Row::new(vec![
            Span::styled("Down", Style::default().fg(settings.theme.aircraft)),
            Span::styled("Move selection downward", Style::default().fg(settings.theme.text)),
        ]),
        Row::new(vec![
            Span::styled("Enter", Style::default().fg(settings.theme.heading)),
            Span::styled(
                "Center Map tab on selected aircraft",
                Style::default().fg(settings.theme.text),
            ),
        ]),
    ];
    let table = Table::new(rows, widths)
        .style(Style::default().fg(settings.theme.text))
        .header(
            Row::new(vec!["Key", "Action"])
                .style(Style::default().fg(settings.theme.table_header))
                .bottom_margin(1),
        )
        .column_spacing(1)
        .block(
            Block::bordered()
                .title("Key Bindings - Airplanes")
                .title_style(Style::default().fg(settings.theme.title))
                .border_style(Style::default().fg(settings.theme.border)),
        );
    f.render_widget(table, vertical_chunks[3]);

    // Range circles help section
    let rows = [
        Row::new(vec![
            Span::styled("--range-circles", Style::default().fg(settings.theme.range_circles)),
            Span::styled(
                "Set range circles (km) as comma-separated values",
                Style::default().fg(settings.theme.text),
            ),
        ]),
        Row::new(vec![
            Span::styled(
                "--disable-range-circles",
                Style::default().fg(settings.theme.range_labels),
            ),
            Span::styled("Hide range circles", Style::default().fg(settings.theme.text)),
        ]),
        Row::new(vec![
            Span::styled("r", Style::default().fg(settings.theme.range_labels)),
            Span::styled(
                "Toggle range circles visibility",
                Style::default().fg(settings.theme.text),
            ),
        ]),
    ];
    let range_widths = &[Constraint::Percentage(30), Constraint::Percentage(70)];
    let table = Table::new(rows, range_widths)
        .style(Style::default().fg(settings.theme.text))
        .header(
            Row::new(vec!["Option", "Description"])
                .style(Style::default().fg(settings.theme.table_header))
                .bottom_margin(1),
        )
        .column_spacing(1)
        .block(
            Block::bordered()
                .title("Range Circles")
                .title_style(Style::default().fg(settings.theme.title))
                .border_style(Style::default().fg(settings.theme.border)),
        );
    f.render_widget(table, vertical_chunks[4]);
}
