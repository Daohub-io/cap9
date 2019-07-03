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
    use cap9_std::proc_table::cap::*;
    use cap9_std::syscalls::*;

    #[eth_abi(ACLAdminEndpoint)]
pub trait ACLAdminInterface {
        /// The constructor set with Initial Entry Procedure
        fn constructor(&mut self);

        fn set_group_procedure(&mut self, group_id: U256, proc_key: H256);

        fn get_group_procedure(&mut self, group_id: U256) -> H256;

        fn set_account_group(&mut self, account: Address, group_id: U256);

        fn get_account_group(&mut self, account: Address) -> U256;

        fn regProc(&mut self, cap_idx: U256, key: H256, address: Address, cap_list: Vec<H256>);

        fn listProcs(&mut self) -> Vec<H256>;

        fn getCap(&mut self, cap_type: U256, cap_index: U256) -> (U256, U256);

        fn getNCaps(&mut self, key: H256) -> u64;

        fn proxy(&mut self, payload: Vec<u8>);

    }

    pub struct ACLContract;

    impl ACLAdminInterface for ACLContract {

        fn constructor(&mut self) {}

        fn set_group_procedure(&mut self, group_id: U256, proc_key: H256) {
            // This relies on a mapping of groups -> procedures. Therefore we
            // need a map mechanism. Here we will just create the mechanism each
            // time at the same address.
            let mut procecedure_map: cap9_std::StorageEnumerableMap<u8,cap9_std::SysCallProcedureKey> = cap9_std::StorageEnumerableMap::from(0);
            procecedure_map.insert(group_id.as_u32() as u8, proc_key.into());
        }

        fn get_group_procedure(&mut self, group_id: U256) -> H256 {
            // This relies on a mapping of groups -> procedures. Therefore we
            // need a map mechanism. Here we will just create the mechanism each
            // time at the same address.
            let procecedure_map: cap9_std::StorageEnumerableMap<u8,cap9_std::SysCallProcedureKey> = cap9_std::StorageEnumerableMap::from(0);
            match procecedure_map.get(group_id.as_u32() as u8) {
                Some(x) => x.into(),
                None => H256::zero(),
            }
        }

        fn set_account_group(&mut self, account: Address, group_id: U256) {
            let mut procecedure_map: cap9_std::StorageEnumerableMap<Address, u8> = cap9_std::StorageEnumerableMap::from(0);
            procecedure_map.insert(account, group_id.as_u32() as u8);
        }

        fn get_account_group(&mut self, account: Address) -> U256 {
            let procecedure_map: cap9_std::StorageEnumerableMap<Address, u8> = cap9_std::StorageEnumerableMap::from(0);
            match procecedure_map.get(account) {
                Some(x) => x.into(),
                None => U256::zero(),
            }
        }

        fn regProc(&mut self, cap_idx: U256, key: H256, address: Address, cap_list: Vec<H256>) {
            cap9_std::reg(cap_idx.as_u32() as u8, key.into(), address, cap_list).unwrap();
            pwasm_ethereum::ret(&cap9_std::result());
        }

        fn listProcs(&mut self) -> Vec<H256> {
            let n_procs = cap9_std::proc_table::get_proc_list_len();
            let mut procs = Vec::new();
            for i in 1..(n_procs.as_usize() + 1) {
                let index = [0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,i as u8];
                procs.push(SysCallProcedureKey(cap9_std::proc_table::get_proc_id(index).unwrap()).into());
            }
            procs
        }

        fn getCap(&mut self, cap_type: U256, cap_index: U256) -> (U256, U256) {
            // Get the key of the currently executing procedure.
            let this_key: cap9_std::proc_table::ProcedureKey = cap9_std::proc_table::get_current_proc_id();
            let cap = cap9_std::proc_table::get_proc_cap(this_key, cap_type.as_u32() as u8, cap_index.as_u32() as u8).unwrap();
            match cap {
                Capability::ProcedureRegister(ProcedureRegisterCap {prefix, key}) => {
                    let h: H256 = SysCallProcedureKey(key).into();
                    (prefix.into(), h.into())
                },
                // ProcedureRegister(ProcedureRegisterCap),
                // ProcedureDelete(ProcedureDeleteCap),
                // ProcedureEntry(ProcedureEntryCap),
                // Capability::StoreWrite(StoreWriteCap {location, size}) => (location.into(), size.into()),
                // Log(LogCap),
                // AccountCall(AccountCallCap),
                _ => panic!("wrong cap")
            }
        }

        fn getNCaps(&mut self, key_raw: H256) -> u64 {
            let key: SysCallProcedureKey = key_raw.into();
            let proc_id: cap9_std::proc_table::ProcedureKey = key.into();
            let mut n_caps: u64 = 0;
            for i in &CAP_TYPES {
                let n: U256 = cap9_std::proc_table::get_proc_cap_list_len(proc_id.clone(), *i).into();
                n_caps += n.as_u64();
            }
            n_caps
        }

        /// The proxy function forwards the transaction to the procedure to
        /// which the sender belongs.
        fn proxy(&mut self, payload: Vec<u8>) {
            let sender = pwasm_ethereum::origin();
            let group_id = self.get_account_group(sender).as_u32() as u8;
            let procecedure_map: cap9_std::StorageEnumerableMap<u8,cap9_std::SysCallProcedureKey> = cap9_std::StorageEnumerableMap::from(0);
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
    let mut endpoint = ACL::ACLAdminEndpoint::new(ACL::ACLContract {});
    // Read http://solidity.readthedocs.io/en/develop/abi-spec.html#formal-specification-of-the-encoding for details
    pwasm_ethereum::ret(&endpoint.dispatch(&pwasm_ethereum::input()));
}

#[no_mangle]
pub fn deploy() {
    let mut endpoint = ACL::ACLAdminEndpoint::new(ACL::ACLContract {});
    endpoint.dispatch_ctor(&pwasm_ethereum::input());
}
