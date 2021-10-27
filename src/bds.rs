use deku::bitvec::{BitSlice, Msb0};
use deku::prelude::*;

use crate::aircraft_identification_read;

#[derive(Debug, PartialEq, DekuRead, Clone)]
#[deku(type = "u8", bits = "8")]
pub enum BDS {
    /// Table A-2-32.
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
            Self::Unknown(s) => {
                writeln!(f, "Comm-B format: unknown format")?;
            },
        }
        Ok(())
    }
}
