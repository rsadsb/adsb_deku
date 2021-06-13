use deku::prelude::*;
pub use deku::DekuContainerRead;
mod crc;

use deku::bitvec::BitSlice;
use deku::bitvec::Msb0;

pub const MODES_LONG_MSG_BYTES: usize = 14;
pub const MODES_SHORT_MSG_BYTES: usize = 7;

#[derive(Debug, PartialEq, DekuRead)]
pub struct Frame {
    /// 5 bits
    pub df: DF,
    /// Calculated from all bits, used as ICAO for Response packets
    #[deku(reader = "Self::read_crc(df, deku::input_bits)")]
    pub crc: u32,
}

impl Frame {
    /// Read and convert to String
    fn read_crc<'a, 'b>(
        df: &'a DF,
        rest: &'b BitSlice<Msb0, u8>,
    ) -> Result<(&'b BitSlice<Msb0, u8>, u32), DekuError> {
        let bit_len = modes_message_len_by_type(df);
        let crc = crc::modes_checksum(rest.as_raw_slice(), bit_len);
        Ok((rest, crc))
    }
}

pub fn modes_message_len_by_type(typ: &DF) -> usize {
    if typ.deku_id().unwrap() & 0x10 != 0 {
        MODES_LONG_MSG_BYTES * 8
    } else {
        MODES_SHORT_MSG_BYTES * 8
    }
}

impl std::fmt::Display for Frame {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match &self.df {
            DF::ShortAirAirSurveillance { altitude, .. } => {
                writeln!(f, " Short Air-Air Surveillance")?;
                // TODO the Mode S ADS-B shouldn't be static
                writeln!(f, "  ICAO Address:  {:06x} (Mode S / ADS-B)", self.crc)?;
                // TODO the airborne? should't be static
                writeln!(f, "  Air/Ground:    airborne?")?;
                writeln!(f, "  Altitude:      {} ft barometric", altitude.0)?;
            }
            DF::SurveillanceAltitudeReply { fs, ac, .. } => {
                writeln!(f, " Surveillance, Altitude Reply")?;
                // TODO: fix me
                writeln!(f, "  ICAO Address:  a3ecce (Mode S / ADS-B)")?;
                writeln!(f, "  Air/Ground:    {}", fs)?;
                writeln!(f, "  Altitude:      {} ft barometric", ac.0)?;
            }
            DF::SurveillanceIdentityReply { fs, id, .. } => {
                writeln!(f, " Surveillance, Identity Reply")?;
                // TODO: fix me
                writeln!(f, "  ICAO Address:  ?????? (Mode S / ADS-B)")?;
                writeln!(f, "  Air/Ground:    {}", fs)?;
                writeln!(f, "  Identity:      {:04x}", id.0)?;
            }
            DF::AllCallReply { capability, icao } => {
                writeln!(f, " All Call Reply")?;
                writeln!(f, "  ICAO Address:  {} (Mode S / ADS-B)", icao)?;
                writeln!(f, "  Air/Ground:    {}", capability)?;
            }
            DF::ADSB {
                capability,
                icao,
                me,
                ..
            } => match me {
                ME::AirbornePositionBaroAltitude(Altitude {
                    alt,
                    odd_flag,
                    lat_cpr,
                    lon_cpr,
                    ..
                }) => {
                    writeln!(
                        f,
                        " Extended Squitter Airborne position (barometric altitude) (11)"
                    );
                    writeln!(f, "  ICAO Address:  {} (Mode S / ADS-B)", icao);
                    writeln!(f, "  Air/Ground:    {}", capability);
                    writeln!(f, "  Altitude:      {} ft barometric", alt);
                    // TODO: fix me
                    writeln!(f, "  CPR type:      Airborne");
                    writeln!(f, "  CPR odd flag:  {}", odd_flag);
                    // TODO: fix me
                    writeln!(f, "  CPR NUCp/NIC:  7");
                    writeln!(f, "  CPR latitude:  ({})", lat_cpr);
                    writeln!(f, "  CPR longitude: ({})", lon_cpr);
                    // TODO: fix me
                    writeln!(f, "  CPR decoding:  none");
                }
                ME::AirborneVelocity(airborne_velocity) => {
                    if let AirborneVelocitySubType::GroundSpeedDecoding(_) =
                        airborne_velocity.sub_type
                    {
                        let (heading, ground_speed, vertical_rate) = airborne_velocity.calculate();
                        println!("{} {} {}", heading, ground_speed, vertical_rate);
                        writeln!(
                            f,
                            " Extended Squitter Airborne velocity over ground, subsonic (19/1)"
                        );
                        writeln!(f, "  ICAO Address:  {} (Mode S / ADS-B)", icao);
                        writeln!(f, "  Air/Ground:    {}", capability);
                        writeln!(
                            f,
                            "  GNSS delta:    {} ft",
                            airborne_velocity.gnss_baro_diff
                        );
                        writeln!(f, "  Heading:       {}", heading.ceil());
                        writeln!(
                            f,
                            "  Speed:         {} kt groundspeed",
                            ground_speed.floor()
                        );
                        writeln!(f, "  Vertical rate: {} ft/min GNSS", vertical_rate);
                    }
                }
                ME::AircraftOperationStatus(OperationStatus::Airborne(opstatus_airborne)) => {
                    writeln!(
                        f,
                        " Extended Quitter Aircraft operational status (airborne) (31/0)"
                    )?;
                    writeln!(f, " ICAO Address:  {} (Mode S / ADS-B)", icao)?;
                    writeln!(f, " Air/Ground:    airborne")?;
                    write!(f, " Aircraft Operational Status:\n{}", opstatus_airborne)?;
                }
                _ => (),
            },
            _ => (),
        }
        Ok(())
    }
}

#[derive(Debug, PartialEq, DekuRead)]
#[deku(type = "u8", bits = "5")]
pub enum DF {
    #[deku(id = "0")]
    ShortAirAirSurveillance {
        /// bit 6
        #[deku(bits = "1")]
        vs: u8,

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
        #[deku(bits = "2")]
        ri: u8,

        ///// bits 18-19
        #[deku(bits = "2")]
        unused2: u8,
        /// bits 20-32
        altitude: AC13Field,
    },
    #[deku(id = "4")]
    SurveillanceAltitudeReply {
        fs: FlightStatus,
        #[deku(bits = "5")]
        dr: u8,
        um: UtilityMessage,
        ac: AC13Field,
        #[deku(bits = "24")]
        ap: u32,
    },
    #[deku(id = "5")]
    SurveillanceIdentityReply {
        fs: FlightStatus,
        #[deku(bits = "5")]
        dr: u8,
        um: UtilityMessage,
        id: IdentityCode,
        #[deku(bits = "24")]
        ap: u32,
    },
    #[deku(id = "11")]
    AllCallReply {
        /// 3 bits
        capability: Capability,
        /// 3 bytes
        icao: ICAO,
    },
    #[deku(id = "17")]
    ADSB {
        /// 3 bits
        capability: Capability,
        /// 3 bytes
        icao: ICAO,
        me: ME,
        #[deku(bits = "24")]
        pi: u32,
    },
    #[deku(id = "18")]
    TisB {
        /// 3 bits
        #[deku(bits = "3")]
        cf: u8,
        /// 3 bytes
        icao: ICAO,
    },
}

#[derive(Debug, PartialEq, DekuRead)]
pub struct UtilityMessage {
    #[deku(bits = "4")]
    pub iis: u8,
    pub ids: UtilityMessageType,
}

#[derive(Debug, PartialEq, DekuRead)]
#[deku(type = "u8", bits = "2")]
pub enum UtilityMessageType {
    NoInformation = 0b00,
    CommB         = 0b01,
    CommC         = 0b10,
    CommD         = 0b11,
}

#[derive(Debug, PartialEq, DekuRead)]
#[deku(type = "u8", bits = "3")]
pub enum FlightStatus {
    NoAlertNoSPIAirborne     = 0b000,
    NoAlertNoSPIOnGround     = 0b001,
    AlertNoSPIAirborne       = 0b010,
    AlertNoSPIOnGround       = 0b011,
    AlertSPIAirborneGround   = 0b100,
    NoAlertSPIAirborneGround = 0b101,
    Reserved                 = 0b110,
    NotAssigned              = 0b111,
}

impl std::fmt::Display for FlightStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}",
            match self {
                Self::NoAlertNoSPIAirborne => "airborne?",
                Self::NoAlertNoSPIOnGround => "ground?",
                Self::AlertNoSPIAirborne => "airborne",
                Self::AlertNoSPIOnGround => "ground",
                Self::AlertSPIAirborneGround => "airborne?",
                Self::NoAlertSPIAirborneGround => "airborne?",
                _ => "reserved",
            }
        )
    }
}

#[derive(Debug, PartialEq, DekuRead)]
#[deku(type = "u8", bits = "5")]
pub enum DownlinkRequest {
    None               = 0b00000,
    RequestSendCommB   = 0b00001,
    CommBBroadcastMsg1 = 0b00100,
    CommBBroadcastMsg2 = 0b00101,
}

#[derive(Debug, PartialEq, DekuRead)]
pub struct ICAO([u8; 3]);

impl std::fmt::Display for ICAO {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:02x}", self.0[0])?;
        write!(f, "{:02x}", self.0[1])?;
        write!(f, "{:02x}", self.0[2])?;
        Ok(())
    }
}

#[derive(Debug, PartialEq, DekuRead)]
pub struct IdentityCode(#[deku(reader = "Self::read(deku::rest)")] u16);

impl IdentityCode {
    fn read(rest: &BitSlice<Msb0, u8>) -> Result<(&BitSlice<Msb0, u8>, u16), DekuError> {
        let (rest, num) =
            u32::read(rest, (deku::ctx::Endian::Big, deku::ctx::Size::Bits(13))).unwrap();

        let c1 = (num & 0b1_0000_0000_0000) >> 12;
        let a1 = (num & 0b0_1000_0000_0000) >> 11;
        let c2 = (num & 0b0_0100_0000_0000) >> 10;
        let a2 = (num & 0b0_0010_0000_0000) >> 9;
        let c4 = (num & 0b0_0001_0000_0000) >> 8;
        let a4 = (num & 0b0_0000_1000_0000) >> 7;
        let _ = (num & 0b0_0000_0100_0000) >> 6;
        let b1 = (num & 0b0_0000_0010_0000) >> 5;
        let d1 = (num & 0b0_0000_0001_0000) >> 4;
        let b2 = (num & 0b0_0000_0000_1000) >> 3;
        let d2 = (num & 0b0_0000_0000_0100) >> 2;
        let b4 = (num & 0b0_0000_0000_0010) >> 1;
        let d4 = (num & 0b0_0000_0000_0001) >> 0;

        let a = a4 << 2 | a2 << 1 | a1;
        let b = b4 << 2 | b2 << 1 | b1;
        let c = c4 << 2 | c2 << 1 | c1;
        let d = d4 << 2 | d2 << 1 | d1;
        println!("{} {} {} {}", a, b, c, d);

        let num: u16 = (a << 12 | b << 8 | c << 4 | d) as u16;
        Ok((rest, num))
    }
}

#[derive(Debug, PartialEq, DekuRead)]
pub struct AC13Field(#[deku(reader = "Self::read(deku::rest)")] u32);

impl AC13Field {
    /// TODO Add unit
    fn read(rest: &BitSlice<Msb0, u8>) -> Result<(&BitSlice<Msb0, u8>, u32), DekuError> {
        let (rest, num) =
            u32::read(rest, (deku::ctx::Endian::Big, deku::ctx::Size::Bits(13))).unwrap();

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
    Meter = 0,
    Feet  = 1,
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

impl std::fmt::Display for Capability {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}",
            match self {
                Capability::AG_UNCERTAIN => "uncertain1",
                Capability::Reserved => "reserved",
                Capability::AG_GROUND => "ground",
                Capability::AG_AIRBORNE => "airborne",
                Capability::AG_UNCERTAIN2 => "uncertain2",
                Capability::AG_UNCERTAIN3 => "uncertain3",
            }
        )
    }
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
    AircraftOperationStatus(OperationStatus),
}

#[derive(Debug, PartialEq, DekuRead)]
#[deku(type = "u8", bits = "3")]
pub enum OperationStatus {
    #[deku(id = "0")]
    Airborne(OperationStatusAirborne),
    #[deku(id = "1")]
    Surface(OperationStatusSurface),
}

#[derive(Debug, PartialEq, DekuRead)]
pub struct OperationStatusAirborne {
    // 16 bits
    pub capability_codes: CapabilityCode,
    pub operational_mode_codes: u16,
    pub version_number: ADSBVersion,
    #[deku(bits = "1")]
    pub nic_supplement_a: u8,
    #[deku(bits = "4")]
    pub navigational_accuracy_category: u8,
    #[deku(bits = "2")]
    pub geometric_vertical_accuracy: u8,
    #[deku(bits = "2")]
    pub source_integrity_level: u8,
    #[deku(bits = "1")]
    pub barometric_altitude_integrity: u8,
    #[deku(bits = "1")]
    pub horizontal_reference_direction: u8,
    #[deku(bits = "1")]
    pub sil_supplement: u8,
    #[deku(bits = "1")]
    pub reserved: u8,
}

impl std::fmt::Display for OperationStatusAirborne {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        writeln!(f, "   Version:            {}", self.version_number)?;
        writeln!(f, "   Capability classes:{}", self.capability_codes)?;
        writeln!(f, "   Operational modes:  {}", self.operational_mode_codes)?;
        writeln!(f, "   NIC-A:              {}", self.nic_supplement_a);
        writeln!(
            f,
            "   NACp:               {}",
            self.navigational_accuracy_category
        );
        writeln!(
            f,
            "   GVA:                {}",
            self.geometric_vertical_accuracy
        );
        writeln!(
            f,
            "   SIL:                {} (per hour)",
            self.source_integrity_level
        );
        writeln!(
            f,
            "   NICbaro:            {}",
            self.barometric_altitude_integrity
        );
        if self.horizontal_reference_direction != 1 {
            writeln!(f, "   Heading reference:  true north")?;
        } else {
            writeln!(f, "   Heading reference:  magnetic north")?;
        }
        Ok(())
    }
}

#[derive(Debug, PartialEq, DekuRead)]
pub struct CapabilityCode {
    #[deku(bits = "2")]
    pub reserved0: u8,
    #[deku(bits = "1")]
    pub acas: u8,
    #[deku(bits = "1")]
    pub cdti: u8,
    #[deku(bits = "2")]
    pub reserved1: u8,
    #[deku(bits = "1")]
    pub arv: u8,
    #[deku(bits = "1")]
    pub ts: u8,
    #[deku(bits = "2")]
    pub tc: u8,
    #[deku(bits = "6")]
    pub reserved2: u8,
}

impl std::fmt::Display for CapabilityCode {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if self.acas == 1 {
            write!(f, " ACAS")?;
        }
        if self.cdti == 1 {
            write!(f, " CDTI")?;
        }
        if self.arv == 1 {
            write!(f, " ARV")?;
        }
        if self.ts == 1 {
            write!(f, " TS")?;
        }
        if self.tc == 1 {
            write!(f, " TC")?;
        }
        Ok(())
    }
}

#[derive(Debug, PartialEq, DekuRead)]
pub struct OperationStatusSurface {
    pub capacity_codes: CapabilityCode,
    #[deku(bits = "4")]
    pub capacity_len_code: u8,
    pub operational_mode_codes: u16,
    pub version_number: ADSBVersion,
    #[deku(bits = "1")]
    pub nic_supplement_a: u8,
    #[deku(bits = "4")]
    pub navigational_accuracy_category: u8,
    #[deku(bits = "1")]
    pub reserved0: u8,
    #[deku(bits = "2")]
    pub source_integrity_level: u8,
    #[deku(bits = "1")]
    pub track_angle_or_heading: u8,
    #[deku(bits = "1")]
    pub horizontal_reference_direction: u8,
    #[deku(bits = "1")]
    pub sil_supplement: u8,
    #[deku(bits = "1")]
    pub reserved1: u8,
}

#[derive(Debug, PartialEq, DekuRead)]
pub struct OperationCodeSurface {
    #[deku(bits = "1")]
    pub poe: u8,
    #[deku(bits = "1")]
    pub cdti: u8,
    #[deku(bits = "1")]
    pub b2_low: u8,
    #[deku(bits = "3")]
    pub lw: u8,
    #[deku(bits = "6")]
    pub reserved: u16,
}

#[derive(Debug, PartialEq, DekuRead)]
#[deku(type = "u8", bits = "3")]
pub enum ADSBVersion {
    DOC9871AppendixA = 0b000,
    DOC9871AppendixB = 0b001,
    DOC9871AppendixC = 0b010,
}

impl std::fmt::Display for ADSBVersion {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}",
            match self {
                Self::DOC9871AppendixA => "0",
                Self::DOC9871AppendixB => "1",
                Self::DOC9871AppendixC => "2",
            }
        )
    }
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
    odd_flag: CPRFormat,
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

impl std::fmt::Display for CPRFormat {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}",
            match self {
                Self::Even => "even",
                Self::Odd => "odd",
            }
        )
    }
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
    #[deku(ctx = "*st")]
    sub_type: AirborneVelocitySubType,
    vrate_src: VerticalRateSource,
    vrate_sign: Sign,
    #[deku(endian = "big", bits = "9")]
    vrate_value: u16,
    #[deku(bits = "2")]
    reverved: u8,
    gnss_sign: Sign,
    #[deku(
        bits = "7",
        map = "|gnss_baro_diff: u16| -> Result<_, DekuError> {Ok((gnss_baro_diff - 1)* 25)}"
    )]
    gnss_baro_diff: u16,
}

impl AirborneVelocity {
    /// Return effective (heading, ground_speed, vertical_rate)
    pub fn calculate(&self) -> (f64, f64, i16) {
        if let AirborneVelocitySubType::GroundSpeedDecoding(ground_speed) = &self.sub_type {
            let v_ew = f64::from((ground_speed.ew_vel as i16 - 1) * ground_speed.ew_sign.value());
            let v_ns = f64::from((ground_speed.ns_vel as i16 - 1) * ground_speed.ns_sign.value());
            let h = v_ew.atan2(v_ns) * (360.0 / (2.0 * std::f64::consts::PI));
            let heading = if h < 0.0 { h + 360.0 } else { h };

            let vrate = self
                .vrate_value
                .checked_sub(1)
                .and_then(|v| v.checked_mul(64))
                .map(|v| (v as i16) * self.vrate_sign.value())
                .unwrap();

            (heading, v_ew.hypot(v_ns), vrate)
        } else {
            panic!();
        }
    }
}

#[derive(Debug, PartialEq, DekuRead)]
#[deku(ctx = "st: u8", id = "st")]
pub enum AirborneVelocitySubType {
    #[deku(id_pat = "1..=2")]
    GroundSpeedDecoding(GroundSpeedDecoding),
    #[deku(id_pat = "3..=4")]
    AirspeedDecoding(AirspeedDecoding),
}

#[derive(Debug, PartialEq, DekuRead)]
pub struct GroundSpeedDecoding {
    ew_sign: Sign,
    #[deku(endian = "big", bits = "10")]
    ew_vel: u16,
    ns_sign: Sign,
    #[deku(endian = "big", bits = "10")]
    ns_vel: u16,
}

#[derive(Debug, PartialEq, DekuRead)]
pub struct AirspeedDecoding {
    #[deku(bits = "1")]
    status_heading: u8,
    #[deku(endian = "big", bits = "10")]
    mag_heading: u16,
    #[deku(bits = "1")]
    airspeed_type: u8,
    #[deku(endian = "big", bits = "10")]
    airspeed: u16,
}

#[derive(Copy, Clone, Debug, PartialEq, DekuRead)]
#[deku(type = "u8", bits = "3")]
pub enum AirborneVelocityType {
    Subsonic   = 1,
    Supersonic = 3,
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
    use assert_hex::assert_eq_hex;
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
                assert_eq!(me.odd_flag, CPRFormat::Even);
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
            assert_eq_hex!(icao.0, [0xac, 0xc0, 0x40]);
            assert_eq!(capability, Capability::AG_AIRBORNE);
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
        if let DF::AllCallReply {
            icao, capability, ..
        } = frame.df
        {
            assert_eq_hex!(icao.0, hex!("ab3d17"));
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
        if let DF::AllCallReply {
            icao, capability, ..
        } = frame.df
        {
            assert_eq_hex!(icao.0, hex!("a039b4"));
            assert_eq!(capability, Capability::AG_AIRBORNE);
            return;
        }
        unreachable!();
    }

    //*02e19cb02512c3;
    //CRC: 0d097e
    //RSSI: -8.1 dBFS
    //Score: 1000
    //Time: 91219304.17us
    //DF:0 addr:0D097E VS:0 CC:1 SL:7 RI:3 AC:7344
    // Short Air-Air Surveillance
    //  ICAO Address:  0D097E (Mode S / ADS-B)
    //  Air/Ground:    airborne?
    //  Altitude:      45000 ft barometric
    #[test]
    fn testing_df_shortairairsurveillance() {
        let bytes = hex!("02e19cb02512c3");
        let frame = Frame::from_bytes((&bytes, 0)).unwrap().1;
        let resulting_string = format!("{}", frame);
        assert_eq!(
            r#" Short Air-Air Surveillance
  ICAO Address:  0d097e (Mode S / ADS-B)
  Air/Ground:    airborne?
  Altitude:      45000 ft barometric
"#,
            resulting_string
        );
    }

    // -----new-----
    // ---deku
    // Frame {
    //    df: ADSB {
    //        capability: AG_AIRBORNE,
    //        icao: [
    //            13,
    //            9,
    //            126,
    //        ],
    //        me: AircraftOperationStatus(
    //            Airborne(
    //                OperationStatusAirborne {
    //                    capacity_class_codes: 35,
    //                    operational_mode_codes: 7,
    //                    version_number: DOC9871AppendixC,
    //                    nic_supplement_a: 1,
    //                    navigational_accuracy_category: 10,
    //                    geometric_vertical_accuracy: 1,
    //                    source_integrity_level: 1,
    //                    barometric_altitude_integrity: 1,
    //                    horizontal_reference_direction: 1,
    //                    sil_supplement: 0,
    //                    reserved: 0,
    //                },
    //            ),
    //        ),
    //        pi: 3422506,
    //    },
    //    crc: 0,
    //}
    // ---regular
    // *8d0d097ef8230007005ab8547268;
    // CRC: 000000
    // RSSI: -10.3 dBFS
    // Score: 1800
    // Time: 92723308.25us
    // DF:17 AA:0D097E CA:5 ME:F8230007005AB8
    //  Extended Squitter Aircraft operational status (airborne) (31/0)
    //   ICAO Address:  0D097E (Mode S / ADS-B)
    //   Air/Ground:    airborne
    //   Aircraft Operational Status:
    //     Version:            2
    //     Capability classes: ACAS ARV TS
    //     Operational modes:  SAF SDA=3
    //     NIC-A:              1
    //     NACp:               10
    //     GVA:                2
    //     SIL:                3 (per hour)
    //     NICbaro:            1
    //     Heading reference:  true north
    #[test]
    fn testing_df_extendedsquitteraircraftopstatus() {
        let bytes = hex!("8d0d097ef8230007005ab8547268");
        let frame = Frame::from_bytes((&bytes, 0)).unwrap().1;
        let resulting_string = format!("{}", frame);
        //TODO: Operational modes:  SAF SDA=3
        assert_eq!(
            r#" Extended Quitter Aircraft operational status (airborne) (31/0)
 ICAO Address:  0d097e (Mode S / ADS-B)
 Air/Ground:    airborne
 Aircraft Operational Status:
   Version:            2
   Capability classes: ACAS ARV TS
   Operational modes:  7
   NIC-A:              1
   NACp:               10
   GVA:                2
   SIL:                3 (per hour)
   NICbaro:            1
   Heading reference:  true north
"#,
            resulting_string
        );
    }

    #[test]
    fn testing_allcall_reply() {
        let bytes = hex!("5da58fd4561b39");
        let frame = Frame::from_bytes((&bytes, 0)).unwrap().1;
        let resulting_string = format!("{}", frame);
        assert_eq!(
            r#" All Call Reply
  ICAO Address:  a58fd4 (Mode S / ADS-B)
  Air/Ground:    airborne
"#,
            resulting_string
        );
    }

    #[test]
    fn testing_airbornepositionbaroaltitude() {
        let bytes = hex!("8dac537858af85d576faed51e731");
        let frame = Frame::from_bytes((&bytes, 0)).unwrap().1;
        let resulting_string = format!("{}", frame);
        assert_eq!(
            r#" Extended Squitter Airborne position (barometric altitude) (11)
  ICAO Address:  ac5378 (Mode S / ADS-B)
  Air/Ground:    airborne
  Altitude:      34000 ft barometric
  CPR type:      Airborne
  CPR odd flag:  odd
  CPR NUCp/NIC:  7
  CPR latitude:  (60091)
  CPR longitude: (64237)
  CPR decoding:  none
"#,
            resulting_string
        );
    }

    #[test]
    fn testing_surveillancealtitudereply() {
        let bytes = hex!("200012b0d96e39");
        let frame = Frame::from_bytes((&bytes, 0)).unwrap().1;
        let resulting_string = format!("{}", frame);
        assert_eq!(
            r#" Surveillance, Altitude Reply
  ICAO Address:  a3ecce (Mode S / ADS-B)
  Air/Ground:    airborne?
  Altitude:      29000 ft barometric
"#,
            resulting_string
        );
    }

    // TODO
    // This test is from mode-s.org, check with the dump1090-rs
    #[test]
    fn testing_surveillanceidentityreply() {
        let bytes = hex!("2A00516D492B80");
        let frame = Frame::from_bytes((&bytes, 0)).unwrap().1;
        let resulting_string = format!("{}", frame);
        assert_eq!(
            r#" Surveillance, Identity Reply
  ICAO Address:  ?????? (Mode S / ADS-B)
  Air/Ground:    airborne
  Identity:      0356
"#,
            resulting_string
        );
    }

    #[test]
    fn testing_airbornevelocity() {
        let bytes = hex!("8dac8e1a9924263950043944cf32");
        let frame = Frame::from_bytes((&bytes, 0)).unwrap().1;
        let resulting_string = format!("{}", frame);
        assert_eq!(
            r#" Extended Squitter Airborne velocity over ground, subsonic (19/1)
  ICAO Address:  ac8e1a (Mode S / ADS-B)
  Air/Ground:    airborne
  GNSS delta:    1400 ft
  Heading:       356
  Speed:         458 kt groundspeed
  Vertical rate: 0 ft/min GNSS
"#,
            resulting_string
        );
    }
}
