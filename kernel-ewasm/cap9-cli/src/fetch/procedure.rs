
use web3::futures::Future;
use web3::contract::{Contract, Options};
use web3::types::{Address, U256, H256};
// use web3::types::TransactionReceipt;
use web3::Transport;
use rustc_hex::FromHex;
use rustc_hex::ToHex;
// use ethabi::Token::Uint;
use crate::conn;
use crate::conn::EthConn;
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
use crate::deploy::{from_common_u256, to_common_u256, to_common_h256,
    from_common_address, to_common_address
};
use std::collections::{HashMap, HashSet};

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
