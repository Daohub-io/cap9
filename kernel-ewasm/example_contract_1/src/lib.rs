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

extern "C" {

    pub fn somin() -> i32;
}

fn syscall() {
    pwasm_ethereum::call_code(pwasm_ethereum::gas_left(), &pwasm_ethereum::sender(), &[], &mut [] );
}

/// The call function is the main function of the *deployed* contract
#[no_mangle]
pub fn call() {
    // // write some value
    // pwasm_ethereum::write(&pwasm_std::types::H256::zero(), &[0,1,0,1,0,1,0,1,0,1,0,1,0,1,0,1,0,1,0,1,0,1,0,1,0,1,0,1,0,1,0,1]);
    // // call another contract
    // pwasm_ethereum::call(0, &Address::zero(), pwasm_std::types::U256::zero(), &[], &mut [] );
    // // delegate call another contract (under the hood this version of call_code
    // // uses delgate call).
    pwasm_ethereum::gas_left();
    pwasm_ethereum::sender();
    pwasm_ethereum::call_code(0, &Address::zero(), &[], &mut [] );
    // unsafe {
    //     somin();
    // }
    // syscall();
    // pwasm_ethereum::sender();
    // // Send a result pointer to the runtime
    pwasm_ethereum::ret(&b"result"[..]);
}
