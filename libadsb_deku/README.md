# adsb_deku library
Minimum required rust version: `1.64`.

Add the following lines to your Cargo.toml file:
```text
adsb_deku = "0.7.0"
```

## Support
### Downlink Format support
|  DF  |  Name                           |  Section    |
| ---- | ------------------------------- | ----------- |
| 0    | Short Air-Air Surveillance      | 3.1.2.8.2   |
| 4    | Surveillance Altitude Reply     | 3.1.2.6.5   |
| 5    | Surveillance Identity Reply     | 3.1.2.6.7   |
| 11   | All Call Reply                  | 2.1.2.5.2.2 |
| 16   | Long Air-Air Surveillance       | 3.1.2.8.3   |
| 17   | Extended Squitter(ADS-B)        | 3.1.2.8.6   |
| 18   | Extended Squitter(TIS-B)        | 3.1.2.8.7   |
| 19   | Extended Squitter(Military)     | 3.1.2.8.8   |
| 20   | Comm-B Altitude Reply           | 3.1.2.6.6   |
| 21   | Comm-B Identity Reply           | 3.1.2.6.8   |
| 24   | Comm-D                          | 3.1.2.7.3   |

### Comm-B support
|  BDS  |  Name                               |  Table      |
| ----  | ----------------------------------- | ----------- |
| (0,0) | Empty                               |             |
| (1,0) | Data Link Capability                | A-2-16      |
| (2,0) | Aircraft Identification             | A-2-32      |

### ME support for ADSB Messages
|  ME(Type Code)  |  Name                          |
| --------------- | ------------------------------ |
| 0               | NoPosition                     |
| 1..=4           | AircraftIdentification         |
| 5..=8           | SurfacePosition                |
| 9..=18          | AirbornePositionBaroAltitude   |
| 19              | AirborneVelocity               |
| 20..=22         | AirbornePositionGNSSAltitude   |
| 23              | Reserved0                      |
| 24              | SurfaceSystemStatus            |
| 25..=27         | Reserved1                      |
| 28              | AircraftStatus                 |
| 29              | TargetStateAndStatusInformation|
| 30              | AircraftOperationalCoordination|
| 31              | AircraftOperationStatus        |

## Example

The following example shows off reading from ADS-B bytes from a demodulation server into our `Frame`
struct and then executing the `fmt::Display` Trait for display of information.
```rust
use hexlit::hex;
use adsb_deku::Frame;

let bytes = hex!("8da2c1bd587ba2adb31799cb802b");
let frame = Frame::from_bytes(&bytes).unwrap();
assert_eq!(
        r#" Extended Squitter Airborne position (barometric altitude)
  Address:       a2c1bd (Mode S / ADS-B)
  Air/Ground:    airborne
  Altitude:      23650 ft barometric
  CPR type:      Airborne
  CPR odd flag:  even
  CPR latitude:  (87769)
  CPR longitude: (71577)
"#,
    frame.to_string()
);
```

Build the docs(`> cargo doc`), or see [docs.rs](https://docs.rs/adsb_deku) for complete public API documentation.

## Contributing

### Testing

Test data was generated using a rtl-sdr with `dump1090-fa`.
```text
> cargo test
```

For testing this library, you can run our app `1090` with the following options for exiting program 
on missing `fmt::Display` or bytes protocol decode.
```text
> cd ../apps
> cargo r --release --bin 1090 -- --debug --disable-airplanes --panic-decode --panic-display
```

This library is also fuzzed, ensuring no panic when parsing from demodulated bytes.
```text
> cargo fuzz run fuzz_target_1
```

### fmt
```text
> cargo +nightly fmt
```

## Benchmark
Benchmarking is done against a file containing `215606` ADS-B messages: [lax-messages.txt](tests/lax-messages.txt).
Quick math `(215606 / 692.80)` says the average speed of decoding is `~311.21 ms` a message.
A `~3%` speedup can be gained on some systems by using  `RUSTFLAGS="-C target-cpu=native"`
```text
> cargo bench
lax_messsages           time:   [680.70 ms 692.82 ms 704.99 ms]
```

## Derivation
Derived from Aeronautical Telecommunications Volume IV: Surveillance and Collision Avoidance Systems, Fifth Edition and ICAO 9871.

## `no_std` support
Add the following to your `Cargo.toml` file to enable `no_std` code only:
```text
default-features = false
features = ["alloc"]
```
