use deku::bitvec::{BitSlice, Msb0};
use deku::prelude::*;

use crate::aircraft_identification_read;

#[derive(Debug, PartialEq, DekuRead, Clone)]
#[deku(type = "u8", bits = "8")]
pub enum BDS {
    /// (1, 0) Table A-2-16
    #[deku(id = "0x10")]
    DataLinkCapability(DataLinkCapability),

    /// (2, 0) Table A-2-32
    #[deku(id = "0x20")]
    AircraftIdentification(#[deku(reader = "aircraft_identification_read(deku::rest)")] String),

    #[deku(id_pat = "_")]
    Unknown([u8; 6]),
}

impl std::fmt::Display for BDS {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::AircraftIdentification(s) => {
                writeln!(f, "Comm-B format: BDS2,0 Aircraft identification")?;
                writeln!(f, "  Ident:         {}", s)?;
            },
            Self::DataLinkCapability(data_link_capability) => {
                writeln!(f, "Comm-B format: BDS1,0 Datalink capabilities")?;
            },
            Self::Unknown(s) => {
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
    continuation_flag: bool,
    #[deku(bits = "5")]
    reserved0: u8,
    #[deku(bits = "1")]
    overlay_command_capability: bool,
    #[deku(bits = "1")]
    acas: bool,
    #[deku(bits = "7")]
    mode_s_subnetwork_version_number: u8,
    #[deku(bits = "1")]
    transponder_enhanced_protocol_indicator: bool,
    #[deku(bits = "1")]
    mode_s_specific_services_capability: bool,
    #[deku(bits = "3")]
    uplink_elm_average_throughput_capability: u8,
    #[deku(bits = "4")]
    downlink_elm: u8,
    #[deku(bits = "1")]
    aircraft_identification_capability: bool,
    #[deku(bits = "1")]
    squitter_capability_subfield: bool,
    #[deku(bits = "1")]
    surveillance_identifier_code: bool,
    #[deku(bits = "1")]
    common_usage_gicb_capability_report: bool,
    #[deku(bits = "4")]
    reserved_acas: u8,
    bit_array: u16,
}
