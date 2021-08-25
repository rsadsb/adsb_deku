use deku::DekuContainerRead;
use mode_s_deku::Frame;

use std::io::{BufRead, BufReader};
use std::net::TcpStream;

use mode_s_deku::{cpr, DF, ICAO, ME};
use std::collections::HashMap;
use std::fmt;

use common_app::{AirplaneCoor, Airplanes};

fn main() {
    let stream = TcpStream::connect(("127.0.0.1", 30002)).unwrap();
    let mut reader = BufReader::new(stream);
    let mut input = String::new();
    let mut airplains = Airplanes::new();

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
                            airplains.add_extended_quitter_ap(adsb.icao, frame.clone());
                        }
                        _ => (),
                    },
                    _ => (),
                };
                if frame.to_string() == "" {
                    panic!("[E] fmt::Display not implemented");
                }
            }
            Err(e) => panic!("[E] {}", e),
        }
        input.clear();
    }
}
