
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
use crate::utils::{from_common_u256, to_common_u256, to_common_h256,
    from_common_address, to_common_address
};
use std::collections::{HashMap, HashSet};
use super::utils::*;
use super::kernel::*;
use super::map::*;

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
