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

// struct CapIndex(pub u8);

pub mod ACL {
    use pwasm_abi::types::*;
    use pwasm_abi_derive::eth_abi;
    use cap9_std;

    #[eth_abi(ACLBootstrapEndpoint)]
    pub trait ACLBootstrapInterface {
        fn constructor(&mut self);

        fn init(&mut self, entry_key: H256, entry_address: Address, entry_cap_list: Vec<H256>, admin_key: H256, admin_address: Address, admin_cap_list: Vec<H256>, admin_account: Address);

    }

    pub struct ACLContract;

    impl ACLBootstrapInterface for ACLContract {

        fn constructor(&mut self) {}

        fn init(&mut self, entry_key: H256, entry_address: Address, entry_cap_list: Vec<H256>, admin_key: H256, admin_address: Address, admin_cap_list: Vec<H256>, admin_account: Address) {
            // Register the admin procedure, uses RegCap 0
            cap9_std::reg(0, admin_key.into(), admin_address, admin_cap_list).unwrap();
            // Register the entry procedure
            cap9_std::reg(1, entry_key.into(), entry_address, entry_cap_list).unwrap();
            // Set the entry procedure
            cap9_std::entry(0, entry_key.into()).unwrap();
            // Add admin to the admin group (1)
            let admin_group: u8 = 1;
            let mut account_map: cap9_std::StorageEnumerableMap<Address, u8>
                = cap9_std::StorageEnumerableMap::from(0);
            account_map.insert(admin_account, admin_group);
            // Set the procedure of the admin group (1) to the admin procedure
            let mut procedure_map: cap9_std::StorageEnumerableMap<u8,cap9_std::SysCallProcedureKey>
                = cap9_std::StorageEnumerableMap::from(0);
            procedure_map.insert(admin_group, admin_key.into());
            // Unregister this bootstrap procedure, note that the contract will
            // not be reaped.
            let current_proc = cap9_std::proc_table::get_current_proc_id();
            cap9_std::delete(0, current_proc.into()).unwrap();
        }

    }
}
// Declares the dispatch and dispatch_ctor methods
use pwasm_abi::eth::EndpointInterface;

#[no_mangle]
pub fn call() {
    let mut endpoint = ACL::ACLBootstrapEndpoint::new(ACL::ACLContract {});
    // Read http://solidity.readthedocs.io/en/develop/abi-spec.html#formal-specification-of-the-encoding for details
    pwasm_ethereum::ret(&endpoint.dispatch(&pwasm_ethereum::input()));
}

#[no_mangle]
pub fn deploy() {
    let mut endpoint = ACL::ACLBootstrapEndpoint::new(ACL::ACLContract {});
    endpoint.dispatch_ctor(&pwasm_ethereum::input());
}
