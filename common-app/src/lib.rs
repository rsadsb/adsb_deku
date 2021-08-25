use mode_s_deku::{cpr, Altitude, CPRFormat, Frame, DF, ICAO, ME};
use std::collections::HashMap;
use std::fmt;

// [odd, even]
#[derive(Debug)]
pub struct AirplaneCoor(pub [Option<Altitude>; 2]);

impl Default for AirplaneCoor {
    fn default() -> Self {
        Self([None, None])
    }
}

#[derive(Debug)]
pub struct Airplanes(pub HashMap<ICAO, AirplaneCoor>);

impl fmt::Display for Airplanes {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        for (key, _) in &self.0 {
            let value = self.lat_long_altitude(*key);
            if let Some(value) = value {
                writeln!(f, "{}: {:?}", key, value);
            }
        }
        Ok(())
    }
}

impl Airplanes {
    pub fn new() -> Self {
        Self(HashMap::new())
    }

    pub fn add_extended_quitter_ap(&mut self, icao: ICAO, frame: Frame) {
        let airplane = self.0.entry(icao).or_insert(AirplaneCoor::default());
        match frame.df {
            DF::ADSB(adsb) => match adsb.me {
                ME::AirbornePositionBaroAltitude(altitude) => match altitude.odd_flag {
                    CPRFormat::Odd => {
                        *airplane = AirplaneCoor([airplane.0[0].clone(), Some(altitude)])
                    }
                    CPRFormat::Even => {
                        *airplane = AirplaneCoor([Some(altitude), airplane.0[1].clone()])
                    }
                },
                _ => (),
            },
            _ => (),
        }
    }

    pub fn lat_long_altitude(&self, icao: ICAO) -> Option<(cpr::Position, u32)> {
        match self.0.get(&icao) {
            Some(altitudes) => {
                if let (Some(first_altitude), Some(second_altitude)) =
                    (altitudes.0[0], altitudes.0[1])
                {
                    cpr::get_position((&first_altitude, &second_altitude))
                        .map(|position| (position, first_altitude.alt))
                } else {
                    None
                }
            }
            None => None,
        }
    }
}
