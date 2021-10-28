/*!
`adsb_deku` provides decoding for the [`ADS-B`] Downlink protocol by using the [`deku`] crate.

## Downlink Format support
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

# Comm-B support
|  BDS  |  Name                                   |  Table      |
| ----  | --------------------------------------- | ----------- |
| (0,0) | [`Empty`]                               |             |
| (1,0) | [`Data Link Capability`]                | A-2-16      |
| (2,0) | [`Aircraft Identification`]             | A-2-32      |

# Example
To begin using `adsb_deku`, import the [`Frame`] struct as well as the trait [`deku::DekuContainerRead`].
This trait is re-exported for your convenience. [`Frame::from_bytes()`] provides the interface for decoding bytes
into adsb data.

```rust
use hexlit::hex;
use adsb_deku::Frame;
use adsb_deku::deku::DekuContainerRead;

let bytes = hex!("8da2c1bd587ba2adb31799cb802b");
let frame = Frame::from_bytes((&bytes, 0)).unwrap().1;
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

[`Empty`]: crate::bds::BDS::Empty
[`Data Link Capability`]: crate::bds::BDS::DataLinkCapability
[`Aircraft Identification`]: crate::bds::BDS::AircraftIdentification
[`DF`]: crate::DF
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
[`apps`]: crate::Frame
[`Frame`]: crate::Frame
[`deku::DekuContainerRead`]: crate::deku::DekuContainerRead
[`README.md`]: https://github.com/wcampbell0x2a/adsb_deku/blob/master/README.md
[`apps/`]: https://github.com/wcampbell0x2a/adsb_deku/tree/master/apps
[`ADS-B`]: https://en.wikipedia.org/wiki/Automatic_Dependent_Surveillance%E2%80%93Broadcast
[`deku`]: https://github.com/sharksforarms/deku
!*/

// good reference: http://www.anteni.net/adsb/Doc/1090-WP30-18-DRAFT_DO-260B-V42.pdf
//
// Maybe always reference this in the future?

/// re-export deku
pub use deku;

pub mod adsb;
pub mod bds;
pub mod cpr;
mod crc;

use adsb::{ControlField, ADSB};
use bds::BDS;
use deku::bitvec::{BitSlice, Msb0};
use deku::prelude::*;

/// Downlink ADS-B Packet
#[derive(Debug, PartialEq, DekuRead, Clone)]
pub struct Frame {
    /// 5 bit identifier
    pub df: DF,
    /// Calculated from all bits, used as ICAO for Response packets
    #[deku(reader = "Self::read_crc(df, deku::input_bits)")]
    pub crc: u32,
}

impl Frame {
    /// Read rest as CRC bits
    fn read_crc<'a, 'b>(
        df: &'a DF,
        rest: &'b BitSlice<Msb0, u8>,
    ) -> Result<(&'b BitSlice<Msb0, u8>, u32), DekuError> {
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

        let crc = crc::modes_checksum(rest.as_raw_slice(), bit_len)?;
        Ok((rest, crc))
    }
}

impl std::fmt::Display for Frame {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match &self.df {
            DF::ShortAirAirSurveillance { altitude, .. } => {
                writeln!(f, " Short Air-Air Surveillance")?;
                writeln!(f, "  ICAO Address:  {:06x} (Mode S / ADS-B)", self.crc)?;
                if altitude.0 > 0 {
                    writeln!(f, "  Air/Ground:    airborne?")?;
                    writeln!(f, "  Altitude:      {} ft barometric", altitude.0)?;
                } else {
                    writeln!(f, "  Air/Ground:    ground")?;
                }
            },
            DF::SurveillanceAltitudeReply { fs, ac, .. } => {
                writeln!(f, " Surveillance, Altitude Reply")?;
                writeln!(f, "  ICAO Address:  {:06x} (Mode S / ADS-B)", self.crc)?;
                writeln!(f, "  Air/Ground:    {}", fs)?;
                if ac.0 > 0 {
                    writeln!(f, "  Altitude:      {} ft barometric", ac.0)?;
                }
            },
            DF::SurveillanceIdentityReply { fs, id, .. } => {
                writeln!(f, " Surveillance, Identity Reply")?;
                writeln!(f, "  ICAO Address:  {:06x} (Mode S / ADS-B)", self.crc)?;
                writeln!(f, "  Air/Ground:    {}", fs)?;
                writeln!(f, "  Identity:      {:04x}", id.0)?;
            },
            DF::AllCallReply {
                capability, icao, ..
            } => {
                writeln!(f, " All Call Reply")?;
                writeln!(f, "  ICAO Address:  {} (Mode S / ADS-B)", icao)?;
                writeln!(f, "  Air/Ground:    {}", capability)?;
            },
            DF::LongAirAir { altitude, .. } => {
                writeln!(f, " Long Air-Air ACAS")?;
                writeln!(f, "  ICAO Address:  {:06x} (Mode S / ADS-B)", self.crc)?;
                // TODO the airborne? should't be static
                if altitude.0 > 0 {
                    writeln!(f, "  Air/Ground:    airborne?")?;
                    writeln!(f, "  Baro altitude: {} ft", altitude.0)?;
                } else {
                    writeln!(f, "  Air/Ground:    ground")?;
                }
            },
            DF::ADSB(adsb) => {
                write!(f, "{}", adsb.to_string("(Mode S / ADS-B)").unwrap())?;
            },
            DF::TisB { cf, .. } => {
                write!(f, "{}", cf)?;
            },
            // TODO
            DF::ExtendedQuitterMilitaryApplication { .. } => {},
            DF::CommBAltitudeReply { bds, alt, .. } => {
                writeln!(f, " Comm-B, Altitude Reply")?;
                writeln!(f, "  ICAO Address:  {:x?} (Mode S / ADS-B)", self.crc)?;
                writeln!(f, "  Altitude:      {} ft", alt.0)?;
                write!(f, "  {}", bds)?;
            },
            DF::CommBIdentityReply { id, bds, .. } => {
                writeln!(f, " Comm-B, Identity Reply")?;
                writeln!(f, "    ICAO Address:  {:x?} (Mode S / ADS-B)", self.crc)?;
                writeln!(f, "    Squawk:        {:x?}", id)?;
                write!(f, "    {}", bds)?;
            },
            DF::CommDExtendedLengthMessage { .. } => {
                writeln!(f, " Comm-D Extended Length Message")?;
                writeln!(f, "    ICAO Address:     {:x?} (Mode S / ADS-B)", self.crc)?;
            },
        }
        Ok(())
    }
}

/// Downlink Format (3.1.2.3.2.1.2)
#[derive(Debug, PartialEq, DekuRead, Clone)]
#[deku(type = "u8", bits = "5")]
pub enum DF {
    /// 0: (Mode S) Short Air-Air Surveillance, Downlink Format 0 (3.1.2.8.2)
    #[deku(id = "0")]
    ShortAirAirSurveillance {
        /// VS: Vertical Status
        #[deku(bits = "1")]
        vs: u8,
        /// CC:
        #[deku(bits = "1")]
        cc: u8,
        /// Spare
        #[deku(bits = "1")]
        unused: u8,
        /// SL: Sensitivity level, ACAS
        #[deku(bits = "3")]
        sl: u8,
        /// Spare
        #[deku(bits = "2")]
        unused1: u8,
        /// RI: Reply Information
        #[deku(bits = "4")]
        ri: u8,
        /// Spare
        #[deku(bits = "2")]
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

    /// 16: (Mode S) Long Air-Air Surveillance Downlink Format 16 (3.1.2.8.3)
    #[deku(id = "16")]
    LongAirAir {
        #[deku(bits = "1")]
        vs: u8,
        #[deku(bits = "2")]
        spare1: u8,
        #[deku(bits = "3")]
        sl: u8,
        #[deku(bits = "2")]
        spare2: u8,
        #[deku(bits = "4")]
        ri: u8,
        #[deku(bits = "2")]
        spare3: u8,
        /// AC: altitude code
        altitude: AC13Field,
        /// MV: message, acas
        #[deku(count = "7")]
        mv: Vec<u8>,
        /// AP: address, parity
        parity: ICAO,
    },

    /// 17: Extended Squitter, Downlink Format 17 (3.1.2.8.6)
    ///
    /// Civil aircraft ADS-B message
    #[deku(id = "17")]
    ADSB(ADSB),

    /// 18: Extended Squitter/Supplementary, Downlink Format 18 (3.1.2.8.7)
    ///
    /// Non-Transponder-based ADS-B Transmitting Subsystems and TIS-B Transmitting equipment.
    /// Equipment that cannot be interrogated.
    ///
    /// reference: Aeronautical Telecommunications Volume IV: Surveillance and
    /// Collision Avoidance Systems, Fifth Edition
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
        #[deku(bits = "3")]
        af: u8,
    },

    /// 20: COMM-B Altitude Reply (3.1.2.6.6)
    ///
    /// TODO: Test me
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
            bits = "13",
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
        /// Spare - 1 bit
        #[deku(bits = "1")]
        spare: u8,
        /// KE: control, ELM
        ke: KE,
        /// ND: number of D-segment
        #[deku(bits = "4")]
        nd: u8,
        /// MD: message, Comm-D, 80 bits
        #[deku(count = "10")]
        md: Vec<u8>,
        /// AP: address/parity
        parity: ICAO,
    },
}

/// [`DF::CommBAltitudeReply`] || ([`DF::ADSB`] && ([`adsb::ME::AirbornePositionBaroAltitude`] || [`adsb::ME::AirbornePositionGNSSAltitude`])
#[derive(Debug, PartialEq, DekuRead, Default, Copy, Clone)]
pub struct Altitude {
    #[deku(bits = "5")]
    pub tc: u8,
    pub ss: SurveillanceStatus,
    #[deku(bits = "1")]
    pub saf_or_imf: u8,
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
        writeln!(f, "  Altitude:      {} ft barometric", self.alt)?;
        writeln!(f, "  CPR type:      Airborne")?;
        writeln!(f, "  CPR odd flag:  {}", self.odd_flag)?;
        writeln!(f, "  CPR latitude:  ({})", self.lat_cpr)?;
        writeln!(f, "  CPR longitude: ({})", self.lon_cpr)?;
        Ok(())
    }
}

impl Altitude {
    /// decodeAC12Field
    fn read(rest: &BitSlice<Msb0, u8>) -> Result<(&BitSlice<Msb0, u8>, u32), DekuError> {
        let (rest, num) = u32::read(rest, (deku::ctx::Endian::Big, deku::ctx::Size::Bits(12)))?;

        let q = num & 0x10;

        if q > 0 {
            let n = ((num & 0x0fe0) >> 1) | (num & 0x000f);
            let n = n * 25;
            if n > 1000 {
                Ok((rest, (n - 1000) as u32))
            } else {
                Ok((rest, 0))
            }
        } else {
            let mut n = ((num & 0x0fc0) << 1) | (num & 0x003f);
            n = mode_ac::decode_id13_field(n);
            if let Ok(n) = mode_ac::mode_a_to_mode_c(n) {
                Ok((rest, ((n as u32) * 100)))
            } else {
                println!("error");
                Ok((rest, (0)))
            }
        }
    }
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

/// Even / Odd
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

/// Positive / Negative
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

mod mode_ac {
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
        if (mode_a & 0xffff_8889) != 0 || (mode_a & 0x0000_00f0) == 0 {
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
            five_hundreds ^= 0x0ff;
        } // D2
        if mode_a & 0x0004 != 0 {
            five_hundreds ^= 0x07f;
        } // D4

        if mode_a & 0x1000 != 0 {
            five_hundreds ^= 0x03f;
        } // A1
        if mode_a & 0x2000 != 0 {
            five_hundreds ^= 0x01f;
        } // A2
        if mode_a & 0x4000 != 0 {
            five_hundreds ^= 0x00f;
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

        let n = (five_hundreds * 5) + one_hundreds;
        if n >= 13 {
            Ok(n - 13)
        } else {
            Err("Invalid altitude")
        }
    }
}

/// 13 bit identity code
#[derive(Debug, PartialEq, DekuRead, Copy, Clone)]
pub struct IdentityCode(#[deku(reader = "Self::read(deku::rest)")] pub u16);

impl IdentityCode {
    fn read(rest: &BitSlice<Msb0, u8>) -> Result<(&BitSlice<Msb0, u8>, u16), DekuError> {
        let (rest, num) = u32::read(rest, (deku::ctx::Endian::Big, deku::ctx::Size::Bits(13)))?;

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
        Ok((rest, num))
    }
}

/// ICAO Address; Mode S transponder code
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

/// Type of DownlinkRequest
#[derive(Debug, PartialEq, DekuRead, Copy, Clone)]
#[deku(type = "u8", bits = "5")]
pub enum DownlinkRequest {
    None               = 0b00000,
    RequestSendCommB   = 0b00001,
    CommBBroadcastMsg1 = 0b00100,
    CommBBroadcastMsg2 = 0b00101,
    #[deku(id_pat = "_")]
    Unknown,
}

/// Uplink / Downlink
#[derive(Debug, PartialEq, DekuRead, Copy, Clone)]
#[deku(type = "u8", bits = "1")]
pub enum KE {
    DownlinkELMTx = 0,
    UplinkELMAck  = 1,
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

/// Airborne / Ground and SPI
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

/// 13 bit encoded altitude
#[derive(Debug, PartialEq, DekuRead, Copy, Clone)]
pub struct AC13Field(#[deku(reader = "Self::read(deku::rest)")] pub u32);

impl AC13Field {
    /// TODO Add unit
    fn read(rest: &BitSlice<Msb0, u8>) -> Result<(&BitSlice<Msb0, u8>, u32), DekuError> {
        let (rest, num) = u32::read(rest, (deku::ctx::Endian::Big, deku::ctx::Size::Bits(13)))?;

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

/// Transponder level and additional information (3.1.2.5.2.2.1)
#[derive(Debug, PartialEq, DekuRead, Copy, Clone)]
#[deku(type = "u8", bits = "3")]
#[allow(non_camel_case_types)]
pub enum Capability {
    /// Level 1 transponder (surveillance only), and either airborne or on the ground
    AG_UNCERTAIN  = 0x00,
    #[deku(id_pat = "0x01..=0x03")]
    Reserved,
    /// Level 2 or above transponder, on ground
    AG_GROUND     = 0x04,
    /// Level 2 or above transponder, airborne
    AG_AIRBORNE   = 0x05,
    /// Level 2 or above transponder, either airborne or on ground
    AG_UNCERTAIN2 = 0x06,
    /// DR field is not equal to 0, or fs field equal 2, 3, 4, or 5, and either airborne or on
    /// ground
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

const CHAR_LOOKUP: &[u8; 64] = b"#ABCDEFGHIJKLMNOPQRSTUVWXYZ##### ###############0123456789######";

pub(crate) fn aircraft_identification_read(
    rest: &BitSlice<Msb0, u8>,
) -> Result<(&BitSlice<Msb0, u8>, String), DekuError> {
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
