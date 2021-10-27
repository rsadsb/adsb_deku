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

/// To indicate common usage GICB services currently supported
#[derive(Debug, PartialEq, DekuRead, Clone)]
pub struct CommonUsageGICBCapabilityReport {
    /// 0,5
    #[deku(bits = "1")]
    extended_squitter_airborne_position: bool,

    /// 0,6
    #[deku(bits = "1")]
    extended_squitter_surface_position: bool,

    /// 0,7
    #[deku(bits = "1")]
    extended_squitter_status: bool,

    /// 0,8
    #[deku(bits = "1")]
    extended_squitter_identification_and_category: bool,

    /// 0,9
    #[deku(bits = "1")]
    extended_squitter_airborne_velocity_information: bool,

    /// 0,a
    #[deku(bits = "1")]
    extended_squitter_event_driven_information: bool,

    /// 2,0
    #[deku(bits = "1")]
    aircraft_identification: bool,

    /// 2,1
    #[deku(bits = "1")]
    aircraft_registration_number: bool,

    /// 4,0
    #[deku(bits = "1")]
    selected_vertical_intention: bool,

    /// 4,1
    #[deku(bits = "1")]
    next_waypoint_ident: bool,

    /// 4,2
    #[deku(bits = "1")]
    next_waypoint_position: bool,

    /// 4,3
    #[deku(bits = "1")]
    next_waypoint_information: bool,

    /// 4,4
    #[deku(bits = "1")]
    meteorological_routine_report: bool,

    /// 4,5
    #[deku(bits = "1")]
    meteorological_hazard_report: bool,

    /// 4,8
    #[deku(bits = "1")]
    vhf_channel_report: bool,

    /// 5,0
    #[deku(bits = "1")]
    track_and_turn_report: bool,

    /// 5,1
    #[deku(bits = "1")]
    position_coarse: bool,

    /// 5,2
    #[deku(bits = "1")]
    position_fine: bool,

    /// 5,3
    #[deku(bits = "1")]
    air_referenced_state_vector: bool,

    /// 5,4
    #[deku(bits = "1")]
    waypoint_1: bool,

    /// 5,5
    #[deku(bits = "1")]
    waypoint_2: bool,

    /// 5,6
    #[deku(bits = "1")]
    waypoint_3: bool,

    /// 5,f
    #[deku(bits = "1")]
    quasi_static_parameter_monitoring: bool,

    /// 6,0
    #[deku(bits = "1")]
    heading_and_speed_report: bool,

    #[deku(bits = "1")]
    reserved_for_aircraft_capability0: bool,

    #[deku(bits = "1")]
    reserved_for_aircraft_capability1: bool,

    /// E,1
    #[deku(bits = "1")]
    reserved_for_mode_s_bite: bool,

    /// E,2
    #[deku(bits = "1")]
    reserved_for_mode_s_bit: bool,

    /// F,1
    #[deku(bits = "1")]
    military_applications: bool,

    /// Reserved0
    #[deku(bits = "3")]
    reserved0: u8,

    /// Reserved1
    reserved1: [u8; 3],
}
