# adsb_deku

[![Latest Version](https://img.shields.io/crates/v/adsb_deku.svg)](https://crates.io/crates/adsb_deku)
[![Rust Documentation](https://docs.rs/adsb_deku/badge.svg)](https://docs.rs/adsb_deku)
[![Actions Status](https://github.com/wcampbell0x2a/adsb_deku/workflows/CI/badge.svg)](https://github.com/wcampbell0x2a/adsb_deku/actions)

Decoder for [ADS-B(Automatic Dependent Surveillance-Broadcast)/Mode-S](https://en.wikipedia.org/wiki/Automatic_Dependent_Surveillance%E2%80%93Broadcast) protocol Downlink Format packets from 1090mhz.
Derived from Aeronautical Telecommunications Volume IV: Surveillance and Collision Avoidance Systems, Fifth Edition and ICAO 9871.

This library uses [deku](https://github.com/sharksforarms/deku) for serialization/deserialization of protocol.

## support
|  DF  |  Name                           |  Section    |
| ---- | ------------------------------- | ----------- |
| 0    | Short Air-Air Surveillance      | 3.1.2.8.2   |
| 4    | Surveillance Altitude Reply     | 3.1.2.6.5   |
| 5    | Surveillance Identity Reply     | 3.1.2.6.7   |
| 11   | All Call Reply                  | 2.1.2.5.2.2 |
| 16   | Long Air-Air Surveillance       | 3.1.2.8.3   |
| 17   | Extended Squitter(ADS-B)        | 3.1.2.8.6   |
| 18   | Extended Squitter(Supplementary)| 3.1.2.8.7   |
| 19   | Extended Squitter(Military)     | 3.1.2.8.8   |
| 20   | Comm-B Altitude Reply           | 3.1.2.6.6   |
| 21   | Comm-B Identity Reply           | 3.1.2.6.8   |
| 24   | Comm-D                          | 3.1.2.7.3   |

## example usage
```rust
use hexlit::hex;
use adsb_deku::Frame;
use adsb_deku::deku::DekuContainerRead;

let bytes = hex!("8da2c1bd587ba2adb31799cb802b");
let frame = Frame::from_bytes((&bytes, 0)).unwrap().1;
assert_eq!(
        r#" Extended Squitter Airborne position (barometric altitude)
  ICAO Address: a2c1bd (Mode S / ADS-B)
  Air/Ground:    airborne
  Altitude:      23650 ft barometric
  CPR type:      Airborne
  CPR odd flag:  even
  CPR NUCp/NIC:  ?
  CPR latitude:  (87769)
  CPR longitude: (71577)
"#,
    frame.to_string()
);
```

## testing and development

### testing

Test data was generated using my rtl-sdr with `dump1090-fa`.
```text
> cargo test
```

This library is also fuzzed, ensuring no panic when parsing from demodulated bytes.
```text
> cargo fuzz run fuzz_target_1
```

### fmt
```text
> cargo +nightly fmt
```

# Applications

## Server/Demodulation(External) Applications
### (C) [dump1090_fa](https://github.com/flightaware/dump1090.git)
This is the most tested application/implementation of 2400 sample rate demodulation used by flightaware.

```text
> ./dump1090 --net --quiet
```

### (Rust) [dump1090_rs](https://github.com/wcampbell0x2a/dump1090_rs.git)
This is a fork of [dump1090_rs](https://github.com/johnwstanford/dump1090_rs) with only demodulation
and data forwarding functions.
```text
> ./cargo r --release
```

## Client Applications
### 1090

Display protocol data structures and currently tracked planes using this library in the same fashion as `dump1090-fa`.

```text
# Startup 1090 decode chain using this library
> cd apps
> cargo r --bin 1090 --release -- --debug
```

![1090 Example](/media/2021-10-15-173245_686x1025_scrot.png)

### radar tui

Display a radar like tui (terminal user interface) showing aircraft: ICAO address, lat, long.
The terminal used is [cool-retro-terminal](https://github.com/Swordfish90/cool-retro-term).

```text
# Startup "radar" display in tui relative to your sdr position
> cd apps
> cargo r --bin radar --release -- --lat="50.0" --long="50.0" --cities "(name,lat,long)" "(name,lat,long)"
```

#### Radar Mode
![Radar Example](/media/2021-09-06-082636_1804x1062_scrot.png)

#### Coverage Mode
![Coverage Example](/media/2021-10-12-194028_1077x998_scrot.png)
