use std::fs::File;

use serde::Deserialize;

#[allow(dead_code)]
#[derive(Clone, Debug, Deserialize)]
pub struct Airport {
    pub icao: String,
    pub iata: String,
    pub name: String,
    pub city: String,
    pub subd: String,
    pub country: String,
    pub elevation: f64,
    pub lat: f64,
    pub lon: f64,
    pub tz: String,
}

impl Airport {
    pub fn from_file(filename: &str, time_zones: &Option<String>) -> Vec<Self> {
        let mut airports = vec![];
        let f = File::open(filename).unwrap();

        let mut rdr = csv::Reader::from_reader(f);
        for result in rdr.deserialize() {
            let record: Self = result.unwrap();

            if let Some(ref time_zones) = time_zones {
                for tz in time_zones.split(',') {
                    if record.tz.contains(tz) {
                        airports.push(record.clone());
                        continue;
                    }
                }
            } else {
                airports.push(record);
                continue;
            }
        }
        airports
    }
}
