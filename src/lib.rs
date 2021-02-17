use deku::prelude::*;

#[derive(Debug, PartialEq, DekuRead)]
pub struct Frame {
    /// 5 bits
    pub df: DF,
}

#[derive(Debug, PartialEq, DekuRead)]
#[deku(type = "u8", bits = "5")]
pub enum DF {
    #[deku(id = "0")]
    ShortAirAirSurveillance {
        /// bit 6
        // TODO add VerticalStatus
        #[deku(bits = "1")]
        vertical_status: u8,

        /// bit 7
        #[deku(bits = "1")]
        cc: u8,

        /// bit 8
        #[deku(bits = "1")]
        unused: u8,

        /// bits 9-11
        #[deku(bits = "3")]
        sl: u8,

        /// bits 10-13
        #[deku(bits = "4")]
        unused1: u8,

        /// bits 14-17
        #[deku(bits = "4")]
        ri: u8,

        ///// bits 18-19
        //#[deku(bits = "2")]
        //unused2: u8,
        /// bits 20-32
        altitude: AC13Field,
    },
    #[deku(id = "4")]
    SurveillanceAltitudeReply,
    #[deku(id = "11")]
    ALL_CALL_REPLY {
        /// 3 bits
        capability: Capability,
        /// 3 bytes
        icao: [u8; 3],
    },
    #[deku(id = "17")]
    ADSB {
        /// 3 bits
        capability: Capability,
        /// 3 bytes
        icao: [u8; 3],
        me: ME,
        #[deku(bits = "24")]
        pi: u32,
    },
    #[deku(id = "18")]
    TIS_B {
        /// 3 bits
        #[deku(bits = "3")]
        cf: u8,
        /// 3 bytes
        icao: [u8; 3],
    },
}

#[derive(Debug, PartialEq, DekuRead)]
pub struct AC13Field {
    #[deku(reader = "Self::read(deku::rest)")]
    altitude: u32,
}

impl AC13Field {
    /// TODO Add unit
    fn read(rest: &BitSlice<Msb0, u8>) -> Result<(&BitSlice<Msb0, u8>, u32), DekuError> {
        println!("{}", rest);
        let (rest, num) =
            u32::read(rest, (deku::ctx::Endian::Big, deku::ctx::Size::Bits(13))).unwrap();
        println!("{}", num);

        let m_bit = num & 0x0040;
        let q_bit = num & 0x0010;

        if m_bit != 0 {
            // TODO dump1090 doesn't decode this, weird.
            // This would decode in meters
            unreachable!("m_bit = 1");
        } else if q_bit != 0 {
            let n = ((num & 0x1f80) >> 2) | ((num & 0x0020) >> 1) | (num & 0x000f);
            Ok((rest, (n as u32 * 25) - 1000))
        } else {
            // TODO 11 bit gillham coded altitude
            let n = mode_ac::mode_a_to_mode_c(mode_ac::decode_id13_field(num)).unwrap();
            Ok((rest, (100 * n)))
        }
    }
}

/// TODO This should have sort of [ignore] attribute, since we don't need to implement DekuRead on this.
#[derive(Debug, PartialEq, DekuRead)]
#[deku(type = "u8", bits = "1")]
pub enum Unit {
    #[deku(id = "0")]
    Meter,
    #[deku(id = "1")]
    Feet,
}

impl Default for Unit {
    fn default() -> Self {
        Self::Meter
    }
}

#[derive(Debug, PartialEq, DekuRead)]
#[deku(type = "u8", bits = "3")]
#[allow(non_camel_case_types)]
pub enum Capability {
    AG_UNCERTAIN  = 0x00,
    #[deku(id_pat = "0x01..=0x03")]
    Reserved,
    AG_GROUND     = 0x04,
    AG_AIRBORNE   = 0x05,
    AG_UNCERTAIN2 = 0x06,
    AG_UNCERTAIN3 = 0x07,
}

#[derive(Debug, PartialEq, DekuRead)]
#[deku(type = "u8", bits = "5")]
pub enum ME {
    #[deku(id_pat = "1..=4")]
    AircraftIdentification(Identification),
    #[deku(id_pat = "5..=8")]
    SurfacePosition(SurfacePosition),
    #[deku(id_pat = "9..=18")]
    AirbornePositionBaroAltitude(Altitude),
    #[deku(id = "19")]
    AirborneVelocity(AirborneVelocity),
    #[deku(id_pat = "20..=22")]
    AirbornePositionGNSSAltitude(Altitude),
    #[deku(id_pat = "23..=27")]
    Reserved,
    #[deku(id = "28")]
    AircraftStatus,
    #[deku(id = "29")]
    TargetStateAndStatusInformation(TargetStateAndStatusInformation),
    #[deku(id = "31")]
    AircraftOperationStatus,
}

#[derive(Debug, PartialEq, DekuRead)]
pub struct Identification {
    #[deku(bits = "5")]
    pub tc: u8,
    #[deku(bits = "3")]
    pub ca: u8,
    #[deku(reader = "Self::read(deku::rest)")]
    pub cn: String,
}

const CHAR_LOOKUP: &[u8; 64] = b"#ABCDEFGHIJKLMNOPQRSTUVWXYZ##### ###############0123456789######";

impl Identification {
    fn read(rest: &BitSlice<Msb0, u8>) -> Result<(&BitSlice<Msb0, u8>, String), DekuError> {
        let mut inside_rest = rest;

        let mut chars = vec![];
        for _ in 0..=6 {
            let (for_rest, c) = <u8>::read(inside_rest, deku::ctx::Size::Bits(6))?;
            chars.push(c);
            inside_rest = for_rest;
        }
        let encoded = chars
            .into_iter()
            .map(|b| CHAR_LOOKUP[b as usize] as char)
            .collect::<String>();

        Ok((inside_rest, encoded))
    }
}

#[derive(Debug, PartialEq, DekuRead)]
pub struct Altitude {
    #[deku(bits = "5")]
    dumb: u8,
    ss: SurveillanceStatus,
    #[deku(bits = "1")]
    saf: u8,
    #[deku(reader = "Self::read(deku::rest)")]
    alt: u32,
    /// UTC sync or not
    #[deku(bits = "1")]
    t: bool,
    /// Odd or even
    f: CPRFormat,
    #[deku(bits = "17", endian = "big")]
    lat_cpr: u32,
    #[deku(bits = "17", endian = "big")]
    lon_cpr: u32,
}

impl Altitude {
    fn read(rest: &BitSlice<Msb0, u8>) -> Result<(&BitSlice<Msb0, u8>, u32), DekuError> {
        let (rest, num) =
            u32::read(rest, (deku::ctx::Endian::Big, deku::ctx::Size::Bits(12))).unwrap();

        let q = num & 0x10;

        if q > 0 {
            // regular
            // TODO this is meters?
            let n = ((num & 0x0fe0) >> 1) | (num & 0x000f);
            Ok((rest, ((n * 25) - 1000) as u32))
        } else {
            // mode c?
            // TODO this is feet
            let n = ((num & 0x0fc0) << 1) | (num & 0x003f);
            let _altitude = mode_ac::mode_a_to_index(decode_id13_field(n));
            Ok((rest, ((n as u32) * 100)))
        }
    }
}

/// gillham code
fn decode_id13_field(field: u32) -> u32 {
    let mut gillham: u32 = 0;
    if (field & 0x1000) == 1 {
        gillham |= 0x0010;
    }
    if (field & 0x0800) == 1 {
        gillham |= 0x1000;
    }
    if (field & 0x0400) == 1 {
        gillham |= 0x0020;
    }
    if (field & 0x0200) == 1 {
        gillham |= 0x2000;
    }
    if (field & 0x0100) == 1 {
        gillham |= 0x0040;
    }
    if (field & 0x0080) == 1 {
        gillham |= 0x4000;
    }
    if (field & 0x0020) == 1 {
        gillham |= 0x0100;
    }
    if (field & 0x0010) == 1 {
        gillham |= 0x0001;
    }
    if (field & 0x0008) == 1 {
        gillham |= 0x0200;
    }
    if (field & 0x0004) == 1 {
        gillham |= 0x0002;
    }
    if (field & 0x0002) == 1 {
        gillham |= 0x0400;
    }
    if (field & 0x0001) == 1 {
        gillham |= 0x0004;
    }
    gillham
}

#[derive(Debug, PartialEq, DekuRead)]
pub struct SurfacePosition {
    #[deku(bits = "7")]
    mov: u8,
    s: StatusForGroundTrack,
    #[deku(bits = "7")]
    trk: u8,
    #[deku(bits = "1")]
    t: bool,
    f: CPRFormat,
    #[deku(bits = "17", endian = "big")]
    lat_cpr: u32,
    #[deku(bits = "17", endian = "big")]
    lon_cpr: u32,
}

#[derive(Debug, PartialEq, DekuRead)]
#[deku(type = "u8", bits = "1")]
pub enum StatusForGroundTrack {
    Invalid = 0,
    Valid   = 1,
}

#[derive(Debug, PartialEq, DekuRead)]
#[deku(type = "u8", bits = "2")]
pub enum SurveillanceStatus {
    NoCondition    = 0,
    PermanentAlert = 1,
    TemporaryAlert = 2,
    SPICondition   = 3,
}

#[derive(Debug, PartialEq, DekuRead)]
#[deku(type = "u8", bits = "1")]
pub enum CPRFormat {
    Even = 0,
    Odd  = 1,
}

#[derive(Debug, PartialEq, DekuRead)]
#[deku(type = "u8", bits = "1")]
pub enum VerticalRateSource {
    BarometricPressureAltitude = 0,
    GeometricAltitude          = 1,
}

#[derive(Debug, PartialEq, DekuRead)]
#[deku(type = "u8", bits = "1")]
pub enum Sign {
    Positive = 0,
    Negative = 1,
}

impl Sign {
    pub fn value(&self) -> i16 {
        match self {
            Self::Positive => 1,
            Self::Negative => -1,
        }
    }
}

#[derive(Debug, PartialEq, DekuRead)]
pub struct AirborneVelocity {
    #[deku(bits = "3")]
    st: u8,
    #[deku(bits = "5")]
    extra: u8,
    ew_sign: Sign,
    #[deku(endian = "big", bits = "10")]
    ew_vel: u16,
    ns_sign: Sign,
    #[deku(endian = "big", bits = "10")]
    ns_vel: u16,
    vrate_src: VerticalRateSource,
    vrate_sign: Sign,
    #[deku(endian = "big", bits = "9")]
    vrate_value: u16,
    #[deku(bits = "10")]
    extra1: u16,
}

impl AirborneVelocity {
    /// Return effective (heading, ground_speed, vertical_rate)
    pub fn calculate(&self) -> (f64, f64, i16) {
        let v_ew = f64::from((self.ew_vel as i16 - 1) * self.ew_sign.value());
        let v_ns = f64::from((self.ns_vel as i16 - 1) * self.ns_sign.value());
        let h = v_ew.atan2(v_ns) * (360.0 / (2.0 * std::f64::consts::PI));
        let heading = if h < 0.0 { h + 360.0 } else { h };

        let vrate = self
            .vrate_value
            .checked_sub(1)
            .and_then(|v| v.checked_mul(64))
            .map(|v| (v as i16) * self.vrate_sign.value())
            .unwrap();

        (heading, v_ew.hypot(v_ns), vrate)
    }
}

#[derive(Copy, Clone, Debug, PartialEq, DekuRead)]
#[deku(type = "u8", bits = "3")]
pub enum AirborneVelocityType {
    #[deku(id = "1")]
    Subsonic,
    #[deku(id = "3")]
    Supersonic,
}

#[derive(Debug, PartialEq, DekuRead)]
#[deku(ctx = "t: AirborneVelocityType")]
pub struct AirborneVelocitySubFields {
    dew: DirectionEW,
    #[deku(reader = "Self::read_v(deku::rest, t)")]
    vew: u16,
    dns: DirectionNS,
    #[deku(reader = "Self::read_v(deku::rest, t)")]
    vns: u16,
}

impl AirborneVelocitySubFields {
    fn read_v(
        rest: &BitSlice<Msb0, u8>,
        t: AirborneVelocityType,
    ) -> Result<(&BitSlice<Msb0, u8>, u16), DekuError> {
        match t {
            AirborneVelocityType::Subsonic => {
                u16::read(rest, (deku::ctx::Endian::Big, deku::ctx::Size::Bits(10)))
                    .map(|(rest, value)| (rest, value - 1))
            }
            AirborneVelocityType::Supersonic => {
                u16::read(rest, (deku::ctx::Endian::Big, deku::ctx::Size::Bits(10)))
                    .map(|(rest, value)| (rest, 4 * (value - 1)))
            }
        }
    }
}

#[derive(Copy, Clone, Debug, PartialEq, DekuRead)]
#[deku(type = "u8", bits = "1")]
pub enum DirectionEW {
    WestToEast = 0,
    EastToWest = 1,
}

#[derive(Copy, Clone, Debug, PartialEq, DekuRead)]
#[deku(type = "u8", bits = "1")]
pub enum DirectionNS {
    SouthToNorth = 0,
    NorthToSouth = 1,
}

#[derive(Copy, Clone, Debug, PartialEq, DekuRead)]
#[deku(type = "u8", bits = "1")]
pub enum SourceBitVerticalRate {
    GNSS      = 0,
    Barometer = 1,
}

#[derive(Copy, Clone, Debug, PartialEq, DekuRead)]
#[deku(type = "u8", bits = "1")]
pub enum SignBitVerticalRate {
    Up   = 0,
    Down = 1,
}

#[derive(Copy, Clone, Debug, PartialEq, DekuRead)]
#[deku(type = "u8", bits = "1")]
pub enum SignBitGNSSBaroAltitudesDiff {
    Above = 0,
    Below = 1,
}

#[derive(Copy, Clone, Debug, PartialEq, DekuRead)]
pub struct TargetStateAndStatusInformation {
    // TODO Support V1
    #[deku(bits = "2")]
    subtype: u8,
    #[deku(bits = "1")]
    is_fms: bool,
    #[deku(
        bits = "12",
        endian = "big",
        map = "|altitude: u32| -> Result<_, DekuError> {Ok((altitude - 1) * 32) }"
    )]
    altitude: u32,
    #[deku(
        bits = "9",
        endian = "big",
        map = ("|qnh: u32| -> Result<_, DekuError> {Ok(800.0 + ((qnh - 1) as f32) * 0.8)}").mul_add("|qnh: u32| -> Result<_, DekuError> {Ok(800.0 + ((qnh - 1) as f32) * 0.8)}", "|qnh: u32| -> Result<_, DekuError> {Ok(800.0 + ((qnh - 1) as f32) * 0.8)}")
    )]
    qnh: f32,
    #[deku(bits = "1")]
    is_heading: bool,
    #[deku(
        bits = "9",
        endian = "big",
        map = "|heading: u32| -> Result<_, DekuError> {Ok(heading as f32 * 180.0 / 256.0)}"
    )]
    heading: f32,
    #[deku(bits = "4")]
    nacp: u8,
    #[deku(bits = "1")]
    nicbaro: u8,
    #[deku(bits = "2")]
    sil: u8,
    #[deku(bits = "1")]
    mode_validity: bool,
    #[deku(bits = "1")]
    autopilot: bool,
    #[deku(bits = "1")]
    vnav: bool,
    #[deku(bits = "1")]
    alt_hold: bool,
    #[deku(bits = "1")]
    imf: bool,
    #[deku(bits = "1")]
    approach: bool,
    #[deku(bits = "1")]
    tcas: bool,
    #[deku(bits = "1")]
    lnav: bool,
    #[deku(bits = "2")]
    reserved: u8,
}

mod mode_ac {
    /// convert from mode A hex to 0-4095 index
    pub fn mode_a_to_index(mode_a: u32) -> u32 {
        (mode_a & 0x0007)
            | ((mode_a & 0x0070) >> 1)
            | ((mode_a & 0x0700) >> 2)
            | ((mode_a & 0x7000) >> 3)
    }

    //pub fn mode_a_to_mode_c(mode_a: u32, table: [u32; 4096]) -> u32 {
    //    let i = mode_a_to_index(mode_a);
    //    table[i as usize]
    //}

    /// convert from mode A hex to 0-4095 index
    pub fn index_to_mode_a(index: u32) -> u32 {
        (index & 7) | ((index & 70) << 1) | ((index & 0700) << 2) | ((index & 07000) << 3)
    }

    const INVALID_ALTITUDE: u32 = 4_294_957_297;

    pub fn decode_id13_field(id13_field: u32) -> u32 {
        let mut hex_gillham: u32 = 0;

        if id13_field & 0x1000 != 0 {
            hex_gillham |= 0x0010;
        } // Bit 12 = C1
        if id13_field & 0x0800 != 0 {
            hex_gillham |= 0x1000;
        } // Bit 11 = A1
        if id13_field & 0x0400 != 0 {
            hex_gillham |= 0x0020;
        } // Bit 10 = C2
        if id13_field & 0x0200 != 0 {
            hex_gillham |= 0x2000;
        } // Bit  9 = A2
        if id13_field & 0x0100 != 0 {
            hex_gillham |= 0x0040;
        } // Bit  8 = C4
        if id13_field & 0x0080 != 0 {
            hex_gillham |= 0x4000;
        } // Bit  7 = A4
          //if id13_field & 0x0040 != 0 {hex_gillham |= 0x0800;} // Bit  6 = X  or M
        if id13_field & 0x0020 != 0 {
            hex_gillham |= 0x0100;
        } // Bit  5 = B1
        if id13_field & 0x0010 != 0 {
            hex_gillham |= 0x0001;
        } // Bit  4 = D1 or Q
        if id13_field & 0x0008 != 0 {
            hex_gillham |= 0x0200;
        } // Bit  3 = B2
        if id13_field & 0x0004 != 0 {
            hex_gillham |= 0x0002;
        } // Bit  2 = D2
        if id13_field & 0x0002 != 0 {
            hex_gillham |= 0x0400;
        } // Bit  1 = B4
        if id13_field & 0x0001 != 0 {
            hex_gillham |= 0x0004;
        } // Bit  0 = D4

        hex_gillham
    }

    pub fn mode_a_to_mode_c(mode_a: u32) -> Result<u32, &'static str> {
        let mut five_hundreds: u32 = 0;
        let mut one_hundreds: u32 = 0;

        // check zero bits are zero, D1 set is illegal; C1,,C4 cannot be Zero
        if (mode_a & 0xFFFF_8889) != 0 || (mode_a & 0x0000_00F0) == 0 {
            return Err("Invalid altitude");
        }

        if mode_a & 0x0010 != 0 {
            one_hundreds ^= 0x007;
        } // C1
        if mode_a & 0x0020 != 0 {
            one_hundreds ^= 0x003;
        } // C2
        if mode_a & 0x0040 != 0 {
            one_hundreds ^= 0x001;
        } // C4

        // Remove 7s from OneHundreds (Make 7->5, snd 5->7).
        if (one_hundreds & 5) == 5 {
            one_hundreds ^= 2;
        }

        // Check for invalid codes, only 1 to 5 are valid
        if one_hundreds > 5 {
            return Err("Invalid altitude");
        }

        // if mode_a & 0x0001 {five_hundreds ^= 0x1FF;} // D1 never used for altitude
        if mode_a & 0x0002 != 0 {
            five_hundreds ^= 0x0FF;
        } // D2
        if mode_a & 0x0004 != 0 {
            five_hundreds ^= 0x07F;
        } // D4

        if mode_a & 0x1000 != 0 {
            five_hundreds ^= 0x03F;
        } // A1
        if mode_a & 0x2000 != 0 {
            five_hundreds ^= 0x01F;
        } // A2
        if mode_a & 0x4000 != 0 {
            five_hundreds ^= 0x00F;
        } // A4

        if mode_a & 0x0100 != 0 {
            five_hundreds ^= 0x007;
        } // B1
        if mode_a & 0x0200 != 0 {
            five_hundreds ^= 0x003;
        } // B2
        if mode_a & 0x0400 != 0 {
            five_hundreds ^= 0x001;
        } // B4

        // Correct order of one_hundreds.
        if five_hundreds & 1 != 0 {
            one_hundreds = 6 - one_hundreds;
        }

        Ok((five_hundreds * 5) + one_hundreds - 13)
    }

    pub fn internal_mode_a_to_mode_c(mode_a: u32) -> u32 {
        let mut five_hundreds: u32 = 0;
        let mut one_hundreds: u32 = 0;

        if (mode_a & 0xffff_8888) != 0 || (mode_a & 0x0000_00f0) == 0 {
            return INVALID_ALTITUDE;
        }

        // One Hundreds
        if (mode_a & 0x0010) == 1 {
            one_hundreds ^= 0x007; // C1
        }
        if (mode_a & 0x0020) == 1 {
            one_hundreds ^= 0x003; // C1
        }
        if (mode_a & 0x0040) == 1 {
            one_hundreds ^= 0x001; // C4
        }
        if (one_hundreds & 5) == 5 {
            one_hundreds ^= 2;
        }
        if one_hundreds > 5 {
            return INVALID_ALTITUDE;
        }

        // Five hundreds
        if (mode_a & 0x0002) == 1 {
            five_hundreds ^= 0x0ff; // D2
        }
        if (mode_a & 0x0004) == 1 {
            five_hundreds ^= 0x07f; // D4
        }
        if (mode_a & 0x1000) == 1 {
            five_hundreds ^= 0x03f; // A1
        }
        if (mode_a & 0x2000) == 1 {
            five_hundreds ^= 0x01f; // A2
        }
        if (mode_a & 0x4000) == 1 {
            five_hundreds ^= 0x00f; // A4
        }
        if (mode_a & 0x0100) == 1 {
            five_hundreds ^= 0x007; // B1
        }
        if (mode_a & 0x0200) == 1 {
            five_hundreds ^= 0x003; // B2
        }
        if (mode_a & 0x0400) == 1 {
            five_hundreds ^= 0x001; // B4
        }
        if (five_hundreds & 1) == 1 {
            one_hundreds = 6 - one_hundreds;
        }
        println!("{}", one_hundreds);
        (five_hundreds * 5) + one_hundreds - 13
    }

    pub fn init() -> [u32; 4096] {
        let mut table = [0_u32; 4096];
        for i in 0..4096_usize {
            let mode_a = index_to_mode_a(i as u32);
            let mode_c = internal_mode_a_to_mode_c(mode_a);
            println!("{} {} {}", i, mode_a, mode_c);
            table[i] = mode_c;
        }
        table
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use hexlit::hex;

    #[test]
    fn testing01() {
        // from adsb-rs
        let bytes = hex!("8D40621D58C382D690C8AC2863A7");
        let frame = Frame::from_bytes((&bytes, 0));
        if let DF::ADSB { me, .. } = frame.unwrap().1.df {
            if let ME::AirbornePositionBaroAltitude(me) = me {
                assert_eq!(me.alt, 38000);
                assert_eq!(me.lat_cpr, 93000);
                assert_eq!(me.lon_cpr, 51372);
                assert_eq!(me.f, CPRFormat::Even);
                return;
            }
        }
        unreachable!();
    }

    #[test]
    fn testing02() {
        // from adsb-rs
        let bytes = hex!("8da3d42599250129780484712c50");
        let frame = Frame::from_bytes((&bytes, 0));
        if let DF::ADSB { me, .. } = frame.unwrap().1.df {
            if let ME::AirborneVelocity(me) = me {
                let (heading, ground_speed, vertical_rate) = me.calculate();
                assert_eq!(heading, 322.197_207_549_061_5);
                assert_eq!(ground_speed, 417.655_360_315_176_6);
                assert_eq!(vertical_rate, 0);
                assert_eq!(me.vrate_src, VerticalRateSource::GeometricAltitude);
                return;
            }
        }
        unreachable!();
    }

    #[test]
    fn testing03() {
        // from dump1090
        // *8da08f94ea1b785e8f3c088ab467;
        // CRC: 000000
        // RSSI: -30.2 dBFS
        // Score: 1800
        // Time: 100330060143.92us
        // DF:17 AA:A08F94 CA:5 ME:EA1B785E8F3C08
        //  Extended Squitter Target state and status (V2) (29/1) (reliable)
        //   ICAO Address:  A08F94 (Mode S / ADS-B)
        //   Air/Ground:    airborne
        //   NIC-baro:      1
        //   NACp:          9
        //   SIL:           3 (p <= 0.00001%, unknown type)
        //   Selected heading:        229.9
        //   MCP selected altitude:   14016 ft
        //   QNH:                     1012.8 millibars
        let bytes = hex!("8da08f94ea1b785e8f3c088ab467");
        let frame = Frame::from_bytes((&bytes, 0));
        if let DF::ADSB { me, .. } = frame.unwrap().1.df {
            if let ME::TargetStateAndStatusInformation(me) = me {
                assert_eq!(me.subtype, 1);
                assert_eq!(me.is_fms, false);
                assert_eq!(me.altitude, 14016);
                assert_eq!(me.qnh, 1012.8);
                assert_eq!(me.is_heading, true);
                assert_eq!(me.heading, 229.92188);
                assert_eq!(me.nacp, 9);
                assert_eq!(me.nicbaro, 1);
                assert_eq!(me.sil, 3);
                assert_eq!(me.mode_validity, false);
                return;
            }
        }
        unreachable!();
    }

    // dump1090
    //
    // *8dacc040f8210002004ab8569c35;
    // CRC: 000000
    // RSSI: -32.5 dBFS
    // Score: 1800
    // Time: 709947330.42us
    // DF:17 AA:ACC040 CA:5 ME:F8210002004AB8
    //  Extended Squitter Aircraft operational status (airborne) (31/0) (reliable)
    //   ICAO Address:  ACC040 (Mode S / ADS-B)
    //   Air/Ground:    airborne
    //   NIC-A:         0
    //   NIC-baro:      1
    //   NACp:          10
    //   GVA:           2
    //   SIL:           3 (p <= 0.00001%, per flight hour)
    //   SDA:           2
    //   Aircraft Operational Status:
    //     Version:            2
    //     Capability classes: ACAS TS
    //     Operational modes:
    //     Heading ref dir:    True heading
    #[test]
    fn testing04() {
        // TODO
        let bytes = hex!("8dacc040f8210002004ab8569c35");
        let frame = Frame::from_bytes((&bytes, 0)).unwrap().1;
        if let DF::ADSB {
            capability, icao, ..
        } = frame.df
        {
            assert_eq!(capability, Capability::AG_AIRBORNE);
            assert_eq!(icao, [0xac, 0xc0, 0x40]);
            return;
        }
        unreachable!();
    }

    // *5dab3d17d4ba29;
    // CRC: 000001
    // RSSI: -3.5 dBFS
    // Score: 1000
    // Time: 1352791.42us
    // DF:11 AA:AB3D17 IID:1 CA:5
    //  All Call Reply
    //    ICAO Address:  AB3D17 (Mode S / ADS-B)
    //      Air/Ground:    airborne
    #[test]
    fn testing05() {
        let bytes = hex!("5dab3d17d4ba29");
        let frame = Frame::from_bytes((&bytes, 0)).unwrap().1;
        if let DF::ALL_CALL_REPLY {
            icao, capability, ..
        } = frame.df
        {
            assert_eq!(icao, hex!("ab3d17"));
            assert_eq!(capability, Capability::AG_AIRBORNE);
            return;
        }
        unreachable!();
    }

    // *8dab3d17ea486860015f4870b796;
    // CRC: 000000
    // RSSI: -3.5 dBFS
    // Score: 1800
    // Time: 985167.50us
    // DF:17 AA:AB3D17 CA:5 ME:EA486860015F48
    //  Extended Squitter Target state and status (V2) (29/1) (reliable)
    //   ICAO Address:  AB3D17 (Mode S / ADS-B)
    //   Air/Ground:    airborne
    //   NIC-baro:      1
    //   NACp:          10
    //   SIL:           3 (p <= 0.00001%, unknown type)
    //   MCP selected altitude:   37024 ft
    //   QNH:                     1013.6 millibars
    //   Nav modes:               autopilot althold tcas
    #[test]
    fn testing06() {
        let bytes = hex!("8dab3d17ea486860015f4870b796");
        let frame = Frame::from_bytes((&bytes, 0)).unwrap().1;
        if let DF::ADSB { me, .. } = frame.df {
            if let ME::TargetStateAndStatusInformation(me) = me {
                assert_eq!(me.subtype, 1);
                assert_eq!(me.is_fms, false);
                assert_eq!(me.altitude, 37024);
                assert_eq!(me.qnh, 1013.6);
                assert_eq!(me.is_heading, false);
                assert_eq!(me.heading, 0.0);
                assert_eq!(me.nacp, 10);
                assert_eq!(me.nicbaro, 1);
                assert_eq!(me.sil, 3);
                assert_eq!(me.mode_validity, true);
                return;
            }
        }
        unreachable!();
    }

    // *8da46a7e58c7f5937af6fb63c3c2;
    // CRC: 000000
    // RSSI: -5.7 dBFS
    // Score: 1800
    // Time: 11202660.42us
    // DF:17 AA:A46A7E CA:5 ME:58C7F5937AF6FB
    //  Extended Squitter Airborne position (barometric altitude) (11) (reliable)
    //   ICAO Address:  A46A7E (Mode S / ADS-B)
    //   Air/Ground:    airborne
    //   Baro altitude: 38975 ft
    //   CPR type:      Airborne
    //   CPR odd flag:  odd
    //   CPR latitude:  39.01436 (51645)
    //   CPR longitude: -84.14093 (63227)
    //   CPR decoding:  global
    //   NIC:           8
    //   Rc:            0.186 km / 0.1 NM
    //   NIC-B:         0
    //   NACp:          8
    //   SIL:           2 (p <= 0.001%, unknown type)
    #[test]
    fn testing07() {
        // TODO
        let bytes = hex!("8da46a7e58c7f5937af6fb63c3c2");
        let frame = Frame::from_bytes((&bytes, 0)).unwrap().1;
        println!("{:#?}", frame);
        if let DF::ADSB { me, .. } = frame.df {
            if let ME::AirbornePositionBaroAltitude(_me) = me {
                return;
            }
        }
        unreachable!();
    }

    // *5da039b46d7d81;
    // CRC: 000000
    // RSSI: -13.9 dBFS
    // Score: 750
    // Time: 183194.00us
    // DF:11 AA:A039B4 IID:0 CA:5
    //  All Call Reply (reliable)
    //   ICAO Address:  A039B4 (Mode S / ADS-B)
    //   Air/Ground:    airborne
    #[test]
    fn testing08() {
        let bytes = hex!("5da039b46d7d81");
        let frame = Frame::from_bytes((&bytes, 0)).unwrap().1;
        if let DF::ALL_CALL_REPLY {
            icao, capability, ..
        } = frame.df
        {
            assert_eq!(icao, hex!("a039b4"));
            assert_eq!(capability, Capability::AG_AIRBORNE);
            return;
        }
        unreachable!();
    }

    // *02e19ab8a0d3fa;
    // CRC: ac952b
    // RSSI: -15.5 dBFS
    // Score: 1000
    // Time: 43944312.67us
    // DF:0 addr:AC952B VS:0 CC:1 SL:7 RI:3 AC:6840
    //  Short Air-Air Surveillance
    //   ICAO Address:  AC952B (Mode S / ADS-B)
    //   Air/Ground:    airborne?
    //   Baro altitude: 42000 ft
    #[test]
    fn testing_DF_0() {
        let bytes = hex!("02e19ab8a0d3fa");
        let frame = Frame::from_bytes((&bytes, 0)).unwrap().1;
        println!("{:#?}", frame);
    }

    // *20001ab8a60bb0;
    // CRC: ac952b
    // RSSI: -15.9 dBFS
    // Score: 1000
    // Time: 44020405.75us
    // DF:4 addr:AC952B FS:0 DR:0 UM:0 AC:6840
    //  Survelliance, Altitude Reply
    //   ICAO Address:  AC952B (Mode S / ADS-B)
    //   Air/Ground:    airborne?
    //   Baro altitude: 42000 ft
}
