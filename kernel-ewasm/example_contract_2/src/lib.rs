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
use pwasm_ethereum::Error;

/// The call function is the main function of the *deployed* contract
#[no_mangle]
pub fn call() {
    let mut endpoint = contract::ExampleContract2Endpoint::new(contract::ExampleContract2{});
    pwasm_ethereum::ret(&endpoint.dispatch(&pwasm_ethereum::input()));
}

// Declares the dispatch and dispatch_ctor methods
use pwasm_abi::eth::EndpointInterface;

#[no_mangle]
pub fn deploy() {
    let mut endpoint = contract::ExampleContract2Endpoint::new(contract::ExampleContract2{});
    endpoint.dispatch_ctor(&pwasm_ethereum::input());
}


pub mod contract {
    use super::*;
    use pwasm_abi_derive::eth_abi;

    #[eth_abi(ExampleContract2Endpoint, ExampleContract2Client)]
    pub trait ExampleContract2Interface {
        /// Check if Procedure Contract is Valid
        fn check_contract(&mut self, _to: Address) -> bool;
    }

    pub struct ExampleContract2;

    impl ExampleContract2Interface for ExampleContract2 {
        fn check_contract(&mut self, _target: Address) -> bool {
            // unimplemented!()
            false
        }
    }
}
