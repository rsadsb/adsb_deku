use std::net::Ipv4Addr;
use std::num::ParseFloatError;
use std::str::FromStr;

use clap::Parser;

/// Parsing struct for the --locations clap parameter
#[derive(Debug, Clone, PartialEq)]
pub struct Location {
    pub name: String,
    pub lat: f64,
    pub long: f64,
}

impl FromStr for Location {
    type Err = ParseFloatError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let coords: Vec<&str> = s.trim_matches(|p| p == '(' || p == ')').split(',').collect();

        let lat_fromstr = coords[1].parse::<f64>()?;
        let long_fromstr = coords[2].parse::<f64>()?;

        Ok(Self { name: coords[0].to_string(), lat: lat_fromstr, long: long_fromstr })
    }
}

/// Parsing struct for the --range-circles clap parameter
#[derive(Debug, Clone, PartialEq)]
pub struct RangeCircles(pub Vec<f64>);

impl FromStr for RangeCircles {
    type Err = ParseFloatError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let ranges: Result<Vec<f64>, ParseFloatError> = s.split(',').map(|r| r.parse::<f64>()).collect();
        Ok(Self(ranges?))
    }
}

const AFTER_TEST: &str = r#"Environment Variables:
    RUST_LOG: See "https://docs.rs/tracing-subscriber/latest/tracing_subscriber/fmt/index.html#filtering-events-with-environment-variables"
"#;

#[derive(Debug, Clone, Parser, PartialEq)]
#[command(
    version,
    name = "radar",
    author = "wcampbell0x2a",
    about = "TUI Display of ADS-B protocol info from demodulator",
    after_help = AFTER_TEST,
)]
pub struct Opts {
    /// ip address / hostname of ADS-B server / demodulator
    #[arg(long, default_value = "127.0.0.1")]
    pub host: Ipv4Addr,

    /// port of ADS-B server / demodulator
    #[arg(long, default_value = "30002")]
    pub port: u16,

    /// Antenna location latitude, this use for aircraft position algorithms.
    ///
    /// This is overwritten when using the `--gpsd` option.
    #[arg(long)]
    pub lat: f64,

    /// Antenna location longitude
    ///
    /// This is overwritten when using the `--gpsd` option.
    #[arg(long)]
    pub long: f64,

    /// Vector of location [(name, lat, long),..] to display on Map
    #[arg(long, num_args = 1..)]
    pub locations: Vec<Location>,

    /// Disable output of latitude and longitude on Map
    #[arg(long)]
    pub disable_lat_long: bool,

    /// Display only ICAO number instead of Callsign / Tail Number
    #[arg(long)]
    pub disable_callsign: bool,

    /// Disable output of icao address of airplane on Map
    #[arg(long)]
    pub disable_icao: bool,

    /// Disable display of angles on aircraft within Map display showing the direction of the aircraft.
    #[arg(long)]
    pub disable_heading: bool,

    /// Disable display of previous positions of aircraft on Map
    #[arg(long)]
    pub disable_track: bool,

    /// Zoom level of Map and Coverage (-=zoom out/+=zoom in)
    #[arg(long, default_value = ".12")]
    pub scale: f64,

    /// Enable automatic updating of lat/lon from gpsd(<https://gpsd.io/>) server.
    ///
    /// This overwrites the `--lat` and `--long`
    #[arg(long)]
    pub gpsd: bool,

    /// Ip address of gpsd
    #[arg(long, default_value = "localhost")]
    pub gpsd_ip: String,

    /// Seconds since last message from airplane, triggers removal of airplane after time is up
    #[arg(long, default_value = "120")]
    pub filter_time: u64,

    #[arg(long, default_value = "logs")]
    pub log_folder: String,

    /// Enable three tabs on left side of screen for zoom out/zoom in/and reset
    #[arg(long)]
    pub touchscreen: bool,

    /// Limit parsing of ADS-B messages to `DF::ADSB(17)` num_messages
    ///
    /// This can improve performance of just needing to read radar related messages
    #[arg(long)]
    pub limit_parsing: bool,

    /// Import downloaded csv file for FAA Airport from <https://github.com/mborsetti/airportsdata>
    #[arg(long)]
    pub airports: Option<String>,

    /// comma seperated filter for --airports timezone data, such as: "America/Chicago,America/New_York"
    #[arg(long)]
    pub airports_tz_filter: Option<String>,

    /// retry TCP connection to dump1090 instance if connecton is lost/disconnected
    #[arg(long)]
    pub retry_tcp: bool,

    /// Control the max range of the receiver in km
    #[arg(long, default_value = "500")]
    pub max_range: f64,
    
    /// Comma-separated list of range circles to display (in km)
    /// Example: --range-circles=100,200,300,400
    #[arg(long, default_value = "100,200,300,400")]
    pub range_circles: RangeCircles,
    
    /// Disable display of range circles on Map and Coverage
    #[arg(long)]
    pub disable_range_circles: bool,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_cli() {
        let t_str = ["--disable-lat-long", "--lat=35.00", "--long=-80.00"];
        let opt = Opts::try_parse_from(t_str).unwrap();
        let exp_opt = Opts {
            host: Ipv4Addr::LOCALHOST,
            port: 30002,
            lat: 35.0,
            long: -80.0,
            locations: vec![],
            disable_lat_long: false,
            disable_callsign: false,
            scale: 0.12,
            gpsd: false,
            gpsd_ip: "localhost".to_string(),
            filter_time: 120,
            log_folder: "logs".to_string(),
            touchscreen: false,
            limit_parsing: false,
            airports: None,
            airports_tz_filter: None,
            disable_icao: false,
            disable_heading: false,
            disable_track: false,
            retry_tcp: false,
            max_range: 500.0,
            range_circles: RangeCircles(vec![100.0, 200.0, 300.0, 400.0]),
            disable_range_circles: false,
        };
        assert_eq!(exp_opt, opt);

        let t_str = [
            "--disable-lat-long",
            "--lat=35.00",
            "--long=-80.00",
            "--locations",
            "(a,56.5,57.2)",
            "(b,1.0,2.0)",
        ];
        let opt = Opts::try_parse_from(t_str).unwrap();
        let exp_opt = Opts {
            host: Ipv4Addr::LOCALHOST,
            port: 30002,
            lat: 35.0,
            long: -80.0,
            locations: vec![
                Location { name: "a".to_string(), lat: 56.5, long: 57.2 },
                Location { name: "b".to_string(), lat: 1.0, long: 2.0 },
            ],
            disable_lat_long: false,
            disable_callsign: false,
            scale: 0.12,
            gpsd: false,
            gpsd_ip: "localhost".to_string(),
            filter_time: 120,
            log_folder: "logs".to_string(),
            touchscreen: false,
            limit_parsing: false,
            airports: None,
            airports_tz_filter: None,
            disable_icao: false,
            disable_heading: false,
            disable_track: false,
            retry_tcp: false,
            max_range: 500.0,
            range_circles: RangeCircles(vec![100.0, 200.0, 300.0, 400.0]),
            disable_range_circles: false,
        };
        assert_eq!(exp_opt, opt);
    }
}
