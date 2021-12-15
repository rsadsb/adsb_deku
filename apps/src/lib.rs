use std::collections::HashMap;
use std::fmt;
use std::time::SystemTime;

use adsb_deku::adsb::{AirborneVelocity, Identification};
use adsb_deku::{cpr, Altitude, CPRFormat, ICAO};
use tracing::{info, debug};

#[derive(Debug)]
pub struct AirplaneState {
    // TODO: rename to coor
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
    /// lat/long
    pub position: Option<cpr::Position>,
}

impl AirplaneCoor {
    /// From Odd/Even Altitudes, update the position of aircraft
    ///
    /// TODO: verify position, such as speed_test
    fn update_position(&mut self) {
        if let [Some(odd), Some(even)] = &self.altitudes {
            self.position = cpr::get_position((odd, even));
            if let Some(position) = &self.position {
                debug!("update_position: odd: (lat: {}, long: {}), even: (lat: {}, long: {}), position: {:?}", odd.lat_cpr, odd.lon_cpr, even.lat_cpr, even.lat_cpr, position);
            }
        }
    }

    /// Return altitude from Odd Altitude
    fn altitude(&self) -> Option<u32> {
        if let Some(odd) = self.altitudes[0] {
            return Some(odd.alt);
        }
        None
    }
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

    /// Increment message count and update last time seen
    pub fn incr_messages(&mut self, icao: ICAO) {
        let mut state = self.0.entry(icao).or_insert_with(AirplaneState::default);
        state.num_messages += 1;
        state.last_time = SystemTime::now();
    }

    pub fn add_identification(&mut self, icao: ICAO, identification: &Identification) {
        let mut state = self.0.entry(icao).or_insert_with(AirplaneState::default);
        state.callsign = Some(identification.cn.clone());
        info!("[{}] with identification: {}", icao, identification.cn);
    }

    pub fn add_airborne_velocity(&mut self, icao: ICAO, vel: &AirborneVelocity) {
        let mut state = self.0.entry(icao).or_insert_with(AirplaneState::default);
        if let Some((_, ground_speed, vert_speed)) = vel.calculate() {
            info!(
                "[{}] with airborne velocity: speed: {}, vertical speed: {}",
                icao, ground_speed, vert_speed
            );
            state.speed = Some(ground_speed);
            state.vert_speed = Some(vert_speed);
        }
    }

    pub fn add_squawk(&mut self, icao: ICAO, squawk: u32) {
        let mut state = self.0.entry(icao).or_insert_with(AirplaneState::default);
        state.squawk = Some(squawk);
        info!("[{}] with squawk: {}", icao, squawk);
    }

    /// Add `Altitude` from adsb frame
    pub fn add_altitude(&mut self, icao: ICAO, altitude: &Altitude) {
        let state = self.0.entry(icao).or_insert_with(AirplaneState::default);
        info!(
            "[{}] with altitude: {}, cpr lat: {}, cpr long: {}",
            icao, altitude.alt, altitude.lat_cpr, altitude.lon_cpr
        );
        match altitude.odd_flag {
            CPRFormat::Odd => {
                state.coords = AirplaneCoor {
                    altitudes: [state.coords.altitudes[0], Some(*altitude)],
                    position: None,
                };
            },
            CPRFormat::Even => {
                state.coords = AirplaneCoor {
                    altitudes: [Some(*altitude), state.coords.altitudes[1]],
                    position: None,
                };
            },
        }
        // updat the position from the new even/odd message
        state.coords.update_position()
    }

    /// return latitude, longitude and altitude of specific ICAO for airplane
    pub fn lat_long_altitude(&self, icao: ICAO) -> Option<(cpr::Position, u32)> {
        match self.0.get(&icao) {
            Some(airplane_state) => {
                let coor = &airplane_state.coords;
                if let (Some(position), Some(altitude)) = (&coor.position, coor.altitude()) {
                    Some((position.clone(), altitude))
                } else {
                    None
                }
            },
            None => None,
        }
    }

    /// return all latitude/longitude from Hashmap of current "seen" aircrafts
    pub fn all_lat_long_altitude(&self) -> Vec<(cpr::Position, ICAO)> {
        let mut all_lat_long = vec![];
        for (key, airplane_state) in self.0.iter() {
            let coor = &airplane_state.coords;
            if let Some(position) = &coor.position {
                all_lat_long.push((position.clone(), *key));
            }
        }

        all_lat_long
    }

    /// Remove airplane after not active for a time
    pub fn prune(&mut self, filter_time: u64) {
        self.0.retain(|k, v| {
            if let Ok(time) = v.last_time.elapsed() {
                if time < std::time::Duration::from_secs(filter_time) {
                    true
                } else {
                    info!("[{}] non-active, removing", k);
                    false
                }
            } else {
                info!("[{}] non-active(time error), removing", k);
                false
            }
        });
    }
}
