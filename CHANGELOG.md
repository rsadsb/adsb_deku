# Changelog
All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

`adsb_deku` follows semvar when versioning, but apps are not required to follow the semvar convention.

## [Unreleased]

### adsb_deku

### apps/radar
- fix breaking clap change, same syntax as before for `--cities`.
- fix time related unwrap(). Thanks ([@Jachdich](https://github.com/Jachdich)) ([!57](https://github.com/rsadsb/adsb_deku/pull/57))
- change logs to rotate daily instead of hourly
- add debug tracing of bytes and `Frame`

### apps/1090

### Other

## [v0.4.0] 2021-12-08
### adsb_deku
- Add `AircraftStatusType::ACASRaBroadcast`
- Add `OperationStatus::Reserved`
- Add `AirborneVelocityMessage::{Reserved0(1), Reserved1(5..=7)}`
- Assert `TargetStateAndStatus.subtype` == 1. This is currently tracked by: [#30](https://github.com/wcampbell0x2a/adsb_deku/issues/30)
- Rename `TargetStateAndStatusInformation.vnac` to `vnav`

### Apps
- [radar] Use TAB key to change tabs ([@wiseman](https://github.com/wiseman)) ([!26](https://github.com/wcampbell0x2a/adsb_deku/pull/26))
- [radar] Add Call sign, Speed, Vertical Speed, Number of messages to Table view. ([@wiseman](https://github.com/wiseman)) ([!29](https://github.com/wcampbell0x2a/adsb_deku/pull/29))
- [radar] fix broken TCP pipe to ADS-B server, exiting correctly
- [radar] *Coverage* Optimize render by reducing the size of Vec
- [radar] Add `--gpsd` and `--gpsd-ip` for automatic updating of lat/long position from a gpsd daemon on port: 2947.
- [radar] Add `--scale` to control scale of Radar and Coverage. Closes: [#36](https://github.com/wcampbell0x2a/adsb_deku/issues/36)
- [radar] Show Airplanes (amount) in tui Titles. Closes: [#31](https://github.com/wcampbell0x2a/adsb_deku/issues/31)
- [radar] Add `--filter-time` for reducing the amount of mis-decodes. Reduces the default max time between messages from 60 to 10 seconds.
- [1090] Remove `--disable-airplanes`. This feature differs from the radar implementation, and thus is outdated.
- [radar] *Coverage* Add gradient of whitespace gray->white to denote how often an aircraft was seen. This functions as a heatmap of sorts.
- [radar] Tracing / Logging
    - Add *tracing* for logging to a default `./logs` directory information about ADS-B information.
    - Control base folder using the `--log-folder` option.
    - We use the environment variable `RUST_LOG` for controlling the level of verbosity. The default is info.
    - The following is an example of using the debug level.
```text
> RUST_LOG=debug cargo r -- ...
```

### Other
- Add Benchmark tools and readme information
- Rework README.md file

## [v0.3.0] 2021-10-31
### adsb_deku
- Fix [#8](https://github.com/wcampbell0x2a/adsb_deku/issues/8) - Support `ME::AircraftOperationStatus` Surface Status message parsing
- Add `QNH` to `fmt::Display` for `ME::TargetStateAndStatusInformation`
- Remove `NUC/NIC` from `fmt::Display` for `Altitude`
- Fix/Add Emergency Status to `fmt::Display` for `ME::AircraftStatus`
- Fix [#10](https://github.com/wcampbell0x2a/adsb_deku/issues/10) - Support `AirborneVelocity` Airspeed `fmt::Display`
- Fix [#11](https://github.com/wcampbell0x2a/adsb_deku/issues/11) and [#12](https://github.com/wcampbell0x2a/adsb_deku/issues/12) - Add `ME::NoPosition` and `fmt::Display`
- Add `fmt::Display` for `ME::Reserved0`
- Add `fmt::Display` for `ME::Reserved1`
- Fix [#13](https://github.com/wcampbell0x2a/adsb_deku/issues/13) - Correct Altitude for Mode C Messages, thanks ([@wiseman](https://github.com/wiseman))
- Support some `BDS` fields for `Comm-B` messages
- Add `ME::AircraftOperationalCoordination`

### Apps
- [radar] Enforce minimum constraint on size of tab text
- [radar] Add `+` and `-` for zooming the map during operation
- [radar] Add `Up`, `Down`, `Left`, and `Right` for moving map (lat/long). `Enter` for resetting Map.
- [radar] Display current Lat/Long in tui
- [radar] Add `Airplanes` tab for display of all airplanes(lat, long, altitude) in table format.
Allow selection in table with Up, Down, and Enter keys for moving to the `Map` tab centered at selected aircraft.

## [v0.2.0] 2021-10-17
### adsb_deku
- Moved all ADS-B related struct/enum parsing into `adsb` module for improve documentation and separation of functionality
- `DF::CommDExtendedLengthMessage` now matches 24..=31
- Improve/Enable parsing/fmt::Display of `Tis-B` messages. Thanks ([@wiseman](https://github.com/wiseman)) for test data
- Add `ME::SurfaceSystemStatus`

### Apps
- Improve clap `--help` for all apps
- [1090] Add `--disable-airplanes` to disable airplane lat/long/altitude output
- [1090] Add `--debug` for displaying Debug trait output
- [1090] Add `--panic-display` and `--panic-decode` for optional testing
- [radar] Rename ADSB Tab to MAP
- [radar] Remove blocking TcpStream, thus making tui work while waiting for new ADS-B message
- [radar, 1090] Add --host and --port ([@wiseman](https://github.com/wiseman)) ([!1](https://github.com/wcampbell0x2a/adsb_deku/pull/1))

## [v0.1.1] 2021-10-12
### Apps
- [radar] Add `--disable-lat-long. This removes the display of the lat/long position in radar mode and just displays ICAO callsign.
- [radar] Add Coverage tab. Instead of displaying the currently tracked aircrafts, display all detected aircrafts over time as plots
- [radar] Fix lat/long scaling issues in both display modes

## [v0.1.0] 2021-09-21
### adsb_deku
- [adsb_deku] Initial Release

### Apps
- [radar] Initial Release
- [1090] Initial Release
