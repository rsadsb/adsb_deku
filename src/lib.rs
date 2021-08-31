use deku::prelude::*;
pub use deku::DekuContainerRead;

pub mod cpr;
mod crc;

use deku::bitvec::BitSlice;
use deku::bitvec::Msb0;

pub const MODES_LONG_MSG_BYTES: usize = 14;
pub const MODES_SHORT_MSG_BYTES: usize = 7;

#[derive(Debug, PartialEq, DekuRead, Clone)]
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
            DF::ShortAirAirSurveillance(address_altitude) => {
                writeln!(f, " Short Air-Air Surveillance")?;
                writeln!(f, "  ICAO Address:  {:06x} (Mode S / ADS-B)", self.crc)?;
                if address_altitude.altitude.0 > 0 {
                    writeln!(f, "  Air/Ground:    airborne?")?;
                    writeln!(
                        f,
                        "  Altitude:      {} ft barometric",
                        address_altitude.altitude.0
                    )?;
                } else {
                    writeln!(f, "  Air/Ground:    ground")?;
                }
            }
            DF::SurveillanceAltitudeReply { fs, ac, .. } => {
                writeln!(f, " Surveillance, Altitude Reply")?;
                writeln!(f, "  ICAO Address:  {:06x} (Mode S / ADS-B)", self.crc)?;
                writeln!(f, "  Air/Ground:    {}", fs)?;
                if ac.0 > 0 {
                    writeln!(f, "  Altitude:      {} ft barometric", ac.0)?;
                }
            }
            DF::SurveillanceIdentityReply { fs, id, .. } => {
                writeln!(f, " Surveillance, Identity Reply")?;
                writeln!(f, "  ICAO Address:  {:06x} (Mode S / ADS-B)", self.crc)?;
                writeln!(f, "  Air/Ground:    {}", fs)?;
                writeln!(f, "  Identity:      {:04x}", id.0)?;
            }
            DF::AllCallReply {
                capability, icao, ..
            } => {
                writeln!(f, " All Call Reply")?;
                writeln!(f, "  ICAO Address:  {} (Mode S / ADS-B)", icao)?;
                writeln!(f, "  Air/Ground:    {}", capability)?;
            }
            DF::LongAirAir(address_altitude) => {
                writeln!(f, " Long Air-Air ACAS")?;
                writeln!(f, "  ICAO Address:  {:06x} (Mode S / ADS-B)", self.crc)?;
                // TODO the airborne? should't be static
                if address_altitude.altitude.0 > 0 {
                    writeln!(f, "  Air/Ground:    airborne?")?;
                    writeln!(f, "  Baro altitude: {} ft", address_altitude.altitude.0)?;
                } else {
                    writeln!(f, "  Air/Ground:    ground")?;
                }
            }
            DF::ADSB(adsb) => {
                write!(f, "{}", adsb.to_string(17).unwrap())?;
            }
            DF::TisB(adsb) => {
                write!(f, "{}", adsb.to_string(18).unwrap())?;
            }
            DF::CommDExtendedLengthMessage => {
                writeln!(f, " Comm-D Extended Length Message");
                writeln!(f, "  ICAO Address:  {:x?} (Mode S / ADS-B)", self.crc);
            }
            DF::CommBIdentityReply {
                id, message_comm_b, ..
            } => {
                writeln!(f, " Comm-B, Identity Reply");
                if message_comm_b == "" {
                    writeln!(f, "    Comm-B format: unknown format");
                } else {
                    writeln!(f, "    Comm-B format: {}", message_comm_b);
                }
                writeln!(f, "    ICAO Address:  {:x?} (Mode S / ADS-B)", self.crc);
                writeln!(f, "    Squawk:        {:x?}", id);
            }
            _ => (),
        }
        Ok(())
    }
}

#[derive(Debug, PartialEq, DekuRead, Clone)]
#[deku(type = "u8", bits = "5")]
pub enum DF {
    #[deku(id = "0")]
    ShortAirAirSurveillance(AddressAltitude),
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
        /// 3 bytes
        p_icao: ICAO,
    },
    #[deku(id = "16")]
    LongAirAir(AddressAltitude),
    #[deku(id = "17")]
    ADSB(ADSB),
    /// Non-transponder-based ADS-B transmitting subsystems and TIS-B transmitting equipment
    #[deku(id = "18")]
    TisB(ADSB),
    #[deku(id = "20")]
    CommBAltitudeReply {
        flight_status: FlightStatus,
        dr: DownlinkRequest,
        um: UtilityMessage,
        #[deku(reader = "Altitude::read(deku::rest)")]
        alt_code: u32,
        #[deku(reader = "read_comm_b(deku::rest)")]
        message_comm_b: String,
        #[deku(endian = "big", bits = "24")]
        parity: u32,
    },
    #[deku(id = "21")]
    CommBIdentityReply {
        fs: FlightStatus,
        dr: DownlinkRequest,
        um: UtilityMessage,
        #[deku(
            bits = "13",
            endian = "big",
            map = "|squawk: u32| -> Result<_, DekuError> {Ok(decode_id13_field(squawk))}"
        )]
        id: u32,
        //#TODO: this works?
        #[deku(reader = "read_comm_b(deku::rest)")]
        message_comm_b: String,
        #[deku(endian = "big", bits = "24")]
        parity: u32,
    },
    //#TODO: Might be an actual field instead of just reading the crc like dump1090 does
    #[deku(id = "24")]
    CommDExtendedLengthMessage,
}

fn read_comm_b(rest: &BitSlice<Msb0, u8>) -> Result<(&BitSlice<Msb0, u8>, String), DekuError> {
    pub const AIS_CHARSET: &str =
        "@ABCDEFGHIJKLMNOPQRSTUVWXYZ[\\]^_ !\"#$%&'()*+,-./0123456789:;<=>?";
    let (rest, ident) = <u8>::read(rest, deku::ctx::Size::Bits(7))?;

    let mut inside_rest = rest;
    let mut callsign = String::new();
    if ident == 0x20 {
        for _ in 0..8 {
            let (for_rest, c) = <u8>::read(inside_rest, deku::ctx::Size::Bits(5))?;
            callsign.push(AIS_CHARSET.chars().nth(c as usize).unwrap());
            inside_rest = for_rest;
        }
    }
    Ok((inside_rest, callsign))
}

#[derive(Debug, PartialEq, DekuRead, Clone)]
pub struct AddressAltitude {
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
}

#[derive(Debug, PartialEq, DekuRead, Clone)]
pub struct ADSB {
    /// 3 bits
    pub capability: Capability,
    /// 3 bytes
    pub icao: ICAO,
    pub me: ME,
    #[deku(bits = "24")]
    pub pi: u32,
}

impl ADSB {
    fn to_string(&self, df: u8) -> Result<String, Box<dyn std::error::Error>> {
        use std::fmt::Write;
        let address_type = if df == 17 {
            "(Mode S / ADS-B)"
        } else if df == 18 {
            "(ADS-R)"
        } else {
            unreachable!();
        };

        let mut f = String::new();
        match &self.me {
            ME::AirbornePositionBaroAltitude(altitude) => {
                writeln!(
                    f,
                    " Extended Squitter Airborne position (barometric altitude) (11)"
                );
                writeln!(f, "  ICAO Address:  {} {}", self.icao, address_type);
                writeln!(f, "  Air/Ground:    {}", self.capability);
                write!(f, "{}", altitude);
            }
            ME::AirbornePositionGNSSAltitude(altitude) => {
                writeln!(
                    f,
                    " Extended Squitter (Non-Transponder) Airborne position (GNSS altitude) (20)"
                );
                writeln!(f, "  ICAO Address:  {} {}", self.icao, address_type);
                //writeln!(f, "  Air/Ground:    {}", self.capability);
                write!(f, "{}", altitude);
            }
            ME::TargetStateAndStatusInformation(target_info) => {
                writeln!(f, " Extended Squitter Target state and status (V2) (29/1)");
                writeln!(f, "  ICAO Address:  {} {}", self.icao, address_type);
                writeln!(f, "  Air/Ground:    {}", self.capability);
                writeln!(f, "  Target State and Status:");
                writeln!(f, "    Target altitude:   MCP, {} ft", target_info.altitude);
                writeln!(f, "    Altimeter setting: {} millibars", target_info.qnh);
                if target_info.is_heading {
                    writeln!(f, "    Target heading:    {}", target_info.heading);
                }
                if target_info.tcas {
                    write!(f, "    ACAS:              operational");
                    if target_info.autopilot {
                        write!(f, " autopilot ");
                    }
                    if target_info.vnav {
                        write!(f, " VNAC ");
                    }
                    if target_info.alt_hold {
                        write!(f, "altitude-hold ");
                    }
                    if target_info.approach {
                        write!(f, "approach");
                    }
                    writeln!(f);
                } else {
                    writeln!(f, "    ACAS:              NOT operational");
                }
                writeln!(f, "    NACp:              {}", target_info.nacp);
                writeln!(f, "    NICbaro:           {}", target_info.nicbaro);
                writeln!(f, "    SIL:               {} (per sample)", target_info.sil);
            }
            ME::AirborneVelocity(airborne_velocity) => {
                if let AirborneVelocitySubType::GroundSpeedDecoding(_) = airborne_velocity.sub_type
                {
                    let (heading, ground_speed, vertical_rate) = airborne_velocity.calculate();
                    writeln!(
                        f,
                        " Extended Squitter Airborne velocity over ground, subsonic (19/1)"
                    );
                    writeln!(f, "  ICAO Address:  {} {}", self.icao, address_type);
                    writeln!(f, "  Air/Ground:    {}", self.capability);
                    writeln!(
                        f,
                        "  GNSS delta:    {}{} ft",
                        airborne_velocity.gnss_sign, airborne_velocity.gnss_baro_diff
                    );
                    writeln!(f, "  Heading:       {}", heading.ceil());
                    writeln!(
                        f,
                        "  Speed:         {} kt groundspeed",
                        ground_speed.floor()
                    );
                    writeln!(
                        f,
                        "  Vertical rate: {} ft/min {}",
                        vertical_rate, airborne_velocity.vrate_src
                    );
                }
            }
            ME::AircraftStatus(AircraftStatus {
                sub_type,
                emergency_state,
                squawk,
                ..
            }) => {
                writeln!(f, " Extended Squitter Emergency/priority status (28/1)")?;
                writeln!(f, "  ICAO Address:  {} {}", self.icao, address_type)?;
                writeln!(f, "  Air/Ground:    {}", self.capability);
                writeln!(f, "  Squawk:        {}", squawk);
            }
            ME::AircraftIdentification(Identification { tc, ca, cn }) => {
                writeln!(
                    f,
                    " Extended Squitter Aircraft identification and category (4)"
                )?;
                writeln!(f, "  ICAO Address:  {} {}", self.icao, address_type)?;
                writeln!(f, "  Air/Ground:    {}", self.capability);
                writeln!(f, "  Ident:         {}", cn)?;
                writeln!(f, "  Category:      {}{}", tc, ca)?;
            }
            ME::AircraftOperationStatus(OperationStatus::Airborne(opstatus_airborne)) => {
                writeln!(
                    f,
                    " Extended Quitter Aircraft operational status (airborne) (31/0)"
                )?;
                writeln!(f, " ICAO Address:  {} {}", self.icao, address_type)?;
                writeln!(f, " Air/Ground:    {}", self.capability);
                write!(f, " Aircraft Operational Status:\n{}", opstatus_airborne)?;
            }
            _ => (),
        }
        Ok(f)
    }
}

#[derive(Debug, PartialEq, DekuRead, Copy, Clone)]
pub struct UtilityMessage {
    #[deku(bits = "4")]
    pub iis: u8,
    pub ids: UtilityMessageType,
}

#[derive(Debug, PartialEq, DekuRead, Copy, Clone)]
#[deku(type = "u8", bits = "2")]
pub enum UtilityMessageType {
    NoInformation = 0b00,
    CommB         = 0b01,
    CommC         = 0b10,
    CommD         = 0b11,
}

#[derive(Debug, PartialEq, DekuRead, Copy, Clone)]
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

#[derive(Debug, PartialEq, DekuRead, Copy, Clone)]
#[deku(type = "u8", bits = "5")]
pub enum DownlinkRequest {
    None               = 0b00000,
    RequestSendCommB   = 0b00001,
    CommBBroadcastMsg1 = 0b00100,
    CommBBroadcastMsg2 = 0b00101,
}

#[derive(Debug, PartialEq, DekuRead, Hash, Eq, Copy, Clone)]
pub struct ICAO(pub [u8; 3]);

impl std::fmt::Display for ICAO {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:02x}", self.0[0])?;
        write!(f, "{:02x}", self.0[1])?;
        write!(f, "{:02x}", self.0[2])?;
        Ok(())
    }
}

#[derive(Debug, PartialEq, DekuRead, Copy, Clone)]
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

        let num: u16 = (a << 12 | b << 8 | c << 4 | d) as u16;
        Ok((rest, num))
    }
}

#[derive(Debug, PartialEq, DekuRead, Copy, Clone)]
pub struct AC13Field(#[deku(reader = "Self::read(deku::rest)")] u32);

impl AC13Field {
    /// TODO Add unit
    fn read(rest: &BitSlice<Msb0, u8>) -> Result<(&BitSlice<Msb0, u8>, u32), DekuError> {
        let (rest, num) =
            u32::read(rest, (deku::ctx::Endian::Big, deku::ctx::Size::Bits(13))).unwrap();

        let m_bit = num & 0x0040;
        let q_bit = num & 0x0010;

        if m_bit != 0 {
            // TODO: this might be wrong?
            Ok((rest, 0))
        } else if q_bit != 0 {
            let n = ((num & 0x1f80) >> 2) | ((num & 0x0020) >> 1) | (num & 0x000f);
            let n = n as u32 * 25;
            if n > 1000 {
                Ok((rest, n - 1000))
            } else {
                // TODO: add error
                Ok((rest, 0))
            }
        } else {
            // TODO 11 bit gillham coded altitude
            if let Ok(n) = mode_ac::mode_a_to_mode_c(mode_ac::decode_id13_field(num)) {
                Ok((rest, (100 * n)))
            } else {
                Ok((rest, 0))
            }
        }
    }
}

/// TODO This should have sort of [ignore] attribute, since we don't need to implement DekuRead on this.
#[derive(Debug, PartialEq, DekuRead, Copy, Clone)]
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

#[derive(Debug, PartialEq, DekuRead, Copy, Clone)]
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
                Capability::AG_UNCERTAIN3 => "airborne?",
            }
        )
    }
}

#[derive(Debug, PartialEq, DekuRead, Clone)]
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
    AircraftStatus(AircraftStatus),
    #[deku(id = "29")]
    TargetStateAndStatusInformation(TargetStateAndStatusInformation),
    #[deku(id = "31")]
    AircraftOperationStatus(OperationStatus),
}

/// Table: A-2-97
#[derive(Debug, PartialEq, DekuRead, Copy, Clone)]
pub struct AircraftStatus {
    pub sub_type: AircraftStatusType,
    pub emergency_state: EmergencyState,
    #[deku(
        bits = "12",
        map = "|squawk: u32| -> Result<_, DekuError> {Ok(mode_ac::decode_id13_field(squawk))}"
    )]
    pub squawk: u32,
}

#[derive(Debug, PartialEq, DekuRead, Copy, Clone)]
#[deku(type = "u8", bits = "3")]
pub enum AircraftStatusType {
    #[deku(id = "0")]
    NoInformation,
    #[deku(id = "1")]
    EmergencyPriorityStatus,
    #[deku(id_pat = "_")]
    Reserved,
}

#[derive(Debug, PartialEq, DekuRead, Copy, Clone)]
#[deku(type = "u8", bits = "3")]
pub enum EmergencyState {
    None                 = 0,
    General              = 1,
    Lifeguard            = 2,
    MinimumFuel          = 4,
    UnlawfulInterference = 5,
    Reserved1            = 6,
    Reserved2            = 7,
}

#[derive(Debug, PartialEq, DekuRead, Copy, Clone)]
#[deku(type = "u8", bits = "3")]
pub enum OperationStatus {
    #[deku(id = "0")]
    Airborne(OperationStatusAirborne),
    #[deku(id = "1")]
    Surface(OperationStatusSurface),
}

#[derive(Debug, PartialEq, DekuRead, Copy, Clone)]
pub struct OperationStatusAirborne {
    // 16 bits
    pub capability_codes: CapabilityCode,
    #[deku(bits = "5")]
    pub operational_mode_unused1: u8,
    #[deku(bits = "1")]
    pub saf: bool,
    #[deku(bits = "2")]
    pub sda: u8,
    pub operational_mode_unused2: u8,
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
        write!(f, "   Operational modes:  ")?;
        if self.saf {
            write!(f, "SAF ");
        }
        if self.sda != 0 {
            write!(f, "SDA={}", self.sda)?;
        }
        writeln!(f)?;
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

#[derive(Debug, PartialEq, DekuRead, Copy, Clone)]
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

#[derive(Debug, PartialEq, DekuRead, Copy, Clone)]
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

#[derive(Debug, PartialEq, DekuRead, Copy, Clone)]
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

#[derive(Debug, PartialEq, DekuRead, Copy, Clone)]
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

#[derive(Debug, PartialEq, DekuRead, Clone)]
pub struct Identification {
    pub tc: TypeCoding,
    #[deku(bits = "3")]
    pub ca: u8,
    #[deku(reader = "Self::read(deku::rest)")]
    pub cn: String,
}

#[derive(Debug, PartialEq, DekuRead, Copy, Clone)]
#[deku(type = "u8", bits = "5")]
pub enum TypeCoding {
    D = 1,
    C = 2,
    B = 3,
    A = 4,
}

impl std::fmt::Display for TypeCoding {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}",
            match self {
                Self::D => "D",
                Self::C => "C",
                Self::B => "B",
                Self::A => "A",
            }
        )
    }
}

const CHAR_LOOKUP: &[u8; 64] = b"#ABCDEFGHIJKLMNOPQRSTUVWXYZ##### ###############0123456789######";

impl Identification {
    fn read(rest: &BitSlice<Msb0, u8>) -> Result<(&BitSlice<Msb0, u8>, String), DekuError> {
        let mut inside_rest = rest;

        let mut chars = vec![];
        for _ in 0..=6 {
            let (for_rest, c) = <u8>::read(inside_rest, deku::ctx::Size::Bits(6))?;
            if c != 32 {
                chars.push(c);
            }
            inside_rest = for_rest;
        }
        let encoded = chars
            .into_iter()
            .map(|b| CHAR_LOOKUP[b as usize] as char)
            .collect::<String>();

        Ok((inside_rest, encoded))
    }
}

#[derive(Debug, PartialEq, DekuRead, Default, Copy, Clone)]
pub struct Altitude {
    #[deku(bits = "5")]
    pub tc: u8,
    pub ss: SurveillanceStatus,
    #[deku(bits = "1")]
    pub saf: u8,
    #[deku(reader = "Self::read(deku::rest)")]
    pub alt: u32,
    /// UTC sync or not
    #[deku(bits = "1")]
    pub t: bool,
    /// Odd or even
    pub odd_flag: CPRFormat,
    #[deku(bits = "17", endian = "big")]
    pub lat_cpr: u32,
    #[deku(bits = "17", endian = "big")]
    pub lon_cpr: u32,
}

impl std::fmt::Display for Altitude {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        writeln!(f, "  Altitude:      {} ft barometric", self.alt);
        // TODO: fix me
        writeln!(f, "  CPR type:      Airborne");
        writeln!(f, "  CPR odd flag:  {}", self.odd_flag);
        // TODO: fix me
        writeln!(f, "  CPR NUCp/NIC:  7");
        writeln!(f, "  CPR latitude:  ({})", self.lat_cpr);
        writeln!(f, "  CPR longitude: ({})", self.lon_cpr);
        // TODO: fix me
        //println!("{}", self.t);
        writeln!(f, "  CPR decoding:  global");
        Ok(())
    }
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
            let n = n * 25;
            if n > 1000 {
                Ok((rest, (n - 1000) as u32))
            } else {
                // TODO add error
                Ok((rest, 0))
            }
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
fn decode_id13_field(id13_field: u32) -> u32 {
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

#[derive(Debug, PartialEq, DekuRead, Copy, Clone)]
pub struct SurfacePosition {
    #[deku(bits = "7")]
    pub mov: u8,
    pub s: StatusForGroundTrack,
    #[deku(bits = "7")]
    pub trk: u8,
    #[deku(bits = "1")]
    pub t: bool,
    pub f: CPRFormat,
    #[deku(bits = "17", endian = "big")]
    pub lat_cpr: u32,
    #[deku(bits = "17", endian = "big")]
    pub lon_cpr: u32,
}

#[derive(Debug, PartialEq, DekuRead, Copy, Clone)]
#[deku(type = "u8", bits = "1")]
pub enum StatusForGroundTrack {
    Invalid = 0,
    Valid   = 1,
}

#[derive(Debug, PartialEq, DekuRead, Copy, Clone)]
#[deku(type = "u8", bits = "2")]
pub enum SurveillanceStatus {
    NoCondition    = 0,
    PermanentAlert = 1,
    TemporaryAlert = 2,
    SPICondition   = 3,
}

impl Default for SurveillanceStatus {
    fn default() -> Self {
        Self::NoCondition
    }
}

#[derive(Debug, PartialEq, DekuRead, Copy, Clone)]
#[deku(type = "u8", bits = "1")]
pub enum CPRFormat {
    Even = 0,
    Odd  = 1,
}

impl Default for CPRFormat {
    fn default() -> Self {
        Self::Even
    }
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

#[derive(Debug, PartialEq, DekuRead, Copy, Clone)]
#[deku(type = "u8", bits = "1")]
pub enum VerticalRateSource {
    BarometricPressureAltitude = 0,
    GeometricAltitude          = 1,
}

impl std::fmt::Display for VerticalRateSource {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}",
            match self {
                Self::BarometricPressureAltitude => "barometric",
                Self::GeometricAltitude => "GNSS",
            }
        )
    }
}

#[derive(Debug, PartialEq, DekuRead, Copy, Clone)]
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

impl std::fmt::Display for Sign {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}",
            match self {
                Self::Positive => "",
                Self::Negative => "-",
            }
        )
    }
}

#[derive(Debug, PartialEq, DekuRead, Clone)]
pub struct AirborneVelocity {
    #[deku(bits = "3")]
    pub st: u8,
    #[deku(bits = "5")]
    pub extra: u8,
    #[deku(ctx = "*st")]
    pub sub_type: AirborneVelocitySubType,
    pub vrate_src: VerticalRateSource,
    pub vrate_sign: Sign,
    #[deku(endian = "big", bits = "9")]
    pub vrate_value: u16,
    #[deku(bits = "2")]
    pub reverved: u8,
    pub gnss_sign: Sign,
    #[deku(
        bits = "7",
        map = "|gnss_baro_diff: u16| -> Result<_, DekuError> {Ok(if gnss_baro_diff > 1 {(gnss_baro_diff - 1)* 25} else { 0 })}"
    )]
    pub gnss_baro_diff: u16,
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

#[derive(Debug, PartialEq, DekuRead, Clone)]
#[deku(ctx = "st: u8", id = "st")]
pub enum AirborneVelocitySubType {
    #[deku(id_pat = "1..=2")]
    GroundSpeedDecoding(GroundSpeedDecoding),
    #[deku(id_pat = "3..=4")]
    AirspeedDecoding(AirspeedDecoding),
}

#[derive(Debug, PartialEq, DekuRead, Copy, Clone)]
pub struct GroundSpeedDecoding {
    pub ew_sign: Sign,
    #[deku(endian = "big", bits = "10")]
    pub ew_vel: u16,
    pub ns_sign: Sign,
    #[deku(endian = "big", bits = "10")]
    pub ns_vel: u16,
}

#[derive(Debug, PartialEq, DekuRead, Clone)]
pub struct AirspeedDecoding {
    #[deku(bits = "1")]
    pub status_heading: u8,
    #[deku(endian = "big", bits = "10")]
    pub mag_heading: u16,
    #[deku(bits = "1")]
    pub airspeed_type: u8,
    #[deku(endian = "big", bits = "10")]
    pub airspeed: u16,
}

#[derive(Copy, Clone, Debug, PartialEq, DekuRead)]
#[deku(type = "u8", bits = "3")]
pub enum AirborneVelocityType {
    Subsonic   = 1,
    Supersonic = 3,
}

#[derive(Debug, PartialEq, DekuRead, Copy, Clone)]
#[deku(ctx = "t: AirborneVelocityType")]
pub struct AirborneVelocitySubFields {
    pub dew: DirectionEW,
    #[deku(reader = "Self::read_v(deku::rest, t)")]
    pub vew: u16,
    pub dns: DirectionNS,
    #[deku(reader = "Self::read_v(deku::rest, t)")]
    pub vns: u16,
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
    pub subtype: u8,
    #[deku(bits = "1")]
    pub is_fms: bool,
    #[deku(
        bits = "12",
        endian = "big",
        map = "|altitude: u32| -> Result<_, DekuError> {Ok(if altitude > 1 {(altitude - 1) * 32} else {0} )}"
    )]
    pub altitude: u32,
    #[deku(
        bits = "9",
        endian = "big",
        map = "|qnh: u32| -> Result<_, DekuError> {if qnh == 0 { Ok(0.0) } else { Ok(800.0 + ((qnh - 1) as f32) * 0.8)}}"
    )]
    pub qnh: f32,
    #[deku(bits = "1")]
    pub is_heading: bool,
    #[deku(
        bits = "9",
        endian = "big",
        map = "|heading: u32| -> Result<_, DekuError> {Ok(heading as f32 * 180.0 / 256.0)}"
    )]
    pub heading: f32,
    #[deku(bits = "4")]
    pub nacp: u8,
    #[deku(bits = "1")]
    pub nicbaro: u8,
    #[deku(bits = "2")]
    pub sil: u8,
    #[deku(bits = "1")]
    pub mode_validity: bool,
    #[deku(bits = "1")]
    pub autopilot: bool,
    #[deku(bits = "1")]
    pub vnav: bool,
    #[deku(bits = "1")]
    pub alt_hold: bool,
    #[deku(bits = "1")]
    pub imf: bool,
    #[deku(bits = "1")]
    pub approach: bool,
    #[deku(bits = "1")]
    pub tcas: bool,
    #[deku(bits = "1")]
    pub lnav: bool,
    #[deku(bits = "2")]
    pub reserved: u8,
}

mod mode_ac {
    /// convert from mode A hex to 0-4095 index
    pub fn mode_a_to_index(mode_a: u32) -> u32 {
        (mode_a & 0x0007)
            | ((mode_a & 0x0070) >> 1)
            | ((mode_a & 0x0700) >> 2)
            | ((mode_a & 0x7000) >> 3)
    }

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
        if five_hundreds & 1 != 0 && one_hundreds <= 6 {
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
        (five_hundreds * 5) + one_hundreds - 13
    }

    pub fn init() -> [u32; 4096] {
        let mut table = [0_u32; 4096];
        for i in 0..4096_usize {
            let mode_a = index_to_mode_a(i as u32);
            let mode_c = internal_mode_a_to_mode_c(mode_a);
            table[i] = mode_c;
        }
        table
    }
}
