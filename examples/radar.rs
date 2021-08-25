use deku::DekuContainerRead;
use mode_s_deku::Frame;

use std::io::{BufRead, BufReader};
use std::net::TcpStream;

use mode_s_deku::{cpr, DF, ICAO, ME};
use std::collections::HashMap;
use std::fmt;

use std::io;
use tui::backend::Backend;
use tui::backend::CrosstermBackend;
use tui::layout::{Constraint, Direction, Layout};
use tui::style::Color;
use tui::widgets::canvas::{Canvas, Line, Points};
use tui::widgets::{Block, Borders};
use tui::Terminal;
use clap::{AppSettings, Clap};

#[derive(Debug)]
pub struct AircraftDeku([Option<Frame>; 2]);

#[derive(Debug)]
pub struct Airplains(HashMap<ICAO, AircraftDeku>);

impl fmt::Display for Airplains {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        for (key, _) in &self.0 {
            let value = self.lat_long_altitude(*key);
            if let Some(value) = value {
                writeln!(f, "{}: {:?}", key, value);
            }
        }
        Ok(())
    }
}

impl Airplains {
    pub fn new() -> Self {
        Self(HashMap::new())
    }

    pub fn add_extended_quitter_ap(&mut self, icao: ICAO, frame: Frame) {
        let airplane = self.0.entry(icao).or_insert(AircraftDeku([None, None]));
        airplane.0 = [Some(frame), airplane.0[0].clone()];
    }

    pub fn lat_long_altitude(&self, icao: ICAO) -> Option<(cpr::Position, u32)> {
        match self.0.get(&icao) {
            Some(frames) => {
                if let (Some(first), Some(second)) = (frames.0[0].as_ref(), frames.0[1].as_ref()) {
                    let first_altitude = match &first.df {
                        DF::ADSB(adsb) => match adsb.me {
                            ME::AirbornePositionBaroAltitude(altitude) => altitude,
                            _ => panic!(),
                        },
                        _ => panic!(),
                    };
                    let second_altitude = match &second.df {
                        DF::ADSB(adsb) => match adsb.me {
                            ME::AirbornePositionBaroAltitude(altitude) => altitude,
                            _ => panic!(),
                        },
                        _ => panic!(),
                    };
                    cpr::get_position((&first_altitude, &second_altitude))
                        .map(|position| (position, first_altitude.alt))
                } else {
                    None
                }
            }
            None => None,
        }
    }
}

#[derive(Clap)]
#[clap(version = "1.0", author = "wcampbell <wcampbell1995@gmail.com>")]
#[clap(setting = AppSettings::ColoredHelp)]
struct Opts {
    #[clap(long)]
    lat: f64,
    #[clap(long)]
    long: f64,
}

fn main() {
    let opts = Opts::parse();
    let local_lat = opts.lat;
    let local_long = opts.long;

    let stream = TcpStream::connect(("127.0.0.1", 30002)).unwrap();
    let mut reader = BufReader::new(stream);
    let mut input = String::new();
    let mut airplains = Airplains::new();

    let stdout = io::stdout();
    let mut backend = CrosstermBackend::new(stdout);
    backend.clear();
    let mut terminal = Terminal::new(backend).unwrap();

    //TODO: add cities as points
    loop {
        let len = reader.read_line(&mut input).unwrap();
        let hex = &input.to_string()[1..len - 2];
        let bytes = hex::decode(&hex).unwrap();
        match Frame::from_bytes((&bytes, 0)) {
            Ok((_, frame)) => {
                match frame.df {
                    DF::ADSB(ref adsb) => match adsb.me {
                        ME::AirbornePositionBaroAltitude(_) => {
                            airplains.add_extended_quitter_ap(adsb.icao, frame.clone());
                        }
                        _ => (),
                    },
                    _ => (),
                };
            }
            Err(_e) => (),
        }
        input.clear();

        terminal
            .draw(|f| {
                let chunks = Layout::default()
                    .direction(Direction::Vertical)
                    .margin(1)
                    .constraints([Constraint::Percentage(100)].as_ref())
                    .split(f.size());

                let canvas = Canvas::default()
                    .block(Block::default().title("ADSB").borders(Borders::ALL))
                    .x_bounds([-180.0, 180.0])
                    .y_bounds([-180.0, 180.0])
                    .paint(|ctx| {
                        ctx.layer();
                        for (key, _) in &airplains.0 {
                            let value = airplains.lat_long_altitude(*key);
                            if let Some((position, _altitude)) = value {
                                let lat = (position.latitude - local_lat) * 200.0;
                                let long = (position.longitude - local_long) * 200.0;

                                ctx.draw(&Points {
                                    coords: &[(long.into(), lat.into())],
                                    color: Color::White,
                                });
                                ctx.print(
                                    long + 5.0,
                                    lat,
                                    Box::leak(
                                        format!(
                                            "{} ({}, {})",
                                            key, position.latitude, position.longitude
                                        )
                                        .into_boxed_str(),
                                    ),
                                    Color::White,
                                );
                            }
                        }
                        // Draw vertical and horizontal lines
                        ctx.draw(&Line {
                            x1: 180.0,
                            y1: 0.0,
                            x2: -180.0,
                            y2: 0.0,
                            color: Color::White,
                        });
                        ctx.draw(&Line {
                            x1: 0.0,
                            y1: 180.0,
                            x2: 0.0,
                            y2: -180.0,
                            color: Color::White,
                        });
                    });

                f.render_widget(canvas, chunks[0]);
            })
            .unwrap();
    }
}
