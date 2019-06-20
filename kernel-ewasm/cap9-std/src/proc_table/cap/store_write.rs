use super::AsCap;
use cap9_core::{Serialize, Deserialize};
use pwasm_abi::types::*;

pub const CAP_STORE_WRITE: u8 = 7;
pub const CAP_STORE_WRITE_SIZE: u8 = 2;

#[derive(Clone, Debug, PartialEq)]
pub struct StoreWriteCap {
    pub location: [u8; 32],
    pub size: [u8; 32],
}


impl AsCap for StoreWriteCap {
    fn is_subset_of(&self, parent_cap: &Self) -> bool {
        // Base storage address
        if U256::from_big_endian(&self.location) < U256::from_big_endian(&parent_cap.location) {
            return false;
        }
        // Number of additional storage keys
        if (U256::from_big_endian(&self.location) + U256::from_big_endian(&self.size)) > (U256::from_big_endian(&parent_cap.location) + U256::from_big_endian(&parent_cap.size)) {
            return false;
        }
        true
    }
}

impl Deserialize<U256> for StoreWriteCap {
    type Error = cap9_core::Error;

    fn deserialize<R: cap9_core::Read<U256>>(reader: &mut R) -> Result<Self, Self::Error> {
        let mut buf = [U256::zero(); 2];
        reader.read(&mut buf).unwrap();
        let location: [u8; 32] = buf[0].into();
        let size: [u8; 32] = buf[1].into();

        Ok(StoreWriteCap {
            location: location,
            size: size,
        })
    }
}

impl Serialize<U256> for StoreWriteCap {
    type Error = cap9_core::Error;

    fn serialize<W: cap9_core::Write<U256>>(self, writer: &mut W) -> Result<(), Self::Error> {
        writer.write(&[U256::from(self.location), U256::from(self.size)])?;
        Ok(())
    }
}
