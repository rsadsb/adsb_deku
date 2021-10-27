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

    /// (1, 7) Table A-2-23
    #[deku(id = "0x17")]
    CommonUsageGICBCapabilityReport(CommonUsageGICBCapabilityReport),

    /// (2, 0) Table A-2-32
    #[deku(id = "0x20")]
    AircraftIdentification(#[deku(reader = "aircraft_identification_read(deku::rest)")] String),

    #[deku(id_pat = "_")]
    Unknown([u8; 6]),
}

impl std::fmt::Display for BDS {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
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
            Self::CommonUsageGICBCapabilityReport(_) => {
                writeln!(
                    f,
                    "Comm-B format: BDS1,7 Common Usage GICB Capability Report"
                )?;
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

/// To indicate common usage GICB services currently supported
#[derive(Debug, PartialEq, DekuRead, Clone)]
pub struct CommonUsageGICBCapabilityReport {
    /// 0,5
    #[deku(bits = "1")]
    pub extended_squitter_airborne_position: bool,

    /// 0,6
    #[deku(bits = "1")]
    pub extended_squitter_surface_position: bool,

    /// 0,7
    #[deku(bits = "1")]
    pub extended_squitter_status: bool,

    /// 0,8
    #[deku(bits = "1")]
    pub extended_squitter_identification_and_category: bool,

    /// 0,9
    #[deku(bits = "1")]
    pub extended_squitter_airborne_velocity_information: bool,

    /// 0,a
    #[deku(bits = "1")]
    pub extended_squitter_event_driven_information: bool,

    /// 2,0
    #[deku(bits = "1")]
    pub aircraft_identification: bool,

    /// 2,1
    #[deku(bits = "1")]
    pub aircraft_registration_number: bool,

    /// 4,0
    #[deku(bits = "1")]
    pub selected_vertical_intention: bool,

    /// 4,1
    #[deku(bits = "1")]
    pub next_waypoint_ident: bool,

    /// 4,2
    #[deku(bits = "1")]
    pub next_waypoint_position: bool,

    /// 4,3
    #[deku(bits = "1")]
    pub next_waypoint_information: bool,

    /// 4,4
    #[deku(bits = "1")]
    pub meteorological_routine_report: bool,

    /// 4,5
    #[deku(bits = "1")]
    pub meteorological_hazard_report: bool,

    /// 4,8
    #[deku(bits = "1")]
    pub vhf_channel_report: bool,

    /// 5,0
    #[deku(bits = "1")]
    pub track_and_turn_report: bool,

    /// 5,1
    #[deku(bits = "1")]
    pub position_coarse: bool,

    /// 5,2
    #[deku(bits = "1")]
    pub position_fine: bool,

    /// 5,3
    #[deku(bits = "1")]
    pub air_referenced_state_vector: bool,

    /// 5,4
    #[deku(bits = "1")]
    pub waypoint_1: bool,

    /// 5,5
    #[deku(bits = "1")]
    pub waypoint_2: bool,

    /// 5,6
    #[deku(bits = "1")]
    pub waypoint_3: bool,

    /// 5,f
    #[deku(bits = "1")]
    pub quasi_static_parameter_monitoring: bool,

    /// 6,0
    #[deku(bits = "1")]
    pub heading_and_speed_report: bool,

    #[deku(bits = "1")]
    pub reserved_for_aircraft_capability0: bool,

    #[deku(bits = "1")]
    pub reserved_for_aircraft_capability1: bool,

    /// E,1
    #[deku(bits = "1")]
    pub reserved_for_mode_s_bite: bool,

    /// E,2
    #[deku(bits = "1")]
    pub reserved_for_mode_s_bit: bool,

    /// F,1
    #[deku(bits = "1")]
    pub military_applications: bool,

    /// Reserved0
    #[deku(bits = "3")]
    pub reserved0: u8,

    /// Reserved1
    pub reserved1: [u8; 3],
}
