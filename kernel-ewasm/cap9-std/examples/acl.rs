#![no_std]
#![allow(non_snake_case)]

extern crate cap9_std;
extern crate pwasm_std;
extern crate pwasm_abi_derive;

// When we are compiling to WASM, unresolved references are left as (import)
// expressions. However, under any other target symbols will have to be linked
// for EVM functions (blocknumber, create, etc.). Therefore, when we are not
// compiling for WASM (be it test, realse, whatever) we want to link in dummy
// functions. pwasm_test provides all the builtins provided by parity, while
// cap9_test covers the few that we have implemented ourselves.
#[cfg(not(target_arch = "wasm32"))]
extern crate pwasm_test;
#[cfg(not(target_arch = "wasm32"))]
extern crate cap9_test;

fn main() {}

pub mod ACL {
    use pwasm_abi::types::*;
    use pwasm_ethereum;
    use pwasm_abi_derive::eth_abi;
    use cap9_std;
    // use cap9_std::proc_table::*;
    use cap9_std::proc_table::cap::*;

    #[eth_abi(TestACLEndpoint, KernelClient)]
    pub trait TestACLInterface {
        /// The constructor set with Initial Entry Procedure
        fn constructor(&mut self);

        fn set_group_procedure(&mut self, group_id: U256, proc_key: H256);

        fn get_group_procedure(&mut self, group_id: U256) -> H256;

        fn set_account_group(&mut self, account: Address, group_id: U256);

        fn get_account_group(&mut self, account: Address) -> U256;

    }

    pub struct ACLContract;

    impl TestACLInterface for ACLContract {

        fn constructor(&mut self) {}

        fn set_group_procedure(&mut self, group_id: U256, proc_key: H256) {
            // This relies on a mapping of groups -> procedures. Therefore we
            // need a map mechanism. Here we will just create the mechanism each
            // time at the same address.
            let location: H256 = [
                0xaa, 0xaa, 0xaa, 0xaa, 0xaa, 0xaa, 0xaa, 0xaa,
                0xaa, 0xaa, 0xaa, 0xaa, 0xaa, 0xaa, 0xaa, 0xaa,
                0xaa, 0xaa, 0xaa, 0xaa, 0xaa, 0xaa, 0xaa, 0xaa,
                0xaa, 0xaa, 0xaa, 0xaa, 0xaa, 0xaa, 0xaa, 0xaa,
            ].into();
            let mut procecedure_map: cap9_std::BigMap<u8,cap9_std::SysCallProcedureKey> = cap9_std::BigMap::new(8, 1, 0);
            procecedure_map.insert(group_id.as_u32() as u8, proc_key.into());
        }

        fn get_group_procedure(&mut self, group_id: U256) -> H256 {
            // This relies on a mapping of groups -> procedures. Therefore we
            // need a map mechanism. Here we will just create the mechanism each
            // time at the same address.
            let location: H256 = [
                0xaa, 0xaa, 0xaa, 0xaa, 0xaa, 0xaa, 0xaa, 0xaa,
                0xaa, 0xaa, 0xaa, 0xaa, 0xaa, 0xaa, 0xaa, 0xaa,
                0xaa, 0xaa, 0xaa, 0xaa, 0xaa, 0xaa, 0xaa, 0xaa,
                0xaa, 0xaa, 0xaa, 0xaa, 0xaa, 0xaa, 0xaa, 0xaa,
            ].into();
            let mut procecedure_map: cap9_std::BigMap<u8,cap9_std::SysCallProcedureKey> = cap9_std::BigMap::new(8, 1, 0);
            match procecedure_map.get(group_id.as_u32() as u8) {
                Some(x) => x.into(),
                None => H256::zero(),
            }
        }

        fn set_account_group(&mut self, account: Address, group_id: U256) {
            let location: H256 = [
                0xbb, 0xbb, 0xbb, 0xbb, 0xbb, 0xbb, 0xbb, 0xbb,
                0xbb, 0xbb, 0xbb, 0xbb, 0xbb, 0xbb, 0xbb, 0xbb,
                0xbb, 0xbb, 0xbb, 0xbb, 0xbb, 0xbb, 0xbb, 0xbb,
                0xbb, 0xbb, 0xbb, 0xbb, 0xbb, 0xbb, 0xbb, 0xbb,
            ].into();
            let mut procecedure_map: cap9_std::BigMap<Address, u8> = cap9_std::BigMap::new(8, 1, 0);
            procecedure_map.insert(account, group_id.as_u32() as u8);
        }

        fn get_account_group(&mut self, account: Address) -> U256 {
            let location: H256 = [
                0xbb, 0xbb, 0xbb, 0xbb, 0xbb, 0xbb, 0xbb, 0xbb,
                0xbb, 0xbb, 0xbb, 0xbb, 0xbb, 0xbb, 0xbb, 0xbb,
                0xbb, 0xbb, 0xbb, 0xbb, 0xbb, 0xbb, 0xbb, 0xbb,
                0xbb, 0xbb, 0xbb, 0xbb, 0xbb, 0xbb, 0xbb, 0xbb,
            ].into();
            let mut procecedure_map: cap9_std::BigMap<Address, u8> = cap9_std::BigMap::new(8, 1, 0);
            match procecedure_map.get(account) {
                Some(x) => x.into(),
                None => U256::zero(),
            }
        }
    }
}
// Declares the dispatch and dispatch_ctor methods
use pwasm_abi::eth::EndpointInterface;

#[no_mangle]
pub fn call() {
    let mut endpoint = ACL::TestACLEndpoint::new(ACL::ACLContract {});
    // Read http://solidity.readthedocs.io/en/develop/abi-spec.html#formal-specification-of-the-encoding for details
    pwasm_ethereum::ret(&endpoint.dispatch(&pwasm_ethereum::input()));
}

#[no_mangle]
pub fn deploy() {
    let mut endpoint = ACL::TestACLEndpoint::new(ACL::ACLContract {});
    endpoint.dispatch_ctor(&pwasm_ethereum::input());
}
