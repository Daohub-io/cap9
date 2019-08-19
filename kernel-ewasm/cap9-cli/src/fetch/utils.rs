
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

pub fn h256_to_u256(h: H256) -> U256 {
    U256::from_big_endian(&h.to_fixed_bytes())
}

pub fn u256_to_h256(u: U256) -> H256 {
    let mut buf: [u8; 32] = [0; 32];
    u.to_big_endian(&mut buf);
    H256::from_slice(&buf)
}
