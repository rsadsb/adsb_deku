rsadsb_common
===============================

[<img alt="github" src="https://img.shields.io/badge/github-rsadsb/rsadsb_common-8da0cb?style=for-the-badge&labelColor=555555&logo=github" height="20">](https://github.com/rsadsb/adsb_deku)
[<img alt="crates.io" src="https://img.shields.io/crates/v/rsadsb_common.svg?style=for-the-badge&color=fc8d62&logo=rust" height="20">](https://crates.io/crates/rsadsb_common)
[<img alt="docs.rs" src="https://img.shields.io/badge/docs.rs-rsadsb_common-66c2a5?style=for-the-badge&labelColor=555555&logo=docs.rs" height="20">](https://docs.rs/rsadsb_common)
[<img alt="build status" src="https://img.shields.io/github/actions/workflow/status/rsadsb/adsb_deku/main.yml?branch=master&style=for-the-badge" height="20">](https://github.com/rsadsb/adsb_deku/actions?query=branch%3Amaster)

Common library data structures and functions for [`adsb_deku`](https://github.com/rsadsb/adsb_deku) applications.

Run `cargo doc` in this directory to generate documentation.

## Usage
```rust, ignore
let mut adsb_airplanes = Airplanes::new();
if let Ok((bytes_left, frame)) = Frame::from_bytes((&bytes, 0)) {
    adsb_airplanes.action(frame, (lat, long), max_range);
}
```

## `no_std` support
Add the following to your `Cargo.toml` file to enable `no_std` code only:
```text
default-features = false
features = ["alloc"]
```
