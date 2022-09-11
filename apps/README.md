# Apps

See main README.md for app sample images.

## radar
See `--help` for more information.
```
radar 0.6.1
wcampbell0x2a
TUI Display of ADS-B protocol info from demodulator

USAGE:
    radar [OPTIONS] --lat <LAT> --long <LONG>

OPTIONS:
        --host <HOST>                                ip address / hostname of ADS-B server / demodulator [default: 127.0.0.1]
        --port <PORT>                                port of ADS-B server / demodulator [default: 30002]
        --lat <LAT>                                  Antenna location latitude, this use for aircraft position algorithms
        --long <LONG>                                Antenna location longitude
        --locations <LOCATIONS>...                   Vector of location [(name, lat, long),..] to display on Map
        --disable-lat-long                           Disable output of latitude and longitude on Map
        --disable-callsign                           Display only ICAO number instead of Callsign / Tail Number
        --disable-icao                               Disable output of icao address of airplane on Map
        --disable-heading                            Disable display of angles on aircraft within Map display showing the direction of the aircraft
        --disable-track                              Disable display of previous positions of aircraft on Map
        --scale <SCALE>                              Zoom level of Map and Coverage (-=zoom out/+=zoom in) [default: .12]
        --gpsd                                       Enable automatic updating of lat/lon from gpsd(<https://gpsd.io/>) server
        --gpsd-ip <GPSD_IP>                          Ip address of gpsd [default: localhost]
        --filter-time <FILTER_TIME>                  Seconds since last message from airplane, triggers removal of airplane after time is up [default: 120]
        --log-folder <LOG_FOLDER>                    [default: logs]
        --touchscreen                                Enable three tabs on left side of screen for zoom out/zoom in/and reset
        --limit-parsing                              Limit parsing of ADS-B messages to `DF::ADSB(17)` num_messages
        --airports <AIRPORTS>                        Import downloaded csv file for FAA Airport from <https://github.com/mborsetti/airportsdata>
        --airports-tz-filter <AIRPORTS_TZ_FILTER>    comma seperated filter for --airports timezone data, such as: "America/Chicago,America/New_York"
        --retry-tcp                                  retry TCP connection to dump1090 instance if connecton is lost/disconnected
        --max-range <MAX_RANGE>                      Control the max range of the receiver in km [default: 500]
    -h, --help                                       Print help information
    -V, --version                                    Print version information

Environment Variables:
    RUST_LOG: See "https://docs.rs/tracing-subscriber/latest/tracing_subscriber/fmt/index.html#filtering-events-with-environment-variables"
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
| F3       | Move to Airplanes screen   |
| F4       | Move to Stat screen        |
| F5       | Move to Help screen        |
| l        | control --disable-lat-long |
| i        | control --disable-icao     |
| h        | control --disable-heading  |
| t        | control --disable-track    |
| n        | toggle --diplay-callsign   |
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

## 1090
See `--help` for more information.
```
1090 0.6.1
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


## Contributing

### fmt
```text
> cargo +nightly fmt
```
