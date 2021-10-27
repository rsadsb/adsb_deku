use std::fs::File;
use std::io::prelude::*;
use std::io::BufReader;

use adsb_deku::deku::prelude::*;
use adsb_deku::Frame;

#[test]
fn lax_messages() {
    // Read from test file and assert display implemented and non panic decode
    let file = File::open("tests/lax-messages.txt").unwrap();
    let reader = BufReader::new(file);

    for line in reader.lines() {
        let line = line.unwrap();
        let len = line.chars().count();
        let hex = &mut line.to_string()[1..len - 1].to_string();
        let bytes = hex::decode(&hex).unwrap();
        // test non panic decode
        let frame = Frame::from_bytes((&bytes, 0)).unwrap().1;
        // test fmt::Display implemented
        assert_ne!("{}", format!("{}", frame));
    }
}
