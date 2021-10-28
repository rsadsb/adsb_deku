use std::collections::HashMap;
use std::fmt;
use std::time::SystemTime;

use adsb_deku::adsb::ME;
use adsb_deku::{cpr, Altitude, CPRFormat, Frame, DF, ICAO};

#[derive(Debug)]
pub struct AirplaneCoor {
    /// [odd, even]
    pub altitudes: [Option<Altitude>; 2],
    /// last time of frame Rx
    pub last_time: SystemTime,
}

impl Default for AirplaneCoor {
    fn default() -> Self {
        Self {
            altitudes: [None, None],
            last_time: SystemTime::now(),
        }
    }
}

#[derive(Debug, Default)]
pub struct Airplanes(pub HashMap<ICAO, AirplaneCoor>);

impl fmt::Display for Airplanes {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        for key in self.0.keys() {
            let value = self.lat_long_altitude(*key);
            if let Some(value) = value {
                writeln!(f, "{}: {:?}", key, value)?;
            }
        }
        Ok(())
    }
}

impl Airplanes {
    pub fn new() -> Self {
        Self(HashMap::new())
    }

    /// Add `Altitude` from adsb frame
    pub fn add_extended_quitter_ap(&mut self, icao: ICAO, frame: Frame) {
        let airplane_coor = self.0.entry(icao).or_insert_with(AirplaneCoor::default);
        if let DF::ADSB(adsb) = frame.df {
            if let ME::AirbornePositionBaroAltitude(altitude) = adsb.me {
                match altitude.odd_flag {
                    CPRFormat::Odd => {
                        *airplane_coor = AirplaneCoor {
                            altitudes: [airplane_coor.altitudes[0], Some(altitude)],
                            last_time: SystemTime::now(),
                        };
                    },
                    CPRFormat::Even => {
                        *airplane_coor = AirplaneCoor {
                            altitudes: [Some(altitude), airplane_coor.altitudes[1]],
                            last_time: SystemTime::now(),
                        };
                    },
                }
            }
        }
    }

    /// Calculate latitude, longitude and altitude
    pub fn lat_long_altitude(&self, icao: ICAO) -> Option<(cpr::Position, u32)> {
        match self.0.get(&icao) {
            Some(altitudes) => {
                if let (Some(first_altitude), Some(second_altitude)) =
                    (altitudes.altitudes[0], altitudes.altitudes[1])
                {
                    cpr::get_position((&first_altitude, &second_altitude))
                        .map(|position| (position, first_altitude.alt))
                } else {
                    None
                }
            },
            None => None,
        }
    }

    /// Calculate all latitude/longitude from Hashmap of current "seen" aircrafts
    pub fn all_lat_long_altitude(&self) -> Vec<cpr::Position> {
        let mut all_lat_long = vec![];
        for altitudes in self.0.values() {
            if let (Some(first_altitude), Some(second_altitude)) =
                (altitudes.altitudes[0], altitudes.altitudes[1])
            {
                if let Some(position) = cpr::get_position((&first_altitude, &second_altitude))
                    .map(|position| (position, first_altitude.alt))
                {
                    all_lat_long.push(position.0);
                }
            }
        }

        all_lat_long
    }

    /// Remove airplane after not active for a time
    pub fn prune(&mut self) {
        self.0
            .retain(|_, v| v.last_time.elapsed().unwrap() < std::time::Duration::from_secs(60));
    }
}
