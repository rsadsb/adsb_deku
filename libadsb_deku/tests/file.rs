use std::io::Cursor;

use adsb_deku::Frame;

const TEST_STR: &str = include_str!("../tests/lax-messages.txt");

#[test]
fn lax_messages() {
    env_logger::init();
    // Read from test file and assert display implemented and non panic decode
    for line in TEST_STR.lines() {
        let len = line.chars().count();
        let hex = &mut line.to_string()[1..len - 1].to_string();
        log::info!("{hex}");
        let bytes = hex::decode(&hex).unwrap();
        // test non panic decode
        let cursor = Cursor::new(bytes);
        let frame = Frame::from_reader(cursor).unwrap();
        // test fmt::Display implemented
        assert_ne!("{}", format!("{frame}"));
    }
}
