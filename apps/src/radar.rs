//! This tui program displays the current ADS-B detected airplanes on a plot with your current
//! position as (0,0) and has the ability to show different information about aircraft locations
//! and testing your coverage.
//!
//! # Tabs
//!
//! ## ADSB
//!
//! Regular display of recently observed aircraft on a lat/long plot
//!
//! ## Coverage
//!
//! Instead of only showing current airplanes, only plot dots for a seen airplane location
//!
//! Instead of using a `HashMap` for only storing an aircraft position for each aircraft, store
//! all aircrafts and only display a dot where detection at the lat/long position. This is for
//! testing the reach of your antenna.
//!
//! ## Airplanes
//!
//! Display all information gathered from observed aircraft

use std::io::{self, BufRead, BufReader, BufWriter};
use std::net::TcpStream;
use std::num::ParseFloatError;
use std::str::FromStr;
use std::sync::{Arc, Mutex};
use std::time::Duration;

use adsb_deku::adsb::ME;
use adsb_deku::cpr::Position;
use adsb_deku::deku::DekuContainerRead;
use adsb_deku::{Frame, DF, ICAO};
use anyhow::{Context, Result};
use apps::Airplanes;
use clap::Parser;
use crossterm::event::{
    poll, read, EnableMouseCapture, Event, KeyCode, KeyEvent, MouseButton, MouseEvent,
    MouseEventKind,
};
use crossterm::terminal::enable_raw_mode;
use crossterm::ExecutableCommand;
use gpsd_proto::{get_data, handshake, ResponseData};
use tracing::{debug, info, trace};
use tracing_subscriber::EnvFilter;
use tui::backend::{Backend, CrosstermBackend};
use tui::layout::{Constraint, Direction, Layout, Rect};
use tui::style::{Color, Modifier, Style};
use tui::symbols::DOT;
use tui::text::{Span, Spans};
use tui::widgets::canvas::{Canvas, Line, Points};
use tui::widgets::{Block, Borders, Row, Table, TableState, Tabs};
use tui::Terminal;

/// Amount of zoom out from your original lat/long position
const MAX_PLOT_HIGH: f64 = 400.0;
const MAX_PLOT_LOW: f64 = MAX_PLOT_HIGH * -1.0;

/// Difference between 1.0 of lat and 1.0 of long when printing
const LAT_LONG_DIFF: f64 = 3.0;

/// Minimum scale an operator can set
const SCALE_MINIMUM: f64 = 0.1;

/// Diff between scale changes
const SCALE_CHANGE: f64 = 0.1;

/// Accuracy of latitude/longitude is affected by this variable.
///
/// ie: 83.912345 -> 83.91. This is specifically so we get more results hitting in the same
/// position for the sake of an usable heatmap
const DIFF: f64 = 100.0;

/// tui top bar margin
const TUI_START_MARGIN: u16 = 1;

/// width of tui top bar
const TUI_BAR_WIDTH: u16 = 3;

/// Parsing struct for the --locations clap parameter
#[derive(Debug, Clone)]
pub struct Location {
    name: String,
    lat: f64,
    long: f64,
}

impl FromStr for Location {
    type Err = ParseFloatError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let coords: Vec<&str> = s
            .trim_matches(|p| p == '(' || p == ')')
            .split(',')
            .collect();

        let lat_fromstr = coords[1].parse::<f64>()?;
        let long_fromstr = coords[2].parse::<f64>()?;

        Ok(Self {
            name: coords[0].to_string(),
            lat: lat_fromstr,
            long: long_fromstr,
        })
    }
}

#[derive(Parser)]
#[clap(
    version,
    name = "radar",
    author = "wcampbell0x2a",
    about = "TUI Display of ADS-B protocol info from demodulator"
)]
#[derive(Debug, Clone)]
struct Opts {
    #[clap(long, default_value = "localhost")]
    host: String,

    #[clap(long, default_value = "30002")]
    port: u16,

    /// Antenna location latitude
    #[clap(long)]
    lat: f64,

    /// Antenna location longitude
    #[clap(long)]
    long: f64,

    /// Vector of location [(name, lat, long),..]
    #[clap(long, multiple_values(true))]
    locations: Vec<Location>,

    /// Disable output of latitude and longitude on display
    #[clap(long)]
    disable_lat_long: bool,

    /// Zoom level of Radar and Coverage (+=zoom out/-=zoom in)
    #[clap(long, default_value = "1.4")]
    scale: f64,

    /// Enable automatic updating of lat/lon from gpsd(https://gpsd.io/) server
    #[clap(long)]
    gpsd: bool,

    /// Ip address of gpsd
    #[clap(long, default_value = "localhost")]
    gpsd_ip: String,

    /// Seconds since last message from airplane, triggers removal of airplane after time is up
    #[clap(long, default_value = "10")]
    filter_time: u64,

    #[clap(long, default_value = "logs")]
    log_folder: String,

    /// Enable three tabs on left side of screen for zoom out/zoom in/and reset.
    #[clap(long)]
    touchscreen: bool,
}

#[derive(Copy, Clone)]
enum Tab {
    Map       = 0,
    Coverage  = 1,
    Airplanes = 2,
}

impl Tab {
    pub fn next_tab(self) -> Self {
        match self {
            Self::Map => Self::Coverage,
            Self::Coverage => Self::Airplanes,
            Self::Airplanes => Self::Map,
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
    /// true: current lat/long/scale differs from gpsd/cmdline input
    /// false: user has changed lat/long/scale with mouse/keyboard
    view_mutated: Arc<Mutex<bool>>,
    /// last seen mouse clicking position
    last_mouse_dragging: Option<(u16, u16)>,
}

impl<'a> Settings<'a> {
    // TODO: the Mutex::new() can be replaced with AtomicBool
    #[allow(clippy::mutex_atomic)]
    fn new(opts: Opts) -> Self {
        Self {
            quit: None,
            tab_selection: Tab::Map,
            scale: opts.scale,
            lat: opts.lat,
            long: opts.long,
            view_mutated: Arc::new(Mutex::new(false)),
            opts,
            last_mouse_dragging: None,
        }
    }

    fn scale_increase(&mut self) {
        self.scale += SCALE_CHANGE;
        self.mutated();
    }

    fn scale_decrease(&mut self) {
        if self.scale > SCALE_MINIMUM {
            self.scale -= SCALE_CHANGE;
        }
        self.mutated();
    }

    fn lat_increase(&mut self) {
        self.lat += 0.005;
        self.mutated();
    }

    fn lat_decrease(&mut self) {
        self.lat -= 0.005;
        self.mutated();
    }

    fn long_increase(&mut self) {
        self.long -= 0.03;
        self.mutated();
    }

    fn long_decrease(&mut self) {
        self.long += 0.03;
        self.mutated();
    }

    fn mutated(&mut self) {
        if let Ok(mut view_mutated) = self.view_mutated.lock() {
            *view_mutated = true;
        }
    }

    fn reset(&mut self) {
        self.lat = self.opts.lat;
        self.long = self.opts.long;
        self.scale = self.opts.scale;
        if let Ok(mut view_mutated) = self.view_mutated.lock() {
            *view_mutated = false;
        }
    }
}

fn main() -> Result<()> {
    let opts = Opts::parse();

    let file_appender = tracing_appender::rolling::daily(&opts.log_folder, "radar.log");
    let (non_blocking, _guard) = tracing_appender::non_blocking(file_appender);

    let env = EnvFilter::from_default_env();

    tracing_subscriber::fmt()
        .with_env_filter(env)
        .with_ansi(true)
        .with_writer(non_blocking)
        .init();

    let version = env!("CARGO_PKG_VERSION");
    info!("starting radar-v{} with options: {:?}", version, opts);

    // Setup non-blocking TcpStream
    let stream = TcpStream::connect((opts.host.clone(), opts.port)).with_context(|| {
        format!(r#"could not open port to ADS-B client at {}:{}, try running https://github.com/rsadsb/dump1090_rs.
see https://github.com/rsadsb/adsb_deku#serverdemodulationexternal-applications for more details"#, opts.host.clone(), opts.port)
    })?;
    stream
        .set_read_timeout(Some(std::time::Duration::from_millis(50)))
        .unwrap();
    let mut reader = BufReader::new(stream);

    // empty containers
    let mut input = String::new();
    let mut coverage_airplanes: Vec<(f64, f64, u8, ICAO)> = Vec::new();
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
    let mut settings = Settings::new(opts.clone());
    drop(opts);

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
        let view_mutated = Arc::clone(&settings.view_mutated);

        // start thread
        std::thread::spawn(move || {
            let gpsd_port = 2947;
            let stream = TcpStream::connect((gpsd_ip.clone(), gpsd_port))
                .with_context(|| {
                    format!(
                        "unable to connect to gpsd server @ {}:{}",
                        gpsd_ip, gpsd_port
                    )
                })
                .unwrap();
            let mut reader = BufReader::new(&stream);
            let mut writer = BufWriter::new(&stream);
            handshake(&mut reader, &mut writer).unwrap();
            info!("[gpsd] connected");

            // keep looping while reading new messages looking for GGA messages which are the
            // normal GPS messages from the NMEA messages.
            loop {
                if let Ok(ResponseData::Tpv(data)) = get_data(&mut reader) {
                    if let Ok(view_mutated) = view_mutated.lock() {
                        if !*view_mutated {
                            if let Ok(mut lat_long) = cloned_gps_lat_long.lock() {
                                if let (Some(lat), Some(lon)) = (data.lat, data.lon) {
                                    info!("[gpsd] lat: {},  long:{}", lat, lon);
                                    *lat_long = Some((lat, lon));
                                }
                            }
                        }
                    }
                }
            }
        });
    }

    info!("tui setup");
    loop {
        if let Some(reason) = settings.quit {
            terminal.clear()?;
            let mut stdout = io::stdout();
            crossterm::execute!(stdout, crossterm::terminal::LeaveAlternateScreen)?;
            crossterm::terminal::disable_raw_mode()?;
            terminal.show_cursor()?;
            println!("radar: {}", reason);
            break;
        }
        input.clear();

        // if `gpsd_ip` is selected, check if there is an update from that thread
        if settings.opts.gpsd {
            if let Ok(lat_long) = gps_lat_long.lock() {
                if let Some((lat, long)) = *lat_long {
                    settings.lat = lat as f64;
                    settings.long = long as f64;
                }
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
            debug!("bytes: {}", hex);
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
            if let Ok((_, frame)) = Frame::from_bytes((&bytes, 0)) {
                debug!("message: {:#?}", frame);
                if let DF::ADSB(ref adsb) = frame.df {
                    adsb_airplanes.incr_messages(adsb.icao);
                    match &adsb.me {
                        ME::AircraftIdentification(identification) => {
                            adsb_airplanes.add_identification(adsb.icao, identification);
                        },
                        ME::AirborneVelocity(vel) => {
                            adsb_airplanes.add_airborne_velocity(adsb.icao, vel);
                        },
                        ME::AirbornePositionGNSSAltitude(altitude)
                        | ME::AirbornePositionBaroAltitude(altitude) => {
                            adsb_airplanes.add_altitude(adsb.icao, altitude);
                        },
                        _ => {},
                    };
                }
            }
        }

        // add lat and long to coverage vector if not existing
        let all_lat_long = adsb_airplanes.all_lat_long_altitude();
        for (
            Position {
                latitude,
                longitude,
                ..
            },
            all_icao,
        ) in all_lat_long
        {
            let latitude = (latitude * DIFF).round() / DIFF;
            let longitude = (longitude * DIFF).round() / DIFF;

            // Add number to seen number if found already
            let mut found = false;
            for coverage in &mut coverage_airplanes {
                // Reduce the precision of the coverage/heatmap display (XX.XX)
                //
                // This is so that more airplanes are seen as being in the same spot and are
                // colored so that is made clear to the user. If this is to accurate you will never
                // see airplanes in the "same" spot
                let (lat, long, seen_number, icao) = coverage;
                let lat = (*lat * DIFF).round() / DIFF;
                let long = (*long * DIFF).round() / DIFF;

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

        input.clear();
        // remove airplanes that timed-out
        adsb_airplanes.prune(filter_time);

        // draw crossterm
        let bottom_touchscreen_rects = draw(
            &mut terminal,
            &adsb_airplanes,
            &settings,
            &coverage_airplanes,
            &mut airplanes_state,
        );

        // handle crossterm events
        //
        // Loop until all MouseEvents are read, if you don't do this it takes forever to read
        // all the moved mouse signals
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

                        // there isn't another keyboard event likely, don't read anymore
                        break;
                    },
                    // handle mouse events
                    Event::Mouse(mouse_event) => {
                        trace!("{:?}", mouse_event);
                        handle_mouseevent(mouse_event, &mut settings, &bottom_touchscreen_rects);
                        //there is most likely another mouse event, jump back to read it
                    },
                    _ => (),
                }
            } else {
                // don't seen anything, don't read anymore
                break;
            }
        }
    }
    info!("quitting");
    Ok(())
}

/// Handle a `KeyEvent`
fn handle_keyevent(
    key_event: KeyEvent,
    settings: &mut Settings,
    adsb_airplanes: &Airplanes,
    airplanes_state: &mut TableState,
) {
    let code = key_event.code;
    let current_selection = settings.tab_selection;
    match (code, current_selection) {
        // All Tabs
        (KeyCode::F(1), _) => settings.tab_selection = Tab::Map,
        (KeyCode::F(2), _) => settings.tab_selection = Tab::Coverage,
        (KeyCode::F(3), _) => settings.tab_selection = Tab::Airplanes,
        (KeyCode::Tab, _) => settings.tab_selection = settings.tab_selection.next_tab(),
        (KeyCode::Char('q'), _) => settings.quit = Some("user requested action: quit"),
        (KeyCode::Char('-'), Tab::Map | Tab::Coverage) => settings.scale_increase(),
        (KeyCode::Char('+'), Tab::Map | Tab::Coverage) => settings.scale_decrease(),
        // Map and Coverage
        (KeyCode::Up, Tab::Map | Tab::Coverage) => settings.lat_increase(),
        (KeyCode::Down, Tab::Map | Tab::Coverage) => settings.lat_decrease(),
        (KeyCode::Left, Tab::Map | Tab::Coverage) => settings.long_increase(),
        (KeyCode::Right, Tab::Map | Tab::Coverage) => settings.long_decrease(),
        (KeyCode::Enter, Tab::Map | Tab::Coverage) => settings.reset(),
        // Airplanes
        (KeyCode::Up, Tab::Airplanes) => {
            if let Some(selected) = airplanes_state.selected() {
                airplanes_state.select(Some(selected - 1));
            } else {
                airplanes_state.select(Some(0));
            }
        },
        (KeyCode::Down, Tab::Airplanes) => {
            if let Some(selected) = airplanes_state.selected() {
                airplanes_state.select(Some(selected + 1));
            } else {
                airplanes_state.select(Some(0));
            }
        },
        (KeyCode::Enter, Tab::Airplanes) => {
            if let Some(selected) = airplanes_state.selected() {
                let key = adsb_airplanes.0.keys().nth(selected).unwrap();
                let pos = adsb_airplanes.lat_long_altitude(*key);
                if let Some((position, _)) = pos {
                    settings.lat = position.latitude;
                    settings.long = position.longitude;
                    settings.tab_selection = Tab::Map;
                }
            }
        },
        _ => (),
    }
}

/// Handle a `MouseEvent`
fn handle_mouseevent(
    mouse_event: MouseEvent,
    settings: &mut Settings,
    bottom_touchscreen_rects: &Option<Vec<Rect>>,
) {
    match mouse_event.kind {
        MouseEventKind::Down(MouseButton::Left) => {
            // Tabs
            match (mouse_event.column, mouse_event.row) {
                (3..=6, TUI_START_MARGIN..=TUI_BAR_WIDTH) => settings.tab_selection = Tab::Map,
                (8..=16, TUI_START_MARGIN..=TUI_BAR_WIDTH) => {
                    settings.tab_selection = Tab::Coverage
                },
                (20..=32, TUI_START_MARGIN..=TUI_BAR_WIDTH) => {
                    settings.tab_selection = Tab::Airplanes
                },
                _ => (),
            }
            // left touchscreen (if enabled)
            if let Some(btr) = bottom_touchscreen_rects {
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
            // if we have a previous mouse drag without a mouse lift, change the current position
            if let Some((column, row)) = &settings.last_mouse_dragging {
                let up =
                    f64::from(i32::from(mouse_event.row).wrapping_sub(i32::from(*row))) * 0.020;
                settings.lat += up;

                let left =
                    f64::from(i32::from(mouse_event.column).wrapping_sub(i32::from(*column)))
                        * 0.020;
                settings.long -= left;
            }
            settings.mutated();
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
    terminal: &mut Terminal<CrosstermBackend<std::io::Stdout>>,
    adsb_airplanes: &Airplanes,
    settings: &Settings,
    coverage_airplanes: &[(f64, f64, u8, ICAO)],
    airplanes_state: &mut TableState,
) -> Option<Vec<Rect>> {
    let mut bottom_touchscreen_rects = None;

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
            let airplane_len = format!("Airplanes({})", adsb_airplanes.0.len());
            let titles = ["Map", "Coverage", &airplane_len]
                .iter()
                .copied()
                .map(Spans::from)
                .collect();

            let view_type = if let Ok(view_mutated) = settings.view_mutated.lock() {
                if *view_mutated {
                    "(CUSTOM)"
                } else {
                    ""
                }
            } else {
                ""
            };

            let tab = Tabs::new(titles)
                .block(
                    Block::default()
                        .title(format!(
                            "({},{}) {}",
                            settings.lat, settings.long, view_type
                        ))
                        .borders(Borders::ALL),
                )
                .style(Style::default().fg(Color::White))
                .highlight_style(Style::default().fg(Color::Green))
                .select(settings.tab_selection as usize)
                .divider(DOT);

            f.render_widget(tab.clone(), chunks[0]);

            bottom_touchscreen_rects = draw_bottom_chunks(
                f,
                chunks,
                settings,
                adsb_airplanes,
                coverage_airplanes,
                airplanes_state,
            );
        })
        .unwrap();

    bottom_touchscreen_rects
}

fn draw_bottom_chunks<A: tui::backend::Backend>(
    f: &mut tui::Frame<A>,
    chunks: Vec<Rect>,
    settings: &Settings,
    adsb_airplanes: &Airplanes,
    coverage_airplanes: &[(f64, f64, u8, ICAO)],
    airplanes_state: &mut TableState,
) -> Option<Vec<Rect>> {
    // Create the bottom chunks, only adding the contraint for the left side if the --touchscreen
    // option is chosen
    let left_size = if settings.opts.touchscreen { 10 } else { 0 };
    let bottom_chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Min(left_size), Constraint::Percentage(100)].as_ref())
        .split(chunks[1]);

    // Optionally create the tui widgets for the touchscreen
    let r_rects = if settings.opts.touchscreen {
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
    }

    r_rects
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
            ctx.layer();

            let (lat_diff, long_diff) = scale_lat_long(settings.scale);

            // draw locations
            draw_locations(ctx, settings, lat_diff, long_diff);

            // draw ADSB tab airplanes
            for key in adsb_airplanes.0.keys() {
                let value = adsb_airplanes.lat_long_altitude(*key);
                if let Some((position, _altitude)) = value {
                    let lat = ((position.latitude - settings.lat) / lat_diff) * MAX_PLOT_HIGH;
                    let long = ((position.longitude - settings.long) / long_diff) * MAX_PLOT_HIGH;

                    // draw dot on location
                    ctx.draw(&Points {
                        coords: &[(long, lat)],
                        color: Color::White,
                    });

                    let name = if settings.opts.disable_lat_long {
                        format!("{}", key).into_boxed_str()
                    } else {
                        format!("{} ({}, {})", key, position.latitude, position.longitude)
                            .into_boxed_str()
                    };

                    // draw plane ICAO name
                    ctx.print(
                        long + LAT_LONG_DIFF,
                        lat,
                        Span::styled(name.to_string(), Style::default().fg(Color::White)),
                    );
                }
            }

            draw_lines(ctx);
        });
    f.render_widget(canvas, chunks[1]);
}

/// Render Coverage tab for tui display
fn build_tab_coverage<A: tui::backend::Backend>(
    f: &mut tui::Frame<A>,
    chunks: Vec<Rect>,
    settings: &Settings,
    coverage_airplanes: &[(f64, f64, u8, ICAO)],
) {
    let canvas = Canvas::default()
        .block(Block::default().title("Coverage").borders(Borders::ALL))
        .x_bounds([MAX_PLOT_LOW, MAX_PLOT_HIGH])
        .y_bounds([MAX_PLOT_LOW, MAX_PLOT_HIGH])
        .paint(|ctx| {
            ctx.layer();

            let (lat_diff, long_diff) = scale_lat_long(settings.scale);

            // draw locations
            draw_locations(ctx, settings, lat_diff, long_diff);

            // draw ADSB tab airplanes
            for (lat, long, seen_number, _) in coverage_airplanes.iter() {
                let lat = ((lat - settings.lat) / lat_diff) * MAX_PLOT_HIGH;
                let long = ((long - settings.long) / long_diff) * MAX_PLOT_HIGH;

                let number: u16 = 100_u16 + (u16::from(*seen_number) * 100);
                let color_number: u8 = if number > u16::from(u8::MAX) {
                    u8::MAX
                } else {
                    number as u8
                };

                // draw dot on location
                ctx.draw(&Points {
                    coords: &[(long, lat)],
                    color: Color::Rgb(color_number, color_number, color_number),
                });
            }

            //draw_lines(ctx);
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
    for key in adsb_airplanes.0.keys() {
        let state = adsb_airplanes.0.get(key).unwrap();
        let pos = adsb_airplanes.lat_long_altitude(*key);
        let mut lat = empty.clone();
        let mut lon = empty.clone();
        let mut alt = empty.clone();
        if let Some((position, altitude)) = pos {
            lat = format!("{}", position.latitude);
            lon = format!("{}", position.longitude);
            alt = format!("{}", altitude);
        }
        rows.push(Row::new(vec![
            format!("{}", key),
            state.callsign.as_ref().unwrap_or(&empty).clone(),
            lat,
            lon,
            format!("{:>8}", alt),
            state
                .vert_speed
                .map_or_else(|| "".into(), |v| format!("{:>6}", v)),
            state
                .speed
                .map_or_else(|| "".into(), |v| format!("{:>5.0}", v)),
            format!("{:>8}", state.num_messages),
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
                "Longitude",
                "Latitude",
                "Altitude",
                "   FPM",
                "Speed",
                "    Msgs",
            ])
            .bottom_margin(1),
        )
        .block(
            Block::default()
                .title(format!("Airplanes({})", rows_len))
                .borders(Borders::ALL),
        )
        .widths(&[
            Constraint::Length(6),
            Constraint::Length(9),
            Constraint::Length(15),
            Constraint::Length(15),
            Constraint::Length(8),
            Constraint::Length(6),
            Constraint::Length(5),
            Constraint::Length(8),
        ])
        .column_spacing(1)
        .highlight_style(Style::default().add_modifier(Modifier::BOLD))
        .highlight_symbol(">> ");
    f.render_stateful_widget(table, chunks[1], &mut airplanes_state.clone());
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
fn draw_locations(
    ctx: &mut tui::widgets::canvas::Context<'_>,
    settings: &Settings,
    lat_diff: f64,
    long_diff: f64,
) {
    for city in &settings.opts.locations {
        let lat = ((city.lat - settings.lat) / lat_diff) * MAX_PLOT_HIGH;
        let long = ((city.long - settings.long) / long_diff) * MAX_PLOT_HIGH;

        // draw city coor
        ctx.draw(&Points {
            coords: &[(long, lat)],
            color: Color::Green,
        });

        // draw city name
        ctx.print(
            long + LAT_LONG_DIFF,
            lat,
            Span::styled(city.name.to_string(), Style::default().fg(Color::Green)),
        );
    }
}

fn scale_lat_long(scale: f64) -> (f64, f64) {
    // Difference between each lat point
    let lat_diff = scale;
    // Difference between each long point
    let long_diff = lat_diff * LAT_LONG_DIFF;

    (lat_diff, long_diff)
}
