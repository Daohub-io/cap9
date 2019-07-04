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

    #[eth_abi(ACLEntryEndpoint)]
    pub trait ACLEntryInterface {
        /// The constructor set with Initial Entry Procedure
        fn constructor(&mut self);

        fn get_group_procedure(&mut self, group_id: U256) -> H256;

        fn get_account_group(&mut self, account: Address) -> U256;

        fn n_accounts(&mut self) -> U256;

        fn accounts(&mut self) -> Vec<Address>;

        fn procedures(&mut self) -> Vec<H256>;

        fn proxy(&mut self, payload: Vec<u8>);

    }

    pub struct ACLContract;

    impl ACLEntryInterface for ACLContract {

        fn constructor(&mut self) {}

        fn get_group_procedure(&mut self, group_id: U256) -> H256 {
            // This relies on a mapping of groups -> procedures. Therefore we
            // need a map mechanism. Here we will just create the mechanism each
            // time at the same address.
            let procecedure_map: cap9_std::StorageEnumerableMap<u8,cap9_std::SysCallProcedureKey> = cap9_std::StorageEnumerableMap::from(1);
            match procecedure_map.get(group_id.as_u32() as u8) {
                Some(x) => x.into(),
                None => H256::zero(),
            }
        }

        fn get_account_group(&mut self, account: Address) -> U256 {
            let procecedure_map: cap9_std::StorageEnumerableMap<Address, u8> = cap9_std::StorageEnumerableMap::from(0);
            match procecedure_map.get(account) {
                Some(x) => x.into(),
                None => U256::zero(),
            }
        }

        fn n_accounts(&mut self) -> U256 {
            let account_map: cap9_std::StorageEnumerableMap<Address, u8> = cap9_std::StorageEnumerableMap::from(0);
            account_map.length()
        }

        fn accounts(&mut self) -> Vec<Address> {
            let account_map: cap9_std::StorageEnumerableMap<Address, u8> = cap9_std::StorageEnumerableMap::from(0);
            let mut accounts = Vec::new();
            for account in account_map.keys() {
                accounts.push(account)
            }
            accounts
        }

        fn procedures(&mut self) -> Vec<H256> {
            let procedure_map: cap9_std::StorageEnumerableMap<u8, cap9_std::SysCallProcedureKey> = cap9_std::StorageEnumerableMap::from(1);
            let mut procedures: Vec<H256> = Vec::new();
            for (_group_id, procedure) in procedure_map.iter() {
                procedures.push(procedure.into())
            }
            procedures
        }

        /// The proxy function forwards the transaction to the procedure to
        /// which the sender belongs.
        fn proxy(&mut self, payload: Vec<u8>) {
            let sender = pwasm_ethereum::origin();
            let group_id = self.get_account_group(sender).as_u32() as u8;
            let procecedure_map: cap9_std::StorageEnumerableMap<u8,cap9_std::SysCallProcedureKey> = cap9_std::StorageEnumerableMap::from(1);
            let procedure_key = procecedure_map.get(group_id).unwrap();
            // Here the cap is hard coded. This procedure expects its first
            // procedure call capability to give it all the necessary
            // permissions.
            cap9_std::call(0_u8, procedure_key, payload).unwrap();
            pwasm_ethereum::ret(&cap9_std::result());
        }

    }
}
// Declares the dispatch and dispatch_ctor methods
use pwasm_abi::eth::EndpointInterface;

#[no_mangle]
pub fn call() {
    let mut endpoint = ACL::ACLEntryEndpoint::new(ACL::ACLContract {});
    // Read http://solidity.readthedocs.io/en/develop/abi-spec.html#formal-specification-of-the-encoding for details
    pwasm_ethereum::ret(&endpoint.dispatch(&pwasm_ethereum::input()));
}

#[no_mangle]
pub fn deploy() {
    let mut endpoint = ACL::ACLEntryEndpoint::new(ACL::ACLContract {});
    endpoint.dispatch_ctor(&pwasm_ethereum::input());
}
