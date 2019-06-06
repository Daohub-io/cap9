#![no_std]
#![allow(non_snake_case)]
#![feature(proc_macro_hygiene)]

extern crate pwasm_std;
extern crate pwasm_ethereum;
extern crate pwasm_abi;
extern crate pwasm_abi_derive;
extern crate parity_wasm;
extern crate validator;

use pwasm_abi::types::*;
use core::default::Default;

// pub mod validator;

pub mod ext {
    extern "C" {
            pub fn extcodesize( address: *const u8) -> i32;
            pub fn extcodecopy( dest: *mut u8, address: *const u8);
    }
}

pub fn extcodesize(address: &Address) -> i32 {
	unsafe { ext::extcodesize(address.as_ptr()) }
}

pub fn extcodecopy(address: &Address) -> pwasm_std::Vec<u8> {
	let len = unsafe { ext::extcodesize(address.as_ptr()) };
    match len {
		0 => pwasm_std::Vec::new(),
		non_zero => {
			let mut data = pwasm_std::Vec::with_capacity(non_zero as usize);
			unsafe {
				data.set_len(non_zero as usize);
				ext::extcodecopy(data.as_mut_ptr(), address.as_ptr());
			}
			data
		}
	}
}

pub mod token {
    use pwasm_ethereum;
    use pwasm_abi::types::*;
    // use parity_wasm::elements::{Module};
    use validator::{Validity};


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

    #[eth_abi(TokenEndpoint, TokenClient)]
    pub trait TokenInterface {
        /// The constructor
        fn constructor(&mut self, _total_supply: U256);
        /// Total amount of tokens
        #[constant]
        fn totalSupply(&mut self) -> U256;
        /// What is the balance of a particular account?
        #[constant]
        fn balanceOf(&mut self, _owner: Address) -> U256;
        /// Transfer the balance from owner's account to another account
        fn transfer(&mut self, _to: Address, _amount: U256) -> bool;
        /// Event declaration
        #[event]
        fn Transfer(&mut self, indexed_from: Address, indexed_to: Address, _value: U256);
        /// Check if Procedure Contract is Valid
        fn check_contract(&mut self, _to: Address) -> bool;
        /// Get the size (in bytes) of another contract
        fn get_code_size(&mut self, _to: Address) -> i32;
        /// Copy the code of another contract into memory
        fn code_copy(&mut self, _to: Address) -> pwasm_std::Vec<u8>;
    }

    pub struct TokenContract;

    impl TokenInterface for TokenContract {
        fn constructor(&mut self, total_supply: U256) {
            let sender = pwasm_ethereum::sender();
            // Set up the total supply for the token
            pwasm_ethereum::write(&TOTAL_SUPPLY_KEY, &total_supply.into());
            // Give all tokens to the contract owner
            pwasm_ethereum::write(&balance_key(&sender), &total_supply.into());
            // Set the contract owner
            pwasm_ethereum::write(&OWNER_KEY, &H256::from(sender).into());
        }

        fn totalSupply(&mut self) -> U256 {
            U256::from_big_endian(&pwasm_ethereum::read(&TOTAL_SUPPLY_KEY))
        }

        fn balanceOf(&mut self, owner: Address) -> U256 {
            read_balance_of(&owner)
        }

        fn transfer(&mut self, to: Address, amount: U256) -> bool {
            let sender = pwasm_ethereum::sender();
            let senderBalance = read_balance_of(&sender);
            let recipientBalance = read_balance_of(&to);
            if amount == 0.into() || senderBalance < amount || to == sender {
                false
            } else {
                let new_sender_balance = senderBalance - amount;
                let new_recipient_balance = recipientBalance + amount;
                pwasm_ethereum::write(&balance_key(&sender), &new_sender_balance.into());
                pwasm_ethereum::write(&balance_key(&to), &new_recipient_balance.into());
                self.Transfer(sender, to, amount);
                true
            }
        }

        fn check_contract(&mut self, target: Address) -> bool {
            // First we check if the target is the null address. If so we return
            // false.
            if target == H160::zero() {
                false
            } else {
                // Next we get the code of the contract, using EXTCODECOPY under
                // the hood.
                let code: pwasm_std::Vec<u8> = self.code_copy(target);
                // code_slice is magic number and version number only
                let code_slice = &[0, 97, 115, 109, 1, 0, 0, 0];
                // big_code_slice is magic number, version number and a simple
                // data section.
                let big_code_slice = &[0x00, 0x61, 0x73, 0x6D, 0x01, 0x00, 0x00, 0x00, 0x05, 0x03, 0x01, 0x00, 0x01, 0x0B, 0x07, 0x01, 0x00, 0x41, 0x01, 0x0B, 0x01, 0x54, 0x00, 0x08, 0x04, 0x6E, 0x61, 0x6D, 0x65, 0x02, 0x01, 0x00];
                // Next we deserialise the code from Vec<u8> into a Module.
                // let module: Module = match deserialize_buffer(code.as_slice()) {
                // // let module: Module = match deserialize_buffer(code_slice) {
                //     Ok(module) => module,
                //     // If we are unable to decode the contract, we assume it is
                //     // not valid, but for now we will panic for testing
                //     // purposes.
                //     Err(_) => panic!("invalid wasm module"),
                // };
                // // Then we perform a boolen is_valid() check.
                code.as_slice().is_valid();
                false
            }
        }

        fn get_code_size(&mut self, to: Address) -> i32 {
            super::extcodesize(&to)
        }

        fn code_copy(&mut self, to: Address) -> pwasm_std::Vec<u8> {
            super::extcodecopy(&to)
        }
    }

    // Reads balance by address
    fn read_balance_of(owner: &Address) -> U256 {
        U256::from_big_endian(&pwasm_ethereum::read(&balance_key(owner)))
    }

    // Generates a balance key for some address.
    // Used to map balances with their owners.
    fn balance_key(address: &Address) -> H256 {
        let mut key = H256::from(*address);
        key.as_bytes_mut()[0] = 1; // just a naive "namespace";
        key
    }
}
// Declares the dispatch and dispatch_ctor methods
use pwasm_abi::eth::EndpointInterface;

#[no_mangle]
pub fn call() {
    let mut endpoint = token::TokenEndpoint::new(token::TokenContract{});
    // Read http://solidity.readthedocs.io/en/develop/abi-spec.html#formal-specification-of-the-encoding for details
    pwasm_ethereum::ret(&endpoint.dispatch(&pwasm_ethereum::input()));
}

#[no_mangle]
pub fn deploy() {
    let mut endpoint = token::TokenEndpoint::new(token::TokenContract{});
    endpoint.dispatch_ctor(&pwasm_ethereum::input());
}

#[cfg(test)]
#[allow(non_snake_case)]
mod tests {
    extern crate pwasm_test;
    // extern crate std;
    use super::*;
    use core::str::FromStr;
    use pwasm_abi::types::*;
    use self::pwasm_test::{ext_reset, ext_get};
    use token::TokenInterface;

    #[test]
    fn should_succeed_transfering_1000_from_owner_to_another_address() {
        let mut contract = token::TokenContract{};
        let owner_address = Address::from_str("ea674fdde714fd979de3edf0f56aa9716b898ec8").unwrap();
        let sam_address = Address::from_str("db6fd484cfa46eeeb73c71edee823e4812f9e2e1").unwrap();
        // Here we're creating an External context using ExternalBuilder and set the `sender` to the `owner_address`
        // so `pwasm_ethereum::sender()` in TokenInterface::constructor() will return that `owner_address`
        ext_reset(|e| e.sender(owner_address.clone()));
        let total_supply = 10000.into();
        contract.constructor(total_supply);
        assert_eq!(contract.balanceOf(owner_address), total_supply);
        assert_eq!(contract.transfer(sam_address, 1000.into()), true);
        assert_eq!(contract.balanceOf(owner_address), 9000.into());
        assert_eq!(contract.balanceOf(sam_address), 1000.into());
        // 1 log entry should be created
        assert_eq!(ext_get().logs().len(), 1);
    }

    #[test]
    fn should_not_transfer_to_self() {
        let mut contract = token::TokenContract{};
        let owner_address = Address::from_str("ea674fdde714fd979de3edf0f56aa9716b898ec8").unwrap();
        ext_reset(|e| e.sender(owner_address.clone()));
        let total_supply = 10000.into();
        contract.constructor(total_supply);
        assert_eq!(contract.balanceOf(owner_address), total_supply);
        assert_eq!(contract.transfer(owner_address, 1000.into()), false);
        assert_eq!(contract.balanceOf(owner_address), 10000.into());
        assert_eq!(ext_get().logs().len(), 0);
    }

}
