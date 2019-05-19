#![no_std]

extern crate pwasm_ethereum;
extern crate pwasm_abi;
extern crate pwasm_token_contract;

use pwasm_abi::eth::EndpointInterface;

#[no_mangle]
pub fn _call() {
	let mut endpoint = pwasm_token_contract::Endpoint::new(pwasm_token_contract::TokenContractInstance{});
	// Read http://solidity.readthedocs.io/en/develop/abi-spec.html#formal-specification-of-the-encoding for details
	pwasm_ethereum::ret(&endpoint.dispatch(&pwasm_ethereum::input()));
}

#[no_mangle]
pub fn _deploy() {
	let mut endpoint = pwasm_token_contract::Endpoint::new(pwasm_token_contract::TokenContractInstance{});
	endpoint.dispatch_ctor(&pwasm_ethereum::input());
}
