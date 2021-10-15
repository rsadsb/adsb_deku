use adsb_deku::deku::DekuContainerRead;
use adsb_deku::{Frame, DF, ME};

use clap::{AppSettings, Clap};
use std::io::{BufRead, BufReader};
use std::net::TcpStream;

use apps::Airplanes;

#[derive(Debug, Clap)]
#[clap(version = "1.0", author = "wcampbell <wcampbell1995@gmail.com>")]
#[clap(setting = AppSettings::ColoredHelp)]
struct Options {
    #[clap(long)]
    host: Option<String>,
    #[clap(long)]
    port: Option<u16>,
}

fn main() {
    let options = Options::parse();
    let stream = TcpStream::connect((
        options.host.unwrap_or_else(|| "localhost".to_string()),
        options.port.unwrap_or(30002),
    ))
    .unwrap();
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
                if let DF::ADSB(ref adsb) = frame.df {
                    if let ME::AirbornePositionBaroAltitude(_) = adsb.me {
                        airplains.add_extended_quitter_ap(adsb.icao, frame.clone());
                    }
                }
                if frame.to_string() == "" {
                    panic!("[E] fmt::Display not implemented");
                }
            }
            Err(e) => panic!("[E] {}", e),
        }
        input.clear();
        airplains.prune();
    }
}
