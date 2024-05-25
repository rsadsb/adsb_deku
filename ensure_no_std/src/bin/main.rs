//! Based on https://github.com/rustwasm/wee_alloc/tree/master/example
//! Run with `cargo +nightly run --release`

#![no_std]
#![no_main]
#![feature(core_intrinsics, alloc_error_handler)]

use adsb_deku::Frame;
use hexlit::hex;
use rsadsb_common::Airplanes;

extern crate alloc;
extern crate wee_alloc;

#[no_mangle]
#[allow(non_snake_case)]
fn _Unwind_Resume() {}

#[global_allocator]
static ALLOC: wee_alloc::WeeAlloc = wee_alloc::WeeAlloc::INIT;

#[no_mangle]
pub extern "C" fn main() {
    let buffer = hex!("8da7c32758ab75f3291315f10261");
    let _ = Frame::from_reader(&buffer[..]).unwrap();
    let _ = Airplanes::new();
}
