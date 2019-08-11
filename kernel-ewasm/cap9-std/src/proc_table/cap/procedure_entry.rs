use super::{AsCap,ProcedureKey,matching_keys};
use cap9_core::{Serialize, Deserialize};
use pwasm_abi::types::*;

#[cfg(feature="std")]
use rustc_hex::ToHex;

pub const CAP_PROC_ENTRY: u8 = 6;
pub const CAP_PROC_ENTRY_SIZE: u8 = 0;

#[derive(Clone, Debug, PartialEq)]
pub struct ProcedureEntryCap;

#[cfg(feature="std")]
impl std::fmt::Display for ProcedureEntryCap {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "ProcedureEntryCap")
    }
}

impl AsCap for ProcedureEntryCap {
    fn is_subset_of(&self, _parent_cap: &Self) -> bool {
        // All of these caps are identical, therefore any cap of this type is
        // the subset of another,
        true
    }
}

impl Deserialize<U256> for ProcedureEntryCap {
    type Error = cap9_core::Error;

    fn deserialize<R: cap9_core::Read<U256>>(_reader: &mut R) -> Result<Self, Self::Error> {
        Ok(ProcedureEntryCap {})
    }
}


impl Serialize<U256> for ProcedureEntryCap {
    type Error = cap9_core::Error;

    fn serialize<W: cap9_core::Write<U256>>(&self, _writer: &mut W) -> Result<(), Self::Error> {
        Ok(())
    }
}
