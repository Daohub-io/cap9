extern crate clap;
extern crate web3;
extern crate rustc_hex;
extern crate std;

use web3::futures::Future;
use web3::contract::{Contract, Options};
use web3::types::{Address, U256, H256};
// use web3::types::TransactionReceipt;
use web3::Transport;
use rustc_hex::FromHex;
use rustc_hex::ToHex;
// use ethabi::Token::Uint;
use crate::conn;
use crate::conn::EthConn;
use crate::project::*;
use cap9_std::proc_table::cap::*;
use pwasm_abi;
use std::fs::File;
use crate::deploy::web3::contract::tokens::Tokenize;
use ethabi;

const REQ_CONFIRMATIONS: usize = 0;

pub fn string_to_proc_key(mut name: String) -> [u8; 24] {
    if !name.is_ascii() {
        println!("{}", name);
        panic!("name is not ascii");
    }
    if name.len() > 24 {
        println!("{}", name);
        panic!("name ({}) is greater than 24 characters, it is {} characters", name, name.len());
    }
    name.push_str("\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0");
    name.truncate(24);
    let mut procedure_key : [u8; 24] = [0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0];
    let byte_name = name.into_bytes();
    procedure_key.clone_from_slice(&byte_name[..24]);
    procedure_key
}

pub fn proc_key_to_string<'a>(key: &'a [u8]) -> &'a str {
    std::str::from_utf8(key).unwrap().trim_end_matches('\0')
}

pub fn proc_key_to_32_bytes(proc_key: &[u8; 24]) -> [u8; 32] {
    let mut buf = [0; 32];
    buf[8..].copy_from_slice(proc_key);
    buf
}

pub fn deploy_kernel<T: Transport>(conn:  &EthConn<T>, local_project: &mut LocalProject) -> (Contract<T>, Contract<T>) {
    // Deploy initial procedure
    let init_contract = deploy_contract(conn, include_bytes!("acl_bootstrap.wasm").to_vec(), include_bytes!("ACLBootstrapInterface.json"));
    // println!("init_contract: {:?}", init_contract);
    let deploy_file = local_project.deploy_file();
    // Deploying a kernel instance
    let kernel_code: &Vec<u8> = &deploy_file.kernel_code.bytes;
    let proc_key = String::from("init");
    let proc_address = init_contract.address();
    let empty_key = string_to_proc_key("".to_string());
    let entry_caps: Vec<NewCapability> = vec![
            NewCapability {
                cap: Capability::ProcedureRegister(ProcedureRegisterCap {
                    prefix: 0,
                    key: empty_key,
                }),
                parent_index: 0,
            },
            NewCapability {
                cap: Capability::ProcedureRegister(ProcedureRegisterCap {
                    prefix: 0,
                    key: empty_key,
                }),
                parent_index: 0,
            },
            NewCapability {
                cap: Capability::ProcedureCall(ProcedureCallCap {
                    prefix: 0,
                    key: empty_key,
                }),
                parent_index: 0,
            },
            NewCapability {
                cap: Capability::ProcedureDelete(ProcedureDeleteCap {
                    prefix: 0,
                    key: empty_key,
                }),
                parent_index: 0,
            },
            // TODO: it might be worth warning about overlapping caps
            NewCapability {
                cap: Capability::StoreWrite(StoreWriteCap {
                    location: [0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00],
                    size:     [0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xfe],
                }),
                parent_index: 0,
            },
            NewCapability {
                cap: Capability::StoreWrite(StoreWriteCap {
                    location: [0x30, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00],
                    size:     [0xaa, 0xaa, 0xaa, 0xaa, 0xaa, 0xaa, 0xaa, 0xaa, 0xaa, 0xaa, 0xaa, 0xaa, 0xaa, 0xaa, 0xaa, 0xaa, 0xaa, 0xaa, 0xaa, 0xaa, 0xaa, 0xaa, 0xaa, 0xaa, 0xaa, 0xaa, 0xaa, 0xaa, 0xaa, 0xaa, 0xaa, 0xaa],
                }),
                parent_index: 0,
            },
            NewCapability {
                cap: Capability::ProcedureEntry(ProcedureEntryCap),
                parent_index: 0,
            },
        ];

    let cap_list: NewCapList = NewCapList(entry_caps.clone());
    let encoded_cap_list: Vec<U256> = from_common_u256_vec(cap_list.to_u256_list());

    let code_hex: String = kernel_code.clone().to_hex();
    // let (kernel_contract, kernel_receipt) = Contract::deploy(conn.web3.eth(), include_bytes!("KernelInterface.json"))
    let kernel_contract = Contract::deploy(conn.web3.eth(), include_bytes!("KernelInterface.json"))
            .expect("deploy construction failed")
            .confirmations(REQ_CONFIRMATIONS)
            .options(Options::with(|opt| {
                opt.gas = Some(200_800_000.into())
            }))
            .execute(
                code_hex,
                (proc_key, proc_address, encoded_cap_list),
                conn.sender,
            )
            .expect("Correct parameters are passed to the constructor.")
            .wait()
            .expect("deployment failed");
    println!("Kernel Instance Address: {:?}", kernel_contract.address());
    let web3::types::Bytes(code_vec_kernel)= conn.web3.eth().code(kernel_contract.address(), None).wait().unwrap();
    println!("Kernel Code Length: {:?}", code_vec_kernel.len());
    // println!("Kernel Gas Used (Deployment): {:?}", kernel_receipt.gas_used);
    // if kernel_receipt.status != Some(web3::types::U64::one()) {
    //     panic!("Kernel Contract deployment failed!");
    // }

    let proxied_init_contract = web3::contract::Contract::from_json(
            conn.web3.eth(),
            kernel_contract.address(),
            include_bytes!("ACLBootstrapInterface.json"),
        ).expect("proxied_init_contract");



    let entry_contract = deploy_contract(conn, include_bytes!("acl_entry.wasm").to_vec(), include_bytes!("ACLEntryInterface.json"));
    let admin_contract = deploy_contract(conn, include_bytes!("acl_admin.wasm").to_vec(), include_bytes!("ACLAdminInterface.json"));
    let entry_key: U256 = proc_key_to_32_bytes(&string_to_proc_key("entry".to_string())).into();
    let admin_key: U256 = proc_key_to_32_bytes(&string_to_proc_key("admin".to_string())).into();
    let prefix = 0;
    let cap_key = string_to_proc_key("write".to_string());
    let caps: Vec<NewCapability> = vec![
            NewCapability {
                cap: Capability::ProcedureRegister(ProcedureRegisterCap {
                    prefix,
                    key: cap_key,
                }),
                parent_index: 0,
            },
            NewCapability {
                cap: Capability::ProcedureRegister(ProcedureRegisterCap {
                    prefix,
                    key: cap_key,
                }),
                parent_index: 1,
            },
            NewCapability {
                cap: Capability::ProcedureCall(ProcedureCallCap {
                    prefix,
                    key: cap_key,
                }),
                parent_index: 0,
            },
            NewCapability {
                cap: Capability::ProcedureDelete(ProcedureDeleteCap {
                    prefix,
                    key: cap_key,
                }),
                parent_index: 0,
            },
            NewCapability {
                cap: Capability::StoreWrite(StoreWriteCap {
                    location: [0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00],
                    size:     [0x10, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00],
                }),
                parent_index: 0,
            },
            NewCapability {
                cap: Capability::StoreWrite(StoreWriteCap {
                    location: [0x30, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00],
                    size:     [0x10, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00],
                }),
                parent_index: 1,
            },
            NewCapability {
                cap: Capability::ProcedureEntry(ProcedureEntryCap),
                parent_index: 0,
            },
        ];

    let encoded_cap_list_entry: NewCapList = NewCapList(caps.clone());
    let encoded_cap_list_admin: NewCapList = NewCapList(caps.clone());

    // Be wary of conflicting U256 types
    let encoded_cap_list_entry_u256: Vec<U256> = from_common_u256_vec(encoded_cap_list_entry.to_u256_list());
    let encoded_cap_list_admin_u256: Vec<U256> = from_common_u256_vec(encoded_cap_list_admin.to_u256_list());


    for i in &encoded_cap_list_admin_u256 {
        println!("entry_caps: {:?}", i);
    }
    let main_account = &conn.sender;

    {
        let entry_proc_address: U256 = U256::from_big_endian(&[0xff, 0xff, 0xff, 0xff, 0x04, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00]);
        // println!("EntryProcAddress: 0x{}", entry_proc_address.to_hex());
        let store_val = conn.web3.eth().storage(kernel_contract.address(), entry_proc_address, None).wait();
        println!("EntryProc: {:?}", store_val);
    }
    {
        let storage_address: U256 = U256::from_big_endian(&[0xff, 0xff, 0xff, 0xff, 0x00, 0x45, 0x6e, 0x74, 0x72, 0x79, 0x50, 0x72, 0x6f, 0x63, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00]);
        let storage_value = conn.web3.eth().storage(kernel_contract.address(), storage_address, None).wait();
        println!("EntryProcAddress: {:?}", storage_value);
    }
    {
        let entry_proc_address: U256 = U256::from_big_endian(&[0xff, 0xff, 0xff, 0xff, 0x01, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00]);
        let store_val2 = conn.web3.eth().storage(kernel_contract.address(), entry_proc_address, None).wait();
        println!("N Procs: {:?}", store_val2);
    }

    println!("entry_key: 0x{:?}", entry_key);
    println!("entry_address: {:?}", entry_contract.address());
    println!("admin_key: 0x{:?}", admin_key);
    println!("admin_address: {:?}", admin_contract.address());
    println!("main_account: {:?}", main_account);


    // Initialise ACL via bootstrap procedure.
    let res = proxied_init_contract.call("init", (
            entry_key, // entry key
            entry_contract.address(), // entry address
            encoded_cap_list_entry_u256, // entry cap list
            admin_key, // admin key
            admin_contract.address(), // admin address
            encoded_cap_list_admin_u256, // admin cap list
            main_account.clone() // admin account
        ), conn.sender,
        Options::with(|opts| {
            opts.gas = Some(550_621_180.into());
        }),
        ).wait().expect("ACL init");
    println!("res: {:?}", res);

    let init_receipt = conn.web3.eth().transaction_receipt(res).wait().expect("init receipt").unwrap();
    println!("Init Receipt: {:?}", init_receipt);


    if init_receipt.status != Some(web3::types::U64::one()) {
        panic!("ACL init failed!");
    }

    // Add a group
    let proc_name = "randomProcName".to_string();
    let proc_key = string_to_proc_key(proc_name);
    let contract_name = "acl_group_5";
    let contract_abi_name = "ACLGroup5Interface";
    let cap_index = 0;

    let contract = deploy_contract(&conn, include_bytes!("acl_group_5.wasm").to_vec(), include_bytes!("ACLGroup5Interface.json"));
    let cap_list: Vec<U256> = vec![];
    // let message = admin_contract.methods.regProc(cap_index, proc_key, contract.address, encodedRequestedCaps).encodeABI();
    // let proxy_message = tester.interface.methods.proxy(message).encodeABI();
    // await web3.eth.sendTransaction({ to: tester.kernel.contract.address, data: proxy_message, gas:2_100_000});
    // regInterface = contract;

    let proxied_admin_contract = web3::contract::Contract::from_json(
            conn.web3.eth(),
            kernel_contract.address(),
            include_bytes!("ACLAdminInterface.json"),
        ).expect("proxied_init_contract");

    let encoded_proc_key: U256 = proc_key_to_32_bytes(&proc_key).into();

    let params = (
            cap_index,
            encoded_proc_key,
            contract.address(),
            cap_list,
        );
    // Register the procedure
    let file: &[u8] = include_bytes!("ACLAdminInterface.json");
    let admin_abi = ethabi::Contract::load(file).expect("no ABI");
    let message: Vec<u8> = admin_abi
            .function("regProc")
            .and_then(|function| function.encode_input(params.into_tokens().as_slice())).expect("message encoding failed");
    println!("message: {:?}", message);
    let proxied_entry_contract = web3::contract::Contract::from_json(
            conn.web3.eth(),
            kernel_contract.address(),
            include_bytes!("ACLEntryInterface.json"),
        ).expect("proxied_entry_contract");


    let res: U256 = proxied_entry_contract.query("n_accounts", ( ), conn.sender,
            Options::with(|opts| {
                opts.gas = Some(550_621_180.into());
            }),
            None,
            ).wait().expect("proxy");
    println!("n_accounts: {:?}", res);
    let res: U256 = proxied_entry_contract.query("get_account_group", ( conn.sender ), conn.sender,
            Options::with(|opts| {
                opts.gas = Some(550_621_180.into());
            }),
            None,
            ).wait().expect("proxy");
    println!("our account group: {:?}", res);
    let res: U256 = proxied_entry_contract.query("get_group_procedure", ( res ), conn.sender,
            Options::with(|opts| {
                opts.gas = Some(550_621_180.into());
            }),
            None,
            ).wait().expect("proxy");
    println!("our proc: {:?}", res);
    let res = proxied_entry_contract.call("proxy", (
            message,
        ), conn.sender,
        Options::with(|opts| {
            opts.gas = Some(550_621_180.into());
        }),
        ).wait().expect("proxy");
    println!("res: {:?}", res);
    let reg_receipt = conn.web3.eth().transaction_receipt(res).wait().expect("reg receipt").unwrap();
    println!("Register Group 5 Procedure Receipt: {:?}", reg_receipt);
    if reg_receipt.status != Some(web3::types::U64::one()) {
        panic!("ACL register proc failed!");
    }
    // use the kernel address as the test account
    let test_account = kernel_contract.address().clone();

    let new_group_params = (
        test_account,
        U256::from(5),
    );
    let new_group_message: Vec<u8> = admin_abi
            .function("set_account_group")
            .and_then(|function| function.encode_input(new_group_params.into_tokens().as_slice())).expect("message encoding failed");
    let res = proxied_entry_contract.call("proxy", (
            new_group_message,
        ), conn.sender,
        Options::with(|opts| {
            opts.gas = Some(550_621_180.into());
        }),
        ).wait().expect("proxy");
    let new_group_receipt = conn.web3.eth().transaction_receipt(res).wait().expect("new_group receipt").unwrap();
    println!("New Group Receipt: {:?}", new_group_receipt);
    if new_group_receipt.status != Some(web3::types::U64::one()) {
        panic!("ACL register proc failed!");
    }

    let new_group_params = (
        U256::from(5),
        encoded_proc_key,
    );
    let new_group_message: Vec<u8> = admin_abi
            .function("set_group_procedure")
            .and_then(|function| function.encode_input(new_group_params.into_tokens().as_slice())).expect("message encoding failed");
    let res = proxied_entry_contract.call("proxy", (
            new_group_message,
        ), conn.sender,
        Options::with(|opts| {
            opts.gas = Some(550_621_180.into());
        }),
        ).wait().expect("proxy");
    let new_group_receipt = conn.web3.eth().transaction_receipt(res).wait().expect("new_group receipt").unwrap();
    println!("New Group Receipt: {:?}", new_group_receipt);
    if new_group_receipt.status != Some(web3::types::U64::one()) {
        panic!("ACL register proc failed!");
    }
    //         // Create a procedure for Group 5
    //         {
    //             const m1 = admin_contract.methods.set_group_procedure(5, proc_key).encodeABI();
    //             const pm1 = tester.interface.methods.proxy(m1).encodeABI();
    //             await web3.eth.sendTransaction({
    //                 from: mainAccount,
    //                 to: tester.interface.address,
    //                 data: pm1,
    //                 gas: 6_000_000,
    //             });
    //         }
    //         // Add testAccount to Group 5
    //         {
    //             const m1 = admin_contract.methods.set_account_group(testAccount, 5).encodeABI();
    //             const pm1 = tester.interface.methods.proxy(m1).encodeABI();
    //             await web3.eth.sendTransaction({
    //                 from: mainAccount,
    //                 to: tester.interface.address,
    //                 data: pm1,
    //                 gas:2_200_000,
    //             });
    //         }



    let entry_proc_address: U256 = U256::from_big_endian(&[0xff, 0xff, 0xff, 0xff, 0x02, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00]);
    println!("EntryProcAddress: 0x{:x?}", entry_proc_address);
    let store_val = conn.web3.eth().storage(kernel_contract.address(), entry_proc_address, None).wait();
    println!("EntryProc: {:?}", store_val);

    let keys: Vec<H256> = serde_json::value::from_value(conn::list_storage_keys(kernel_contract.address()).result.unwrap()).unwrap();
    for key in keys {
        let val = conn.web3.eth().storage(kernel_contract.address(), key.as_fixed_bytes().into(), None).wait().expect("storage value");
        println!("key: {:?}, val: {:?}", key, val);
    }

    local_project.add_status_file(kernel_contract.address());

    (proxied_init_contract, kernel_contract)
}


/// Convert one U256 of the ABI type a U256 of the web3 library type (the web3
/// library and the ABI use different U256s).
pub fn from_common_u256(u: pwasm_abi::types::U256) -> U256 {
    let mut buf = [0; 32];
    u.to_little_endian(&mut buf);
    U256::from_little_endian(&buf)
}

pub fn to_common_u256(u: U256) -> pwasm_abi::types::U256 {
    let mut buf = [0; 32];
    u.to_big_endian(&mut buf);
    pwasm_abi::types::U256::from_big_endian(&buf)
}

pub fn to_common_h256(h: H256) -> pwasm_abi::types::H256 {
    let buf = h.as_fixed_bytes();
    pwasm_abi::types::H256::from_slice(buf)
}

pub fn from_common_address(a: pwasm_abi::types::Address) -> Address {
    let buf = a.as_fixed_bytes();
    Address::from_slice(buf)
}

pub fn to_common_address(a: Address) -> pwasm_abi::types::Address {
    let buf = a.as_fixed_bytes();
    pwasm_abi::types::Address::from_slice(buf)
}


/// Convert a vector of U256 of the ABI type a U256 of the web3 library type
/// (the web3 library and the ABI use different U256s).
fn from_common_u256_vec(v: Vec<pwasm_abi::types::U256>) -> Vec<U256> {
    let mut new_v = Vec::new();
    for n in v {
        new_v.push(from_common_u256(n))
    }
    new_v
}

// Deploy a contract
pub fn deploy_contract<T: Transport>(conn:  &EthConn<T>, code: Vec<u8>, interface: &[u8]) -> Contract<T> {
    println!("Deploying contract");
    conn.web3.personal().unlock_account(conn.sender, "user", None).wait().unwrap();
    // let (contract, receipt) = Contract::deploy(conn.web3.eth(), interface)
    let code_hex: String = code.to_hex();
    let contract = Contract::deploy(conn.web3.eth(), interface)
            .expect("deploy construction failed")
            .confirmations(REQ_CONFIRMATIONS)
            .options(Options::with(|opt| {
                opt.gas = Some(200_800_000.into())
            }))
            .execute(
                code_hex,
                ( ),
                conn.sender,
            )
            .expect("Correct parameters are passed to the constructor.")
            .wait()
            .expect("deployment failed");
    println!("Contract Address: {:?}", contract.address());
    let web3::types::Bytes(code_vec_kernel)= conn.web3.eth().code(contract.address(), None).wait().unwrap();
    println!("Code Length: {:?}", code_vec_kernel.len());
    // println!("Gas Used (Deployment): {:?}", receipt.gas_used);
    // println!("Receipt: {:?}", receipt);
    // if receipt.status != Some(web3::types::U64::one()) {
    //     panic!("Contract deployment failed!");
    // }
    contract
}
