# Apps

See main README.md for app sample images.

## 1090
```
1090 0.5.0
wcampbell0x2a
Dump ADS-B protocol info from demodulator

USAGE:
    1090 [OPTIONS]

OPTIONS:
        --debug            Display debug of adsb::Frame
    -h, --help             Print help information
        --host <HOST>      ip address of ADS-B demodulated bytes server [default: localhost]
        --panic-decode     Panic on adsb_deku::Frame::from_bytes() error
        --panic-display    Panic on adsb_deku::Frame::fmt::Display not implemented
        --port <PORT>      port of ADS-B demodulated bytes server [default: 30002]
    -V, --version          Print version information
```

## radar
```
radar 0.5.0
wcampbell0x2a
TUI Display of ADS-B protocol info from demodulator

USAGE:
    radar [OPTIONS] --lat <LAT> --long <LONG>

OPTIONS:
        --disable-lat-long             Disable output of latitude and longitude on display
        --filter-time <FILTER_TIME>    Seconds since last message from airplane, triggers removal of airplane after time is up [default:
                                       10]
        --gpsd                         Enable automatic updating of lat/lon from gpsd(https://gpsd.io/) server
        --gpsd-ip <GPSD_IP>            Ip address of gpsd [default: localhost]
    -h, --help                         Print help information
        --host <HOST>                  ip address / hostname of ADS-B server / demodulator [default: 127.0.0.1]
        --lat <LAT>                    Antenna location latitude
        --limit-parsing                Limit parsing of ADS-B messages to `DF::ADSB(17)` messages
        --locations <LOCATIONS>...     Vector of location [(name, lat, long),..]
        --log-folder <LOG_FOLDER>      [default: logs]
        --long <LONG>                  Antenna location longitude
        --port <PORT>                  port of ADS-B server / demodulator [default: 30002]
        --scale <SCALE>                Zoom level of Radar and Coverage (-=zoom out/+=zoom in) [default: .12]
        --touchscreen                  Enable three tabs on left side of screen for zoom out/zoom in/and reset
    -V, --version                      Print version information
```

### Logging
`radar` is enabled with logging. Use the `RUST_LOG=?` environment variable to control trace level and `--log-folder` to control log base folder location.

### Mouse Bindings
#### Tabs
Control the current tab by clicking on the top-right text.

#### Map and Coverage
Control the position of the lat/long center by dragging your mouse/finger and scroll out/in to control zoom.

#### Touchsreen
Use the `--touchscreen` option for enabling three buttoms for Zoom In/Zoom Out/Reset screen.
This enables those features for platforms without keyboard and mouse usage.

### Key Bindings

#### Any Tab
|  Key     |  Action                    |
| -------- | -------------------------- |
| F1       | Move to Radar screen       |
| F2       | Move to Coverage screen    |
| F2       | Move to Airplanes screen   |
| TAB      | Move to next tab           |
| q        | Quit the app               |
| ctrl + C | Quit the app               |


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

## Contributing

### fmt
```text
> cargo +nightly fmt
```
