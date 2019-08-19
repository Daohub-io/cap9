/// Module for the fetch interface.

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

/// A representation and connection to a deployed kernel. This has both a
/// connection to the node and a filesystem representation.
pub struct DeployedKernel<'a, 'b, T: Transport> {
    conn: &'a EthConn<T>,
    local_project: &'b LocalProject,
    address: Address,
}

impl<'a, 'b, T: Transport> DeployedKernel<'a, 'b, T> {

    pub fn new(conn: &'a EthConn<T>, local_project: &'b LocalProject,) -> Self {
        let status_file = match local_project.status_file() {
            Some(status_file) => status_file,
            None => panic!("Project not deployed"),
        };
        DeployedKernel {
            conn,
            local_project,
            address: status_file.kernel_address
        }
    }

    pub fn get_storage(&self, storage_address: U256) -> H256 {
        self.conn.web3.eth().storage(self.address, storage_address, None).wait().expect("storage value")
    }

    pub fn current_proc(&self) -> cap9_std::SysCallProcedureKey {
        let h_val: H256 = self.get_storage(*constants::CURRENT_PROC_ADDRESS);
        to_common_h256(h_val).into()
    }

    pub fn entry_proc(&self) -> cap9_std::SysCallProcedureKey {
        let h_val: H256 = self.get_storage(*constants::ENTRY_PROC_ADDRESS);
        to_common_h256(h_val).into()
    }

    pub fn get_proc_cap(&self, key: proc_table::ProcedureKey, cap_type: u8, cap_index: u8) -> Option<proc_table::cap::Capability> {
        let proc_pointer: ProcPointer = ProcPointer::from_key(key);
        let procs = parse_procs(self.conn, self.address.clone(), proc_pointer.clone(), cap_type);
        procs.get(cap_index as usize).cloned()
    }

    /// List the procedures registered in the kernel.
    pub fn procedures(&self) -> Vec<Procedure> {
        let status_file = match self.local_project.status_file() {
            Some(status_file) => status_file,
            None => panic!("Project not deployed"),
        };
        let kernel_address = self.address;
        let n_proc_address: U256 = U256::from_big_endian(&[0xff, 0xff, 0xff, 0xff, 0x01, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00]);
        let store_val = self.conn.web3.eth().storage(kernel_address, n_proc_address, None).wait();
        let n_procs: U256 = U256::from_big_endian(store_val.unwrap().as_bytes());
        let mut procs = Vec::new();
        for i in 1..(n_procs.as_u64() + 1) {
            let storage_address = get_idx_proc_address(i);
            let proc_key_raw: H256 = self.conn.web3.eth().storage(kernel_address, storage_address, None).wait().expect("proc key raw");
            let proc_key_bytes: [u8; 32] = proc_key_raw.to_fixed_bytes();
            let mut proc_bytes = [0; 24];
            proc_bytes.copy_from_slice(&proc_key_bytes[8..]);
            let proc_pointer: ProcPointer = ProcPointer::from_key(proc_bytes);
            let address_raw: H256 = self.conn.web3.eth().storage(kernel_address, U256::from_big_endian(&proc_pointer.get_addr_ptr()), None).wait().expect("proc key raw");
            let address = Address::from_slice(&address_raw[12..]);

            let caps = Caps {
                proc_call: parse_procs(self.conn, kernel_address.clone(), proc_pointer.clone(), CAP_PROC_CALL),
                proc_register: parse_procs(self.conn, kernel_address.clone(), proc_pointer.clone(), CAP_PROC_REGISTER),
                proc_delete: parse_procs(self.conn, kernel_address.clone(), proc_pointer.clone(), CAP_PROC_DELETE),
                proc_entry: parse_procs(self.conn, kernel_address.clone(), proc_pointer.clone(), CAP_PROC_ENTRY),
                store_write: parse_procs(self.conn, kernel_address.clone(), proc_pointer.clone(), CAP_STORE_WRITE),
                log: parse_procs(self.conn, kernel_address.clone(), proc_pointer.clone(), CAP_LOG),
                acc_call: parse_procs(self.conn, kernel_address.clone(), proc_pointer.clone(), CAP_ACC_CALL),
            };

            let procedure = Procedure {
                key: proc_bytes,
                index: i.into(),
                address,
                caps,
            };
            procs.push(procedure);
        }
        procs
    }
}

fn parse_procs<T: Transport>(conn: &EthConn<T>, kernel_address: Address, proc_pointer: ProcPointer, cap_type: u8) -> Vec<Capability> {
    let n_proc_call_caps = U256::from_big_endian(&conn.web3.eth().storage(kernel_address, U256::from_big_endian(&proc_pointer.get_cap_type_len_ptr(cap_type)), None).wait().expect("proc key raw").to_fixed_bytes());
    let mut proc_call_caps = Vec::new();
    for i in 0..(n_proc_call_caps.as_u64() as u8 ) {
        let mut proc_call_caps_reader = CapReader {
            conn: conn,
            kernel_address,
            proc_pointer: proc_pointer.clone(),
            cap_type: cap_type,
            cap_index: i,
            current_val: 0,
        };
        match cap_type {
            CAP_PROC_CALL => {
                let procedure = ProcedureCallCap::deserialize(&mut proc_call_caps_reader);
                proc_call_caps.push(Capability::ProcedureCall(procedure.unwrap()));
            },
            CAP_PROC_REGISTER => {
                let procedure = ProcedureRegisterCap::deserialize(&mut proc_call_caps_reader);
                proc_call_caps.push(Capability::ProcedureRegister(procedure.unwrap()));
            },
            CAP_PROC_DELETE => {
                let procedure = ProcedureDeleteCap::deserialize(&mut proc_call_caps_reader);
                proc_call_caps.push(Capability::ProcedureDelete(procedure.unwrap()));
            },
            CAP_PROC_ENTRY => {
                let procedure = ProcedureEntryCap::deserialize(&mut proc_call_caps_reader);
                proc_call_caps.push(Capability::ProcedureEntry(procedure.unwrap()));
            },
            CAP_STORE_WRITE => {
                let procedure = StoreWriteCap::deserialize(&mut proc_call_caps_reader);
                proc_call_caps.push(Capability::StoreWrite(procedure.unwrap()));
            },
            CAP_LOG => {
                let procedure = LogCap::deserialize(&mut proc_call_caps_reader);
                proc_call_caps.push(Capability::Log(procedure.unwrap()));
            },
            CAP_ACC_CALL => {
                let procedure = AccountCallCap::deserialize(&mut proc_call_caps_reader);
                proc_call_caps.push(Capability::AccountCall(procedure.unwrap()));
            },
            _ => panic!("invalid cap type"),
        }
    }
    proc_call_caps
}

/// As with [DeployKernel] but with a standard ACL.
pub struct DeployedKernelWithACL<'a, 'b, T: Transport> {
    kernel: DeployedKernel<'a, 'b, T>,
}


impl<'a, 'b, T: Transport> DeployedKernelWithACL<'a, 'b, T> {

    pub fn new(kernel: DeployedKernel<'a, 'b, T>) -> Self {
        DeployedKernelWithACL {
            kernel: kernel,
        }
    }

    pub fn groups(&self) -> HashMap<u8,Group> {
        // Currently we assume the group map is at cap index 1
        let groups: LocalEnumerableMap<_, u8, cap9_std::SysCallProcedureKey> = LocalEnumerableMap::from(&self.kernel, 1).expect("could not create group map");
        // Currently we assume the users map is at cap index 0
        let users: LocalEnumerableMap<_, pwasm_abi::types::Address, u8> = LocalEnumerableMap::from(&self.kernel, 0).expect("could not create user map");
        let mut group_map: HashMap<u8, Group> = HashMap::new();
        for (k, v) in groups.iter() {
            group_map.insert(k, Group {
                id: k,
                procedure_key: v,
                users: HashSet::new(),
            });
        }
        for (k, v) in users.iter() {
            let group = group_map.get_mut(&v).expect(format!("no such group exists: {}", v).as_str());
            group.users.insert(from_common_address(k));
        }
        group_map
    }

    pub fn users(&self) -> HashMap<Address,u8> {
        // Currently we assume the users map is at cap index 0
        let users: LocalEnumerableMap<_, pwasm_abi::types::Address, u8> = LocalEnumerableMap::from(&self.kernel, 0).expect("could not create user map");
        let mut users_map = HashMap::new();
        for (k, v) in users.iter() {
            users_map.insert(from_common_address(k), v);
        }
        users_map
    }
}

#[derive(Clone, Debug)]
pub struct Group {
    pub id: u8,
    pub procedure_key: cap9_std::SysCallProcedureKey,
    pub users: HashSet<Address>,
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

fn get_idx_proc_address(i: u64) -> U256 {
    let idx: u8 = i as u8;
    U256::from_big_endian(&[0xff, 0xff, 0xff, 0xff, 0x01, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, idx, 0x00, 0x00, 0x00])
}

use cap9_std::data::{Keyable, Storable, DataStructureError};
use core::marker::PhantomData;

pub struct LocalEnumerableMap<'a, 'b, T: Transport, K: Keyable, V: Storable> {
    cap_index: u8,
    /// The start location of the map.
    location: H256,
    /// The key type of the map.
    key_type: PhantomData<K>,
    /// The data type of the map.
    data_type: PhantomData<V>,
    /// Possible the cached number of elements in the map.
    length: Option<U256>,
    /// The deployed kernel used as the source of information
    kernel: &'a DeployedKernel<'a, 'b, T>,
}

impl<'a, 'b, T: Transport, K: Keyable, V: Storable> LocalEnumerableMap<'a, 'b, T, K, V> {

    pub fn from(kernel: &'a DeployedKernel<'a, 'b, T>, cap_index: u8) -> Result<Self, DataStructureError> {
        // The size of the cap needs to be key_width+1 in bytes
        let address_bytes = K::key_width()+1;
        let address_bits = address_bytes*8;
        let address_size = U256::from(2).pow(U256::from(address_bits));
        // The address also need to be aligned.

        // The cap_index is an index into the caplist of the entry procedure
        // let this_proc_key = proc_table::get_current_proc_id();
        let this_proc_key = kernel.entry_proc().0;
        // We need to get
        if let Some(proc_table::cap::Capability::StoreWrite(proc_table::cap::StoreWriteCap {location, size})) =
                kernel.get_proc_cap(this_proc_key, proc_table::cap::CAP_STORE_WRITE, cap_index) {
                    // Check that the size of the cap is correct.
                    if U256::from(size) < address_size {
                        Err(DataStructureError::TooSmall)
                    } else if U256::from(location).trailing_zeros() < (address_bits as u32 + 1 + 1 + 6) {
                        // the trailing number of 0 bits should be equal to or greater than the address_bits
                        Err(DataStructureError::MisAligned)
                    } else {
                        Ok(LocalEnumerableMap {
                            cap_index,
                            location: location.into(),
                            key_type: PhantomData,
                            data_type: PhantomData,
                            length: None,
                            kernel,
                        })
                    }
        } else {
            Err(DataStructureError::BadCap)
        }
    }

    /// Return the start/base location of the map.
    pub fn location(&self) -> H256 {
        self.location
    }

    /// Return the base storage key of a given map key.
    fn base_key(&self, key: &K) -> [u8; 32] {
        let mut base: [u8; 32] = [0; 32];
        // The key start 32 - width - 1, the - 1 is for data and presence. This
        // is in bytes.
        let key_start = 32 - K::key_width() as usize - 1;
        // First we copy in the relevant parts of the location.
        base[0..key_start].copy_from_slice(&self.location().as_bytes()[0..key_start]);
        // Then we copy in the key
        // TODO: overflow
        base[key_start..(key_start+K::key_width() as usize)].clone_from_slice(key.key_slice().as_slice());
        base
    }

    fn presence_key(&self, key: &K) -> H256 {
        // The presence_key is the storage key which indicates whether there is
        // a value associated with this key.
        let mut presence_key = self.base_key(&key);
        // The first bit of the data byte indicates presence
        presence_key[31] = presence_key[31] | 0b10000000;
        presence_key.into()
    }

    fn length_key(&self) -> H256 {
        // The presence_key is the storage key which indicates whether there is
        // a value associated with this key.
        let mut location = self.location.clone();
        let length_key = location.as_fixed_bytes_mut();
        let index = 31;
        length_key[index as usize] = length_key[index as usize] | 0b01000000;
        length_key.into()
    }

    /// Return the number of elements in the map.
    pub fn length(&self) -> U256 {
        let mut buf: [u8; 32] = [0; 32];
        h256_to_u256(self.length_key()).to_big_endian(&mut buf);
        match self.length {
            // A cached value exists, use that.
            Some(l) => l,
            // No cached value exists, read from storage.
            None => {
                let length = self.kernel.get_storage(h256_to_u256(self.length_key()));
                h256_to_u256(length)
            }
        }
    }

    // fn increment_length(&mut self) {
    //     self.length = Some(self.length().checked_add(1.into()).unwrap());
    //     // Store length value.
    //     write(self.cap_index, &self.length_key().to_fixed_bytes(), &self.length().into()).unwrap();
    // }

    // fn decrement_length(&mut self) {
    //     self.length = Some(self.length().checked_sub(1.into()).unwrap());
    //     // Store length value.
    //     write(self.cap_index, &self.length_key().to_fixed_bytes(), &self.length().into()).unwrap();
    // }

    /// Return true if the given key is associated with a value in the map.
    pub fn present(&self, key: &K) -> bool {
        // If the value at the presence key is non-zero, then a value is
        // present.
        let presence_key = h256_to_u256(self.presence_key(key));
        let mut buf = [0;32];
        presence_key.to_big_endian(&mut buf);
        let present = self.kernel.get_storage(presence_key);
        let null: [u8; 32] = [0; 32];
        present.as_fixed_bytes() != &null
    }

    // fn index(&self, key: &K) -> Option<U256> {
    //     let present = pwasm_ethereum::read(&self.presence_key(key));
    //     Some(present.into())
    // }

    // fn set_present(&self, key: &K, index: U256) {
    //     // For the enumerable map, the presence value is a 1-based index into
    //     // the enumeration vector.
    //     let storable_index: StorageValue = index.into();
    //     write(self.cap_index, &self.presence_key(key).as_fixed_bytes(), &storable_index.into()).unwrap();
    // }

    // fn set_absent(&self, key: K) {
    //     write(self.cap_index, &self.presence_key(&key).as_fixed_bytes(), H256::repeat_byte(0x00).as_fixed_bytes()).unwrap();
    // }

    // /// Get the value associated with a given key, if it exists.
    pub fn get(&self, key: K) -> Option<V> {
        let base = self.base_key(&key);
        if self.present(&key) {
            let mut vals = Vec::new();
            let base_address: U256 = base.into();
            for i in 0..V::n_keys().as_u64() {
                let val = self.kernel.get_storage(base_address + U256::from(i));
                vals.push(to_common_u256(h256_to_u256(val)));
            }
            V::read_vec_u256(vals)
        } else {
            None
        }
    }

    /// Return the key at a given index in the map. The ordering of keys is not
    /// well defined, and this should only be used for enumeration.
    pub fn get_key_at_index(&self, index: U256) -> Option<K> {
        if index >= self.length() {
            return None;
        }
        let mut storage_key_h = self.length_key().clone();
        let storage_key: U256 = h256_to_u256(storage_key_h) + index + U256::from(1);
        let mut store_buf: [u8; 32] = [0; 32];
        storage_key.to_big_endian(&mut store_buf);
        let storage_value: StorageValue = to_common_h256(self.kernel.get_storage(storage_key)).into();
        Some(storage_value.into())
    }

    /// Produce an iterator over keys and values.
    pub fn iter(&self) -> LocalEnumerableMapIter<T, K, V> {
        LocalEnumerableMapIter::new(self)
    }

    // /// Produce an iterator over keys.
    // pub fn keys(&self) -> StorageEnumerableMapKeys<K,V> {
    //     StorageEnumerableMapKeys::new(self)
    // }

    // /// Produce an iterator over values.
    // pub fn values(&self) -> StorageEnumerableMapValues<K,V> {
    //     StorageEnumerableMapValues::new(self)
    // }
}


/// An iterator over the keys and values of a [`StorageEnumerableMap`].
pub struct LocalEnumerableMapIter<'a, 'b, 'c, T: Transport, K: Keyable, V: Storable> {
    /// The StorageVec we are iterating over.
    storage_map: &'a LocalEnumerableMap<'b, 'c, T, K, V>,
    /// The current offset into the StorageVec.
    offset: U256,
}

impl<'a, 'b, 'c, T: Transport, K: Keyable, V: Storable> LocalEnumerableMapIter<'a, 'b, 'c, T, K, V> {
    fn new(storage_map: &'a LocalEnumerableMap<'b, 'c, T, K, V>) -> Self {
        LocalEnumerableMapIter {
            storage_map,
            offset: U256::zero(),
        }
    }
}

impl<'a, 'b, 'c, T: Transport, K: Keyable, V: Storable> Iterator for LocalEnumerableMapIter<'a, 'b, 'c, T, K, V> {
    type Item = (K, V);

    fn next(&mut self) -> Option<Self::Item> {
        let key = match self.storage_map.get_key_at_index(self.offset) {
            Some(val) => {
                self.offset += U256::from(1);
                val
            },
            None => {
                return None;
            },
        };
        Some((key.clone(), self.storage_map.get(key)?))
    }
}


fn h256_to_u256(h: H256) -> U256 {
    U256::from_big_endian(&h.to_fixed_bytes())
}

fn u256_to_h256(u: U256) -> H256 {
    let mut buf: [u8; 32] = [0; 32];
    u.to_big_endian(&mut buf);
    H256::from_slice(&buf)
}
