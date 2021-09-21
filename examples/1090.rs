use adsb_deku::Frame;
use deku::DekuContainerRead;

use std::io::Read;

use std::net::TcpStream;

use adsb_deku::{DF, ME};

use common_app::Airplanes;

fn main() {
    let mut stream = TcpStream::connect(("127.0.0.1", 30002)).expect("ADS-B server not running");
    let mut airplains = Airplanes::new();

    loop {
        let mut bytes = [0_u8; u8::MAX as usize];
        let len = stream.read(&mut bytes).unwrap();
        println!("{:02x?}", &bytes[..len]);
        match Frame::from_bytes((&bytes[..len], 0)) {
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
        airplains.prune();
    }
}
