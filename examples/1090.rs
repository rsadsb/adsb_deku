use deku::DekuContainerRead;
use mode_s_deku::Frame;

use std::io::{BufRead, BufReader};
use std::net::TcpStream;

use mode_s_deku::{cpr, DF, ICAO, ME};
use std::collections::HashMap;
use std::fmt;

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

fn main() {
    let stream = TcpStream::connect(("127.0.0.1", 30002)).unwrap();
    let mut reader = BufReader::new(stream);
    let mut input = String::new();
    let mut airplains = Airplains::new();

    loop {
        let len = reader.read_line(&mut input).unwrap();
        let hex = &input.to_string()[1..len - 2];
        println!("{}", hex);
        let bytes = hex::decode(&hex).unwrap();
        match Frame::from_bytes((&bytes, 0)) {
            Ok((_, frame)) => {
                println!("{:#?}", frame);
                println!("{}", frame);
                println!("{}", airplains);
                match frame.df {
                    DF::ADSB(ref adsb) => match adsb.me {
                        ME::AirbornePositionBaroAltitude(_) => {
                            airplains.add_extended_quitter_ap(adsb.icao, frame);
                        }
                        _ => (),
                    },
                    _ => (),
                };
            }
            Err(e) => println!("{}", e),
        }
        input.clear();
    }
}
