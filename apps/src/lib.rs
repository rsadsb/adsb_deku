use std::collections::HashMap;
use std::fmt;
use std::time::SystemTime;

use adsb_deku::adsb::{AirborneVelocity, Identification, ME};
use adsb_deku::{cpr, Altitude, CPRFormat, Frame, DF, ICAO};
use tracing::{debug, info, warn};

// Max distance from the receiver to the aircraft
const MAX_RECEIVER_DISTANCE: f64 = 300.0;

// Max obsurd distance an aircraft travelled between messages
const MAX_AIRCRAFT_DISTANCE: f64 = 100.0;

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

#[derive(Debug, Default, Clone, Copy)]
pub struct AirplaneCoor {
    /// [odd, even]
    pub altitudes: [Option<Altitude>; 2],
    /// lat/long
    pub position: Option<cpr::Position>,
    /// last good time
    pub last_time: Option<SystemTime>,
    /// distance from receiver lat/long
    pub kilo_distance: Option<f64>,
}

impl AirplaneCoor {
    /// After checking the range of the new lat / long, new position from last position, update the
    /// position of an aircraft
    fn update_position(&mut self, lat_long: (f64, f64)) -> bool {
        if let [Some(odd), Some(even)] = self.altitudes {
            let test_position = cpr::get_position((&odd, &even));

            // Check kilometer range from receiver
            if let Some(test_position) = test_position {
                let kilo_distance = Self::haversine_distance(
                    lat_long,
                    (test_position.latitude, test_position.longitude),
                );
                if kilo_distance > MAX_RECEIVER_DISTANCE {
                    warn!("range: {kilo_distance} -  old: {lat_long:?} new: {test_position:?}");
                    return false;
                }
                self.kilo_distance = Some(kilo_distance);
                debug!("range: {kilo_distance}");
            }

            // if previous position, check against for range. This is a non-great way of doing
            // this, but maybe in the future we can check against the speed of the aircraft
            if let (Some(current_position), Some(test_position)) = (self.position, test_position) {
                let distance = Self::haversine_distance_position(current_position, test_position);
                if distance > MAX_AIRCRAFT_DISTANCE {
                    warn!("distance: {distance} old: {current_position:?}, invalid: {test_position:?}");
                    return false;
                }
                debug!("distance: {distance}");
            }

            // Good new position!
            self.position = test_position;
            debug!("update_position: odd: (lat: {}, long: {}), even: (lat: {}, long: {}), position: {:?}",
                odd.lat_cpr,
                odd.lon_cpr,
                even.lat_cpr,
                even.lat_cpr,
                self.position);
            self.last_time = Some(SystemTime::now());
        }
        true
    }

    /// Return altitude from Odd Altitude
    fn altitude(&self) -> Option<u32> {
        if let Some(odd) = self.altitudes[0] {
            return Some(odd.alt);
        }
        None
    }

    /// Calculate the kilometers between two lat/long points
    fn haversine_distance_position(position: cpr::Position, other: cpr::Position) -> f64 {
        let lat1 = position.latitude;
        let lat2 = other.latitude;
        let long1 = position.longitude;
        let long2 = other.longitude;
        Self::haversine_distance((lat1, long1), (lat2, long2))
    }

    // https://en.wikipedia.org/wiki/Haversine_formula
    fn haversine_distance(s: (f64, f64), other: (f64, f64)) -> f64 {
        // kilometers
        let r = 6371.00;
        let lat1_rad = s.0.to_radians();
        let lat2_rad = other.0.to_radians();
        let long1_rad = s.1.to_radians();
        let long2_rad = other.1.to_radians();

        let a = ((lat2_rad - lat1_rad) / 2.00).sin().mul_add(
            ((lat2_rad - lat1_rad) / 2.00).sin(),
            lat1_rad.cos() * lat2_rad.cos() * ((long2_rad - long1_rad) / 2.00).sin().powi(2),
        );
        let c = 2.00 * ((a).sqrt().atan2((1.00 - a).sqrt()));
        r * c
    }
}

#[derive(Debug, Default)]
pub struct Airplanes(pub HashMap<ICAO, AirplaneState>);

impl fmt::Display for Airplanes {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        for key in self.0.keys() {
            let value = self.aircraft_details(*key);
            if let Some(value) = value {
                writeln!(f, "{key}: {value:?}").unwrap();
            }
        }
        Ok(())
    }
}

impl Airplanes {
    #[must_use]
    pub fn new() -> Self {
        Self(HashMap::new())
    }

    /// Increment message count and update last time seen
    pub fn incr_messages(&mut self, icao: ICAO) {
        let mut state = self.0.entry(icao).or_insert_with(AirplaneState::default);
        state.num_messages += 1;
        state.last_time = SystemTime::now();
    }

    pub fn action(&mut self, frame: Frame, lat_long: (f64, f64)) {
        if let DF::ADSB(ref adsb) = frame.df {
            match &adsb.me {
                ME::AircraftIdentification(identification) => {
                    self.add_identification(adsb.icao, identification);
                },
                ME::AirborneVelocity(vel) => {
                    self.add_airborne_velocity(adsb.icao, vel);
                },
                ME::AirbornePositionGNSSAltitude(altitude)
                | ME::AirbornePositionBaroAltitude(altitude) => {
                    self.add_altitude(adsb.icao, altitude, lat_long);
                },
                _ => {},
            };
            self.incr_messages(adsb.icao);
        }
    }

    fn add_identification(&mut self, icao: ICAO, identification: &Identification) {
        let mut state = self.0.entry(icao).or_insert_with(AirplaneState::default);
        state.callsign = Some(identification.cn.clone());
        info!("[{icao}] with identification: {}", identification.cn);
    }

    fn add_airborne_velocity(&mut self, icao: ICAO, vel: &AirborneVelocity) {
        let mut state = self.0.entry(icao).or_insert_with(AirplaneState::default);
        if let Some((_, ground_speed, vert_speed)) = vel.calculate() {
            info!(
                "[{icao}] with airborne velocity: speed: {}, vertical speed: {}",
                ground_speed, vert_speed
            );
            state.speed = Some(ground_speed);
            state.vert_speed = Some(vert_speed);
        }
    }

    /// Add `Altitude` from adsb frame
    fn add_altitude(&mut self, icao: ICAO, altitude: &Altitude, lat_long: (f64, f64)) {
        let state = self.0.entry(icao).or_insert_with(AirplaneState::default);
        info!(
            "[{icao}] with altitude: {}, cpr lat: {}, cpr long: {}",
            altitude.alt, altitude.lat_cpr, altitude.lon_cpr
        );
        let mut temp_coords = match altitude.odd_flag {
            CPRFormat::Odd => AirplaneCoor {
                altitudes: [state.coords.altitudes[0], Some(*altitude)],
                ..state.coords
            },
            CPRFormat::Even => AirplaneCoor {
                altitudes: [Some(*altitude), state.coords.altitudes[1]],
                ..state.coords
            },
        };
        // update the position from the new even/odd message if it's a good new position
        if temp_coords.update_position(lat_long) {
            state.coords = temp_coords;
        } else {
            // clear record
            state.coords = AirplaneCoor::default();
        }
    }

    // return display detail of aircraft
    #[must_use]
    pub fn aircraft_details(&self, icao: ICAO) -> Option<(cpr::Position, u32, f64)> {
        match self.0.get(&icao) {
            Some(airplane_state) => {
                let coor = &airplane_state.coords;
                if let (Some(position), Some(altitude), Some(kilo_distance)) =
                    (&coor.position, coor.altitude(), coor.kilo_distance)
                {
                    Some((*position, altitude, kilo_distance))
                } else {
                    None
                }
            },
            None => None,
        }
    }

    /// return all latitude/longitude from Hashmap of current "seen" aircrafts
    #[must_use]
    pub fn all_lat_long_altitude(&self) -> Vec<(cpr::Position, ICAO)> {
        let mut all_lat_long = vec![];
        for (key, airplane_state) in &self.0 {
            let coor = &airplane_state.coords;
            if let Some(position) = &coor.position {
                all_lat_long.push((*position, *key));
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
                    info!("[{k}] non-active, removing");
                    false
                }
            } else {
                info!("[{k}] non-active(time error), removing");
                false
            }
        });
    }
}
