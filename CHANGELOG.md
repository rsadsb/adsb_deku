# Changelog
All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

`adsb_deku` follows semvar when versioning, but apps are not required to follow the semvar convention.

## [Unreleased]
### adsb_deku
### Apps
- [radar] Use TAB key to change tabs ([@wiseman](https://github.com/wiseman)) ([!26](https://github.com/wcampbell0x2a/adsb_deku/pull/26))

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
