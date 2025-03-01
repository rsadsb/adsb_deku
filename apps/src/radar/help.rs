use ratatui::layout::{Constraint, Direction, Layout, Rect};
use ratatui::style::{Color, Style};
use ratatui::widgets::{Block, Row, Table};

/// Render Help tab for tui display
pub fn build_tab_help(f: &mut ratatui::Frame, chunks: &[Rect]) {
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
        Row::new(vec!["F1", "Move to Radar screen"]),
        Row::new(vec!["F2", "Move to Coverage screen"]),
        Row::new(vec!["F3", "Move to Airplanes screen"]),
        Row::new(vec!["F4", "Move to Stats screen"]),
        Row::new(vec!["F5", "Move to Help screen"]),
        Row::new(vec!["l", "control --disable-lat-long"]),
        Row::new(vec!["i", "control --disable-icao"]),
        Row::new(vec!["h", "control --disable-heading"]),
        Row::new(vec!["t", "control --disable-track"]),
        Row::new(vec!["n", "toggle --disable-callsign"]),
        Row::new(vec!["r", "toggle --disable-range-circles"]),
        Row::new(vec!["TAB", "Move to Next screen"]),
        Row::new(vec!["q", "Quit this app"]),
        Row::new(vec!["ctrl+c", "Quit this app"]),
    ];
    let widths = &[Constraint::Percentage(10), Constraint::Percentage(90)];
    let table = Table::new(rows, widths)
        .style(Style::default().fg(Color::White))
        .header(Row::new(vec!["Key", "Action"]).bottom_margin(1))
        .column_spacing(1)
        .block(Block::bordered().title("Key Bindings - Any Tab"));
    f.render_widget(table, vertical_chunks[1]);

    // Second help section
    let rows = vec![
        Row::new(vec!["-", "Zoom out"]),
        Row::new(vec!["+", "Zoom in"]),
        Row::new(vec!["Up", "Move map up"]),
        Row::new(vec!["Down", "Move map down"]),
        Row::new(vec!["Left", "Move map left"]),
        Row::new(vec!["Right", "Move map right"]),
        Row::new(vec!["Enter", "Map position reset"]),
    ];
    let table = Table::new(rows, widths)
        .style(Style::default().fg(Color::White))
        .header(Row::new(vec!["Key", "Action"]).bottom_margin(1))
        .column_spacing(1)
        .block(Block::bordered().title("Key Bindings - Map or Coverage"));
    f.render_widget(table, vertical_chunks[2]);

    // Third help section
    let rows = [
        Row::new(vec!["Up", "Move selection upward"]),
        Row::new(vec!["Down", "Move selection downward"]),
        Row::new(vec!["Enter", "Center Map tab on selected aircraft"]),
    ];
    let table = Table::new(rows, widths)
        .style(Style::default().fg(Color::White))
        .header(Row::new(vec!["Key", "Action"]).bottom_margin(1))
        .column_spacing(1)
        .block(Block::bordered().title("Key Bindings - Airplanes"));
    f.render_widget(table, vertical_chunks[3]);
    
    // Range circles help section
    let rows = [
        Row::new(vec!["--range-circles", "Set range circles (km) as comma-separated values"]),
        Row::new(vec!["--disable-range-circles", "Hide range circles"]),
        Row::new(vec!["r", "Toggle range circles visibility"]),
    ];
    let table = Table::new(rows, widths)
        .style(Style::default().fg(Color::White))
        .header(Row::new(vec!["Option", "Description"]).bottom_margin(1))
        .column_spacing(1)
        .block(Block::bordered().title("Range Circles"));
    f.render_widget(table, vertical_chunks[4]);
}
