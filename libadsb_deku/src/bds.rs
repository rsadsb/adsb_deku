//! B-Definition Subfield for Comm-B Messages

use alloc::format;
use alloc::string::String;
#[cfg(feature = "alloc")]
use core::{
    clone::Clone, cmp::PartialEq, fmt, fmt::Debug, prelude::rust_2021::derive, result::Result,
    result::Result::Ok, writeln,
};

use deku::prelude::*;

use crate::aircraft_identification_read;

#[derive(Debug, PartialEq, DekuRead, Clone)]
#[deku(type = "u8", bits = "8")]
pub enum BDS {
    /// (1, 0) Table A-2-16
    #[deku(id = "0x00")]
    Empty([u8; 6]),

    /// (1, 0) Table A-2-16
    #[deku(id = "0x10")]
    DataLinkCapability(DataLinkCapability),

    /// (2, 0) Table A-2-32
    #[deku(id = "0x20")]
    AircraftIdentification(#[deku(reader = "aircraft_identification_read(deku::rest)")] String),

    #[deku(id_pat = "_")]
    Unknown([u8; 6]),
}

impl fmt::Display for BDS {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Empty(_) => {
                writeln!(f, "Comm-B format: empty response")?;
            },
            Self::AircraftIdentification(s) => {
                writeln!(f, "Comm-B format: BDS2,0 Aircraft identification")?;
                writeln!(f, "  Ident:         {}", s)?;
            },
            Self::DataLinkCapability(_) => {
                writeln!(f, "Comm-B format: BDS1,0 Datalink capabilities")?;
            },
            Self::Unknown(_) => {
                writeln!(f, "Comm-B format: unknown format")?;
            },
        }
        Ok(())
    }
}

/// To report the data link capability of the Mode S transponder/data link installation
#[derive(Debug, PartialEq, DekuRead, Clone)]
pub struct DataLinkCapability {
    #[deku(bits = "1")]
    pub continuation_flag: bool,
    #[deku(bits = "5")]
    pub reserved0: u8,
    #[deku(bits = "1")]
    pub overlay_command_capability: bool,
    #[deku(bits = "1")]
    pub acas: bool,
    #[deku(bits = "7")]
    pub mode_s_subnetwork_version_number: u8,
    #[deku(bits = "1")]
    pub transponder_enhanced_protocol_indicator: bool,
    #[deku(bits = "1")]
    pub mode_s_specific_services_capability: bool,
    #[deku(bits = "3")]
    pub uplink_elm_average_throughput_capability: u8,
    #[deku(bits = "4")]
    pub downlink_elm: u8,
    #[deku(bits = "1")]
    pub aircraft_identification_capability: bool,
    #[deku(bits = "1")]
    pub squitter_capability_subfield: bool,
    #[deku(bits = "1")]
    pub surveillance_identifier_code: bool,
    #[deku(bits = "1")]
    pub common_usage_gicb_capability_report: bool,
    #[deku(bits = "4")]
    pub reserved_acas: u8,
    pub bit_array: u16,
}
