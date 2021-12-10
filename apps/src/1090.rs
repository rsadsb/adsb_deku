use std::io::{BufRead, BufReader};
use std::net::TcpStream;

use adsb_deku::deku::DekuContainerRead;
use adsb_deku::Frame;
use clap::Parser;

#[derive(Debug, Parser)]
#[clap(
    version,
    name = "1090",
    author = "wcampbell0x2a",
    about = "Dump ADS-B protocol info from demodulator"
)]
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
}

fn main() {
    let options = Options::parse();
    let stream = TcpStream::connect((options.host, options.port)).unwrap();
    stream
        .set_read_timeout(Some(std::time::Duration::from_millis(50)))
        .unwrap();
    let mut reader = BufReader::new(stream);
    let mut input = String::new();

    loop {
        input.clear();
        if let Ok(len) = reader.read_line(&mut input) {
            if len == 0 {
                continue;
            }
            // convert from string hex -> bytes
            let hex = &mut input.to_string()[1..len - 2].to_string();
            println!("{}", hex.to_lowercase());
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
            match Frame::from_bytes((&bytes, 0)) {
                Ok((_, frame)) => {
                    if options.debug {
                        println!("{:#?}", frame);
                    }
                    println!("{}", frame);
                    assert!(
                        !((frame.to_string() == "") && options.panic_display),
                        "[E] fmt::Display not implemented"
                    );
                },
                Err(e) => {
                    assert!(!options.panic_decode, "[E] {}", e);
                },
            }
            input.clear();
        }
    }
}
