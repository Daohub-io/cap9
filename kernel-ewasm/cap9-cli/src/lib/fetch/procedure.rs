
use web3::futures::Future;
use web3::contract::{Contract, Options};
use web3::types::{Address, U256, H256};
// use web3::types::TransactionReceipt;
use web3::Transport;
use rustc_hex::FromHex;
use rustc_hex::ToHex;
// use ethabi::Token::Uint;
use crate::connection;
use crate::connection::EthConn;
use crate::project::LocalProject;
use cap9_std::proc_table::cap::*;
use pwasm_abi;
use std::fs::File;
use std::fmt;
use cap9_std::proc_table::ProcPointer;
use cap9_std::proc_table;
use cap9_core::*;
use cap9_core::Error;
use cap9_core::Read;
use crate::constants;
use crate::utils;
use std::collections::{HashMap, HashSet};
use serde_json::json;
use serde::ser::{Serialize, Serializer, SerializeSeq, SerializeMap, SerializeStruct};

#[derive(Clone, Debug)]
pub struct Procedure {
    pub key: [u8; 24],
    pub index: U256,
    pub address: Address,
    pub caps: Caps,
}

impl fmt::Display for Procedure {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let key_hex: String = self.key.to_hex();
        let key_utf8: &str = std::str::from_utf8(&self.key).unwrap().trim_end_matches('\0');
        write!(f, "Procedure[{}]: 0x{} (\"{}\")\n  Address: {:?}\n  Caps({}):\n{}",
            self.index.as_u64(), key_hex, key_utf8, self.address, self.caps.len(), self.caps)
    }
}

#[derive(Clone, Debug)]
pub struct Caps {
    pub proc_call: Vec<Capability>,
    pub proc_register: Vec<Capability>,
    pub proc_delete: Vec<Capability>,
    pub proc_entry: Vec<Capability>,
    pub store_write: Vec<Capability>,
    pub log: Vec<Capability>,
    pub acc_call: Vec<Capability>,
}

impl Caps {
    pub fn len(&self) -> usize {
        self.proc_call.len()
            + self.proc_register.len()
            + self.proc_delete.len()
            + self.proc_entry.len()
            + self.store_write.len()
            + self.log.len()
            + self.acc_call.len()
    }
}


impl fmt::Display for Caps {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        if self.proc_call.len() > 0 {
            write!(f, "    CAP_PROC_CALL({}):\n", self.proc_call.len())?;
            for (i, cap) in self.proc_call.iter().enumerate() {
                write!(f, "        {}: {}\n", i, cap)?;
            }
        }
        if self.proc_register.len() > 0 {
            write!(f, "    CAP_PROC_REGISTER({}):\n", self.proc_register.len())?;
            for (i, cap) in self.proc_register.iter().enumerate() {
                write!(f, "        {}: {}\n", i, cap)?;
            }
        }
        if self.proc_delete.len() > 0 {
            write!(f, "    CAP_PROC_DELETE({}):\n", self.proc_delete.len())?;
            for (i, cap) in self.proc_delete.iter().enumerate() {
                write!(f, "        {}: {}\n", i, cap)?;
            }
        }
        if self.proc_entry.len() > 0 {
            write!(f, "    CAP_PROC_CALL({}):\n", self.proc_entry.len())?;
            for (i, cap) in self.proc_entry.iter().enumerate() {
                write!(f, "        {}: {}\n", i, cap)?;
            }
        }
        if self.store_write.len() > 0 {
            write!(f, "    CAP_STORE_WRITE({}):\n", self.store_write.len())?;
            for (i, cap) in self.store_write.iter().enumerate() {
                write!(f, "        {}: {}\n", i, cap)?;
            }
        }
        if self.log.len() > 0 {
            write!(f, "    CAP_LOG({}):\n", self.log.len())?;
            for (i, cap) in self.log.iter().enumerate() {
                write!(f, "        {}: {}\n", i, cap)?;
            }
        }
        if self.acc_call.len() > 0 {
            write!(f, "    CAP_ACC_CALL({}):\n", self.acc_call.len())?;
            for (i, cap) in self.acc_call.iter().enumerate() {
                write!(f, "        {}: {}\n", i, cap)?;
            }
        }
        write!(f, "")
    }
}

struct CapReader<'a, T> where T: Transport {
    conn: &'a EthConn<T>,
    kernel_address: Address,
    proc_pointer: ProcPointer,
    cap_type: u8,
    cap_index: u8,
    current_val: u8,
}

impl<'a, T: Transport> Read<pwasm_abi::types::U256> for CapReader<'a, T> {
    fn read(&mut self, buf: &mut [pwasm_abi::types::U256]) -> Result<(), Error> {
        for i in 0..buf.len() {
            let next_val_ptr = self.proc_pointer.get_cap_val_ptr(self.cap_type, self.cap_index, self.current_val);
            let next_val = self.conn.web3.eth().storage(self.kernel_address, U256::from_big_endian(&next_val_ptr), None).wait().expect("proc key raw");
            self.current_val += 1;
            buf[i] = pwasm_abi::types::U256::from_big_endian(&next_val.to_fixed_bytes());
        }
        Ok(())
    }

    fn remaining(&self) -> usize {
        1_usize
    }
}


fn get_idx_proc_address(i: u64) -> U256 {
    let idx: u8 = i as u8;
    U256::from_big_endian(&[0xff, 0xff, 0xff, 0xff, 0x01, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, idx, 0x00, 0x00, 0x00])
}

pub struct SerialNewCapList(pub NewCapList);
pub struct SerialNewCap(NewCapability);
pub struct SerialCapability(Capability);
pub struct SerialAddress(Address);

impl Serialize for SerialNewCapList {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let cap_list = &(self.0).0;

        let mut seq = serializer.serialize_seq(Some(cap_list.len()))?;
        for e in cap_list {
            seq.serialize_element(&SerialNewCap(e.clone()))?;
        }
        seq.end()
    }
}

impl Serialize for SerialNewCap {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let cap = &(self.0).cap;
        let parent_index = (self.0).parent_index;

        let mut state = serializer.serialize_struct("NewCapability", 2)?;
        state.serialize_field("cap", &SerialCapability(cap.clone()))?;
        state.serialize_field("parent_index", &parent_index)?;
        state.end()
    }
}

fn key_to_str(key: [u8; 24]) -> String {
    let mut key_hex: String = String::from("0x");;
    let s: String = key.to_hex();
    key_hex.push_str(&s);
    key_hex
}

fn b32_to_str(key: [u8; 32]) -> String {
    let mut key_hex: String = String::from("0x");;
    let s: String = key.to_hex();
    key_hex.push_str(&s);
    key_hex
}

impl Serialize for SerialCapability {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        match &self.0 {
            Capability::ProcedureCall(cap) => {
                let mut state = serializer.serialize_struct("ProcedureCallCap", 2)?;
                state.serialize_field("prefix", &cap.prefix)?;
                state.serialize_field("key", &key_to_str(cap.key))?;
                state.end()
            },
            Capability::ProcedureRegister(cap) => {
                let mut state = serializer.serialize_struct("ProcedureRegisterCap", 2)?;
                state.serialize_field("prefix", &cap.prefix)?;
                state.serialize_field("key", &key_to_str(cap.key))?;
                state.end()

            },
            Capability::ProcedureDelete(cap) => {
                let mut state = serializer.serialize_struct("ProcedureDeleteCap", 2)?;
                state.serialize_field("prefix", &cap.prefix)?;
                state.serialize_field("key", &key_to_str(cap.key))?;
                state.end()

            },
            Capability::ProcedureEntry(_cap) => {
                let state = serializer.serialize_struct("ProcedureEntryCap", 0)?;
                state.end()

            },
            Capability::StoreWrite(cap) => {
                let mut state = serializer.serialize_struct("StoreWriteCap", 2)?;
                state.serialize_field("location", &b32_to_str(cap.location))?;
                state.serialize_field("size", &b32_to_str(cap.size))?;
                state.end()

            },
            Capability::Log(cap) => {
                let mut state = serializer.serialize_struct("LogCap", 5)?;
                state.serialize_field("topics", &cap.topics)?;
                state.serialize_field("t1", &b32_to_str(cap.t1))?;
                state.serialize_field("t2", &b32_to_str(cap.t2))?;
                state.serialize_field("t3", &b32_to_str(cap.t3))?;
                state.serialize_field("t4", &b32_to_str(cap.t4))?;
                state.end()

            },
            Capability::AccountCall(cap) => {
                let mut state = serializer.serialize_struct("AccountCallCap", 2)?;
                state.serialize_field("can_call_any", &cap.can_call_any)?;
                state.serialize_field("can_send", &cap.can_send)?;
                state.serialize_field("address", &SerialAddress(utils::from_common_address(cap.address)))?;
                state.end()

            },
        }
    }
}

impl Serialize for SerialAddress {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.serialize_str(format!("{}",(self.0)).as_ref())
    }
}
