#![no_std]
#![allow(non_snake_case)]
#![feature(proc_macro_hygiene)]

extern crate pwasm_abi;
extern crate pwasm_abi_derive;
extern crate pwasm_ethereum;
extern crate pwasm_std;

pub mod proc_table;
pub mod validator;

type ProcedureKey = [u8; 24];

pub mod token {
    use pwasm_abi::types::*;
    use pwasm_ethereum;

    // eth_abi is a procedural macros https://doc.rust-lang.org/book/first-edition/procedural-macros.html
    use pwasm_abi_derive::eth_abi;

    lazy_static::lazy_static! {
        static ref TOTAL_SUPPLY_KEY: H256 =
            H256::from(
                [2,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0]
            );
        static ref OWNER_KEY: H256 =
            H256::from(
                [3,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0]
            );
    }

    #[eth_abi(TokenEndpoint, KernelClient)]
    pub trait KernelInterface {
        /// The constructor set with Initial Entry Procedure
        fn constructor(&mut self, _entry_proc_key: String, _entry_proc_address: Address);
        /// Get Entry Procedure
        #[constant]
        fn entryProcedure(&mut self) -> String;
        /// Get Current Executing Procedure
        #[constant]
        fn currentProcedure(&mut self) -> String;

        /// Get Procedure Address By Key
        /// Returns 0 if Procedure Not Found
        fn getProcedureByKey(&mut self, _proc_key: String) -> Address;
    }

    pub struct KernelContract;

    impl KernelInterface for KernelContract {
        fn constructor(&mut self, _entry_proc_key: String, _entry_proc_address: Address) {
            // // Set up the total supply for the token
            // pwasm_ethereum::write(&TOTAL_SUPPLY_KEY, &total_supply.into());
            // // Give all tokens to the contract owner
            // pwasm_ethereum::write(&balance_key(&sender), &total_supply.into());
            // // Set the contract owner
            // pwasm_ethereum::write(&OWNER_KEY, &H256::from(sender).into());
            unimplemented!()
        }

        fn entryProcedure(&mut self) -> String {
            unimplemented!()
        }

        fn currentProcedure(&mut self) -> String {
            unimplemented!()
        }

        fn getProcedureByKey(&mut self, _proc_key: String) -> Address {
            unimplemented!()
        }
    }
}
// Declares the dispatch and dispatch_ctor methods
use pwasm_abi::eth::EndpointInterface;

#[no_mangle]
pub fn call() {
    let mut endpoint = token::TokenEndpoint::new(token::KernelContract {});
    // Read http://solidity.readthedocs.io/en/develop/abi-spec.html#formal-specification-of-the-encoding for details
    pwasm_ethereum::ret(&endpoint.dispatch(&pwasm_ethereum::input()));
}

#[no_mangle]
pub fn deploy() {
    let mut endpoint = token::TokenEndpoint::new(token::KernelContract {});
    endpoint.dispatch_ctor(&pwasm_ethereum::input());
}

#[cfg(test)]
#[allow(non_snake_case)]
mod tests {
    extern crate pwasm_test;
    extern crate std;
    use self::pwasm_test::{ext_get, ext_reset};
    use super::*;
    use core::str::FromStr;
    use pwasm_abi::types::*;
    use token::KernelInterface;

    #[test]
    fn should_initialize_with_entry_procedure() {
        let mut contract = token::KernelContract {};

        let owner_address = Address::from_str("ea674fdde714fd979de3edf0f56aa9716b898ec8").unwrap();
        let entry_proc_key = pwasm_abi::types::String::from("init");
        let entry_proc_address =
            Address::from_str("db6fd484cfa46eeeb73c71edee823e4812f9e2e1").unwrap();

        // Here we're creating an External context using ExternalBuilder and set the `sender` to the `owner_address`
        // so `pwasm_ethereum::sender()` in KernelInterface::constructor() will return that `owner_address`
        ext_reset(|e| e.sender(owner_address.clone()));

        contract.constructor(entry_proc_key.clone(), entry_proc_address.clone());

        assert_eq!(contract.entryProcedure(), entry_proc_key);
        assert_eq!(contract.currentProcedure(), unsafe {
            String::from_utf8_unchecked([0; 32].to_vec())
        });
    }

}
