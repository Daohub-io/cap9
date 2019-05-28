use pwasm_abi;
use pwasm_ethereum;
use pwasm_std;
use pwasm_abi_derive;

use pwasm_abi::types::*;
use pwasm_abi_derive::eth_abi;

// use validator::*;

pub fn check_contract(bytecode: &[u8]) -> bool {
    false
}

pub mod contract {
    use super::*;

    #[eth_abi(ValidatorEndpoint, ValidatorClient)]
    pub trait ValidatorInterface {
        /// Check if Procedure Contract is Valid
        fn check_contract(&mut self, _to: Address) -> bool;
    }

    pub struct ValidatorContract;

    impl ValidatorInterface for ValidatorContract {
        fn check_contract(&mut self, _target: Address) -> bool {
            unimplemented!()
        }
    }
}

#[cfg(test)]
#[allow(non_snake_case)]
mod tests {
    extern crate pwasm_test;
    extern crate std;

    use super::contract;
    use super::contract::*;

    use self::pwasm_test::{ext_get, ext_reset};
    use core::str::FromStr;
    use pwasm_abi::types::*;

    #[test]
    fn should_reject_invalid_address() {
        let mut contract = contract::ValidatorContract {};
        let owner_address = Address::from_str("ea674fdde714fd979de3edf0f56aa9716b898ec8").unwrap();
        let invalid_address = Address::from_str("0").unwrap();

        // Here we're creating an External context using ExternalBuilder and set the `sender` to the `owner_address`
        // so `pwasm_ethereum::sender()` in TokenInterface::constructor() will return that `owner_address`
        ext_reset(|e| e.sender(owner_address.clone()));
        assert_eq!(contract.check_contract(invalid_address), false);
    }

}
