# Apps

See main README.md for app sample images.

## 1090
```
1090 0.3.0

wcampbell0x2a

Dump ADS-B protocol info from demodulator

USAGE:
    1090 [OPTIONS]

OPTIONS:
        --debug                Display debug of adsb::Frame
        --disable-airplanes    Disable display of currently tracked airplanes lat/long/altitude
    -h, --help                 Print help information
        --host <HOST>          ip address of ADS-B demodulated bytes server [default: localhost]
        --panic-decode         Panic on adsb_deku::Frame::from_bytes() error
        --panic-display        Panic on adsb_deku::Frame::fmt::Display not implemented
        --port <PORT>          port of ADS-B demodulated bytes server [default: 30002]
    -V, --version              Print version information
```

## radar
```
radar 0.3.0

wcampbell0x2a

TUI Display of ADS-B protocol info from demodulator

USAGE:
    radar [OPTIONS] --lat <LAT> --long <LONG>

OPTIONS:
        --cities <CITIES>...    Vector of cities [(name, lat, long),..]
        --disable-lat-long      Disable output of latitude and longitude on display
    -h, --help                  Print help information
        --host <HOST>           [default: localhost]
        --lat <LAT>             Antenna location latitude
        --long <LONG>           Antenna location longitude
        --port <PORT>           [default: 30002]
    -V, --version               Print version information
```

### Key Bindings

### Any Tab
|  Key  |  Action                    |
| ----- | -------------------------- |
| F1    | Move to Radar screen       |
| F2    | Move to Coverage screen    |
| F2    | Move to Airplanes screen   |


### Map or Coverage
|  Key  |  Action                    |
| ----- | -------------------------- |
| -     | Zoom out                   |
| +     | Zoom in                    |
| Up    | Move Map Up                |
| Down  | Move Map Down              |
| Left  | Move Map Left              |
| Right | Move Map Right             |
| Enter | Reset Map                  |

### Airplanes
|  Key  |  Action                    |
| ----- | -------------------------- |
| Up    | Move selection upward      |
| Down  | Move selection downward    |
| Enter | Center Map tab on aircraft |
