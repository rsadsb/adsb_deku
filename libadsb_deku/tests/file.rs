use adsb_deku::Frame;
use test_log::test;

const TEST_STR: &str = include_str!("../tests/lax-messages.txt");

#[test]
fn lax_messages() {
    // Read from test file and assert display implemented and non panic decode
    for (line_num, line) in TEST_STR.lines().enumerate() {
        let len = line.chars().count();
        let hex = &mut line.to_string()[1..len - 1].to_string();
        let bytes = hex::decode(&hex).unwrap();
        // test non panic decode
        println!("Line {}: {:02x?}", line_num + 1, bytes);
        let frame = Frame::from_bytes(&bytes).unwrap_or_else(|e| {
            panic!("Line {} (hex: {}) failed: {:?}", line_num + 1, hex, e);
        });
        // test fmt::Display implemented
        assert_ne!("{}", format!("{frame}"));
    }
}
