//! This tui program displays the current ADS-B detected airplanes on a plot with your current
//! position as (0,0) and has the ability to show different information about aircraft locations
//! and testing your coverage.

mod airport;
mod cli;

use std::io::{self, BufRead, BufReader, BufWriter};
use std::net::{SocketAddr, TcpStream};
use std::sync::{Arc, Mutex};
use std::time::{Duration, SystemTime};

use adsb_deku::cpr::Position;
use adsb_deku::deku::DekuContainerRead;
use adsb_deku::{Frame, ICAO};
use anyhow::{Context, Result};
use clap::Parser;
use crossterm::event::{
    poll, read, EnableMouseCapture, Event, KeyCode, KeyEvent, MouseButton, MouseEvent,
    MouseEventKind,
};
use crossterm::terminal::enable_raw_mode;
use crossterm::ExecutableCommand;
use gpsd_proto::{get_data, handshake, ResponseData};
use rsadsb_common::{AirplaneCoor, AirplaneDetails, Airplanes};
use time::UtcOffset;
use tracing::{debug, error, info, trace};
use tracing_subscriber::EnvFilter;
use tui::backend::{Backend, CrosstermBackend};
use tui::layout::{Constraint, Direction, Layout, Rect};
use tui::style::{Color, Modifier, Style};
use tui::symbols::DOT;
use tui::text::{Span, Spans};
use tui::widgets::canvas::{Canvas, Line, Points};
use tui::widgets::{Block, Borders, Row, Table, TableState, Tabs};
use tui::Terminal;

use crate::airport::Airport;
use crate::cli::Opts;

/// Amount of zoom out from your original lat/long position
const MAX_PLOT_HIGH: f64 = 400.0;
const MAX_PLOT_LOW: f64 = MAX_PLOT_HIGH * -1.0;

mod scale {
    /// Diff between scale changes
    pub const CHANGE: f64 = 1.1;

    /// Value used as mutiplier in map scaling for projection
    pub const DEFAULT: f64 = 500_000.0;
}

/// Accuracy of latitude/longitude for Coverage is affected by this variable.
///
/// ie: 83.912345 -> 83.91. This is specifically so we get more results hitting in the same
/// position for the sake of an usable heatmap
const COVERAGE_MASK: f64 = 100.0;

/// tui top bar margin
const TUI_START_MARGIN: u16 = 1;

/// width of tui top bar
const TUI_BAR_WIDTH: u16 = 3;

/// default precision of latitude, longitude, and distance
const DEFAULT_PRECISION: usize = 3;

/// Available top row Tabs
#[derive(Copy, Clone)]
enum Tab {
    Map,
    Coverage,
    Airplanes,
    Stats,
    Help,
}

impl Tab {
    pub fn next_tab(self) -> Self {
        match self {
            Self::Map => Self::Coverage,
            Self::Coverage => Self::Airplanes,
            Self::Airplanes => Self::Stats,
            Self::Stats => Self::Help,
            Self::Help => Self::Map,
        }
    }
}

#[derive(Debug, Default)]
struct Stats {
    most_distance: Option<(SystemTime, ICAO, AirplaneCoor)>,
    most_airplanes: Option<(SystemTime, u32)>,
}

impl Stats {
    fn update(&mut self, airplanes: &Airplanes) {
        // Update most_distance
        let current_distance = self.most_distance.map_or(0.0, |most_distance| {
            if let Some(kilo_distance) = most_distance.2.kilo_distance {
                kilo_distance
            } else {
                0.0
            }
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
        let most_airplanes = if let Some(most_airplanes) = self.most_airplanes {
            most_airplanes.1
        } else {
            0
        };
        if most_airplanes < current_len as u32 {
            info!("new most airplanes: {}", current_len);
            self.most_airplanes = Some((SystemTime::now(), current_len as u32));
        }
    }
}

/// After parsing from `Opts` contains more settings mutated in program
struct Settings<'a> {
    /// opts from clap command line
    opts: Opts,
    /// when Some(), imply quitting with msg
    quit: Option<&'a str>,
    /// mutable current map selection
    tab_selection: Tab,
    /// current scale from operator
    scale: f64,
    /// current lat from operator
    lat: f64,
    /// current long from operator
    long: f64,
    /// current lat from operator
    custom_lat: Option<f64>,
    /// current long from operator
    custom_long: Option<f64>,
    /// last seen mouse clicking position
    last_mouse_dragging: Option<(u16, u16)>,
    /// Parsed list of airport locations
    airports: Option<Vec<Airport>>,
    /// DateTime offset
    utc_offset: UtcOffset,
}

impl<'a> Settings<'a> {
    fn new(opts: Opts, utc_offset: UtcOffset) -> Self {
        Self {
            quit: None,
            tab_selection: Tab::Map,
            scale: opts.scale,
            lat: opts.lat,
            long: opts.long,
            custom_lat: None,
            custom_long: None,
            opts,
            last_mouse_dragging: None,
            airports: None,
            utc_offset,
        }
    }

    /// Convert new lat/long into mercator using current location from `Settings`
    fn to_xy(&self, latitude: f64, longitude: f64) -> (f64, f64) {
        // TODO save before, it's kinda costly
        let (local_x, local_y) = self.local_lat_lon();
        let (x, y) = self.to_mercator(latitude, longitude);
        let (x, y) = (x - local_x, y - local_y);
        (x, y * -1.0)
    }

    /// Calculate mercator for local lat/long
    fn local_lat_lon(&self) -> (f64, f64) {
        let lat = self.custom_lat.map_or(self.lat, |lat| lat);
        let long = self.custom_long.map_or(self.long, |long| long);
        self.to_mercator(lat, long)
    }

    /// Convert lat/long to mercator coordinates
    fn to_mercator(&self, lat: f64, long: f64) -> (f64, f64) {
        let scale: f64 = self.scale * scale::DEFAULT;

        let x = (long + 180.0) * (scale / 360.0);
        let lat_rad = lat.to_radians();
        let merc_n = f64::ln(f64::tan((std::f64::consts::PI / 4.0) + (lat_rad / 2.0)));
        let y = (scale / 2.0) - (scale * merc_n / (2.0 * std::f64::consts::PI));

        (x, y)
    }

    fn scale_increase(&mut self) {
        self.scale /= scale::CHANGE;
    }

    fn scale_decrease(&mut self) {
        self.scale *= scale::CHANGE;
    }

    fn lat_increase(&mut self) {
        if let Some(lat) = &mut self.custom_lat {
            *lat += 0.005;
        } else {
            self.custom_lat = Some(self.lat + 0.005);
        }
    }

    fn lat_decrease(&mut self) {
        if let Some(lat) = &mut self.custom_lat {
            *lat -= 0.005;
        } else {
            self.custom_lat = Some(self.lat - 0.005);
        }
    }

    fn long_increase(&mut self) {
        if let Some(long) = &mut self.custom_long {
            *long -= 0.03;
        } else {
            self.custom_long = Some(self.long - 0.03);
        }
    }

    fn long_decrease(&mut self) {
        if let Some(long) = &mut self.custom_long {
            *long += 0.03;
        } else {
            self.custom_long = Some(self.long + 0.03);
        }
    }

    fn reset(&mut self) {
        self.custom_lat = None;
        self.custom_long = None;
        self.scale = self.opts.scale;
    }
}

/// Information generated by tui during runtime that is needed for `MouseEvents`
#[derive(Default, Debug, Clone)]
struct TuiInfo {
    bottom_chunks: Option<Vec<Rect>>,
    touchscreen_buttons: Option<Vec<Rect>>,
}

fn main() -> Result<()> {
    // grab the local offset from localtime_r while we are a single thread for safety
    let utc_offset = time::OffsetDateTime::now_local().unwrap().offset();

    // Parse arguments
    let opts = Opts::parse();

    // Generate logs file and start logging
    let file_appender = tracing_appender::rolling::daily(&opts.log_folder, "radar.log");
    let (non_blocking, _guard) = tracing_appender::non_blocking(file_appender);
    let env = EnvFilter::from_default_env();
    tracing_subscriber::fmt()
        .with_env_filter(env)
        .with_ansi(true)
        .with_writer(non_blocking)
        .with_line_number(true)
        .with_file(true)
        .init();

    // print current version
    let version = env!("CARGO_PKG_VERSION");
    info!(
        "starting rsadsb/radar-v{} with options: {:?}",
        version, opts
    );

    // Setup non-blocking TcpStream
    let socket = SocketAddr::from((opts.host, opts.port));
    let host = opts.host;
    let port = opts.port;
    let stream = TcpStream::connect_timeout(&socket, Duration::from_secs(5)).with_context(|| {
        format!(r#"could not open port to ADS-B client at {host}:{port}, try running https://github.com/rsadsb/dump1090_rs.
see https://github.com/rsadsb/adsb_deku#serverdemodulationexternal-applications for more details"#)
    })?;
    stream
        .set_read_timeout(Some(std::time::Duration::from_millis(50)))
        .unwrap();
    let mut reader = BufReader::new(stream);

    // empty containers
    let mut input = String::new();
    let mut coverage_airplanes: Vec<(f64, f64, u32, ICAO)> = Vec::new();
    let mut adsb_airplanes = Airplanes::new();

    // setup tui params
    let mut stdout = io::stdout();
    stdout.execute(EnableMouseCapture).unwrap();
    let mut backend = CrosstermBackend::new(stdout);
    backend.clear().unwrap();
    let mut terminal = Terminal::new(backend).unwrap();
    enable_raw_mode().unwrap();

    // setup tui variables
    let mut airplanes_state = TableState::default();
    let filter_time = opts.filter_time;

    // create settings, dropping opts to prevent bad usage of variable
    let mut settings = Settings::new(opts.clone(), utc_offset);
    drop(opts);

    let mut airports = vec![];
    if let Some(airport) = &settings.opts.airports {
        airports = Airport::from_file(airport, &settings.opts.airports_tz_filter);
    }
    settings.airports = Some(airports);

    // This next group of functions and variables handle if `gpsd_ip` is set from the command
    // line.
    //
    // When set, read from the gpsd daemon at (gpsd_ip, 2947) and update the lat/long Arc<Mutex<T>
    // accordingly
    let gps_lat_long = Arc::new(Mutex::new(None));
    let gpsd = settings.opts.gpsd;
    let gpsd_ip = settings.opts.gpsd_ip.clone();
    if gpsd {
        // clone locally
        let cloned_gps_lat_long = Arc::clone(&gps_lat_long);

        // start thread
        std::thread::spawn(move || {
            gpsd_thread(gpsd_ip, cloned_gps_lat_long);
        });
    }

    let mut stats = Stats::default();

    info!("tui setup");
    while settings.quit.is_none() {
        // check the Mutex from the gpsd thread, update lat/long
        if let Ok(lat_long) = gps_lat_long.lock() {
            if let Some((lat, long)) = *lat_long {
                settings.lat = lat as f64;
                settings.long = long as f64;
            }
        }

        if let Ok(len) = reader.read_line(&mut input) {
            // a length of 0 would indicate a broken pipe/input, quit program
            if len == 0 {
                settings.quit = Some("TCP connection aborted, quitting radar tui");
                continue;
            }

            // convert from string hex -> bytes
            let hex = &mut input.to_string()[1..len - 2].to_string();
            debug!("bytes: {hex}");
            let bytes = if let Ok(bytes) = hex::decode(&hex) {
                bytes
            } else {
                continue;
            };

            // check for all 0's
            if bytes.iter().all(|&b| b == 0) {
                continue;
            }

            // decode
            // first check if the option is selected that limits the parsing by first checking the
            // first 5 bits if they are the known adsb header DF field
            let df_adsb = if settings.opts.limit_parsing {
                ((bytes[0] & 0b1111_1000) >> 3) == 17
            } else {
                true
            };
            if df_adsb {
                // parse the entire DF frame
                let frame = Frame::from_bytes((&bytes, 0));
                match frame {
                    Ok((left_over, frame)) => {
                        debug!("ADS-B Frame: {frame}");
                        adsb_airplanes.action(frame, (settings.lat, settings.long));
                        if left_over.1 != 0 {
                            error!("{left_over:x?}");
                        }
                        // update stats
                        stats.update(&adsb_airplanes);
                    },
                    Err(e) => error!("{e:?}"),
                }
            }
        }
        input.clear();

        populate_coverage(&adsb_airplanes, &mut coverage_airplanes);

        // remove airplanes that timed-out
        adsb_airplanes.prune(filter_time);

        // draw crossterm tui display
        let tui_info = draw(
            version,
            &mut terminal,
            &adsb_airplanes,
            &settings,
            &coverage_airplanes,
            &mut airplanes_state,
            &stats,
        );

        // handle crossterm events
        //
        // Loop until all MouseEvents are read, if you don't do this it takes forever to read
        // all the moved mouse signals and repeated keyboard events
        loop {
            if poll(Duration::from_millis(10))? {
                match read()? {
                    // handle keyboard events
                    Event::Key(key_event) => {
                        trace!("{:?}", key_event);
                        handle_keyevent(
                            key_event,
                            &mut settings,
                            &adsb_airplanes,
                            &mut airplanes_state,
                        );
                    },
                    // handle mouse events
                    Event::Mouse(mouse_event) => {
                        trace!("{:?}", mouse_event);
                        handle_mouseevent(mouse_event, &mut settings, &tui_info);
                    },
                    _ => (),
                }
            } else {
                // don't seen anything, don't read anymore
                break;
            }
        }
    }

    // cleanup and quit
    //
    // PANIC: this won't panic, because main loop will continue until this is Some
    let reason = settings.quit.unwrap();
    terminal.clear()?;
    let mut stdout = io::stdout();
    crossterm::execute!(
        stdout,
        crossterm::terminal::LeaveAlternateScreen,
        crossterm::event::DisableMouseCapture
    )?;
    crossterm::terminal::disable_raw_mode()?;
    terminal.show_cursor()?;
    println!("radar: {}", reason);
    info!("quitting: {}", reason);
    Ok(())
}

// Add to the coverage tab data structure `coverage_airplanes`.
//
// Two events cause an addition:
// 1: New plot from a lat/long position that didn't exist before
// 2: New ICAO(plane) at a previously seen location
fn populate_coverage(
    adsb_airplanes: &Airplanes,
    coverage_airplanes: &mut Vec<(f64, f64, u32, ICAO)>,
) {
    let all_position = adsb_airplanes.all_position();
    for (
        all_icao,
        Position {
            latitude,
            longitude,
            ..
        },
    ) in all_position
    {
        let latitude = (latitude * COVERAGE_MASK).round() / COVERAGE_MASK;
        let longitude = (longitude * COVERAGE_MASK).round() / COVERAGE_MASK;

        // Add number to seen number if found already
        let mut found = false;
        for (lat, long, seen_number, icao) in coverage_airplanes.iter_mut() {
            // Reduce the precision of the coverage/heatmap display (XX.XX)
            //
            // This is so that more airplanes are seen as being in the same spot and are
            // colored so that is made clear to the user. If this is to accurate you will never
            // see airplanes in the "same" spot
            let lat = (*lat * COVERAGE_MASK).round() / COVERAGE_MASK;
            let long = (*long * COVERAGE_MASK).round() / COVERAGE_MASK;

            // Found already, but it is a diff icao? if so, update to new icao and add to
            // seen_number for the color to be more "white" later on
            if (latitude, longitude) == (lat, long) && (all_icao != *icao) {
                *seen_number += 1;
                *icao = all_icao;
                found = true;
                break;
            }

            if (latitude, longitude) == (lat, long) {
                found = true;
                break;
            }
        }

        // If an airplane wasn't seen in this position, add a new entry
        if !found {
            coverage_airplanes.push((latitude, longitude, 0, all_icao));
        }
    }
}

/// Handle a `KeyEvent`
fn handle_keyevent(
    key_event: KeyEvent,
    settings: &mut Settings,
    adsb_airplanes: &Airplanes,
    airplanes_state: &mut TableState,
) {
    let modifiers = key_event.modifiers;
    let code = key_event.code;
    let current_selection = settings.tab_selection;
    match (code, current_selection) {
        // All Tabs
        (KeyCode::F(1), _) => settings.tab_selection = Tab::Map,
        (KeyCode::F(2), _) => settings.tab_selection = Tab::Coverage,
        (KeyCode::F(3), _) => settings.tab_selection = Tab::Airplanes,
        (KeyCode::F(4), _) => settings.tab_selection = Tab::Stats,
        (KeyCode::F(5), _) => settings.tab_selection = Tab::Help,
        (KeyCode::Tab, _) => settings.tab_selection = settings.tab_selection.next_tab(),
        (KeyCode::Char('q'), _) => settings.quit = Some("user requested action: quit"),
        (KeyCode::Char('c'), _) => {
            if modifiers == crossterm::event::KeyModifiers::CONTROL {
                settings.quit = Some("user requested action: quit");
            }
        },
        (KeyCode::Char('l'), _) => settings.opts.disable_lat_long ^= true,
        (KeyCode::Char('i'), _) => settings.opts.disable_icao ^= true,
        (KeyCode::Char('h'), _) => settings.opts.disable_heading ^= true,
        (KeyCode::Char('t'), _) => settings.opts.disable_track ^= true,
        // Map and Coverage
        (KeyCode::Char('-'), Tab::Map | Tab::Coverage) => settings.scale_increase(),
        (KeyCode::Char('+'), Tab::Map | Tab::Coverage) => settings.scale_decrease(),
        (KeyCode::Up, Tab::Map | Tab::Coverage) => settings.lat_increase(),
        (KeyCode::Down, Tab::Map | Tab::Coverage) => settings.lat_decrease(),
        (KeyCode::Left, Tab::Map | Tab::Coverage) => settings.long_increase(),
        (KeyCode::Right, Tab::Map | Tab::Coverage) => settings.long_decrease(),
        (KeyCode::Enter, Tab::Map | Tab::Coverage) => settings.reset(),
        // Airplanes
        (KeyCode::Up, Tab::Airplanes) => {
            let index = airplanes_state
                .selected()
                .and_then(|selected| selected.checked_sub(1))
                .unwrap_or(0);
            airplanes_state.select(Some(index));
        },
        (KeyCode::Down, Tab::Airplanes) => {
            let index = airplanes_state
                .selected()
                .map_or(0, |selected| selected + 1);
            airplanes_state.select(Some(index));
        },
        (KeyCode::Enter, Tab::Airplanes) => {
            if let Some(selected) = airplanes_state.selected() {
                let key = adsb_airplanes.keys().nth(selected).unwrap();
                let aircraft_details = adsb_airplanes.aircraft_details(*key);
                if let Some(AirplaneDetails { position, .. }) = aircraft_details {
                    settings.custom_lat = Some(position.latitude);
                    settings.custom_long = Some(position.longitude);
                    settings.tab_selection = Tab::Map;
                }
            }
        },
        _ => (),
    }
}

/// Handle a `MouseEvent`
fn handle_mouseevent(mouse_event: MouseEvent, settings: &mut Settings, tui_info: &TuiInfo) {
    match mouse_event.kind {
        MouseEventKind::Down(MouseButton::Left) => {
            // Tabs
            match (mouse_event.column, mouse_event.row) {
                (3..=6, TUI_START_MARGIN..=TUI_BAR_WIDTH) => settings.tab_selection = Tab::Map,
                (8..=16, TUI_START_MARGIN..=TUI_BAR_WIDTH) => {
                    settings.tab_selection = Tab::Coverage;
                },
                (20..=34, TUI_START_MARGIN..=TUI_BAR_WIDTH) => {
                    settings.tab_selection = Tab::Airplanes;
                },
                (36..=42, TUI_START_MARGIN..=TUI_BAR_WIDTH) => {
                    settings.tab_selection = Tab::Stats;
                },
                (42..=48, TUI_START_MARGIN..=TUI_BAR_WIDTH) => {
                    settings.tab_selection = Tab::Help;
                },
                _ => (),
            }
            // left touchscreen (if enabled)
            if let Some(btr) = &tui_info.touchscreen_buttons {
                let scale_i_start = btr[0].y;
                let scale_i_end = btr[0].y + btr[0].height;
                let scale_o_start = btr[1].y;
                let scale_o_end = btr[1].y + btr[0].height;
                let reset_start = btr[2].y;
                let reset_end = btr[2].y + btr[0].height;

                // zoom out
                if (1..=10_u16).contains(&mouse_event.column)
                    && (scale_i_start..=scale_i_end).contains(&mouse_event.row)
                {
                    settings.scale_increase();
                // zoom in
                } else if (1..=10_u16).contains(&mouse_event.column)
                    && (scale_o_start..=scale_o_end).contains(&mouse_event.row)
                {
                    settings.scale_decrease();
                // reset
                } else if (1..=10_u16).contains(&mouse_event.column)
                    && (reset_start..=reset_end).contains(&mouse_event.row)
                {
                    settings.reset();
                }
            }
        },
        MouseEventKind::Drag(MouseButton::Left) => {
            // check tab
            match settings.tab_selection {
                Tab::Map | Tab::Coverage => (),
                Tab::Airplanes | Tab::Stats | Tab::Help => return,
            }

            // check bounds below tab selection
            if mouse_event.row < TUI_BAR_WIDTH {
                return;
            }

            // check bounds if in map view, ignoring touchscreen controls
            if let Some(bottom_chunks) = &tui_info.bottom_chunks {
                let minimum_left_bound = bottom_chunks[1].x;
                if mouse_event.column < minimum_left_bound {
                    return;
                }
            }

            // if we have a previous mouse drag without a mouse lift, change the current position
            if let Some((column, row)) = &settings.last_mouse_dragging {
                let up =
                    f64::from(i32::from(mouse_event.row).wrapping_sub(i32::from(*row))) * 0.020;
                if let Some(lat) = &mut settings.custom_lat {
                    *lat += up;
                } else {
                    settings.custom_lat = Some(settings.lat + up);
                }

                let left =
                    f64::from(i32::from(mouse_event.column).wrapping_sub(i32::from(*column)))
                        * 0.020;
                if let Some(long) = &mut settings.custom_long {
                    *long -= left;
                } else {
                    settings.custom_long = Some(settings.long - left);
                }
            }
            settings.last_mouse_dragging = Some((mouse_event.column, mouse_event.row));
        },
        MouseEventKind::Up(_) => {
            settings.last_mouse_dragging = None;
        },
        MouseEventKind::ScrollDown => settings.scale_increase(),
        MouseEventKind::ScrollUp => settings.scale_decrease(),
        _ => (),
    }
}

fn draw(
    version: &str,
    terminal: &mut Terminal<CrosstermBackend<std::io::Stdout>>,
    adsb_airplanes: &Airplanes,
    settings: &Settings,
    coverage_airplanes: &[(f64, f64, u32, ICAO)],
    airplanes_state: &mut TableState,
    stats: &Stats,
) -> TuiInfo {
    let mut tui_info = TuiInfo::default();

    // tui drawing
    terminal
        .draw(|f| {
            // create layout
            let chunks = Layout::default()
                .direction(Direction::Vertical)
                .margin(1)
                .constraints([Constraint::Min(3), Constraint::Percentage(100)].as_ref())
                .split(f.size());

            // render tabs
            let airplane_len = format!("Airplanes({})", adsb_airplanes.len());
            let titles = ["Map", "Coverage", &airplane_len, "Stats", "Help"]
                .iter()
                .copied()
                .map(Spans::from)
                .collect();

            let mut view_type = "";

            let lat = settings.custom_lat.map_or(settings.lat, |lat| {
                view_type = "(CUSTOM)";
                lat
            });

            let long = settings.custom_long.map_or(settings.long, |long| {
                view_type = "(CUSTOM)";
                long
            });

            let tab = Tabs::new(titles)
                .block(
                    Block::default()
                        .title(format!(
                            "rsadsb/radar(v{}) - ({:.DEFAULT_PRECISION$},{:.DEFAULT_PRECISION$}) {view_type}",
                            version, lat, long
                        ))
                        .borders(Borders::ALL),
                )
                .style(Style::default().fg(Color::White))
                .highlight_style(Style::default().fg(Color::Green))
                .select(settings.tab_selection as usize)
                .divider(DOT);

            f.render_widget(tab.clone(), chunks[0]);

            // render everything under tab
            tui_info = draw_bottom_chunks(
                f,
                chunks,
                settings,
                adsb_airplanes,
                coverage_airplanes,
                airplanes_state,
                stats,
            );
        })
        .unwrap();

    tui_info
}

fn draw_bottom_chunks<A: tui::backend::Backend>(
    f: &mut tui::Frame<A>,
    chunks: Vec<Rect>,
    settings: &Settings,
    adsb_airplanes: &Airplanes,
    coverage_airplanes: &[(f64, f64, u32, ICAO)],
    airplanes_state: &mut TableState,
    stats: &Stats,
) -> TuiInfo {
    let mut tui_info = TuiInfo::default();

    // touchscreen is enabled when operator enabled and Map or Coverage.
    let touchscreen_enable =
        settings.opts.touchscreen && matches!(settings.tab_selection, Tab::Map | Tab::Coverage);

    // if --touchscreen was used, create 10 percent of the screen on the left for the three
    // required buttoms to appear
    let left_size = if touchscreen_enable { 10 } else { 0 };
    let bottom_chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Min(left_size), Constraint::Percentage(100)].as_ref())
        .split(chunks[1]);

    tui_info.bottom_chunks = Some(bottom_chunks.clone());

    // Optionally create the tui widgets for the touchscreen
    tui_info.touchscreen_buttons = if touchscreen_enable {
        let touchscreen_chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Percentage(33),
                Constraint::Percentage(33),
                Constraint::Percentage(33),
            ])
            .split(bottom_chunks[0]);

        let block01 = Block::default().title("Zoom Out").borders(Borders::ALL);
        f.render_widget(block01, touchscreen_chunks[0]);

        let block02 = Block::default().title("Zoom In").borders(Borders::ALL);
        f.render_widget(block02, touchscreen_chunks[1]);

        let block03 = Block::default().title("Reset").borders(Borders::ALL);
        f.render_widget(block03, touchscreen_chunks[2]);

        Some(touchscreen_chunks)
    } else {
        None
    };

    // render the bottom cavas depending on the chosen tab
    match settings.tab_selection {
        Tab::Map => build_tab_map(f, bottom_chunks, settings, adsb_airplanes),
        Tab::Coverage => build_tab_coverage(f, bottom_chunks, settings, coverage_airplanes),
        Tab::Airplanes => build_tab_airplanes(f, bottom_chunks, adsb_airplanes, airplanes_state),
        Tab::Stats => build_tab_stats(f, bottom_chunks, stats, settings),
        Tab::Help => build_tab_help(f, &bottom_chunks),
    }

    tui_info
}

/// Render Map tab for tui display
fn build_tab_map<A: tui::backend::Backend>(
    f: &mut tui::Frame<A>,
    chunks: Vec<Rect>,
    settings: &Settings,
    adsb_airplanes: &Airplanes,
) {
    let canvas = Canvas::default()
        .block(Block::default().title("Map").borders(Borders::ALL))
        .x_bounds([MAX_PLOT_LOW, MAX_PLOT_HIGH])
        .y_bounds([MAX_PLOT_LOW, MAX_PLOT_HIGH])
        .paint(|ctx| {
            draw_lines(ctx);

            // draw locations
            draw_locations(ctx, settings);

            // draw ADSB tab airplanes
            for key in adsb_airplanes.keys() {
                let aircraft_details = adsb_airplanes.aircraft_details(*key);
                if let Some(AirplaneDetails {
                    position,
                    heading,
                    track,
                    ..
                }) = aircraft_details
                {
                    let (x, y) = settings.to_xy(position.latitude, position.longitude);

                    // draw previous positions ("track")
                    if !settings.opts.disable_track {
                        if let Some(track) = track {
                            for coor in track {
                                if let Some(position) = coor.position {
                                    let (x, y) =
                                        settings.to_xy(position.latitude, position.longitude);

                                    // draw dot on location
                                    ctx.draw(&Points {
                                        coords: &[(x, y)],
                                        color: Color::White,
                                    });
                                }
                            }
                        }
                    }

                    // make wings for the angle directions facing toward the heading. This tried to
                    // account for the angles not showing up around the 90 degree mark, of which I
                    // add degrees of the angle before displaying
                    if !settings.opts.disable_heading {
                        if let Some(heading) = heading {
                            const ANGLE: f32 = 20.0;
                            const LENGTH: f32 = 8.0;

                            let addition_heading = (heading % 90.0) / 10.0;
                            let angle: f32 = ANGLE + addition_heading;

                            let heading = heading + 180.0 % 360.0;
                            // wrap around the angle since we are are subtracting
                            let n_heading = if heading > angle {
                                heading - angle
                            } else {
                                (360.0 + heading) - angle
                            };

                            // move the first point out, so that the green point of the aircraft
                            // _usually_ shows.
                            let y_1 = y + f64::from(2.0 * (n_heading.to_radians()).cos());
                            let x_1 = x + f64::from(2.0 * (n_heading.to_radians()).sin());

                            // draw the line out from the aircraft at an angle
                            let y_2 = y + f64::from(LENGTH * (n_heading.to_radians()).cos());
                            let x_2 = x + f64::from(LENGTH * (n_heading.to_radians()).sin());

                            ctx.draw(&Line {
                                x1: x_1,
                                x2: x_2,
                                y1: y_1,
                                y2: y_2,
                                color: Color::Blue,
                            });

                            // repeat for the other side (addition, so just modding)
                            let n_heading = (heading + angle) % 360.0;
                            let y_1 = y + f64::from(2.0 * (n_heading.to_radians()).cos());
                            let x_1 = x + f64::from(2.0 * (n_heading.to_radians()).sin());
                            let y_2 = y + f64::from(LENGTH * (n_heading.to_radians()).cos());
                            let x_2 = x + f64::from(LENGTH * (n_heading.to_radians()).sin());

                            ctx.draw(&Line {
                                x1: x_1,
                                x2: x_2,
                                y1: y_1,
                                y2: y_2,
                                color: Color::Blue,
                            });
                        }
                    }

                    let name = if settings.opts.disable_lat_long {
                        format!("{key}").into_boxed_str()
                    } else {
                        format!("{key} ({}, {})", position.latitude, position.longitude)
                            .into_boxed_str()
                    };

                    if !settings.opts.disable_icao {
                        // draw plane ICAO name
                        ctx.print(
                            x,
                            y + 20.0,
                            Span::styled(name.to_string(), Style::default().fg(Color::White)),
                        );
                    }

                    // draw dot on actual lat/lon
                    ctx.draw(&Points {
                        coords: &[(x, y)],
                        color: Color::Blue,
                    });
                }
            }
        });
    f.render_widget(canvas, chunks[1]);
}

/// Render Coverage tab for tui display
fn build_tab_coverage<A: tui::backend::Backend>(
    f: &mut tui::Frame<A>,
    chunks: Vec<Rect>,
    settings: &Settings,
    coverage_airplanes: &[(f64, f64, u32, ICAO)],
) {
    let canvas = Canvas::default()
        .block(Block::default().title("Coverage").borders(Borders::ALL))
        .x_bounds([MAX_PLOT_LOW, MAX_PLOT_HIGH])
        .y_bounds([MAX_PLOT_LOW, MAX_PLOT_HIGH])
        .paint(|ctx| {
            // draw locations
            draw_locations(ctx, settings);

            // draw ADSB tab airplanes
            for (lat, long, seen_number, _) in coverage_airplanes.iter() {
                let (x, y) = settings.to_xy(*lat, *long);

                let number: u32 = 100 + *seen_number * 50;
                let color_number: u8 = if number > u32::from(u8::MAX) {
                    u8::MAX
                } else {
                    number as u8
                };

                // draw dot on location
                ctx.draw(&Points {
                    coords: &[(x, y)],
                    color: Color::Rgb(color_number, color_number, color_number),
                });
            }
        });
    f.render_widget(canvas, chunks[1]);
}

/// Render Airplanes tab for tui display
fn build_tab_airplanes<A: tui::backend::Backend>(
    f: &mut tui::Frame<A>,
    chunks: Vec<Rect>,
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
        if let Some(AirplaneDetails {
            position,
            altitude,
            kilo_distance,
            ..
        }) = aircraft_details
        {
            lat = format!("{:.DEFAULT_PRECISION$}", position.latitude);
            lon = format!("{:.DEFAULT_PRECISION$}", position.longitude);
            s_kilo_distance = format!("{:.DEFAULT_PRECISION$}", kilo_distance);
            alt = altitude.to_string();
        }
        let heading = if let Some(heading) = state.heading {
            format!("{heading:>7.1}")
        } else {
            "".to_string()
        };
        rows.push(Row::new(vec![
            format!("{key}"),
            state.callsign.as_ref().unwrap_or(&empty).clone(),
            lat,
            lon,
            heading,
            format!("{alt:>8}"),
            state
                .vert_speed
                .map_or_else(|| "".into(), |v| format!("{v:>6}")),
            state
                .speed
                .map_or_else(|| "".into(), |v| format!("{v:>5.0}")),
            format!("{:>8}", s_kilo_distance),
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
    let table = Table::new(rows)
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
        .block(
            Block::default()
                .title(format!("Airplanes({rows_len})"))
                .borders(Borders::ALL),
        )
        .widths(&[
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
        ])
        .column_spacing(1)
        .highlight_style(Style::default().add_modifier(Modifier::BOLD))
        .highlight_symbol(">> ");
    f.render_stateful_widget(table, chunks[1], &mut airplanes_state.clone());
}

/// Render Help tab for tui display
fn build_tab_stats<A: tui::backend::Backend>(
    f: &mut tui::Frame<A>,
    chunks: Vec<Rect>,
    stats: &Stats,
    settings: &Settings,
) {
    let format =
        time::format_description::parse("[month]/[day]/[year] [hour]:[minute]:[second]").unwrap();
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
        .header(Row::new(vec!["Type", "Time", "Value"]).bottom_margin(1))
        .block(Block::default().title("Stats").borders(Borders::ALL))
        .widths(&[
            Constraint::Length(14),
            Constraint::Length(20),
            Constraint::Length(200),
        ])
        .column_spacing(1);
    f.render_widget(table, chunks[1]);
}

/// Render Help tab for tui display
fn build_tab_help<A: tui::backend::Backend>(f: &mut tui::Frame<A>, chunks: &[Rect]) {
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
            Constraint::Percentage(30),
            Constraint::Percentage(30),
            Constraint::Percentage(30),
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
        Row::new(vec!["TAB", "Move to Next screen"]),
        Row::new(vec!["q", "Quit this app"]),
        Row::new(vec!["ctrl+c", "Quit this app"]),
    ];
    let table = Table::new(rows)
        .style(Style::default().fg(Color::White))
        .header(Row::new(vec!["Key", "Action"]).bottom_margin(1))
        .widths(&[Constraint::Percentage(10), Constraint::Percentage(90)])
        .column_spacing(1)
        .block(
            Block::default()
                .title("Key Bindings - Any Tab")
                .borders(Borders::ALL),
        );
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
    let table = Table::new(rows)
        .style(Style::default().fg(Color::White))
        .header(Row::new(vec!["Key", "Action"]).bottom_margin(1))
        .widths(&[Constraint::Percentage(10), Constraint::Percentage(90)])
        .column_spacing(1)
        .block(
            Block::default()
                .title("Key Bindings - Map or Coverage")
                .borders(Borders::ALL),
        );
    f.render_widget(table, vertical_chunks[2]);

    // Third help section
    let rows = [
        Row::new(vec!["Up", "Move selection upward"]),
        Row::new(vec!["Down", "Move selection downward"]),
        Row::new(vec!["Enter", "Center Map tab on selected aircraft"]),
    ];
    let table = Table::new(rows)
        .style(Style::default().fg(Color::White))
        .header(Row::new(vec!["Key", "Action"]).bottom_margin(1))
        .widths(&[Constraint::Percentage(10), Constraint::Percentage(90)])
        .column_spacing(1)
        .block(
            Block::default()
                .title("Key Bindings - Airplanes")
                .borders(Borders::ALL),
        );
    f.render_widget(table, vertical_chunks[3]);
}

/// Draw vertical and horizontal lines
fn draw_lines(ctx: &mut tui::widgets::canvas::Context<'_>) {
    ctx.draw(&Line {
        x1: MAX_PLOT_HIGH,
        y1: 0.0,
        x2: MAX_PLOT_LOW,
        y2: 0.0,
        color: Color::White,
    });
    ctx.draw(&Line {
        x1: 0.0,
        y1: MAX_PLOT_HIGH,
        x2: 0.0,
        y2: MAX_PLOT_LOW,
        color: Color::White,
    });
}

/// Draw locations on the map
fn draw_locations(ctx: &mut tui::widgets::canvas::Context<'_>, settings: &Settings) {
    for location in &settings.opts.locations {
        let (x, y) = settings.to_xy(location.lat, location.long);

        // draw location coor
        ctx.draw(&Points {
            coords: &[(x, y)],
            color: Color::Green,
        });

        // draw location name
        ctx.print(
            x,
            y,
            Span::styled(location.name.clone(), Style::default().fg(Color::Green)),
        );
    }
    if let Some(ref airports) = settings.airports {
        for Airport { icao, lat, lon, .. } in airports {
            let (x, y) = settings.to_xy(*lat, *lon);

            // draw city coor
            ctx.draw(&Points {
                coords: &[(x, y)],
                color: Color::Green,
            });

            // draw city name
            ctx.print(
                x,
                y,
                Span::styled(icao.to_string(), Style::default().fg(Color::Green)),
            );
        }
    }
}

/// function ran withint a thread for updating `gps_lat_long` when the gpsd shows a new `lat_long`
/// position.
fn gpsd_thread(gpsd_ip: String, gps_lat_long: Arc<Mutex<Option<(f64, f64)>>>) {
    let gpsd_port = 2947;
    if let Ok(stream) = TcpStream::connect((gpsd_ip.clone(), gpsd_port))
        .with_context(|| format!("unable to connect to gpsd server @ {gpsd_ip}:{gpsd_port}"))
    {
        let mut reader = BufReader::new(&stream);
        let mut writer = BufWriter::new(&stream);
        handshake(&mut reader, &mut writer).unwrap();
        info!("[gpsd] connected");

        // keep looping while reading new messages looking for GGA messages which are the
        // normal GPS messages from the NMEA messages.
        loop {
            if let Ok(ResponseData::Tpv(data)) = get_data(&mut reader) {
                // only update if the operator hasn't set a lat/long position already
                if let Ok(mut lat_long) = gps_lat_long.lock() {
                    if let (Some(lat), Some(lon)) = (data.lat, data.lon) {
                        info!("[gpsd] lat: {lat},  long:{lon}");
                        *lat_long = Some((lat, lon));
                    }
                }
            }
        }
    } else {
        error!("could not connect to gpsd");
    }
}
