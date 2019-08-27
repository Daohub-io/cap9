
use web3::futures::Future;
use web3::types::{Address, U256, H256};
// use web3::types::TransactionReceipt;
use web3::Transport;
use rustc_hex::ToHex;
// use ethabi::Token::Uint;
use crate::connection::EthConn;
use crate::project::LocalProject;
use cap9_std::proc_table::cap::*;
use pwasm_abi;
use std::fmt;
use cap9_std::proc_table::ProcPointer;
use cap9_std::proc_table;
use cap9_core::*;
use cap9_core::Error;
use cap9_core::Read;
use crate::constants;
use crate::utils::to_common_h256;
/// A representation and connection to a deployed kernel. This has both a
/// connection to the node and a filesystem representation.
pub struct DeployedKernel<'a, T: Transport> {
    pub conn: &'a EthConn<T>,
    pub local_project: LocalProject,
    pub address: Address,
}

impl<'a, T: Transport> DeployedKernel<'a, T> {

    pub fn new(conn: &'a EthConn<T>, local_project: LocalProject,) -> Self {
        let kernel_address = match local_project.status_file() {
            Some(status_file) => status_file.kernel_address.clone(),
            None => panic!("Project not deployed"),
        };
        DeployedKernel {
            conn,
            local_project,
            address: kernel_address
        }
    }

    pub fn address(&self) -> Address {
        self.address
    }

    pub fn gas(&self) -> U256 {
        self.conn.web3.eth().balance(self.address, None).wait().expect("could not retrieve gas")
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

            let caps = Capabilities {
                proc_call_caps:     parse_proc_call_caps(self.conn, kernel_address.clone(), proc_pointer.clone()),
                proc_register_caps: parse_proc_register_caps(self.conn, kernel_address.clone(), proc_pointer.clone()),
                proc_delete_caps:   parse_proc_delete_caps(self.conn, kernel_address.clone(), proc_pointer.clone()),
                proc_entry_caps:    parse_proc_entry_caps(self.conn, kernel_address.clone(), proc_pointer.clone()),
                store_write_caps:   parse_store_write_caps(self.conn, kernel_address.clone(), proc_pointer.clone()),
                log_caps:           parse_log_caps(self.conn, kernel_address.clone(), proc_pointer.clone()),
                account_call_caps:  parse_account_call_caps(self.conn, kernel_address.clone(), proc_pointer.clone()),
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

    /// Retrieve a specific procedure.
    /// TODO: this is currently inefficient as it retrieves all procs first.
    pub fn procedure(&self, proc_key: cap9_std::SysCallProcedureKey) -> Option<Procedure> {
        let procs = self.procedures();
        for procedure in procs {
            if procedure.key == proc_key.0 {
                return Some(procedure);
            }
        }
        None
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

fn parse_proc_call_caps<T: Transport>(conn: &EthConn<T>, kernel_address: Address, proc_pointer: ProcPointer) -> Vec<ProcedureCallCap> {
    let cap_type: u8 = CAP_PROC_CALL;
    let n_caps = U256::from_big_endian(&conn.web3.eth().storage(kernel_address, U256::from_big_endian(&proc_pointer.get_cap_type_len_ptr(cap_type)), None).wait().expect("proc key raw").to_fixed_bytes());
    let mut caps = Vec::new();
    for i in 0..(n_caps.as_u64() as u8 ) {
        let mut caps_reader = CapReader {
            conn: conn,
            kernel_address,
            proc_pointer: proc_pointer.clone(),
            cap_type: cap_type,
            cap_index: i,
            current_val: 0,
        };
        let procedure = ProcedureCallCap::deserialize(&mut caps_reader);
        caps.push(procedure.unwrap());
    }
    caps
}

fn parse_proc_register_caps<T: Transport>(conn: &EthConn<T>, kernel_address: Address, proc_pointer: ProcPointer) -> Vec<ProcedureRegisterCap> {
    let cap_type: u8 = CAP_PROC_REGISTER;
    let n_caps = U256::from_big_endian(&conn.web3.eth().storage(kernel_address, U256::from_big_endian(&proc_pointer.get_cap_type_len_ptr(cap_type)), None).wait().expect("proc key raw").to_fixed_bytes());
    let mut caps = Vec::new();
    for i in 0..(n_caps.as_u64() as u8 ) {
        let mut caps_reader = CapReader {
            conn: conn,
            kernel_address,
            proc_pointer: proc_pointer.clone(),
            cap_type: cap_type,
            cap_index: i,
            current_val: 0,
        };
        let procedure = ProcedureRegisterCap::deserialize(&mut caps_reader);
        caps.push(procedure.unwrap());
    }
    caps
}

fn parse_proc_delete_caps<T: Transport>(conn: &EthConn<T>, kernel_address: Address, proc_pointer: ProcPointer) -> Vec<ProcedureDeleteCap> {
    let cap_type: u8 = CAP_PROC_DELETE;
    let n_caps = U256::from_big_endian(&conn.web3.eth().storage(kernel_address, U256::from_big_endian(&proc_pointer.get_cap_type_len_ptr(cap_type)), None).wait().expect("proc key raw").to_fixed_bytes());
    let mut caps = Vec::new();
    for i in 0..(n_caps.as_u64() as u8 ) {
        let mut caps_reader = CapReader {
            conn: conn,
            kernel_address,
            proc_pointer: proc_pointer.clone(),
            cap_type: cap_type,
            cap_index: i,
            current_val: 0,
        };
        let procedure = ProcedureDeleteCap::deserialize(&mut caps_reader);
        caps.push(procedure.unwrap());
    }
    caps
}

fn parse_proc_entry_caps<T: Transport>(conn: &EthConn<T>, kernel_address: Address, proc_pointer: ProcPointer) -> Vec<ProcedureEntryCap> {
    let cap_type: u8 = CAP_PROC_ENTRY;
    let n_caps = U256::from_big_endian(&conn.web3.eth().storage(kernel_address, U256::from_big_endian(&proc_pointer.get_cap_type_len_ptr(cap_type)), None).wait().expect("proc key raw").to_fixed_bytes());
    let mut caps = Vec::new();
    for i in 0..(n_caps.as_u64() as u8 ) {
        let mut caps_reader = CapReader {
            conn: conn,
            kernel_address,
            proc_pointer: proc_pointer.clone(),
            cap_type: cap_type,
            cap_index: i,
            current_val: 0,
        };
        let procedure = ProcedureEntryCap::deserialize(&mut caps_reader);
        caps.push(procedure.unwrap());
    }
    caps
}

fn parse_store_write_caps<T: Transport>(conn: &EthConn<T>, kernel_address: Address, proc_pointer: ProcPointer) -> Vec<StoreWriteCap> {
    let cap_type: u8 = CAP_STORE_WRITE;
    let n_caps = U256::from_big_endian(&conn.web3.eth().storage(kernel_address, U256::from_big_endian(&proc_pointer.get_cap_type_len_ptr(cap_type)), None).wait().expect("proc key raw").to_fixed_bytes());
    let mut caps = Vec::new();
    for i in 0..(n_caps.as_u64() as u8 ) {
        let mut caps_reader = CapReader {
            conn: conn,
            kernel_address,
            proc_pointer: proc_pointer.clone(),
            cap_type: cap_type,
            cap_index: i,
            current_val: 0,
        };
        let procedure = StoreWriteCap::deserialize(&mut caps_reader);
        caps.push(procedure.unwrap());
    }
    caps
}

fn parse_log_caps<T: Transport>(conn: &EthConn<T>, kernel_address: Address, proc_pointer: ProcPointer) -> Vec<LogCap> {
    let cap_type: u8 = CAP_LOG;
    let n_caps = U256::from_big_endian(&conn.web3.eth().storage(kernel_address, U256::from_big_endian(&proc_pointer.get_cap_type_len_ptr(cap_type)), None).wait().expect("proc key raw").to_fixed_bytes());
    let mut caps = Vec::new();
    for i in 0..(n_caps.as_u64() as u8 ) {
        let mut caps_reader = CapReader {
            conn: conn,
            kernel_address,
            proc_pointer: proc_pointer.clone(),
            cap_type: cap_type,
            cap_index: i,
            current_val: 0,
        };
        let procedure = LogCap::deserialize(&mut caps_reader);
        caps.push(procedure.unwrap());
    }
    caps
}

fn parse_account_call_caps<T: Transport>(conn: &EthConn<T>, kernel_address: Address, proc_pointer: ProcPointer) -> Vec<AccountCallCap> {
    let cap_type: u8 = CAP_ACC_CALL;
    let n_caps = U256::from_big_endian(&conn.web3.eth().storage(kernel_address, U256::from_big_endian(&proc_pointer.get_cap_type_len_ptr(cap_type)), None).wait().expect("proc key raw").to_fixed_bytes());
    let mut caps = Vec::new();
    for i in 0..(n_caps.as_u64() as u8 ) {
        let mut caps_reader = CapReader {
            conn: conn,
            kernel_address,
            proc_pointer: proc_pointer.clone(),
            cap_type: cap_type,
            cap_index: i,
            current_val: 0,
        };
        let procedure = AccountCallCap::deserialize(&mut caps_reader);
        caps.push(procedure.unwrap());
    }
    caps
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
pub struct Procedure {
    pub key: [u8; 24],
    pub index: U256,
    pub address: Address,
    pub caps: Capabilities,
}

impl fmt::Display for Procedure {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let key_hex: String = self.key.to_hex();
        let key_utf8: &str = std::str::from_utf8(&self.key).unwrap().trim_end_matches('\0');
        write!(f, "Procedure[{}]: 0x{} (\"{}\")\n  Address: {:?}\n  Caps({}):\n{}",
            self.index.as_u64(), key_hex, key_utf8, self.address, self.caps.len(), self.caps)
    }
}

fn get_idx_proc_address(i: u64) -> U256 {
    let idx: u8 = i as u8;
    U256::from_big_endian(&[0xff, 0xff, 0xff, 0xff, 0x01, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, idx, 0x00, 0x00, 0x00])
}
