#![cfg_attr(not(feature = "std"), no_std)]
#![cfg_attr(docsrs, feature(doc_auto_cfg))]
#![doc(html_logo_url = "https://raw.githubusercontent.com/rsadsb/adsb_deku/master/media/logo.png")]
/*!
`adsb_deku` provides decoding for the [`ADS-B`] Downlink protocol by using the [`deku`] crate.

See [`rsadsb.github.io`] for more details.

# Downlink Format Support
|  [`DF`]  |  Name                               |  Section    |
| -------- | ----------------------------------- | ----------- |
| 0        | [`Short Air-Air Surveillance`]      | 3.1.2.8.2   |
| 4        | [`Surveillance Altitude Reply`]     | 3.1.2.6.5   |
| 5        | [`Surveillance Identity Reply`]     | 3.1.2.6.7   |
| 11       | [`All Call Reply`]                  | 2.1.2.5.2.2 |
| 16       | [`Long Air-Air Surveillance`]       | 3.1.2.8.3   |
| 17       | [`Extended Squitter(ADS-B)`]        | 3.1.2.8.6   |
| 18       | [`Extended Squitter(TIS-B)`]        | 3.1.2.8.7   |
| 19       | [`Extended Squitter(Military)`]     | 3.1.2.8.8   |
| 20       | [`Comm-B Altitude Reply`]           | 3.1.2.6.6   |
| 21       | [`Comm-B Identity Reply`]           | 3.1.2.6.8   |
| 24       | [`Comm-D`]                          | 3.1.2.7.3   |

# [`Comm-B Altitude Reply`] and [`Comm-B Identity Reply`] Comm-B Support

|  [`BDS`]  |  Name                                   |  Table      |
| --------- | --------------------------------------- | ----------- |
| (0,0)     | [`Empty`]                               |             |
| (1,0)     | [`Data Link Capability`]                | A-2-16      |
| (2,0)     | [`Aircraft Identification`]             | A-2-32      |

# [`Extended Squitter(ADS-B)`] and [`Extended Squitter(TIS-B)`] Type Code Support

|  [`ME`](Type Code)  |  Name                                  |
| ------------------- | -----------------------------------    |
| 0                   | [`ME::NoPosition`]                     |
| 1..=4               | [`ME::AircraftIdentification`]         |
| 5..=8               | [`ME::SurfacePosition`]                |
| 9..=18              | [`ME::AirbornePositionBaroAltitude`]   |
| 19                  | [`ME::AirborneVelocity`]               |
| 20..=22             | [`ME::AirbornePositionGNSSAltitude`]   |
| 23                  | [`ME::Reserved0`]                      |
| 24                  | [`ME::SurfaceSystemStatus`]            |
| 25..=27             | [`ME::Reserved1`]                      |
| 28                  | [`ME::AircraftStatus`]                 |
| 29                  | [`ME::TargetStateAndStatusInformation`]|
| 30                  | [`ME::AircraftOperationalCoordination`]|
| 31                  | [`ME::AircraftOperationStatus`]        |

# Example
To begin using `adsb_deku`, import the [`Frame`] struct.
This trait is re-exported for your convenience. [`Frame::from_reader()`] provides the interface for decoding bytes
into adsb data.

```rust
use hexlit::hex;
use adsb_deku::Frame;

let bytes = hex!("8da2c1bd587ba2adb31799cb802b");
let frame = Frame::from_reader(bytes.as_ref()).unwrap();
assert_eq!(
        r#" Extended Squitter Airborne position (barometric altitude)
  Address:       a2c1bd (Mode S / ADS-B)
  Air/Ground:    airborne
  Altitude:      23650 ft barometric
  CPR type:      Airborne
  CPR odd flag:  even
  CPR latitude:  (87769)
  CPR longitude: (71577)
"#,
    frame.to_string()
);
```

# Apps
The [`apps/`] directory of the project repository contains programs `radar` and `1090` for showcasing
different `adsb_deku` uses. See the [`README.md`] for examples of use.

[`DF`]: crate::DF
[`ME`]: crate::adsb::ME
[`BDS`]: crate::bds::BDS
[`Short Air-Air Surveillance`]: crate::DF::ShortAirAirSurveillance
[`Surveillance Altitude Reply`]: crate::DF::SurveillanceAltitudeReply
[`Surveillance Identity Reply`]: crate::DF::SurveillanceIdentityReply
[`All Call Reply`]: crate::DF::AllCallReply
[`Long Air-Air Surveillance`]: crate::DF::LongAirAir
[`Extended Squitter(ADS-B)`]: crate::DF::ADSB
[`Extended Squitter(TIS-B)`]: crate::DF::TisB
[`Extended Squitter(Military)`]: crate::DF::ExtendedQuitterMilitaryApplication
[`Comm-B Altitude Reply`]: crate::DF::CommBAltitudeReply
[`Comm-B Identity Reply`]: crate::DF::CommBIdentityReply
[`Comm-D`]: crate::DF::CommDExtendedLengthMessage

[`Empty`]: crate::bds::BDS::Empty
[`Data Link Capability`]: crate::bds::BDS::DataLinkCapability
[`Aircraft Identification`]: crate::bds::BDS::AircraftIdentification
[`ME::NoPosition`]: crate::adsb::ME::NoPosition
[`ME::AircraftIdentification`]: crate::adsb::ME::AircraftIdentification
[`ME::SurfacePosition`]: crate::adsb::ME::SurfacePosition
[`ME::AirbornePositionBaroAltitude`]: crate::adsb::ME::AirbornePositionBaroAltitude
[`ME::AirborneVelocity`]: crate::adsb::ME::AirborneVelocity
[`ME::AirbornePositionGNSSAltitude`]: crate::adsb::ME::AirbornePositionGNSSAltitude
[`ME::Reserved0`]: crate::adsb::ME::Reserved0
[`ME::SurfaceSystemStatus`]: crate::adsb::ME::SurfaceSystemStatus
[`ME::Reserved1`]: crate::adsb::ME::Reserved1
[`ME::AircraftStatus`]: crate::adsb::ME::AircraftStatus
[`ME::TargetStateAndStatusInformation`]: crate::adsb::ME::TargetStateAndStatusInformation
[`ME::AircraftOperationalCoordination`]: crate::adsb::ME::AircraftOperationalCoordination
[`ME::AircraftOperationStatus`]: crate::adsb::ME::AircraftOperationStatus

[`README.md`]: https://github.com/rsadsb/adsb_deku/blob/master/README.md
[`apps/`]: https://github.com/rsadsb/adsb_deku/tree/master/apps
[`ADS-B`]: https://en.wikipedia.org/wiki/Automatic_Dependent_Surveillance%E2%80%93Broadcast
[`deku`]: https://github.com/sharksforarms/deku
[`rsadsb.github.io`]: https://rsadsb.github.io/
*/

// good reference: http://www.anteni.net/adsb/Doc/1090-WP30-18-DRAFT_DO-260B-V42.pdf
//
// Maybe always reference this in the future?

extern crate alloc;

use alloc::borrow::Cow;
#[cfg(feature = "alloc")]
use alloc::{fmt, format, string::String, string::ToString, vec, vec::Vec};
#[cfg(feature = "alloc")]
use core::{
    clone::Clone,
    cmp::Eq,
    cmp::PartialEq,
    default::Default,
    fmt::Debug,
    hash::Hash,
    iter::IntoIterator,
    marker::Copy,
    prelude::rust_2021::derive,
    result,
    result::Result::{Err, Ok},
    write, writeln,
};

pub mod adsb;
pub mod bds;
pub mod cpr;
mod crc;
mod mode_ac;

#[doc = include_str!("../README.md")]
mod readme_test {}

use adsb::{ControlField, ADSB};
use bds::BDS;
use deku::no_std_io::Read;
use deku::prelude::*;

/// Every read to this struct will be saved into an internal cache. This is to keep the cache
/// around for the crc without reading from the buffer twice!
struct ReaderCrc<R: Read> {
    reader: R,
    cache: Vec<u8>,
}

impl<R: Read> Read for ReaderCrc<R> {
    fn read(&mut self, buf: &mut [u8]) -> deku::no_std_io::Result<usize> {
        let n = self.reader.read(buf);
        self.cache.extend_from_slice(buf);
        n
    }
}

/// Downlink ADS-B Packet
#[derive(Debug, PartialEq, Clone)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct Frame {
    /// Starting with 5 bit identifier, decode packet
    pub df: DF,
    /// Calculated from all bits, used as ICAO for Response packets
    pub crc: u32,
}

impl Frame {
    pub fn from_reader<R>(r: R) -> Result<Frame, DekuError>
    where
        R: Read,
    {
        let mut reader_crc = ReaderCrc { reader: r, cache: vec![] };
        let mut reader = Reader::new(&mut reader_crc);
        let df = DF::from_reader_with_ctx(&mut reader, ())?;

        let crc = Self::read_crc(&df, &mut reader_crc)?;

        Ok(Self { df, crc })
    }
}

impl Frame {
    /// Read rest as CRC bits
    fn read_crc<R: Read>(df: &DF, reader: &mut ReaderCrc<R>) -> result::Result<u32, DekuError> {
        const MODES_LONG_MSG_BYTES: usize = 14;
        const MODES_SHORT_MSG_BYTES: usize = 7;

        let bit_len = if let Ok(id) = df.deku_id() {
            if id & 0x10 != 0 {
                MODES_LONG_MSG_BYTES * 8
            } else {
                MODES_SHORT_MSG_BYTES * 8
            }
        } else {
            // In this case, it's the DF::CommD, which has multiple ids
            MODES_LONG_MSG_BYTES * 8
        };

        if bit_len > reader.cache.len() * 8 {
            let mut buf = vec![];
            reader.read_to_end(&mut buf).unwrap();
            reader.cache.append(&mut buf);
        }

        let crc = crc::modes_checksum(&reader.cache, bit_len)?;
        Ok(crc)
    }
}

impl fmt::Display for Frame {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let crc = self.crc;
        match &self.df {
            DF::ShortAirAirSurveillance { altitude, .. } => {
                writeln!(f, " Short Air-Air Surveillance")?;
                writeln!(f, "  ICAO Address:  {crc:06x} (Mode S / ADS-B)")?;
                if altitude.0 > 0 {
                    let altitude = altitude.0;
                    writeln!(f, "  Air/Ground:    airborne?")?;
                    writeln!(f, "  Altitude:      {altitude} ft barometric")?;
                } else {
                    writeln!(f, "  Air/Ground:    ground")?;
                }
            }
            DF::SurveillanceAltitudeReply { fs, ac, .. } => {
                writeln!(f, " Surveillance, Altitude Reply")?;
                writeln!(f, "  ICAO Address:  {crc:06x} (Mode S / ADS-B)")?;
                writeln!(f, "  Air/Ground:    {fs}")?;
                if ac.0 > 0 {
                    let altitude = ac.0;
                    writeln!(f, "  Altitude:      {altitude} ft barometric")?;
                }
            }
            DF::SurveillanceIdentityReply { fs, id, .. } => {
                let identity = id.0;
                writeln!(f, " Surveillance, Identity Reply")?;
                writeln!(f, "  ICAO Address:  {crc:06x} (Mode S / ADS-B)")?;
                writeln!(f, "  Air/Ground:    {fs}")?;
                writeln!(f, "  Identity:      {identity:04x}")?;
            }
            DF::AllCallReply { capability, icao, .. } => {
                writeln!(f, " All Call Reply")?;
                writeln!(f, "  ICAO Address:  {icao} (Mode S / ADS-B)")?;
                writeln!(f, "  Air/Ground:    {capability}")?;
            }
            DF::LongAirAir { altitude, .. } => {
                writeln!(f, " Long Air-Air ACAS")?;
                writeln!(f, "  ICAO Address:  {crc:06x} (Mode S / ADS-B)")?;
                // TODO the airborne? should't be static
                if altitude.0 > 0 {
                    let altitude = altitude.0;
                    writeln!(f, "  Air/Ground:    airborne?")?;
                    writeln!(f, "  Baro altitude: {altitude} ft")?;
                } else {
                    writeln!(f, "  Air/Ground:    ground")?;
                }
            }
            DF::ADSB(adsb) => {
                write!(f, "{}", adsb.to_string("(Mode S / ADS-B)")?)?;
            }
            DF::TisB { cf, .. } => {
                write!(f, "{cf}")?;
            }
            // TODO
            DF::ExtendedQuitterMilitaryApplication { .. } => {}
            DF::CommBAltitudeReply { bds, alt, .. } => {
                writeln!(f, " Comm-B, Altitude Reply")?;
                writeln!(f, "  ICAO Address:  {crc:x?} (Mode S / ADS-B)")?;
                let altitude = alt.0;
                writeln!(f, "  Altitude:      {altitude} ft")?;
                write!(f, "  {bds}")?;
            }
            DF::CommBIdentityReply { id, bds, .. } => {
                writeln!(f, " Comm-B, Identity Reply")?;
                writeln!(f, "    ICAO Address:  {crc:x?} (Mode S / ADS-B)")?;
                writeln!(f, "    Squawk:        {id:x?}")?;
                write!(f, "    {bds}")?;
            }
            DF::CommDExtendedLengthMessage { .. } => {
                writeln!(f, " Comm-D Extended Length Message")?;
                writeln!(f, "    ICAO Address:     {crc:x?} (Mode S / ADS-B)")?;
            }
        }
        Ok(())
    }
}

/// Downlink Format (3.1.2.3.2.1.2)
///
/// Starting with 5 bits, decode the rest of the message as the correct data packets
#[derive(Debug, PartialEq, DekuRead, Clone)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[deku(id_type = "u8", bits = 5)]
pub enum DF {
    /// 17: Extended Squitter, Downlink Format 17 (3.1.2.8.6)
    ///
    /// Civil aircraft ADS-B message
    #[deku(id = "17")]
    ADSB(ADSB),

    /// 11: (Mode S) All-call reply, Downlink format 11 (2.1.2.5.2.2)
    #[deku(id = "11")]
    AllCallReply {
        /// CA: Capability
        capability: Capability,
        /// AA: Address Announced
        icao: ICAO,
        /// PI: Parity/Interrogator identifier
        p_icao: ICAO,
    },

    /// 0: (Mode S) Short Air-Air Surveillance, Downlink Format 0 (3.1.2.8.2)
    #[deku(id = "0")]
    ShortAirAirSurveillance {
        /// VS: Vertical Status
        #[deku(bits = 1)]
        vs: u8,
        /// CC:
        #[deku(bits = 1)]
        cc: u8,
        /// Spare
        #[deku(bits = 1)]
        unused: u8,
        /// SL: Sensitivity level, ACAS
        #[deku(bits = 3)]
        sl: u8,
        /// Spare
        #[deku(bits = 2)]
        unused1: u8,
        /// RI: Reply Information
        #[deku(bits = 4)]
        ri: u8,
        /// Spare
        #[deku(bits = 2)]
        unused2: u8,
        /// AC: altitude code
        altitude: AC13Field,
        /// AP: address, parity
        parity: ICAO,
    },

    /// 4: (Mode S) Surveillance Altitude Reply, Downlink Format 4 (3.1.2.6.5)
    #[deku(id = "4")]
    SurveillanceAltitudeReply {
        /// FS: Flight Status
        fs: FlightStatus,
        /// DR: DownlinkRequest
        dr: DownlinkRequest,
        /// UM: Utility Message
        um: UtilityMessage,
        /// AC: AltitudeCode
        ac: AC13Field,
        /// AP: Address/Parity
        ap: ICAO,
    },

    /// 5: (Mode S) Surveillance Identity Reply (3.1.2.6.7)
    #[deku(id = "5")]
    SurveillanceIdentityReply {
        /// FS: Flight Status
        fs: FlightStatus,
        /// DR: Downlink Request
        dr: DownlinkRequest,
        /// UM: UtilityMessage
        um: UtilityMessage,
        /// ID: Identity
        id: IdentityCode,
        /// AP: Address/Parity
        ap: ICAO,
    },

    /// 16: (Mode S) Long Air-Air Surveillance Downlink Format 16 (3.1.2.8.3)
    #[deku(id = "16")]
    LongAirAir {
        #[deku(bits = 1)]
        vs: u8,
        #[deku(bits = 2)]
        spare1: u8,
        #[deku(bits = 3)]
        sl: u8,
        #[deku(bits = 2)]
        spare2: u8,
        #[deku(bits = 4)]
        ri: u8,
        #[deku(bits = 2)]
        spare3: u8,
        /// AC: altitude code
        altitude: AC13Field,
        /// MV: message, acas
        #[deku(count = "7")]
        mv: Vec<u8>,
        /// AP: address, parity
        parity: ICAO,
    },

    /// 18: Extended Squitter/Supplementary, Downlink Format 18 (3.1.2.8.7)
    ///
    /// Non-Transponder-based ADS-B Transmitting Subsystems and TIS-B Transmitting equipment.
    /// Equipment that cannot be interrogated.
    #[deku(id = "18")]
    TisB {
        /// Enum containing message
        cf: ControlField,
        /// PI: parity/interrogator identifier
        pi: ICAO,
    },

    /// 19: Extended Squitter Military Application, Downlink Format 19 (3.1.2.8.8)
    #[deku(id = "19")]
    ExtendedQuitterMilitaryApplication {
        /// Reserved
        #[deku(bits = 3)]
        af: u8,
    },

    /// 20: COMM-B Altitude Reply (3.1.2.6.6)
    #[deku(id = "20")]
    CommBAltitudeReply {
        /// FS: Flight Status
        flight_status: FlightStatus,
        /// DR: Downlink Request
        dr: DownlinkRequest,
        /// UM: Utility Message
        um: UtilityMessage,
        /// AC: Altitude Code
        alt: AC13Field,
        /// MB Message, Comm-B
        bds: BDS,
        /// AP: address/parity
        parity: ICAO,
    },

    /// 21: COMM-B Reply, Downlink Format 21 (3.1.2.6.8)
    #[deku(id = "21")]
    CommBIdentityReply {
        /// FS: Flight Status
        fs: FlightStatus,
        /// DR: Downlink Request
        dr: DownlinkRequest,
        /// UM: Utility Message
        um: UtilityMessage,
        /// ID: Identity
        #[deku(
            bits = 13,
            endian = "big",
            map = "|squawk: u32| -> Result<_, DekuError> {Ok(mode_ac::decode_id13_field(squawk))}"
        )]
        id: u32,
        /// MB Message, Comm-B
        bds: BDS,
        /// AP address/parity
        parity: ICAO,
    },

    /// 24..=31: Comm-D(ELM), Downlink Format 24 (3.1.2.7.3)
    #[deku(id_pat = "24..=31")]
    CommDExtendedLengthMessage {
        // Read from DF, which is usually 5 bits but is listed as only 1 bit for this format.
        //
        // So the 4th bit is KE, 5th bit is ND.
        id: u8,
        /// MD: message, Comm-D, 80 bits
        #[deku(count = "10")]
        md: Vec<u8>,
        /// AP: address/parity
        parity: ICAO,
    },
}

/// Latitude, Longitude and Altitude information
#[derive(Debug, PartialEq, Eq, DekuRead, Default, Copy, Clone)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct Altitude {
    pub ss: SurveillanceStatus,
    #[deku(bits = 1)]
    pub saf_or_imf: u8,
    #[deku(reader = "Self::read(deku::reader)")]
    pub alt: Option<u16>,
    /// UTC sync or not
    #[deku(bits = 1)]
    pub t: bool,
    /// Odd or even
    pub odd_flag: CPRFormat,
    #[deku(bits = 17, endian = "big")]
    pub lat_cpr: u32,
    #[deku(bits = 17, endian = "big")]
    pub lon_cpr: u32,
}

impl fmt::Display for Altitude {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let altitude = self
            .alt
            .map_or_else(|| "None".to_string(), |altitude| format!("{altitude} ft barometric"));
        writeln!(f, "  Altitude:      {altitude}")?;
        writeln!(f, "  CPR type:      Airborne")?;
        writeln!(f, "  CPR odd flag:  {}", self.odd_flag)?;
        writeln!(f, "  CPR latitude:  ({})", self.lat_cpr)?;
        writeln!(f, "  CPR longitude: ({})", self.lon_cpr)?;
        Ok(())
    }
}

impl Altitude {
    /// `decodeAC12Field`
    fn read<R: Read>(reader: &mut Reader<R>) -> Result<Option<u16>, DekuError> {
        let num =
            u32::from_reader_with_ctx(reader, (deku::ctx::Endian::Big, deku::ctx::BitSize(12)))?;

        let q = num & 0x10;

        if q > 0 {
            let n = ((num & 0x0fe0) >> 1) | (num & 0x000f);
            let n = n * 25;
            if n > 1000 {
                // TODO: maybe replace with Result->Option
                Ok(u16::try_from(n - 1000).ok())
            } else {
                Ok(None)
            }
        } else {
            let mut n = ((num & 0x0fc0) << 1) | (num & 0x003f);
            n = mode_ac::decode_id13_field(n);
            if let Ok(n) = mode_ac::mode_a_to_mode_c(n) {
                Ok(u16::try_from(n * 100).ok())
            } else {
                Ok(None)
            }
        }
    }
}

/// SPI Condition
#[derive(Debug, PartialEq, Eq, DekuRead, Copy, Clone)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[deku(id_type = "u8", bits = 2)]
pub enum SurveillanceStatus {
    NoCondition = 0,
    PermanentAlert = 1,
    TemporaryAlert = 2,
    SPICondition = 3,
}

impl Default for SurveillanceStatus {
    fn default() -> Self {
        Self::NoCondition
    }
}

/// Even / Odd
#[derive(Debug, PartialEq, Eq, DekuRead, Copy, Clone)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[deku(id_type = "u8", bits = 1)]
pub enum CPRFormat {
    Even = 0,
    Odd = 1,
}

impl Default for CPRFormat {
    fn default() -> Self {
        Self::Even
    }
}

impl fmt::Display for CPRFormat {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
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

/// Positive / Negative
#[derive(Debug, PartialEq, Eq, DekuRead, Copy, Clone)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[deku(id_type = "u8", bits = 1)]
pub enum Sign {
    Positive = 0,
    Negative = 1,
}

impl Sign {
    #[must_use]
    pub fn value(&self) -> i16 {
        match self {
            Self::Positive => 1,
            Self::Negative => -1,
        }
    }
}

impl fmt::Display for Sign {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
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

/// 13 bit identity code
#[derive(Debug, PartialEq, Eq, DekuRead, Copy, Clone)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct IdentityCode(#[deku(reader = "Self::read(deku::reader)")] pub u16);

impl IdentityCode {
    fn read<R: Read>(reader: &mut Reader<R>) -> result::Result<u16, DekuError> {
        let num =
            u32::from_reader_with_ctx(reader, (deku::ctx::Endian::Big, deku::ctx::BitSize(13)))?;

        let c1 = (num & 0b1_0000_0000_0000) >> 12;
        let a1 = (num & 0b0_1000_0000_0000) >> 11;
        let c2 = (num & 0b0_0100_0000_0000) >> 10;
        let a2 = (num & 0b0_0010_0000_0000) >> 9;
        let c4 = (num & 0b0_0001_0000_0000) >> 8;
        let a4 = (num & 0b0_0000_1000_0000) >> 7;
        let b1 = (num & 0b0_0000_0010_0000) >> 5;
        let d1 = (num & 0b0_0000_0001_0000) >> 4;
        let b2 = (num & 0b0_0000_0000_1000) >> 3;
        let d2 = (num & 0b0_0000_0000_0100) >> 2;
        let b4 = (num & 0b0_0000_0000_0010) >> 1;
        let d4 = num & 0b0_0000_0000_0001;

        let a = a4 << 2 | a2 << 1 | a1;
        let b = b4 << 2 | b2 << 1 | b1;
        let c = c4 << 2 | c2 << 1 | c1;
        let d = d4 << 2 | d2 << 1 | d1;

        let num: u16 = (a << 12 | b << 8 | c << 4 | d) as u16;
        Ok(num)
    }
}

/// ICAO Address; Mode S transponder code
#[derive(Debug, PartialEq, Eq, PartialOrd, DekuRead, Hash, Copy, Clone, Ord)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct ICAO(pub [u8; 3]);

impl fmt::Display for ICAO {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{:02x}", self.0[0])?;
        write!(f, "{:02x}", self.0[1])?;
        write!(f, "{:02x}", self.0[2])?;
        Ok(())
    }
}

impl core::str::FromStr for ICAO {
    type Err = core::num::ParseIntError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let num = u32::from_str_radix(s, 16)?;
        let bytes = num.to_be_bytes();
        let num = [bytes[1], bytes[2], bytes[3]];
        Ok(Self(num))
    }
}

/// Type of `DownlinkRequest`
#[derive(Debug, PartialEq, Eq, DekuRead, Copy, Clone)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[deku(id_type = "u8", bits = 5)]
pub enum DownlinkRequest {
    None = 0b00000,
    RequestSendCommB = 0b00001,
    CommBBroadcastMsg1 = 0b00100,
    CommBBroadcastMsg2 = 0b00101,
    #[deku(id_pat = "_")]
    Unknown,
}

/// Uplink / Downlink
#[derive(Debug, PartialEq, Eq, DekuRead, Copy, Clone)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[deku(id_type = "u8", bits = 1)]
pub enum KE {
    DownlinkELMTx = 0,
    UplinkELMAck = 1,
}

#[derive(Debug, PartialEq, Eq, DekuRead, Copy, Clone)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct UtilityMessage {
    #[deku(bits = 4)]
    pub iis: u8,
    pub ids: UtilityMessageType,
}

/// Message Type
#[derive(Debug, PartialEq, Eq, DekuRead, Copy, Clone)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[deku(id_type = "u8", bits = 2)]
pub enum UtilityMessageType {
    NoInformation = 0b00,
    CommB = 0b01,
    CommC = 0b10,
    CommD = 0b11,
}

/// Airborne / Ground and SPI
#[derive(Debug, PartialEq, Eq, DekuRead, Copy, Clone)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[deku(id_type = "u8", bits = 3)]
pub enum FlightStatus {
    NoAlertNoSPIAirborne = 0b000,
    NoAlertNoSPIOnGround = 0b001,
    AlertNoSPIAirborne = 0b010,
    AlertNoSPIOnGround = 0b011,
    AlertSPIAirborneGround = 0b100,
    NoAlertSPIAirborneGround = 0b101,
    Reserved = 0b110,
    NotAssigned = 0b111,
}

impl fmt::Display for FlightStatus {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "{}",
            match self {
                Self::NoAlertNoSPIAirborne
                | Self::AlertSPIAirborneGround
                | Self::NoAlertSPIAirborneGround => "airborne?",
                Self::NoAlertNoSPIOnGround => "ground?",
                Self::AlertNoSPIAirborne => "airborne",
                Self::AlertNoSPIOnGround => "ground",
                _ => "reserved",
            }
        )
    }
}

/// 13 bit encoded altitude
#[derive(Debug, PartialEq, Eq, DekuRead, Copy, Clone)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct AC13Field(#[deku(reader = "Self::read(deku::reader)")] pub u16);

impl AC13Field {
    // TODO Add unit
    fn read<R: Read>(reader: &mut Reader<R>) -> result::Result<u16, DekuError> {
        let num =
            u32::from_reader_with_ctx(reader, (deku::ctx::Endian::Big, deku::ctx::BitSize(13)))?;

        let m_bit = num & 0x0040;
        let q_bit = num & 0x0010;

        if m_bit != 0 {
            // TODO: read altitude when meter is selected
            Ok(0)
        } else if q_bit != 0 {
            let n = ((num & 0x1f80) >> 2) | ((num & 0x0020) >> 1) | (num & 0x000f);
            let n = n * 25;
            if n > 1000 {
                Ok((n - 1000) as u16)
            } else {
                Ok(0)
            }
        } else {
            // TODO 11 bit gillham coded altitude
            match mode_ac::mode_a_to_mode_c(mode_ac::decode_id13_field(num)) {
                Ok(n) => {
                    if n < -12 {
                        Err(DekuError::InvalidParam(Cow::from(
                            "Invalid altitude field: mode_a_to_mode_c invalid",
                        )))
                    } else {
                        Ok((n as u16).wrapping_mul(100))
                    }
                }
                Err(_e) => Ok(0),
            }
        }
    }
}

/// Transponder level and additional information (3.1.2.5.2.2.1)
#[derive(Debug, PartialEq, Eq, DekuRead, Copy, Clone)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[deku(id_type = "u8", bits = 3)]
#[allow(non_camel_case_types)]
pub enum Capability {
    /// Level 1 transponder (surveillance only), and either airborne or on the ground
    AG_UNCERTAIN = 0x00,
    #[deku(id_pat = "0x01..=0x03")]
    Reserved,
    /// Level 2 or above transponder, on ground
    AG_GROUND = 0x04,
    /// Level 2 or above transponder, airborne
    AG_AIRBORNE = 0x05,
    /// Level 2 or above transponder, either airborne or on ground
    AG_UNCERTAIN2 = 0x06,
    /// DR field is not equal to 0, or fs field equal 2, 3, 4, or 5, and either airborne or on
    /// ground
    AG_UNCERTAIN3 = 0x07,
}

impl fmt::Display for Capability {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "{}",
            match self {
                Self::AG_UNCERTAIN => "uncertain1",
                Self::Reserved => "reserved",
                Self::AG_GROUND => "ground",
                Self::AG_AIRBORNE => "airborne",
                Self::AG_UNCERTAIN2 => "uncertain2",
                Self::AG_UNCERTAIN3 => "airborne?",
            }
        )
    }
}

const CHAR_LOOKUP: &[u8; 64] = b"#ABCDEFGHIJKLMNOPQRSTUVWXYZ##### ###############0123456789######";

pub(crate) fn aircraft_identification_read<R: Read>(
    reader: &mut Reader<R>,
) -> Result<String, DekuError> {
    let mut chars = vec![];
    for _ in 0..=6 {
        let c = <u8>::from_reader_with_ctx(reader, deku::ctx::BitSize(6))?;
        if c != 32 {
            chars.push(c);
        }
    }
    let encoded = chars.into_iter().map(|b| CHAR_LOOKUP[b as usize] as char).collect::<String>();

    Ok(encoded)
}
