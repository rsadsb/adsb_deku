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
use apps::Airplanes;
use clap::Parser;
use crossterm::event::{poll, read, Event, KeyCode, KeyEvent};
use crossterm::terminal::enable_raw_mode;
use gpsd_proto::{get_data, handshake, ResponseData};
use tracing::{info, trace};
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

/// Accuracy of latitude/longitude is affected by this variable.
///
/// ie: 83.912345 -> 83.91. This is specifically so we get more results hitting in the same
/// position for the sake of an usable heatmap
const DIFF: f64 = 100.0;

#[derive(Clone)]
pub struct City {
    name: String,
    lat: f64,
    long: f64,
}

impl FromStr for City {
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
    /// Vector of cities [(name, lat, long),..]
    #[clap(long, multiple_values(true))]
    cities: Vec<City>,
    /// Disable output of latitude and longitude on display
    #[clap(long)]
    disable_lat_long: bool,
    /// Zoom level of Radar and Coverage
    #[clap(long, default_value = "1.2")]
    scale: f64,
    /// Enable automatic updating of lat/lon from gpsd
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
struct Settings {
    scale: f64,
    lat: f64,
    long: f64,
}

fn main() {
    let opts = Opts::parse();

    let file_appender = tracing_appender::rolling::hourly(&opts.log_folder, "radar.log");
    let (non_blocking, _guard) = tracing_appender::non_blocking(file_appender);

    let env = EnvFilter::from_default_env();

    let _ = tracing_subscriber::fmt()
        .with_env_filter(env)
        .with_ansi(true)
        .with_writer(non_blocking)
        .init();

    // Setup non-blocking TcpStream
    let stream = TcpStream::connect((opts.host.clone(), opts.port))
        .expect("Could not open port to ADS-B daemon");
    stream
        .set_read_timeout(Some(std::time::Duration::from_millis(50)))
        .unwrap();
    let mut reader = BufReader::new(stream);

    // This next group of functions and variables handle if `gpsd_ip` is set from the command
    // line.
    //
    // When set, read from the gpsd daemon at (gpsd_ip, 2947) and update the lat/long Arc<Mutex<T>
    // accordingly
    let gps_lat_long = Arc::new(Mutex::new(None));
    let gpsd = opts.gpsd;
    let gpsd_ip = opts.gpsd_ip.clone();
    if gpsd {
        // clone locally
        let cloned_gps_lat_long = Arc::clone(&gps_lat_long);

        // start thread
        std::thread::spawn(move || {
            let stream = TcpStream::connect((gpsd_ip, 2947)).unwrap();
            let mut reader = BufReader::new(&stream);
            let mut writer = BufWriter::new(&stream);
            handshake(&mut reader, &mut writer).unwrap();
            info!("[gpsd] connected");

            // keep looping while reading new messages looking for GGA messages which are the
            // normal GPS messages from the NMEA messages.
            loop {
                if let Ok(ResponseData::Tpv(data)) = get_data(&mut reader) {
                    if let Ok(mut lat_long) = cloned_gps_lat_long.lock() {
                        if let (Some(lat), Some(lon)) = (data.lat, data.lon) {
                            info!("[gpsd] lat: {},  long:{}", lat, lon);
                            *lat_long = Some((lat, lon));
                        }
                    }
                }
            }
        });
    }

    // empty containers
    let mut input = String::new();
    let mut coverage_airplanes: Vec<(f64, f64, u8, ICAO)> = Vec::new();
    let mut adsb_airplanes = Airplanes::new();

    // setup tui params
    let stdout = io::stdout();
    let mut backend = CrosstermBackend::new(stdout);
    backend.clear().unwrap();
    let mut terminal = Terminal::new(backend).unwrap();
    enable_raw_mode().unwrap();

    // setup tui variables
    let mut tab_selection = Tab::Map;
    let mut quit = None;
    let original_scale = opts.scale;
    let mut airplanes_state = TableState::default();
    let filter_time = opts.filter_time;

    let mut settings = Settings {
        scale: opts.scale,
        lat: opts.lat,
        long: opts.long,
    };

    info!("tui setup");
    loop {
        if let Some(reason) = quit {
            terminal.clear().unwrap();
            println!("{}", reason);
            break;
        }
        input.clear();

        // if `gpsd_ip` is selected, check if there is an update from that thread
        if opts.gpsd {
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
                quit = Some("TCP connection aborted, quitting radar tui");
                continue;
            }
            // convert from string hex -> bytes
            let hex = &mut input.to_string()[1..len - 2].to_string();
            trace!("new msg: {}", hex);
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
                let tab = Tabs::new(titles)
                    .block(
                        Block::default()
                            .title(format!("({},{})", settings.lat, settings.long))
                            .borders(Borders::ALL),
                    )
                    .style(Style::default().fg(Color::White))
                    .highlight_style(Style::default().fg(Color::Green))
                    .select(tab_selection as usize)
                    .divider(DOT);

                f.render_widget(tab, chunks[0]);

                // render the tab depending on the selection
                match tab_selection {
                    Tab::Map => build_tab_map(f, chunks, &settings, &opts, &adsb_airplanes),
                    Tab::Coverage => {
                        build_tab_coverage(f, chunks, &settings, &opts, &coverage_airplanes)
                    },
                    Tab::Airplanes => {
                        build_tab_airplanes(f, chunks, &adsb_airplanes, &mut airplanes_state)
                    },
                }
            })
            .unwrap();

        // handle keyboard events
        if poll(Duration::from_millis(10)).unwrap() {
            if let Event::Key(KeyEvent { code, .. }) = read().unwrap() {
                match (code, tab_selection) {
                    // All Tabs
                    (KeyCode::F(1), _) => tab_selection = Tab::Map,
                    (KeyCode::F(2), _) => tab_selection = Tab::Coverage,
                    (KeyCode::F(3), _) => tab_selection = Tab::Airplanes,
                    (KeyCode::Tab, _) => tab_selection = tab_selection.next_tab(),
                    (KeyCode::Char('q'), _) => quit = Some("user requested quit"),
                    (KeyCode::Char('-'), Tab::Map | Tab::Coverage) => settings.scale += 0.1,
                    (KeyCode::Char('+'), Tab::Map | Tab::Coverage) => {
                        if settings.scale > 0.2 {
                            settings.scale -= 0.1;
                        }
                    },
                    // Map and Coverage
                    (KeyCode::Up, Tab::Map | Tab::Coverage) => settings.lat += 0.005,
                    (KeyCode::Down, Tab::Map | Tab::Coverage) => settings.lat -= 0.005,
                    (KeyCode::Left, Tab::Map | Tab::Coverage) => settings.long -= 0.03,
                    (KeyCode::Right, Tab::Map | Tab::Coverage) => settings.long += 0.03,
                    (KeyCode::Enter, Tab::Map | Tab::Coverage) => {
                        settings.lat = opts.lat;
                        settings.long = opts.long;
                        settings.scale = original_scale;
                    },
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
                                tab_selection = Tab::Map;
                            }
                        }
                    },
                    _ => (),
                }
            }
        }
    }
    info!("quitting");
}

/// Render Map tab for tui display
fn build_tab_map<A: tui::backend::Backend>(
    f: &mut tui::Frame<A>,
    chunks: Vec<Rect>,
    settings: &Settings,
    opts: &Opts,
    adsb_airplanes: &Airplanes,
) {
    let canvas = Canvas::default()
        .block(Block::default().title("Map").borders(Borders::ALL))
        .x_bounds([MAX_PLOT_LOW, MAX_PLOT_HIGH])
        .y_bounds([MAX_PLOT_LOW, MAX_PLOT_HIGH])
        .paint(|ctx| {
            ctx.layer();

            let (lat_diff, long_diff) = scale_lat_long(settings.scale);

            // draw cities
            draw_cities(
                ctx,
                &opts.cities,
                settings.lat,
                settings.long,
                lat_diff,
                long_diff,
            );

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

                    let name = if opts.disable_lat_long {
                        format!("{}", key).into_boxed_str()
                    } else {
                        format!("{} ({}, {})", key, position.latitude, position.longitude)
                            .into_boxed_str()
                    };

                    // draw plane ICAO name
                    ctx.print(
                        long + 3.0,
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
    opts: &Opts,
    coverage_airplanes: &[(f64, f64, u8, ICAO)],
) {
    let canvas = Canvas::default()
        .block(Block::default().title("Coverage").borders(Borders::ALL))
        .x_bounds([MAX_PLOT_LOW, MAX_PLOT_HIGH])
        .y_bounds([MAX_PLOT_LOW, MAX_PLOT_HIGH])
        .paint(|ctx| {
            ctx.layer();

            let (lat_diff, long_diff) = scale_lat_long(settings.scale);

            // draw cities
            draw_cities(
                ctx,
                &opts.cities,
                settings.lat,
                settings.long,
                lat_diff,
                long_diff,
            );

            // draw ADSB tab airplanes
            for (lat, long, seen_number, _) in coverage_airplanes.iter() {
                let lat = ((lat - settings.lat) / lat_diff) * MAX_PLOT_HIGH;
                let long = ((long - settings.long) / long_diff) * MAX_PLOT_HIGH;

                let number: u16 = 100_u16 + (*seen_number as u16 * 100);
                let color_number: u8 = if number > u8::MAX as u16 {
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
            Constraint::Length(15),
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

/// Draw cities on the map
fn draw_cities(
    ctx: &mut tui::widgets::canvas::Context<'_>,
    cities: &[City],
    local_lat: f64,
    local_long: f64,
    lat_diff: f64,
    long_diff: f64,
) {
    for city in cities {
        let lat = ((city.lat - local_lat) / lat_diff) * MAX_PLOT_HIGH;
        let long = ((city.long - local_long) / long_diff) * MAX_PLOT_HIGH;

        // draw city coor
        ctx.draw(&Points {
            coords: &[(long, lat)],
            color: Color::Green,
        });

        // draw city name
        ctx.print(
            long + 3.0,
            lat,
            Span::styled(city.name.to_string(), Style::default().fg(Color::Green)),
        );
    }
}

fn scale_lat_long(scale: f64) -> (f64, f64) {
    // Difference between each lat point
    let lat_diff = scale;
    // Difference between each long point
    let long_diff = lat_diff * 3.0;

    (lat_diff, long_diff)
}
