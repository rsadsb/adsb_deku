#![no_main]
use libfuzzer_sys::fuzz_target;
use adsb_deku::{Frame, DF};
use adsb_deku::deku::{DekuContainerRead};

fuzz_target!(|data: &[u8]| {
    if let Ok((_, frame)) = Frame::from_bytes((&data, 0)) {
        println!("{}", frame);
        println!("{:?}", frame);
    }
});
