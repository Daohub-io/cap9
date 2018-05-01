#![no_std]
#![allow(non_snake_case)]
#![feature(alloc)]
#![feature(proc_macro)]

extern crate parity_hash;
extern crate pwasm_std;
extern crate pwasm_ethereum;
extern crate alloc;
extern crate pwasm_abi;
extern crate pwasm_abi_derive;
/// Bigint used for 256-bit arithmetic
extern crate bigint;

pub mod kernel {
    use parity_hash::{H256, Address};
    use pwasm_ethereum;
    use pwasm_std;
    use bigint::U256;
    use pwasm_abi;
    use core;

    // eth_abi is a procedural macros https://doc.rust-lang.org/book/first-edition/procedural-macros.html
    use pwasm_abi_derive::eth_abi;
    use alloc::Vec;

    static KERNEL_VERSION: H256 = H256([0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,1]);
    static PROCEDURE_TABLE_ID: &[u8] = b"KERNEL_PROCEDURES"; 

    trait Store: Sized {
        fn store_write(&self, key: U256);
        fn store_read(key: U256) -> Self;
    }

    struct Table<T: Store> {
        namespace: U256,
        size: u8,
        _marker: core::marker::PhantomData<T>
    }

    impl <T: Store> Table<T> {
        fn open<N: Into<U256>>(namespace: N) -> Self {
            Table {
                namespace: namespace.into(),
                size: 0,
                _marker: core::marker::PhantomData
            }
        }

        fn set<K: Into<U256>>(&mut self, key: K, value: T) {
            // Mask Key
            let key: U256 = key.into() & self.namespace;
            // Store Value
            value.store_write(key);
        }

        fn get<K: Into<U256>>(&self, key: K) -> T {
            // Mask Key
            let key: U256 = key.into() & self.namespace;
            Store::store_read(key)
        }
    }

    struct ProcedureTable {
        table: Table<Procedure>
    }

    struct Procedure(U256);

    impl Store for Procedure {
        fn store_write(&self, key: U256) {
            pwasm_ethereum::write(&key.into(), &self.0.into());
        }
        fn store_read(key: U256) -> Self {
            let val = U256::from(pwasm_ethereum::read(&key.into()));
            Procedure(val)
        }
    }


    #[eth_abi(KernelEndpoint, KernelClient)]
    pub trait KernelContract {
        /// The constructor
        fn constructor(&mut self, _total_supply: U256);
        /// Total amount of kernels
        #[constant]
        fn version(&mut self) -> U256;

        #[constant]
        fn procedures(&mut self) -> Vec<U256>;

        // /// Create Procedure and push it to the procedure table
        fn create_procedure(&mut self, name: U256);

        #[event]
        fn Transfer(&mut self, indexed_from: Address, indexed_to: Address, _value: U256);
    }

    pub struct KernelContractInstance;

    impl KernelContract for KernelContractInstance {
        fn constructor(&mut self, total_supply: U256) {
            // Set up the total supply for the kernel
            pwasm_ethereum::write(&KERNEL_VERSION, &total_supply.into());
        }

        fn version(&mut self) -> U256 {
            pwasm_ethereum::read(&KERNEL_VERSION).into()
        }

        fn procedures(&mut self) -> Vec<U256> {
            let mut vec = Vec::with_capacity(10);
            vec.push(23.into());
            vec
        }

        fn create_procedure(&mut self, name: U256) {
            // Open Procedure Table
            let mut procedures = Table::open(PROCEDURE_TABLE_ID);
            procedures.set(name, Procedure(0.into()))
        }
    }
}
// Declares the dispatch and dispatch_ctor methods
use pwasm_abi::eth::EndpointInterface;

#[no_mangle]
pub fn call() {
    let mut endpoint = kernel::KernelEndpoint::new(kernel::KernelContractInstance{});
    // Read http://solidity.readthedocs.io/en/develop/abi-spec.html#formal-specification-of-the-encoding for details
    pwasm_ethereum::ret(&endpoint.dispatch(&pwasm_ethereum::input()));
}

#[no_mangle]
pub fn deploy() {
    let mut endpoint = kernel::KernelEndpoint::new(kernel::KernelContractInstance{});
    endpoint.dispatch_ctor(&pwasm_ethereum::input());
}
