#![cfg_attr(not(feature="std"), no_std)]

#![allow(non_snake_case)]

extern crate tiny_keccak;
extern crate pwasm_std;
extern crate pwasm_ethereum;
extern crate pwasm_abi;
extern crate pwasm_abi_derive;

use tiny_keccak::Keccak;
use pwasm_ethereum as eth;
use pwasm_abi::types::*;
use pwasm_abi_derive::eth_abi;


/// The call function is the main function of the *deployed* contract
#[no_mangle]
pub fn call() {
    // write some value
    pwasm_ethereum::write(&pwasm_std::types::H256::zero(), &[0,1,0,1,0,1,0,1,0,1,0,1,0,1,0,1,0,1,0,1,0,1,0,1,0,1,0,1,0,1,0,1]);
    // call another contract
    pwasm_ethereum::call(0, &Address::zero(), pwasm_std::types::U256::zero(), &[], &mut [] );
    // Send a result pointer to the runtime
    pwasm_ethereum::ret(&b"result"[..]);
}
