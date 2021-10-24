/*!
Compact Position Reporting

This module turns an ADS-B CPR format into Latitude/Longitude: `Position`.

reference: ICAO 9871 (D.2.4.7)
!*/

use crate::{Altitude, CPRFormat};
use std::cmp;

const NZ: f64 = 15.0;
const D_LAT_EVEN: f64 = 360.0 / (4.0 * NZ);
const D_LAT_ODD: f64 = 360.0 / (4.0 * NZ - 1.0);
const CPR_MAX: f64 = 131_072.0;

/// Post-processing of CPR into Latitude/Longitude
#[derive(Debug, PartialEq, Clone)]
pub struct Position {
    pub latitude: f64,
    pub longitude: f64,
}

// The NL function uses the precomputed table from 1090-WP-9-14
// This code is translated from https://github.com/wiedehopf/readsb/blob/dev/cpr.c
pub(crate) fn cpr_nl(lat: f64) -> u64 {
    let mut lat = lat;
    if lat < 0.0 {
        // Table is symmetric about the equator
        lat = -lat;
    }
    if lat < 29.91135686 {
        if lat < 10.47047130 {
            return 59;
        }
        if lat < 14.82817437 {
            return 58;
        }
        if lat < 18.18626357 {
            return 57;
        }
        if lat < 21.02939493 {
            return 56;
        }
        if lat < 23.54504487 {
            return 55;
        }
        if lat < 25.82924707 {
            return 54;
        }
        if lat < 27.93898710 {
            return 53;
        }
        // < 29.91135686
        return 52;
    }
    if lat < 44.19454951 {
        if lat < 31.77209708 {
            return 51;
        }
        if lat < 33.53993436 {
            return 50;
        }
        if lat < 35.22899598 {
            return 49;
        }
        if lat < 36.85025108 {
            return 48;
        }
        if lat < 38.41241892 {
            return 47;
        }
        if lat < 39.92256684 {
            return 46;
        }
        if lat < 41.38651832 {
            return 45;
        }
        if lat < 42.80914012 {
            return 44;
        }
        // < 44.19454951
        return 43;
    }
    if lat < 59.95459277 {
        if lat < 45.54626723 {
            return 42;
        }
        if lat < 46.86733252 {
            return 41;
        }
        if lat < 48.16039128 {
            return 40;
        }
        if lat < 49.42776439 {
            return 39;
        }
        if lat < 50.67150166 {
            return 38;
        }
        if lat < 51.89342469 {
            return 37;
        }
        if lat < 53.09516153 {
            return 36;
        }
        if lat < 54.27817472 {
            return 35;
        }
        if lat < 55.44378444 {
            return 34;
        }
        if lat < 56.59318756 {
            return 33;
        }
        if lat < 57.72747354 {
            return 32;
        }
        if lat < 58.84763776 {
            return 31;
        }
        // < 59.95459277
        return 30;
    }
    if lat < 61.04917774 {
        return 29;
    }
    if lat < 62.13216659 {
        return 28;
    }
    if lat < 63.20427479 {
        return 27;
    }
    if lat < 64.26616523 {
        return 26;
    }
    if lat < 65.31845310 {
        return 25;
    }
    if lat < 66.36171008 {
        return 24;
    }
    if lat < 67.39646774 {
        return 23;
    }
    if lat < 68.42322022 {
        return 22;
    }
    if lat < 69.44242631 {
        return 21;
    }
    if lat < 70.45451075 {
        return 20;
    }
    if lat < 71.45986473 {
        return 19;
    }
    if lat < 72.45884545 {
        return 18;
    }
    if lat < 73.45177442 {
        return 17;
    }
    if lat < 74.43893416 {
        return 16;
    }
    if lat < 75.42056257 {
        return 15;
    }
    if lat < 76.39684391 {
        return 14;
    }
    if lat < 77.36789461 {
        return 13;
    }
    if lat < 78.33374083 {
        return 12;
    }
    if lat < 79.29428225 {
        return 11;
    }
    if lat < 80.24923213 {
        return 10;
    }
    if lat < 81.19801349 {
        return 9;
    }
    if lat < 82.13956981 {
        return 8;
    }
    if lat < 83.07199445 {
        return 7;
    }
    if lat < 83.99173563 {
        return 6;
    }
    if lat < 84.89166191 {
        return 5;
    }
    if lat < 85.75541621 {
        return 4;
    }
    if lat < 86.53536998 {
        return 3;
    }
    if lat < 87.00000000 {
        return 2;
    }
    1
}

/// Calculate Globally unambiguous position decoding
///
/// Using both an Odd and Even `Altitude`, calculate the latitude/longitude
///
/// reference: ICAO 9871 (D.2.4.7.7)
pub fn get_position(cpr_frames: (&Altitude, &Altitude)) -> Option<Position> {
    let latest_frame = cpr_frames.1;
    let (even_frame, odd_frame) = match cpr_frames {
        (
            even @ Altitude {
                odd_flag: CPRFormat::Even,
                ..
            },
            odd @ Altitude {
                odd_flag: CPRFormat::Odd,
                ..
            },
        )
        | (
            odd @ Altitude {
                odd_flag: CPRFormat::Odd,
                ..
            },
            even @ Altitude {
                odd_flag: CPRFormat::Even,
                ..
            },
        ) => (even, odd),
        _ => return None,
    };

    let cpr_lat_even = f64::from(even_frame.lat_cpr) / CPR_MAX;
    let cpr_lon_even = f64::from(even_frame.lon_cpr) / CPR_MAX;
    let cpr_lat_odd = f64::from(odd_frame.lat_cpr) / CPR_MAX;
    let cpr_lon_odd = f64::from(odd_frame.lon_cpr) / CPR_MAX;

    let j = (59.0 * cpr_lat_even - 60.0 * cpr_lat_odd + 0.5).floor();

    let mut lat_even = D_LAT_EVEN * (j % 60.0 + cpr_lat_even);
    let mut lat_odd = D_LAT_ODD * (j % 59.0 + cpr_lat_odd);

    if lat_even >= 270.0 {
        lat_even -= 360.0;
    }

    if lat_odd >= 270.0 {
        lat_odd -= 360.0;
    }

    let lat = if latest_frame == even_frame {
        lat_even
    } else {
        lat_odd
    };

    let (lat, lon) = get_lat_lon(lat, cpr_lon_even, cpr_lon_odd, &latest_frame.odd_flag);

    Some(Position {
        latitude: lat,
        longitude: lon,
    })
}

fn get_lat_lon(
    lat: f64,
    cpr_lon_even: f64,
    cpr_lon_odd: f64,
    cpr_format: &CPRFormat,
) -> (f64, f64) {
    let (p, c) = if cpr_format == &CPRFormat::Even {
        (0, cpr_lon_even)
    } else {
        (1, cpr_lon_odd)
    };
    let ni = cmp::max(cpr_nl(lat) - p, 1) as f64;
    let m =
        (cpr_lon_even * (cpr_nl(lat) - 1) as f64 - cpr_lon_odd * cpr_nl(lat) as f64 + 0.5).floor();
    let mut lon = (360.0 / ni) * (m % ni + c);
    if lon >= 180.0 {
        lon -= 360.0;
    }
    (lat, lon)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn cpr_nl_high_low_lat() {
        assert_eq!(cpr_nl(89.9), 1);
        assert_eq!(cpr_nl(-89.9), 1);
        assert_eq!(cpr_nl(86.9), 2);
        assert_eq!(cpr_nl(-86.9), 2);
    }

    #[test]
    fn cpr_calculate_position() {
        let odd = Altitude {
            odd_flag: CPRFormat::Odd,
            lat_cpr: 74158,
            lon_cpr: 50194,
            ..Altitude::default()
        };
        let even = Altitude {
            odd_flag: CPRFormat::Even,
            lat_cpr: 93000,
            lon_cpr: 51372,
            ..Altitude::default()
        };

        let position = get_position((&odd, &even)).unwrap();
        assert!((position.latitude - 52.257_202_148_437_5).abs() < f64::EPSILON);
        assert!((position.longitude - 3.919_372_558_593_75).abs() < f64::EPSILON);
    }

    #[test]
    fn cpr_calculate_position_high_lat() {
        let even = Altitude {
            odd_flag: CPRFormat::Even,
            lat_cpr: 108_011,
            lon_cpr: 110_088,
            ..Altitude::default()
        };
        let odd = Altitude {
            odd_flag: CPRFormat::Odd,
            lat_cpr: 75_050,
            lon_cpr: 36_777,
            ..Altitude::default()
        };
        let position = get_position((&even, &odd)).unwrap();
        assert!((position.latitude - 88.917_474_261_784_96).abs() < f64::EPSILON);
        assert!((position.longitude - 101.011_047_363_281_25).abs() < f64::EPSILON);
    }
}
