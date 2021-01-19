use deku::prelude::*;

#[derive(Debug, PartialEq, DekuRead, DekuWrite)]
pub struct Frame {
    /// 5 bits
    pub df: DF,
    /// 3 bits
    pub capability: Capability,
    /// 3 bytes
    pub icao: [u8; 3],
    pub me: ME,
    #[deku(bits = "24")]
    pub pi: u32,
}

#[derive(Debug, PartialEq, DekuRead, DekuWrite)]
#[deku(type = "u8", bits = "5")]
pub enum DF {
    #[deku(id = "0b10001")]
    ADSB,
    #[deku(id = "0b10010")]
    TIS_B,
}

#[derive(Debug, PartialEq, DekuRead, DekuWrite)]
#[deku(type = "u8", bits = "3")]
#[allow(non_camel_case_types)]
pub enum Capability {
    #[deku(id = "0x00")]
    AG_UNCERTAIN,
    #[deku(id_pat = "0x01..=0x03")]
    Reserved,
    #[deku(id = "0x04")]
    AG_GROUND,
    #[deku(id = "0x05")]
    AG_AIRBORNE,
    #[deku(id = "0x06")]
    AG_UNCERTAIN2,
    #[deku(id = "0x07")]
    AG_UNCERTAIN3,
}

#[derive(Debug, PartialEq, DekuRead, DekuWrite)]
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
    AircraftOperationStatus(AircraftOperationStatus),
}

#[derive(Debug, PartialEq, DekuRead, DekuWrite)]
pub struct Identification {
    #[deku(bits = "5")]
    pub tc: u8,
    #[deku(bits = "3")]
    pub ca: u8,
    #[deku(
        reader = "Self::read(deku::rest)",
        writer = "Self::write(&self.cn, deku::output)"
    )]
    pub cn: String,
}

const CHAR_LOOKUP: &[u8; 64] = b"#ABCDEFGHIJKLMNOPQRSTUVWXYZ##### ###############0123456789######";

impl Identification {
    fn read(rest: &BitSlice<Msb0, u8>) -> Result<(&BitSlice<Msb0, u8>, String), DekuError> {
        let mut inside_rest = rest;

        let mut chars = vec![];
        for _ in 0..=6 {
            let (for_rest, c) = <u8>::read(inside_rest, (deku::ctx::Size::Bits(6)))?;
            chars.push(c);
            inside_rest = for_rest;
        }
        let encoded = chars
            .into_iter()
            .map(|b| CHAR_LOOKUP[b as usize] as char)
            .collect::<String>();

        Ok((inside_rest, encoded))
    }

    fn write(cn: &String, output: &mut BitVec<Msb0, u8>) -> Result<(), DekuError> {
        //TODO
        Ok(())
    }
}

#[derive(Debug, PartialEq, DekuRead, DekuWrite)]
pub struct Altitude {
    #[deku(bits = "5")]
    tc: u8,
    ss: SurveillanceStatus,
    #[deku(bits = "1")]
    saf: u8,
    #[deku(
        reader = "Self::read(deku::rest)",
        writer = "Self::write(&self.alt, deku::output)"
    )]
    alt: u16,
    #[deku(bits = "1")]
    t: bool,
    f: CPRFormat,
    #[deku(bits = "17", endian = "big")]
    lat_cpr: u32,
    #[deku(bits = "17", endian = "big")]
    lon_cpr: u32,
}

impl Altitude {
    fn read(rest: &BitSlice<Msb0, u8>) -> Result<(&BitSlice<Msb0, u8>, u16), DekuError> {
        let (rest, l) =
            u16::read(rest, (deku::ctx::Endian::Little, deku::ctx::Size::Bits(7))).unwrap();

        let (rest, q) =
            u16::read(rest, (deku::ctx::Endian::Little, deku::ctx::Size::Bits(1))).unwrap();

        let (rest, r) =
            u16::read(rest, (deku::ctx::Endian::Little, deku::ctx::Size::Bits(4))).unwrap();

        let altitude = (l.rotate_left(4) + r)
            .checked_mul(if q == 0 { 100 } else { 25 })
            .and_then(|r| r.checked_sub(1000));
        Ok((rest, altitude.unwrap()))
    }

    fn write(calt: &u16, output: &mut BitVec<Msb0, u8>) -> Result<(), DekuError> {
        //TODO
        Ok(())
    }
}

#[derive(Debug, PartialEq, DekuRead, DekuWrite)]
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

#[derive(Debug, PartialEq, DekuRead, DekuWrite)]
#[deku(type = "u8", bits = "1")]
pub enum StatusForGroundTrack {
    #[deku(id = "0")]
    Invalid,
    #[deku(id = "1")]
    Valid,
}

#[derive(Debug, PartialEq, DekuRead, DekuWrite)]
#[deku(type = "u8", bits = "2")]
pub enum SurveillanceStatus {
    #[deku(id = "0")]
    NoCondition,
    #[deku(id = "1")]
    PermanentAlert,
    #[deku(id = "2")]
    TemporaryAlert,
    #[deku(id = "3")]
    SPICondition,
}

#[derive(Debug, PartialEq, DekuRead, DekuWrite)]
#[deku(type = "u8", bits = "1")]
pub enum CPRFormat {
    #[deku(id = "0")]
    Even,
    #[deku(id = "1")]
    Odd,
}

#[derive(Debug, PartialEq, DekuRead, DekuWrite)]
#[deku(type = "u8", bits = "1")]
pub enum VerticalRateSource {
    #[deku(id = "0")]
    BarometricPressureAltitude,
    #[deku(id = "1")]
    GeometricAltitude,
}

#[derive(Debug, PartialEq, DekuRead, DekuWrite)]
#[deku(type = "u8", bits = "1")]
pub enum Sign {
    #[deku(id = "0")]
    Positive,
    #[deku(id = "1")]
    Negative,
}

impl Sign {
    pub fn value(&self) -> i16 {
        match self {
            Self::Positive => 1,
            Self::Negative => -1,
        }
    }
}

#[derive(Debug, PartialEq, DekuRead, DekuWrite)]
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
        let v_ew = ((self.ew_vel as i16 - 1) * self.ew_sign.value()) as f64;
        let v_ns = ((self.ns_vel as i16 - 1) * self.ns_sign.value()) as f64;
        let h = v_ew.atan2(v_ns) * (360.0 / (2.0 * std::f64::consts::PI));
        let heading = if h < 0.0 { h + 360.0 } else { h };

        let vrate = self
            .vrate_value
            .checked_sub(1)
            .and_then(|v| v.checked_mul(64))
            .map(|v| (v as i16) * self.vrate_sign.value())
            .unwrap();

        (heading, ((v_ew.powi(2) + v_ns.powi(2)).sqrt()), vrate)
    }
}

#[derive(Copy, Clone, Debug, PartialEq, DekuRead, DekuWrite)]
#[deku(type = "u8", bits = "3")]
pub enum AirborneVelocityType {
    #[deku(id = "1")]
    Subsonic,
    #[deku(id = "3")]
    Supersonic,
}

#[derive(Debug, PartialEq, DekuRead, DekuWrite)]
#[deku(ctx = "t: AirborneVelocityType")]
pub struct AirborneVelocitySubFields {
    dew: DirectionEW,
    #[deku(
        reader = "Self::read_v(deku::rest, t)",
        writer = "Self::write_v(&self.vew, deku::output)"
    )]
    vew: u16,
    dns: DirectionNS,
    #[deku(
        reader = "Self::read_v(deku::rest, t)",
        writer = "Self::write_v(&self.vns, deku::output)"
    )]
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

    fn write_v(val: &u16, output: &mut BitVec<Msb0, u8>) -> Result<(), DekuError> {
        //TODO
        Ok(())
    }
}

#[derive(Copy, Clone, Debug, PartialEq, DekuRead, DekuWrite)]
#[deku(type = "u8", bits = "1")]
pub enum DirectionEW {
    #[deku(id = "0")]
    WestToEast,
    #[deku(id = "1")]
    EastToWest,
}

#[derive(Copy, Clone, Debug, PartialEq, DekuRead, DekuWrite)]
#[deku(type = "u8", bits = "1")]
pub enum DirectionNS {
    #[deku(id = "0")]
    SouthToNorth,
    #[deku(id = "1")]
    NorthToSouth,
}

#[derive(Copy, Clone, Debug, PartialEq, DekuRead, DekuWrite)]
#[deku(type = "u8", bits = "1")]
pub enum SourceBitVerticalRate {
    #[deku(id = "0")]
    GNSS,
    #[deku(id = "1")]
    Barometer,
}

#[derive(Copy, Clone, Debug, PartialEq, DekuRead, DekuWrite)]
#[deku(type = "u8", bits = "1")]
pub enum SignBitVerticalRate {
    #[deku(id = "0")]
    Up,
    #[deku(id = "1")]
    Down,
}

#[derive(Copy, Clone, Debug, PartialEq, DekuRead, DekuWrite)]
#[deku(type = "u8", bits = "1")]
pub enum SignBitGNSSBaroAltitudesDiff {
    #[deku(id = "0")]
    Above,
    #[deku(id = "1")]
    Below,
}

/// Version 2
#[derive(Debug, PartialEq, DekuRead, DekuWrite)]
pub struct AircraftOperationStatus {
    st: AircraftOperationStatusSubType,
    #[deku(ctx = "*st")]
    cc: CapacityClassCodes,
    #[deku(bits = "16")]
    om: u16,
    #[deku(bits = "3")]
    ver: u8,
    #[deku(bits = "1")]
    nica: u8,
    #[deku(bits = "4")]
    nacp: u8,
    #[deku(bits = "2")]
    gva: u8,
    #[deku(bits = "2")]
    sil: u8,
    #[deku(bits = "1")]
    something: u8,
    #[deku(bits = "1")]
    hrd: u8,
    #[deku(bits = "1")]
    sils: u8,
    #[deku(bits = "1")]
    reserved: u8,
}

#[derive(Debug, PartialEq, DekuRead, DekuWrite)]
#[deku(id = "t", ctx = "t: AircraftOperationStatusSubType")]
pub enum CapacityClassCodes {
    #[deku(id = "AircraftOperationStatusSubType::Airborne")]
    Airborne(u16),
    #[deku(id = "AircraftOperationStatusSubType::Surface")]
    Surface(#[deku(bits = "12")] u16, #[deku(bits = "4")] u8),
}

#[derive(Copy, Clone, Debug, PartialEq, DekuRead, DekuWrite)]
#[deku(type = "u8", bits = "3")]
pub enum AircraftOperationStatusSubType {
    #[deku(id = "0")]
    Airborne,
    #[deku(id = "1")]
    Surface,
}

#[derive(Copy, Clone, Debug, PartialEq, DekuRead, DekuWrite)]
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
        map = "|qnh: u32| -> Result<_, DekuError> {Ok(800.0 + ((qnh - 1) as f32) * 0.8)}"
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

#[cfg(test)]
mod tests {
    use super::*;
    use hexlit::hex;

    // #[test]
    // fn identification() {
    //     let bytes = hex!("8D4840D6202CC371C32CE0576098");
    //     let v = BitSlice::<Msb0, _>::from_slice(&bytes).unwrap();
    //     println!("{}", v);
    //     let frame = Frame::from_bytes((&bytes, 0));
    //     println!("{:#x?}", frame);
    // }

    //#[test]
    //fn altitude() {
    //    let bytes = hex!("8D40058B58C901375147EFD09357");
    //    let frame = Frame::from_bytes((&bytes, 0));
    //    println!("{:#x?}", frame);
    //}

    //#[test]
    //fn surface_position() {
    //    let bytes = hex!("8C4841753AAB238733C8CD4020B1");
    //    let frame = Frame::from_bytes((&bytes, 0));
    //    println!("{:#x?}", frame);
    //}

    //#[test]
    //fn airborne_velocity() {
    //    let bytes = hex!("8D485020994409940838175B284F");
    //    let frame = Frame::from_bytes((&bytes, 0));
    //    println!("{:#x?}", frame);
    //}

    //#[test]
    //fn identification() {
    //    let bytes = hex!("8D4840D6202CC371C32CE0576098");
    //    let frame = Frame::from_bytes((&bytes, 0));
    //    println!("{:#x?}", frame);
    //}

    #[test]
    fn testing01() {
        // from adsb-rs
        let bytes = hex!("8D40621D58C382D690C8AC2863A7");
        let frame = Frame::from_bytes((&bytes, 0));
        if let ME::AirbornePositionBaroAltitude(me) = frame.unwrap().1.me {
            assert_eq!(me.alt, 38000);
            assert_eq!(me.lat_cpr, 93000);
            assert_eq!(me.lon_cpr, 51372);
            assert_eq!(me.f, CPRFormat::Even);
        } else {
            unreachable!();
        }
    }

    #[test]
    fn testing02() {
        // from adsb-rs
        let bytes = hex!("8da3d42599250129780484712c50");
        let frame = Frame::from_bytes((&bytes, 0));
        if let ME::AirborneVelocity(me) = frame.unwrap().1.me {
            let (heading, ground_speed, vertical_rate) = me.calculate();
            assert_eq!(heading, 322.1972075490615);
            assert_eq!(ground_speed, 417.6553603151766);
            assert_eq!(vertical_rate, 0);
            assert_eq!(me.vrate_src, VerticalRateSource::GeometricAltitude);
        } else {
            unreachable!();
        }
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
        if let ME::TargetStateAndStatusInformation(me) = frame.unwrap().1.me {
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
        } else {
            unreachable!();
        }
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
    //#[test]
    //fn example() {
    //    let bytes = hex!("8dacc040f8210002004ab8569c35");
    //    let frame = Frame::from_bytes((&bytes, 0)).unwrap().1;
    //    //println!("{:#?}", frame);
    //    assert_eq!(frame.capability, Capability::Level2Airborne);
    //    assert_eq!(frame.icao, [0xac, 0xc0, 0x40]);
    //    if let ME::AircraftOperationStatus(status) = frame.me {
    //        assert_eq!(status.nica, 0);
    //        assert_eq!(status.nica, 0);
    //    } else {
    //        unreachable!();
    //    }
    //}

    //#[test]
    //fn identification() {
    //    let bytes = hex!("8da2b728589b7256649518d79c30");
    //    let frame = Frame::from_bytes((&bytes, 0));
    //    println!("{:#?}", frame);
    //}
}

//
//  dump1090
//
//*8da2b728589b7256649518d79c30;
// CRC: 000000
// RSSI: -25.0 dBFS
// Score: 1800
// Time: 5636190.17us
// DF:17 AA:A2B728 CA:5 ME:589B7256649518
//  Extended Squitter Airborne position (barometric altitude) (11) (reliable)
//   ICAO Address:  A2B728 (Mode S / ADS-B)
//   Air/Ground:    airborne
//   Baro altitude: 29975 ft
//   CPR type:      Airborne
//   CPR odd flag:  even
//   CPR latitude:  39.50620 (76594)
//   CPR longitude: -83.80801 (38168)
//   CPR decoding:  global
//   NIC:           8
//   Rc:            0.186 km / 0.1 NM
//   NIC-B:         0
//   NACp:          8
//   SIL:           2 (p <= 0.001%, unknown type)
//
//
// Frame {
//            df: ADSB,
//            capability: Level2Airborne,
//            icao: [
//                162,
//                183,
//                40,
//            ],
//            me: AirbornePositionBaroAltitude(
//                Altitude {
//                    tc: WithBarometricAltitude,
//                    ss: PermanentAlert,
//                    saf: false,
//                    alt: 3524,
//                    t: true,
//                    f: Even,
//                    lat_cpr: 109796,
//                    lon_cpr: 21650,
//                },
//            ),
//            pi: 14793926,
//        },
//    ),
