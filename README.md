# adsb_deku

![adsb_deku](media/logo.png)

[![Latest Version](https://img.shields.io/crates/v/adsb_deku.svg)](https://crates.io/crates/adsb_deku)
[![Rust Documentation](https://docs.rs/adsb_deku/badge.svg)](https://docs.rs/adsb_deku)
[![Actions Status](https://github.com/wcampbell0x2a/adsb_deku/workflows/CI/badge.svg)](https://github.com/wcampbell0x2a/adsb_deku/actions)


Decoder for [ADS-B(Automatic Dependent Surveillance-Broadcast)](https://en.wikipedia.org/wiki/Automatic_Dependent_Surveillance%E2%80%93Broadcast) Downlink Format protocol packets from 1090mhz.

This library uses [deku](https://github.com/sharksforarms/deku) for deserialization of protocol.

## Applications
### Client Applications

Client applications use this library to display the data accumulated from an ADS-B demodulation server in various ways.

#### radar tui
An ADS-B client for the terminal written in Rust. `Radar` connects to an ADS-B demodulation server
and stores this info, allowing an operator to control the display of the data.

- **Map** - Plot currently detected aircraft on lat/long grid.
- **Coverage** - Plot all detected aircraft lat/long positions since starting radar.
- **Aircraft** - Show detailed table of information about currently detected aircraft.

See [apps/README.md](https://github.com/wcampbell0x2a/adsb_deku/tree/update-readme/apps) for keybindings.

This application uses [tui-rs](https://github.com/fdehau/tui-rs) for generating the display to terminal.

```text
# Startup "radar" display in tui relative to your antenna position
> cd apps
> cargo r --bin radar --release -- --lat="50.0" --long="50.0" --cities "(name,lat,long)" "(name,lat,long)"
```

![Radar Example](/media/peek_2021_10_31.gif)

#### 1090
Display protocol data structures and currently tracked planes using this library in the same fashion as `dump1090-fa`
to a terminal stdout. Optionally panic on missing implementation or `fmt::Display`, see `> ./1090 -h`.

```text
# Startup 1090 decode chain using this library
> cd apps
> cargo r --bin 1090 --release -- --debug
```

![1090 Example](/media/2021-10-31-093905_676x659_scrot.png)

### Server/Demodulation(External) Applications

This library contains logic for decoding a message, you must use a server for demodulating the message
from 1090mhz into bytes usable by this library. These are called `Server` applications.

#### (Rust) [dump1090_rs](https://github.com/wcampbell0x2a/dump1090_rs.git)
This is a fork of [dump1090_rs](https://github.com/johnwstanford/dump1090_rs) with only demodulation
and data forwarding functions. I recommend this for the _full_ Rust experience. In my testing
this has the same performance as the C demodulator.
```text
> cargo r --release
```

#### (C) [dump1090_fa](https://github.com/flightaware/dump1090.git)
This is the more tested application/implementation of 2400 sample rate demodulation used by flightaware.

```text
> ./dump1090 --net --quiet
```

## adsb_deku library
Add the following lines to your Cargo.toml file:
```text
adsb_deku = "*"
```

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

### example

The following example shows off reading from ADS-B bytes from a demodulation server into our `Frame`
struct and then executing the `fmt::Display` Trait for display of information.
```rust
use hexlit::hex;
use adsb_deku::Frame;
use adsb_deku::deku::DekuContainerRead;

let bytes = hex!("8da2c1bd587ba2adb31799cb802b");
let frame = Frame::from_bytes((&bytes, 0)).unwrap().1;
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

### Contributing

#### testing

Test data was generated using a rtl-sdr with `dump1090-fa`.
```text
> cargo test
```

For testing this library, you can run our app `1090` with the following options for exiting program 
on missing `fmt::Display` or bytes protocol decode.
```text
> cargo r --release --bin 1090 -- --debug --disable-airplanes --panic-decode --panic-display
```

This library is also fuzzed, ensuring no panic when parsing from demodulated bytes.
```text
> cargo fuzz run fuzz_target_1
```

#### fmt
```text
> cargo +nightly fmt
```

## Derivation
Derived from Aeronautical Telecommunications Volume IV: Surveillance and Collision Avoidance Systems, Fifth Edition and ICAO 9871.

