# Changelog
All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

* `adsb_deku`(published library) follows semvar when versioning
* `apps` follow semvar of `adsb_deku` 

## [Unreleased]
### rsadsb_common
- Added this library to have a library for the common data structures required for keeping aircraft
  details in memory for embedded and non-embedded environments.
- This library was previously in `apps/src/lib.rs`,
  and has been updated to support embedded `no_std` environments.

### adsb_deku
- `Altitude::alt` has been changed to the size of `u16`, and is now correctly enclosed in an `Option`.
- `AC13Field::altitude` has been change to the size of `u16`.
- Add asserts for empty `CapabilityClassAirborne::{reserved0, reserved1}`.
- `ControlField` was refactored for TisB support, moving from an enum into `ME` and `ICAO` fields.
- add `no_std` support through `--default-features = false` and `features = alloc`. See [rsadsb-embedded](https://github.com/rsadsb/rsadsb-embedded) for example usage.

### radar
- Release binary is now stripped. ~2.26MB -> ~1.24MB. MSRV is bumped to `1.59`.
- Remove unsecure `chrono` crate with secure `time` crate usage.
- Add `track`, `heading` to Map tab.
   - `Track`: Display the previous positions of an aircraft since it was tracked. Use key: `t` to turn off, or 
     `--disable-track` cli option.
   - `Heading`: Display an arrow to show the direction of the tracked aircraft. Use key: `h` to turn off, or 
     `--disable-heading` cli option.
- Add key: `i` to trigger disable of ICAO names above aircraft positions.
- Add Stats Tab, with `Max Distance`, `Most Airplanes`, and `Total Airplanes Tracked`.
- Fix terminal escape codes for mouse control. Thanks ([@paunstefan](https://github.com/paunstefan)) ([!124](https://github.com/rsadsb/adsb_deku/pull/124)).
- Reduce precision of all `f32`s to 3. (for longitude, latitude, heading displays).
- Add `--retry-tcp` for trying to connect to a dump1090 instance if it crashes. Add tui screen to alert user instead of crashing.
- Fix usize overflow when selecting airplanes. Thanks ([@andelf](https://github.com/andelf)) ([!140](https://github.com/rsadsb/adsb_deku/pull/140)).

### 1090
- feat: Release binary is now stripped. ~1.2MB -> 440KB. MSRV is bumped to `1.59`.

## [v0.5.1] 2022-02-13

### radar
- fix: Swap Airplanes table "Latitude" and "Longitude", found by reddit user @BanksOfTheOuchita ([!111](https://github.com/rsadsb/adsb_deku/pull/111))

## [v0.5.0] 2022-02-12
### adsb_deku
- refactor: improve `fmt::Display` of `ControlField`
- fix(breaking): change `OperationalCodeSurface.reserved` from `u16` to `u8`
- fix: Handle negative cpr `m` value. Thanks ([@amacd31](https://github.com/amacd31)) ([!78](https://github.com/rsadsb/adsb_deku/pull/78))
- bump msrv to `1.58.1`

### apps/radar
- fix: breaking clap change, same syntax as before for `--cities`
- fix: time related unwrap(). Thanks ([@Jachdich](https://github.com/Jachdich)) ([!57](https://github.com/rsadsb/adsb_deku/pull/57))
- feat: change logs to rotate daily instead of hourly
- feat: add debug and error tracing of bytes and `adsb_deku::Frame`
- feat: improve performance of latitude/longitude calculation
- feat: add Mouse control for Map/Coverage lat/long position
- feat: add Mouse control for tab selection
- feat: add `--touchscreen`, three left-side buttons for zoom out, zoom in, reset screen actions
- refactor: general code improvements and adding const usage
- feat: add version info to logging
- feat: improve user facing errors
- feat(breaking): use mercator projection for map/coverage tabs, change `--scale` usage
- feat: Update to clap v3.0.0
- feat(breaking): change `--scale` to use * and /
- feat: Add ctrl+c as quit option
- feat: fix position mis decoding with ([!101](https://github.com/rsadsb/adsb_deku/pull/101)), fixing: ([#21](https://github.com/rsadsb/adsb_deku/issues/21))
- feat(breaking): `--cities` has been renamed to `--locations`
- bump msrv to `1.58.1`
- feat: add `--airports` and `--airports-tz-filter` for import csv file from https://github.com/mborsetti/airportsdata. ([!103](https://github.com/rsadsb/adsb_deku/pull/103)), fixing: ([#39](https://github.com/rsadsb/adsb_deku/issues/39))


### apps/1090
- bump msrv to `1.58.1`

### Other
- feat: add test, check, and release binaries for x86_64-unknown-linux-gnu, armv7-unknown-linux-gnueabihf, and aarch64-linux-android

## [v0.4.0] 2021-12-08
### adsb_deku
- Add `AircraftStatusType::ACASRaBroadcast`
- Add `OperationStatus::Reserved`
- Add `AirborneVelocityMessage::{Reserved0(1), Reserved1(5..=7)}`
- Assert `TargetStateAndStatus.subtype` == 1. This is currently tracked by: [#30](https://github.com/rsadsb/adsb_deku/issues/30)
- Rename `TargetStateAndStatusInformation.vnac` to `vnav`

### Apps
- [radar] Use TAB key to change tabs ([@wiseman](https://github.com/wiseman)) ([!26](https://github.com/rsadsb/adsb_deku/pull/26))
- [radar] Add Call sign, Speed, Vertical Speed, Number of messages to Table view. ([@wiseman](https://github.com/wiseman)) ([!29](https://github.com/rsadsb/adsb_deku/pull/29))
- [radar] fix broken TCP pipe to ADS-B server, exiting correctly
- [radar] *Coverage* Optimize render by reducing the size of Vec
- [radar] Add `--gpsd` and `--gpsd-ip` for automatic updating of lat/long position from a gpsd daemon on port: 2947.
- [radar] Add `--scale` to control scale of Radar and Coverage. Closes: [#36](https://github.com/rsadsb/adsb_deku/issues/36)
- [radar] Show Airplanes (amount) in tui Titles. Closes: [#31](https://github.com/rsadsb/adsb_deku/issues/31)
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
- Fix [#8](https://github.com/rsadsb/adsb_deku/issues/8) - Support `ME::AircraftOperationStatus` Surface Status message parsing
- Add `QNH` to `fmt::Display` for `ME::TargetStateAndStatusInformation`
- Remove `NUC/NIC` from `fmt::Display` for `Altitude`
- Fix/Add Emergency Status to `fmt::Display` for `ME::AircraftStatus`
- Fix [#10](https://github.com/rsadsb/adsb_deku/issues/10) - Support `AirborneVelocity` Airspeed `fmt::Display`
- Fix [#11](https://github.com/rsadsb/adsb_deku/issues/11) and [#12](https://github.com/rsadsb/adsb_deku/issues/12) - Add `ME::NoPosition` and `fmt::Display`
- Add `fmt::Display` for `ME::Reserved0`
- Add `fmt::Display` for `ME::Reserved1`
- Fix [#13](https://github.com/rsadsb/adsb_deku/issues/13) - Correct Altitude for Mode C Messages, thanks ([@wiseman](https://github.com/wiseman))
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
- [radar, 1090] Add --host and --port ([@wiseman](https://github.com/wiseman)) ([!1](https://github.com/rsadsb/adsb_deku/pull/1))

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
