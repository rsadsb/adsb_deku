# adsb_deku

[![Latest Version](https://img.shields.io/crates/v/adsb_deku.svg)](https://crates.io/crates/adsb_deku)
[![Rust Documentation](https://docs.rs/adsb_deku/badge.svg)](https://docs.rs/adsb_deku)
[![Actions Status](https://github.com/rsadsb/adsb_deku/workflows/CI/badge.svg)](https://github.com/rsadsb/adsb_deku/actions)
[![dependency status](https://deps.rs/repo/github/rsadsb/adsb_deku/status.svg)](https://deps.rs/repo/github/rsadsb/adsb_deku)

Decoder for [ADS-B(Automatic Dependent Surveillance-Broadcast)](https://en.wikipedia.org/wiki/Automatic_Dependent_Surveillance%E2%80%93Broadcast) Downlink Format protocol packets from 1090mhz. See [dump1090_rs](https://github.com/rsadsb/dump1090_rs.git) for a Rust demodulator. View planes in the sky around you, with only a [rtl-sdr](https://www.rtl-sdr.com/)!

See [quickstart-guide](https://rsadsb.github.io/quickstart.html) for a quick installation guide.

See [rsadsb-2022.12.29](https://rsadsb.github.io/2022.12.29.html) for latest major release details.

This library uses [deku](https://github.com/sharksforarms/deku) for deserialization of protocol.

![Radar Example](media/peek_2022_08_19.gif)

## Applications

Minimum required rust version: `1.64.0`.

### Client Applications

Client applications use this library to display the data accumulated from an ADS-B demodulation server in various ways.

#### radar tui
An ADS-B client for the terminal written in Rust. Radar connects to an ADS-B demodulation server
and stores this info, allowing an operator to control the display of the data.

- **Map Tab** - Plot currently detected aircraft on lat/long grid. Add known locations with `--locations` option.
- **Coverage Tab** - Plot all detected aircraft lat/long positions since starting radar. Features grayscale heatmap.
- **Aircraft Tab** - Show detailed table of information about currently detected aircraft. Set aircraft position as center lat/long.
- **gpsd** - Derive lat/long from a gpsd instance using `--gpsd` flag.

See [apps/README.md](apps) for map control keybindings.

This application uses [ratatui](https://github.com/ratatui-org/ratatui) for generating the display to the terminal.

```text
# Startup "radar" display in tui relative to your antenna position
> cargo r --bin radar --release -- --lat="50.0" --long="50.0" --locations "(name,lat,long)" "(name,lat,long)"
```

#### 1090
Display protocol data structures and currently tracked planes using this library in the same fashion as `dump1090-fa`
to a terminal stdout. Optionally panic on missing implementation or `fmt::Display`, see `> ./1090 -h`.

```text
# Startup 1090 decode chain using this library
> cargo r --bin 1090 --release -- --debug
```

![1090 Example](media/2021-10-31-093905_676x659_scrot.png)

### Server/Demodulation(External) Applications

This library contains logic for decoding a message, you must use a server for demodulating the message
from 1090mhz into bytes usable by this library. These are called `Server` applications.

#### (Rust) [dump1090_rs](https://github.com/rsadsb/dump1090_rs.git)
Rust translation of the popular C `dump1090` tools, with demodulation and data forwarding functions.
```text
> cargo r --release
```

## Library
See [libadsb_deku](libadsb_deku) for more details.

## Common
See [rsadsb_common](rsadsb_common) for more details.
