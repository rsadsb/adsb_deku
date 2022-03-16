# rsadsb_common

Common library data structures and functions for `adsb_deku` applications.

This library is not published on `crates.io`. If needed, it could be published.

Run `cargo doc` in this directory to generate documentation.

## Usage
```rust
    # use adsb_deku::Frame;
    # use adsb_deku::deku::DekuContainerRead;
    # use rsadsb_common::Airplanes;
    #
    # let lat = 0.0;
    # let long = 0.0;
    # let bytes = vec![];
    let mut adsb_airplanes = Airplanes::new();
    if let Ok((bytes_left, frame)) = Frame::from_bytes((&bytes, 0)) {
        adsb_airplanes.action(frame, (lat, long));
    }
```

