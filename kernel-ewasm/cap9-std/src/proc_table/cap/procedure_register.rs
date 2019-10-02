use super::{AsCap,ProcedureKey,matching_keys};
use cap9_core::{Serialize, Deserialize};
use pwasm_abi::types::*;

#[cfg(feature="std")]
use rustc_hex::ToHex;

pub const CAP_PROC_REGISTER: u8 = 4;
pub const CAP_PROC_REGISTER_SIZE: u8 = 1;

#[derive(Clone, Debug, PartialEq)]
pub struct ProcedureRegisterCap {
    pub prefix: u8,
    pub key: ProcedureKey,
}

#[cfg(feature="std")]
impl std::fmt::Display for ProcedureRegisterCap {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        let key_hex: String = self.key.to_hex();
        write!(f, "ProcedureRegisterCap: prefix: {}, key: 0x{}", self.prefix, key_hex)
    }
}

impl AsCap for ProcedureRegisterCap {
    fn is_subset_of(&self, parent_cap: &Self) -> bool {
        // Check that the prefix of B is >= than the prefix of A.
        if parent_cap.prefix > self.prefix {
            return false;
        }
        // The keys must match
        matching_keys(parent_cap.prefix, &parent_cap.key, &self.key)
    }
}

impl Deserialize<U256> for ProcedureRegisterCap {
    type Error = cap9_core::Error;

    fn deserialize<R: cap9_core::Read<U256>>(reader: &mut R) -> Result<Self, Self::Error> {
        let mut buf = [U256::zero(); 1];
        reader.read(&mut buf).unwrap();
        let val: U256 = buf[0];
        let mut key = [0u8; 24];
        key.copy_from_slice(&<[u8; 32]>::from(val)[8..]);

        Ok(ProcedureRegisterCap {
            prefix: val.byte(31),
            key: key,
        })
    }
}

impl Serialize<U256> for ProcedureRegisterCap {
    type Error = cap9_core::Error;

    fn serialize<W: cap9_core::Write<U256>>(&self, writer: &mut W) -> Result<(), Self::Error> {
        let mut res = [0u8; 32];
        res[0] = self.prefix;
        res[8..].copy_from_slice(&self.key);
        writer.write(&[res.into()])?;
        Ok(())
    }
}
