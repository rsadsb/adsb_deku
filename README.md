# adsb_deku

[![Actions Status](https://github.com/wcampbell0x2a/adsb_deku/workflows/CI/badge.svg)](https://github.com/wcampbell0x2a/adsb_deku/actions)

Decoder for ADS-B(Automatic Dependent Surveillance-Broadcast)/Mode-S protocol Downlink Format packets from 1090mhz.
Derived from Aeronautical Telecommunications Volume IV: Surveillance and Collision Avoidance Systems, Fifth Edition.

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
use adsb_deku::{Frame, DekuContainerRead};

let bytes = hex!("8da2c1bd587ba2adb31799cb802b");
let frame = Frame::from_bytes((&bytes, 0)).unwrap().1;
assert_eq!(
        r#" Extended Squitter Airborne position (barometric altitude) (11)
  ICAO Address:  a2c1bd (Mode S / ADS-B)
  Air/Ground:    airborne
  Altitude:      23650 ft barometric
  CPR type:      Airborne
  CPR odd flag:  even
  CPR NUCp/NIC:  7
  CPR latitude:  (87769)
  CPR longitude: (71577)
  CPR decoding:  global
"#,
    frame.to_string()
);
```

## testing and development

### testing

Test data was generated using my rtl-sdr with `dump1090-fa`.
```text
cargo test
```

### fmt
```
cargo +nightly fmt
```

### applications

## dump1090-fa

Dump protocol bytes using this library in the same fashion as `dump1090-fa`.

```text
# Startup dump1090-fa (https://github.com/flightaware/dump1090.git)
./dump1090 --net --quiet

# Startup 1090 decode chain using this library
cargo r --example 1090
```

## radar tui

Display a radar like tui (terminal user interface) showing aircraft: ICAO address, lat, long.
The terminal used is [cool-retro-terminal](https://github.com/Swordfish90/cool-retro-term).

![Radar Example](/media/2021-09-06-082636_1804x1062_scrot.png)
```text
# Startup dump1090-fa (https://github.com/flightaware/dump1090.git)
./dump1090 --net --quiet

# Startup "radar" display in tui relative to your sdr position
cargo r --example radar -- --lat="50.0" --long="50.0" --cities "(name,lat,long)" "(name,lat,long)"
```
