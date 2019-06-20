use super::{AsCap,ProcedureKey,matching_keys};
use cap9_core::{Serialize, Deserialize};
use pwasm_abi::types::*;
pub const CAP_PROC_REGISTER: u8 = 4;
pub const CAP_PROC_REGISTER_SIZE: u8 = 1;

#[derive(Clone, Debug, PartialEq)]
pub struct ProcedureRegisterCap {
    pub prefix: u8,
    pub key: ProcedureKey,
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
