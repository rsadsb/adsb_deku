use std::collections::HashMap;
use std::fmt;
use std::time::SystemTime;

use adsb_deku::adsb::{AirborneVelocity, Identification};
use adsb_deku::{cpr, Altitude, CPRFormat, ICAO};

#[derive(Debug)]
pub struct AirplaneState {
    pub coords: AirplaneCoor,
    pub squawk: Option<u32>,
    pub callsign: Option<String>,
    pub speed: Option<f64>,
    pub vert_speed: Option<i16>,
    pub on_ground: Option<bool>,
    pub num_messages: u64,
    pub last_time: SystemTime,
}

impl Default for AirplaneState {
    fn default() -> Self {
        Self {
            coords: AirplaneCoor::default(),
            squawk: None,
            callsign: None,
            speed: None,
            vert_speed: None,
            on_ground: None,
            num_messages: 0,
            last_time: SystemTime::now(),
        }
    }
}

#[derive(Debug, Default)]
pub struct AirplaneCoor {
    /// [odd, even]
    pub altitudes: [Option<Altitude>; 2],
}

#[derive(Debug, Default)]
pub struct Airplanes(pub HashMap<ICAO, AirplaneState>);

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

    pub fn incr_messages(&mut self, icao: ICAO) {
        let mut state = self.0.entry(icao).or_insert_with(AirplaneState::default);
        state.num_messages += 1;
        state.last_time = SystemTime::now();
    }

    pub fn add_identification(&mut self, icao: ICAO, identification: &Identification) {
        let mut state = self.0.entry(icao).or_insert_with(AirplaneState::default);
        state.callsign = Some(identification.cn.clone());
    }

    pub fn add_airborne_velocity(&mut self, icao: ICAO, vel: &AirborneVelocity) {
        let mut state = self.0.entry(icao).or_insert_with(AirplaneState::default);
        if let Some((_, ground_speed, vert_speed)) = vel.calculate() {
            state.speed = Some(ground_speed);
            state.vert_speed = Some(vert_speed);
        }
    }

    pub fn add_squawk(&mut self, icao: ICAO, squawk: u32) {
        let mut state = self.0.entry(icao).or_insert_with(AirplaneState::default);
        state.squawk = Some(squawk);
    }

    /// Add `Altitude` from adsb frame
    pub fn add_altitude(&mut self, icao: ICAO, altitude: &Altitude) {
        let state = self.0.entry(icao).or_insert_with(AirplaneState::default);
        match altitude.odd_flag {
            CPRFormat::Odd => {
                state.coords = AirplaneCoor {
                    altitudes: [state.coords.altitudes[0], Some(*altitude)],
                };
            },
            CPRFormat::Even => {
                state.coords = AirplaneCoor {
                    altitudes: [Some(*altitude), state.coords.altitudes[1]],
                };
            },
        }
    }

    /// Calculate latitude, longitude and altitude
    pub fn lat_long_altitude(&self, icao: ICAO) -> Option<(cpr::Position, u32)> {
        match self.0.get(&icao) {
            Some(state) => {
                if let (Some(first_altitude), Some(second_altitude)) =
                    (state.coords.altitudes[0], state.coords.altitudes[1])
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
        for state in self.0.values() {
            if let (Some(first_altitude), Some(second_altitude)) =
                (state.coords.altitudes[0], state.coords.altitudes[1])
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
    pub fn prune(&mut self, filter_time: u64) {
        self.0.retain(|_, v| {
            v.last_time.elapsed().unwrap() < std::time::Duration::from_secs(filter_time)
        });
    }
}
