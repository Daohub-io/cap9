extern crate pwasm_abi;
extern crate pwasm_ethereum;
extern crate pwasm_std;

use cap9_core;
use cap9_core::{Cursor, Serialize, Deserialize, Write};

use pwasm_abi::eth;
use pwasm_abi::types::*;

use core::convert::TryFrom;

mod store_write;
pub use store_write::*;

mod procedure_call;
pub use procedure_call::*;

mod procedure_register;
pub use procedure_register::*;

mod procedure_delete;
pub use procedure_delete::*;

mod procedure_entry;
pub use procedure_entry::*;

mod log;
pub use log::*;

mod account_call;
pub use account_call::*;

/// A list of the cap types which we can use for iterating over all cap types.
pub const CAP_TYPES: [u8; 7] = [
    CAP_PROC_CALL,
    CAP_PROC_REGISTER,
    CAP_PROC_DELETE,
    CAP_PROC_ENTRY,
    CAP_STORE_WRITE,
    CAP_LOG,
    CAP_ACC_CALL
];

type ProcedureKey = [u8; 24];
type ProcedureIndex = [u8; 24];

#[derive(Clone, Debug, PartialEq)]
pub enum Capability {
    ProcedureCall(ProcedureCallCap),
    ProcedureRegister(ProcedureRegisterCap),
    ProcedureDelete(ProcedureDeleteCap),
    ProcedureEntry(ProcedureEntryCap),
    StoreWrite(StoreWriteCap),
    Log(LogCap),
    AccountCall(AccountCallCap),
}

impl Capability {
    #[inline]
    pub fn get_cap_size(&self) -> u8 {
        match self {
            Capability::ProcedureCall(_) => CAP_PROC_CALL_SIZE,
            Capability::ProcedureRegister(_) => CAP_PROC_REGISTER_SIZE,
            Capability::ProcedureDelete(_) => CAP_PROC_DELETE_SIZE,
            Capability::ProcedureEntry(_) => CAP_PROC_ENTRY_SIZE,
            Capability::StoreWrite(_) => CAP_STORE_WRITE_SIZE,
            Capability::Log(_) => CAP_LOG_SIZE,
            Capability::AccountCall(_) => CAP_ACC_CALL_SIZE,
        }
    }

    pub fn cap_type(&self) -> u8 {
        match self {
            Capability::ProcedureCall(_) => CAP_PROC_CALL,
            Capability::ProcedureRegister(_) => CAP_PROC_REGISTER,
            Capability::ProcedureDelete(_) => CAP_PROC_DELETE,
            Capability::ProcedureEntry(_) => CAP_PROC_ENTRY,
            Capability::StoreWrite(_) => CAP_STORE_WRITE,
            Capability::Log(_) => CAP_LOG,
            Capability::AccountCall(_) => CAP_ACC_CALL,
        }
    }

    pub fn is_subset_of(&self, parent_cap: &Capability) -> bool {
        match (self, parent_cap) {
            (Capability::ProcedureCall(cap),Capability::ProcedureCall(parent)) => cap.is_subset_of(parent),
            (Capability::ProcedureCall(_),_) => false,

            (Capability::StoreWrite(cap),Capability::StoreWrite(parent)) => cap.is_subset_of(parent),
            (Capability::StoreWrite(_),_) => false,

            (Capability::ProcedureRegister(cap),Capability::ProcedureRegister(parent)) => cap.is_subset_of(parent),
            (Capability::ProcedureRegister(_),_) => false,

            (Capability::ProcedureDelete(cap),Capability::ProcedureDelete(parent)) => cap.is_subset_of(parent),
            (Capability::ProcedureDelete(_),_) => false,

            (Capability::ProcedureEntry(cap),Capability::ProcedureEntry(parent)) => cap.is_subset_of(parent),
            (Capability::ProcedureEntry(_),_) => false,

            (Capability::Log(cap),Capability::Log(parent)) => cap.is_subset_of(parent),
            (Capability::Log(_),_) => false,

            (Capability::AccountCall(cap),Capability::AccountCall(parent)) => cap.is_subset_of(parent),
            (Capability::AccountCall(_),_) => false,
        }
    }
}

pub trait AsCap {
    fn is_subset_of(&self, parent_cap: &Self) -> bool;
}

pub fn matching_keys(prefix: u8, required_key: &ProcedureKey, requested_key: &ProcedureKey) -> bool {
    // We only want to keep the first $prefix bits of $key, the
    // rest should be zero. We then XOR this value with the
    // requested proc id and the value should be zero. TODO:
    // consider using the unstable BitVec type. For now we will
    // just a u128 and a u64.
    let mut mask_a_array = [0;16];
    mask_a_array.copy_from_slice(&required_key[0..16]);
    let mut mask_b_array = [0;8];
    mask_b_array.copy_from_slice(&required_key[16..24]);

    let shift_amt: u32 = 192_u8.checked_sub(prefix).unwrap_or(0) as u32;

    let prefix_mask_a: u128 = u128::max_value().checked_shl(shift_amt.checked_sub(64).unwrap_or(0)).unwrap_or(0);
    let prefix_mask_b:u64 = u64::max_value().checked_shl(shift_amt).unwrap_or(0);

    // mask_a + mask_b is the key we are allowed
    let mask_a: u128 = u128::from_be_bytes(mask_a_array) & prefix_mask_a;
    let mask_b: u64 = u64::from_be_bytes(mask_b_array) & prefix_mask_b;

    // This is the key we are requesting but cleared
    let mut req_a_array = [0;16];
    req_a_array.copy_from_slice(&requested_key[0..16]);
    let mut req_b_array = [0;8];
    req_b_array.copy_from_slice(&requested_key[16..24]);
    let req_a: u128 = u128::from_be_bytes(req_a_array) & prefix_mask_a;
    let req_b: u64 = u64::from_be_bytes(req_b_array) & prefix_mask_b;

    return (req_a == mask_a) && (req_b == mask_b);
}

#[derive(Clone, Debug)]
pub enum CapDecodeErr {
    InvalidCapType(u8),
    InvalidCapLen(u8),
}

impl Serialize<U256> for Capability {
    type Error = cap9_core::Error;

    fn serialize<W: cap9_core::Write<U256>>(self, writer: &mut W) -> Result<(), Self::Error> {
        // TODO: replace all these identical match arms with something like .inner()
        match self {
            Capability::ProcedureCall(cap) => cap.serialize(writer)?,
            Capability::ProcedureRegister(cap) => cap.serialize(writer)?,
            Capability::ProcedureDelete(cap) => cap.serialize(writer)?,
            Capability::ProcedureEntry(cap) => cap.serialize(writer)?,
            Capability::StoreWrite(cap) => cap.serialize(writer)?,
            Capability::Log(cap) => cap.serialize(writer)?,
            Capability::AccountCall(cap) => cap.serialize(writer)?,
        }
        Ok(())
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct NewCapability {
    pub cap: Capability,
    pub parent_index: u8,
}

#[derive(Clone, Debug, PartialEq)]
pub struct NewCapList(pub Vec<NewCapability>);

impl NewCapList {
    /// Create Empty CapList
    pub fn empty() -> NewCapList {
        NewCapList(Vec::new())
    }

    pub fn inner(self) -> Vec<NewCapability> {
        self.0
    }
}

impl NewCapList {
    // TODO: add error handling
    pub fn to_u256_list(&self) -> Vec<U256> {
        let mut res: Vec<U256> = Vec::with_capacity(self.0.len() * (CAP_LOG_SIZE + 3) as usize);
        self.clone().serialize(&mut res).unwrap();
        res
    }

    pub fn from_u256_list(list: &[U256]) -> Result<Self, CapDecodeErr> {
        let mut cursor = Cursor::new(list);
        NewCapList::deserialize(&mut cursor).map_err(|_| CapDecodeErr::InvalidCapType(0))
    }
}


impl Serialize<U256> for NewCapList {
    type Error = cap9_core::Error;

    fn serialize<W: cap9_core::Write<U256>>(self, writer: &mut W) -> Result<(), Self::Error> {
        // TODO: figure out whether move/clone is the right choice.
        for new_cap in self.0.iter() {
            let cap_size = U256::from(new_cap.cap.get_cap_size() + 3);
            writer.write(&[cap_size])?;
            let cap_type = U256::from(new_cap.cap.cap_type());
            writer.write(&[cap_type])?;
            let parent_index = U256::from(new_cap.parent_index);
            writer.write(&[parent_index])?;
            new_cap.cap.clone().serialize(writer)?;
        }

        Ok(())
    }
}


impl Deserialize<U256> for NewCapList {
    type Error = cap9_core::Error;

    fn deserialize<R: cap9_core::Read<U256>>(reader: &mut R) -> Result<Self, Self::Error> {
        let mut result = Vec::new();
        while reader.remaining() > 0 {
            // Check List Length
            if reader.remaining() < 3 {
                return Err(cap9_core::Error::InvalidData);
            }
            // Get Values
            let cap_size = u8::deserialize(reader)?;
            let cap_type = u8::deserialize(reader)?;
            let parent_index = u8::deserialize(reader)?;


            let inner_cap_size = cap_size.checked_sub(3).ok_or(cap9_core::Error::InvalidData)?;

            // Check Cap Size
            if reader.remaining() < inner_cap_size as usize {
                return Err(cap9_core::Error::InvalidData);
            }

            let cap: Capability = match (cap_type, inner_cap_size) {
                (CAP_PROC_CALL, CAP_PROC_CALL_SIZE) =>
                    Capability::ProcedureCall(ProcedureCallCap::deserialize(reader)?),
                (CAP_PROC_REGISTER, CAP_PROC_REGISTER_SIZE) =>
                    Capability::ProcedureRegister(ProcedureRegisterCap::deserialize(reader)?),
                (CAP_PROC_DELETE, CAP_PROC_DELETE_SIZE) =>
                    Capability::ProcedureDelete(ProcedureDeleteCap::deserialize(reader)?),
                (CAP_PROC_ENTRY, CAP_PROC_ENTRY_SIZE) =>
                    Capability::ProcedureEntry(ProcedureEntryCap),
                (CAP_STORE_WRITE, CAP_STORE_WRITE_SIZE) =>
                    Capability::StoreWrite(StoreWriteCap::deserialize(reader)?),
                (CAP_LOG, CAP_LOG_SIZE) =>
                    Capability::Log(LogCap::deserialize(reader)?),
                (CAP_ACC_CALL, CAP_ACC_CALL_SIZE) =>
                    Capability::AccountCall(AccountCallCap::deserialize(reader)?),
                _ => return Err(cap9_core::Error::InvalidData),
            };
            let new_cap = NewCapability {
                cap,
                parent_index,
            };
            result.push(new_cap);
        }
        Ok(NewCapList(result))
    }
}


#[cfg(test)]
#[allow(non_snake_case)]
mod tests {
    extern crate pwasm_test;

    use self::pwasm_test::{ext_get, ext_reset};
    use core::str::FromStr;
    use pwasm_abi::types::*;

    use super::*;

    use pwasm_abi::eth;
    use pwasm_abi::eth::AbiType;


    #[test]
    fn should_encode_empty_cap_list() {
        let EMPTY_LIST = Vec::new();
        let result = NewCapList(EMPTY_LIST).to_u256_list();

        assert_eq!(result.len(), 0);
    }

    #[test]
    fn should_decode_empty_cap_list() {
        let result = NewCapList::from_u256_list(&Vec::new()).expect("An Empty CapList is a Valid List");
        assert_eq!(result.inner().len(), 0);
    }

    #[test]
    fn should_encode_cap_list() {

        let sample_cap = StoreWriteCap {
            location: U256::from(1234).into(),
            size: U256::from(2345).into(),
        };
        let sample_new_cap = NewCapability {
            cap: Capability::StoreWrite(sample_cap),
            parent_index: 0,
        };

        let ENCODED_SAMPLE_WRITE_CAP: Vec<U256> = [
            U256::from(CAP_STORE_WRITE_SIZE + 3),
            U256::from(CAP_STORE_WRITE),
            U256::from(0),
            U256::from(1234),
            U256::from(2345),
        ]
        .to_vec();

        let result = NewCapList([sample_new_cap].to_vec()).to_u256_list();

        assert_eq!(result, ENCODED_SAMPLE_WRITE_CAP);
    }

    #[test]
    fn should_decode_cap_list() {

        let SAMPLE_WRITE_CAP: Vec<U256> = [
            U256::from(CAP_STORE_WRITE_SIZE + 3),
            U256::from(CAP_STORE_WRITE),
            U256::from(0),
            U256::from(1234),
            U256::from(2345),
        ]
        .to_vec();

        let cap_list_result = NewCapList::from_u256_list(&SAMPLE_WRITE_CAP)
            .expect("Should decode to a capability");

        let write_cap = match &cap_list_result.0[0].cap {
            Capability::StoreWrite(write_cap) => write_cap,
            _ => panic!("Invalid Cap"),
        };

        // Get Location and Size
        let loc = U256::from(write_cap.location);
        let size = U256::from(write_cap.size);

        assert_eq!(loc, U256::from(1234));
        assert_eq!(size, U256::from(2345));
    }

    #[test]
    fn should_decode_call_cap() {

        let mut arr = [0; 32];
        arr[0] = 3;

        let sample_cap = ProcedureCallCap {
            prefix: 3,
            key: [1; 24]
        };
        let sample_new_cap = NewCapability {
            cap: Capability::ProcedureCall(sample_cap),
            parent_index: 0,
        };

        let input = NewCapList([sample_new_cap].to_vec());
        let encoded = input.to_u256_list();

        let decoded = NewCapList::from_u256_list(&encoded).expect("Should decode call cap");

        assert_eq!(input.inner(), decoded.inner());
    }

    #[test]
    fn should_decode_encode_call_cap() {

        let prefix: u8 = 3;
        let key: [u8; 24] = [0x1,0x3,0x4,0x5,0x6,0x7,0x8,0x9,0xa,0xb,0xc,0xd,0xe,0xf,0x10,0x12,0x13,0x14,0x15,0x16,0x17,0x18,0x19,0x1a];
        let mut arr = [0; 32];
        arr[0] = 3;
        arr[8..32].copy_from_slice(&key);

        let sample_cap = ProcedureCallCap {
            prefix,
            key,
        };

        let list = [arr.into()].to_vec();

        assert_eq!(ProcedureCallCap::deserialize(&mut Cursor::new(&list)).unwrap(), sample_cap);
    }

    #[test]
    fn should_decode_encode_account_call_cap() {

        let raw: [u8;32] = [0xc0,0x00,0x00,0x00,0x00,0x00,0x00,0x00,0x00,0x00,0x00,0x00,0xaa,0xaa,0xaa,0xaa,0xaa,0xaa,0xaa,0xaa,0xaa,0xaa,0xaa,0xaa,0xaa,0xaa,0xaa,0xaa,0xaa,0xaa,0xaa,0x00];
        let raw_address: [u8;20] = [0xaa,0xaa,0xaa,0xaa,0xaa,0xaa,0xaa,0xaa,0xaa,0xaa,0xaa,0xaa,0xaa,0xaa,0xaa,0xaa,0xaa,0xaa,0xaa,0x00];

        let val: U256 = raw.into();
        let address: Address = raw_address.into();
        let cap: AccountCallCap = val.into();
        let expected_cap = AccountCallCap {
            can_send: true,
            can_call_any: true,
            address: address,
        };
        assert_eq!(cap, expected_cap);
    }

}
