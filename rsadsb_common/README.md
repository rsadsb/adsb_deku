# rsadsb_common

Common library data structures and functions for `adsb_deku` applications.

This library is not published on `crates.io`. If needed, it could be published.

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
