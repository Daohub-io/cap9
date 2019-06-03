extern crate pwasm_abi;
extern crate pwasm_abi_derive;
extern crate pwasm_ethereum;
extern crate pwasm_std;

use pwasm_abi::eth;
use pwasm_abi::types::*;
use pwasm_abi_derive::eth_abi;

pub const CAP_PROC_CALL: u8 = 3;
pub const CAP_PROC_CALL_SIZE: u8 = 1;

pub const CAP_PROC_REGISTER: u8 = 4;
pub const CAP_PROC_REGISTER_SIZE: u8 = 1;

pub const CAP_PROC_DELETE: u8 = 5;
pub const CAP_PROC_DELETE_SIZE: u8 = 1;

pub const CAP_PROC_ENTRY: u8 = 6;
pub const CAP_PROC_ENTRY_SIZE: u8 = 0;

pub const CAP_STORE_WRITE: u8 = 7;
pub const CAP_STORE_WRITE_SIZE: u8 = 2;

pub const CAP_LOG: u8 = 8;
pub const CAP_LOG_SIZE: u8 = 5;

pub const CAP_ACC_CALL: u8 = 9;
pub const CAP_ACC_CALL_SIZE: u8 = 1;

type ProcedureKey = [u8; 24];
type ProcedureIndex = [u8; 24];

#[derive(Clone, Debug)]
pub struct ProcedureCallCap {
    pub prefix: u8,
    pub key: ProcedureKey,
}

#[derive(Clone, Debug)]
pub struct ProcedureRegisterCap {
    pub prefix: u8,
    pub key: ProcedureKey,
}

#[derive(Clone, Debug)]
pub struct ProcedureDeleteCap {
    pub prefix: u8,
    pub key: ProcedureKey,
}

#[derive(Clone, Debug)]
pub struct ProcedureEntryCap;

#[derive(Clone, Debug)]
pub struct StoreWriteCap {
    pub location: [u8; 32],
    pub size: [u8; 32],
}

#[derive(Clone, Debug)]
pub struct LogCap {
    pub topics: u8,
    pub t1: [u8; 32],
    pub t2: [u8; 32],
    pub t3: [u8; 32],
    pub t4: [u8; 32],
}

#[derive(Clone, Debug)]
pub struct AccountCallCap {
    pub can_call_any: bool,
    pub can_send: bool,
    pub address: Address,
}

#[derive(Clone, Debug)]
pub enum Capability {
    ProcedureCall(ProcedureCallCap),
    ProcedureRegister(ProcedureRegisterCap),
    ProcedureDelete(ProcedureDeleteCap),
    ProcedureEntry(ProcedureEntryCap),
    StoreWrite(StoreWriteCap),
    Log(LogCap),
    AccountCall(AccountCallCap),
}

#[derive(Clone, Debug)]
pub struct NewCapability {
    pub cap: Capability,
    pub parent_index: u8,
}

#[derive(Debug)]
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

#[derive(Clone, Debug)]
pub enum CapDecodeErr {
    InvalidCapType(u8),
    InvalidCapLen(u8),
}

impl NewCapList {
    pub fn to_u256_list(&self) -> Vec<U256> {
        // Allocate Vector with Max Cap Size
        let mut res = Vec::with_capacity(self.0.len() * (CAP_LOG_SIZE + 3) as usize);

        for new_cap in self.0.iter() {
            let raw_cap_slice = match &new_cap.cap {
                Capability::ProcedureCall(proc_call_cap) => {
                    let cap_size = U256::from(CAP_PROC_CALL_SIZE + 3);
                    let cap_type = U256::from(CAP_PROC_CALL);
                    let parent_index = U256::from(new_cap.parent_index);

                    let mut res = [0u8; 32];
                    res[0] = proc_call_cap.prefix;
                    res[8..].copy_from_slice(&proc_call_cap.key);

                    [cap_size, cap_type, parent_index, U256::from(res)].to_vec()
                }
                Capability::ProcedureRegister(proc_register_cap) => {
                    let cap_size = U256::from(CAP_PROC_REGISTER_SIZE + 3);
                    let cap_type = U256::from(CAP_PROC_REGISTER);
                    let parent_index = U256::from(new_cap.parent_index);

                    let mut res = [0u8; 32];
                    res[0] = proc_register_cap.prefix;
                    res[8..].copy_from_slice(&proc_register_cap.key);

                    [cap_size, cap_type, parent_index, U256::from(res)].to_vec()
                }
                Capability::ProcedureDelete(proc_delete_cap) => {
                    let cap_size = U256::from(CAP_PROC_DELETE_SIZE + 3);
                    let cap_type = U256::from(CAP_PROC_DELETE);
                    let parent_index = U256::from(new_cap.parent_index);

                    let mut res = [0u8; 32];
                    res[0] = proc_delete_cap.prefix;
                    res[8..].copy_from_slice(&proc_delete_cap.key);

                    [cap_size, cap_type, parent_index, U256::from(res)].to_vec()
                }
                Capability::ProcedureEntry(_) => {
                    let cap_size = U256::from(CAP_PROC_ENTRY_SIZE + 3);
                    let cap_type = U256::from(CAP_PROC_ENTRY);
                    let parent_index = U256::from(new_cap.parent_index);

                    [cap_size, cap_type, parent_index].to_vec()
                }
                Capability::StoreWrite(store_write_cap) => {
                    let cap_size = U256::from(CAP_STORE_WRITE_SIZE + 3);
                    let cap_type = U256::from(CAP_STORE_WRITE);
                    let parent_index = U256::from(new_cap.parent_index);

                    [
                        cap_size,
                        cap_type,
                        parent_index,
                        U256::from(store_write_cap.location),
                        U256::from(store_write_cap.size),
                    ]
                    .to_vec()
                }
                Capability::Log(log_cap) => {
                    let cap_size = U256::from(CAP_LOG_SIZE + 3);
                    let cap_type = U256::from(CAP_LOG);
                    let parent_index = U256::from(new_cap.parent_index);

                    let topics_len = U256::from(log_cap.topics);
                    let t1 = U256::from(log_cap.t1);
                    let t2 = U256::from(log_cap.t2);
                    let t3 = U256::from(log_cap.t3);
                    let t4 = U256::from(log_cap.t4);

                    [cap_size, cap_type, parent_index, topics_len, t1, t2, t3, t4].to_vec()
                }
                Capability::AccountCall(account_call_cap) => {
                    let cap_size = U256::from(CAP_ACC_CALL_SIZE + 3);
                    let cap_type = U256::from(CAP_ACC_CALL);
                    let parent_index = U256::from(new_cap.parent_index);

                    let mut res = [0u8; 32];
                    res[0] |= if account_call_cap.can_call_any {
                        0x80
                    } else {
                        0
                    };
                    res[0] |= if account_call_cap.can_send { 0x40 } else { 0 };

                    res[12..].copy_from_slice(account_call_cap.address.as_fixed_bytes());

                    [cap_size, cap_type, parent_index, U256::from(res)].to_vec()
                }
            };

            res.extend_from_slice(&raw_cap_slice);
        }

        res
    }

    pub fn from_u256_list(list: &[U256]) -> Result<Self, CapDecodeErr> {
        let mut result = Vec::new();

        // List Length
        let end = list.len();

        // Set Start
        let mut start = 0;

        while start < end {
            // Check List Length
            if end - start < 3 {
                return Err(CapDecodeErr::InvalidCapLen(start as u8));
            }

            // Get Values
            let cap_size = list[start].byte(0);
            let cap_type = list[start + 1].byte(0);
            let parent_index = list[start + 2].byte(0);

            // Check Cap Size
            if end - start > cap_size as usize {
                return Err(CapDecodeErr::InvalidCapLen(cap_size));
            }

            // Increment Start
            start += 3;

            let new_cap = match (cap_type, cap_size - 3) {
                (CAP_PROC_CALL, CAP_PROC_CALL_SIZE) => {
                    let val = list[start];
                    let mut key = [0u8; 24];
                    key.copy_from_slice(&<[u8; 32]>::from(val)[8..]);

                    let proc_call_cap = ProcedureCallCap {
                        prefix: val.byte(0),
                        key: key,
                    };

                    NewCapability {
                        cap: Capability::ProcedureCall(proc_call_cap),
                        parent_index,
                    }
                }
                (CAP_PROC_REGISTER, CAP_PROC_REGISTER_SIZE) => {
                    let val = list[start];

                    let mut key = [0u8; 24];
                    key.copy_from_slice(&<[u8; 32]>::from(val)[8..]);

                    let proc_reg_cap = ProcedureRegisterCap {
                        prefix: val.byte(0),
                        key: key,
                    };

                    NewCapability {
                        cap: Capability::ProcedureRegister(proc_reg_cap),
                        parent_index,
                    }
                }
                (CAP_PROC_DELETE, CAP_PROC_DELETE_SIZE) => {
                    let val = list[start];

                    let mut key = [0u8; 24];
                    key.copy_from_slice(&<[u8; 32]>::from(val)[8..]);

                    let proc_del_cap = ProcedureDeleteCap {
                        prefix: val.byte(0),
                        key: key,
                    };

                    NewCapability {
                        cap: Capability::ProcedureDelete(proc_del_cap),
                        parent_index,
                    }
                }
                (CAP_PROC_ENTRY, CAP_PROC_ENTRY_SIZE) => NewCapability {
                    cap: Capability::ProcedureEntry(ProcedureEntryCap),
                    parent_index,
                },
                (CAP_STORE_WRITE, CAP_STORE_WRITE_SIZE) => {
                    let location: [u8; 32] = list[start].into();
                    let size: [u8; 32] = list[start + 1].into();

                    let store_write_cap = StoreWriteCap {
                        location: location,
                        size: size,
                    };

                    NewCapability {
                        cap: Capability::StoreWrite(store_write_cap),
                        parent_index,
                    }
                }
                (CAP_LOG, CAP_LOG_SIZE) => {
                    let topics_len: usize = list[start].byte(0) as usize;
                    let mut topics = [[0; 32]; 4];
                    if topics_len != 0 {
                        match topics_len {
                            1...4 => {
                                for i in 0..topics_len {
                                    topics[i] = list[start + i + 1].into()
                                }
                            }
                            _ => return Err(CapDecodeErr::InvalidCapLen(topics_len as u8)),
                        }
                    }

                    NewCapability {
                        cap: Capability::Log(LogCap {
                            topics: topics_len as u8,
                            t1: topics[0],
                            t2: topics[1],
                            t3: topics[2],
                            t4: topics[3],
                        }),
                        parent_index,
                    }
                }
                (CAP_ACC_CALL, CAP_ACC_CALL_SIZE) => {
                    let val = list[start];

                    let can_call_any = val.bit(0);
                    let can_send = val.bit(1);

                    let mut address = [0u8; 20];
                    address.copy_from_slice(&<[u8; 32]>::from(val)[12..]);
                    let address = H160::from(address);

                    let account_call_cap = AccountCallCap {
                        can_call_any: can_call_any,
                        can_send: can_send,
                        address: address,
                    };

                    NewCapability {
                        cap: Capability::AccountCall(account_call_cap),
                        parent_index,
                    }
                }
                _ => return Err(CapDecodeErr::InvalidCapType(cap_type)),
            };

            start += cap_size as usize - 3;
            result.push(new_cap);
        }

        Ok(NewCapList(result))
    }
}

#[cfg(test)]
#[allow(non_snake_case)]
mod tests {
    extern crate pwasm_test;
    extern crate std;

    use self::pwasm_test::{ext_get, ext_reset};
    use core::str::FromStr;
    use pwasm_abi::types::*;

    use super::*;

    use pwasm_abi::eth;
    use pwasm_abi::eth::AbiType;

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

        let mut sink = eth::Sink::new(10 * 256);

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
            cap => panic!("Invalid Cap"),
        };

        // Get Location and Size
        let loc = U256::from(write_cap.location);
        let size = U256::from(write_cap.size);

        assert_eq!(loc, U256::from(1234));
        assert_eq!(size, U256::from(2345));
    }

}
