use adsb_deku::adsb::ME;
use adsb_deku::deku::DekuContainerRead;
use adsb_deku::{Frame, DF};

use clap::{AppSettings, Clap};
use std::io::{BufRead, BufReader};
use std::net::TcpStream;

use apps::Airplanes;

#[derive(Debug, Clap)]
#[clap(version = "1.0", author = "wcampbell <wcampbell1995@gmail.com>")]
#[clap(setting = AppSettings::ColoredHelp)]
struct Options {
    /// ip address of ADS-B demodulated bytes server
    #[clap(long, default_value = "localhost")]
    host: String,
    /// port of ADS-B demodulated bytes server
    #[clap(long, default_value = "30002")]
    port: u16,
    /// Panic on adsb_deku::Frame::fmt::Display not implemented
    #[clap(long)]
    panic_display: bool,
    /// Panic on adsb_deku::Frame::from_bytes() error
    #[clap(long)]
    panic_decode: bool,
    /// Display debug of adsb::Frame
    #[clap(long)]
    debug: bool,
    /// Disable display of currently tracked airplanes lat/long/altitude
    #[clap(long)]
    disable_airplanes: bool,
}

fn main() {
    let options = Options::parse();
    let stream = TcpStream::connect((options.host, options.port)).unwrap();
    let mut reader = BufReader::new(stream);
    let mut input = String::new();
    let mut airplanes = Airplanes::new();

    loop {
        let len = reader.read_line(&mut input).unwrap();
        let hex = &input.to_string()[1..len - 2];
        println!("{}", hex);
        let bytes = hex::decode(&hex).unwrap();
        match Frame::from_bytes((&bytes, 0)) {
            Ok((_, frame)) => {
                if options.debug {
                    println!("{:#?}", frame);
                }
                println!("{}", frame);
                if !options.disable_airplanes {
                    println!("{}", airplanes);
                }
                if let DF::ADSB(ref adsb) = frame.df {
                    if let ME::AirbornePositionBaroAltitude(_) = adsb.me {
                        airplanes.add_extended_quitter_ap(adsb.icao, frame.clone());
                    }
                }
                if (frame.to_string() == "") && options.panic_display {
                    panic!("[E] fmt::Display not implemented");
                }
            }
            Err(e) => {
                if options.panic_decode {
                    panic!("[E] {}", e);
                }
            }
        }
        input.clear();
        airplanes.prune();
    }
}
