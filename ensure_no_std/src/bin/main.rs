//! Based on https://github.com/rustwasm/wee_alloc/tree/master/example
//! Run with `cargo +nightly run --release`

#![no_std]
#![no_main]
#![feature(core_intrinsics, lang_items, alloc_error_handler)]

use adsb_deku::deku::DekuContainerRead;
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

// Need to provide a tiny `panic` implementation for `#![no_std]`.
// This translates into an `unreachable` instruction that will
// raise a `trap` the WebAssembly execution if we panic at runtime.
#[panic_handler]
#[no_mangle]
unsafe fn panic(_info: &::core::panic::PanicInfo) -> ! {
    ::core::intrinsics::abort();
}

// Need to provide an allocation error handler which just aborts
// the execution with trap.
#[alloc_error_handler]
#[no_mangle]
unsafe fn oom(_: ::core::alloc::Layout) -> ! {
    ::core::intrinsics::abort();
}

// Needed for non-wasm targets.
#[lang = "eh_personality"]
#[no_mangle]
extern "C" fn eh_personality() {}

#[no_mangle]
pub extern "C" fn main() {
    let buffer = hex!("8da7c32758ab75f3291315f10261");
    let _ = Frame::from_bytes((&buffer, 0)).unwrap().0;
    let _ = Airplanes::new();
}
