# Changelog
All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

adsb_deku follows semvar when versioning, but apps are not required to follow the semvar convention.

## [Unreleased]
- [apps/radar] Removed blocking TcpStream, thus making tui work while waiting for new ADSB message.

## [v0.1.1] 2021-10-12
### Apps
- [apps/radar] Add `--disable-lat-long. This removes the display of the lat/long position in radar mode and just displays ICAO callsign.
- [apps/radar] Add Coverage tab. Instead of displaying the currently tracked aircrafts, display all detected aircrafts over time as plots.
- [apps/radar] Fix lat/long scaling issues in both display modes

## [v0.1.0] 2021-09-21
- [adsb_deku] Initial Release
- [apps/radar] Initial Release
- [apps/1090] Initial Release
