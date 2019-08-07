extern crate clap;
extern crate web3;
extern crate rustc_hex;
extern crate ethabi;
extern crate std;

use web3::futures::Future;
use web3::contract::{Contract, Options};
use web3::types::{Address, U256};
// use web3::types::TransactionReceipt;
use web3::Transport;
// use rustc_hex::FromHex;
// use ethabi::Token::Uint;
use crate::conn::EthConn;
use crate::project::*;

const REQ_CONFIRMATIONS: usize = 0;

// pub fn string_to_proc_key(mut name: String) -> [u8; 24] {
//     if !name.is_ascii() {
//         println!("{}", name);
//         panic!("name is not ascii");
//     }
//     if name.len() > 24 {
//         println!("{}", name);
//         panic!("name ({}) is greater than 24 characters, it is {} characters", name, name.len());
//     }
//     name.push_str("\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0");
//     name.truncate(24);
//     let mut procedure_key : [u8; 24] = [0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0];
//     let byte_name = name.into_bytes();
//     procedure_key.clone_from_slice(&byte_name[..24]);
//     procedure_key
// }

// pub fn register_procedure<'a, T: Transport>(conn:  &'a EthConn<T>, kernel_contract: &'a Contract<T>, procedure_address: Address, name: String, caps : Vec<Cap>) -> Box<Future<Item = TransactionReceipt, Error = String>+'a> {
//     let caps_vals = caps_into_u256s(caps);
//     let procedure_key = string_to_proc_key(name);
//     let params = (procedure_key, procedure_address, caps_vals); // this is needed by the future

//     // Do a test run of the registration
//     let gg1: web3::contract::CallFuture<U256,_> = kernel_contract.estimate_gas("registerAnyProcedure", params.clone(), conn.sender, Options::default());
//     let gggg = kernel_contract.estimate_gas("registerAnyProcedure", params.clone(), conn.sender, Options::default()).map_err(|_| String::from("ss")).and_then(move |gas_estimate| {
//             let opts = Options::with(|opts| opts.gas = Some(gas_estimate));
//             // let query_result: Result<(ethabi::Token, Address),web3::contract::Error> = kernel_contract.query("registerAnyProcedure", params.clone(), Some(conn.sender), opts.clone(), Some(web3::types::BlockNumber::Latest)).wait();
//             let result = kernel_contract.call_with_confirmations("registerAnyProcedure", params.clone(), conn.sender, opts.clone(), REQ_CONFIRMATIONS);
//             // web3::futures::future::ok((query_result,result))
//             result.map_err(|_| String::from("ss"))
//         });
//     Box::new(gggg.and_then(|result| {
//         web3::futures::future::ok(result)
//     }))
// }

// pub fn deploy_example<T: Transport>(conn:  &EthConn<T>) {
//     // Deploy a kernel instance
//     let kernel_contract = deploy_kernel(conn);

//     // Deploying a contract and register it as a procedure
//     let caps: Vec<Cap> = vec![Cap::WriteCap{address: U256::from(0x8000), add_keys: U256::from(1)},Cap::LogCap(vec![])];
//     let p1 = deploy_register_procedure_f(conn, &kernel_contract, String::from("testName"), vec![]);
//     let p2 = deploy_register_procedure_f(conn, &kernel_contract, String::from("another one"), caps.clone());
//     let p3 = deploy_register_procedure_f(conn, &kernel_contract, String::from("member's procedure"), vec![Cap::WriteCap{address: U256::from(0x8000), add_keys: U256::from(1)},Cap::LogCap(vec![U256::from(0x41)]),Cap::CallCap(Vec::new()),Cap::LogCap(vec![U256::from(0x41),U256::from(0x123456)])]);
//     let p4 = deploy_register_procedure_f(conn, &kernel_contract, String::from("Bob's procedure"), caps.clone());
//     let p5 = deploy_register_procedure_f(conn, &kernel_contract, String::from("Jane's procedure"), caps.clone());
//     let ps = vec![p1,p2,p3,p4,p5];
//     web3::futures::future::join_all(ps).wait().map_err(|_| String::from("ss")).expect("Procedures deployed successfully");
//     kernel_contract.call("setEntryProcedure", (string_to_proc_key(String::from("member's procedure")),), conn.sender, Options::default()).wait().unwrap();
//     println!("Kernel Instance Address: {:?}", &kernel_contract.address());
// }

// pub fn deploy_big_example<T: Transport>(conn:  &EthConn<T>) {
//     // Deploy a kernel instance
//     let kernel_contract = deploy_kernel(conn);

//     // Deploying a contract and register it as a procedure
//     let caps: Vec<Cap> = vec![Cap::WriteCap{address: U256::from(0x8000), add_keys: U256::from(1)},Cap::LogCap(vec![])];

//     let p1 = deploy_register_procedure_f(conn, &kernel_contract, String::from("testName"), vec![]);
//     let p2 = deploy_register_procedure_f(conn, &kernel_contract, String::from("another one"), caps.clone());
//     let p3 = deploy_register_procedure_f(conn, &kernel_contract, String::from("member's procedure"), vec![Cap::WriteCap{address: U256::from(0x8000), add_keys: U256::from(1)},Cap::LogCap(vec![U256::from(0x41)]),Cap::CallCap(Vec::new()),Cap::LogCap(vec![U256::from(0x41),U256::from(0x123456)])]);
//     let p4 = deploy_register_procedure_f(conn, &kernel_contract, String::from("Bob's procedure"), caps.clone());
//     let p5 = deploy_register_procedure_f(conn, &kernel_contract, String::from("Jane's procedure"), caps.clone());
//     let n_procs = 250;
//     let mut ps = Vec::with_capacity(n_procs+5);
//     ps.push(p1);
//     ps.push(p2);
//     ps.push(p3);
//     ps.push(p4);
//     ps.push(p5);
//     for proc_num in 0..n_procs {
//         let n_caps = std::cmp::min(32,proc_num);
//         // let n_caps = proc_num;

//         let these_caps: Vec<Cap> = (0..n_caps).map(|cap_num| Cap::WriteCap{address: U256::from(0x8000+proc_num*n_caps+cap_num), add_keys: U256::from(1)}).collect();
//         println!("----------------------------------------------");
//         println!("Registering Procedure #{} with {} capabilities", proc_num, n_caps);
//         ps.push(deploy_register_procedure_f(conn, &kernel_contract, String::from(format!("Jane's proc #{}",proc_num)), these_caps));
//         println!("----------------------------------------------");
//     }
//     web3::futures::future::join_all(ps).wait().map_err(|_| String::from("ss")).expect("Procedures deployed successfully");
//     kernel_contract.call("setEntryProcedure", (string_to_proc_key(String::from("member's procedure")),), conn.sender, Options::default()).wait().unwrap();
//     println!("Kernel Instance Address: {:?}", &kernel_contract.address());
// }

// deploy_file.deploy_spec.initial_entry.bytes
pub fn deploy_kernel<T: Transport>(conn:  &EthConn<T>, deploy_file: DeployFile) -> Contract<T> {
    // Deploy initial procedure
    let init_contract = deploy_contract(conn, deploy_file.deploy_spec.initial_entry.bytes, include_bytes!("ACLBootstrapInterface.json"));
    println!("init_contract: {:?}", init_contract);
    // Deploying a kernel instance
    let kernel_code: Vec<u8> = deploy_file.kernel_code.bytes;
    let proc_key = String::from("EntryProc");
    let proc_address = Address::zero();
    let encoded_cap_list: Vec<U256> = vec![];
    let (kernel_contract, kernel_receipt) = Contract::deploy(conn.web3.eth(), include_bytes!("KernelInterface.json"))
            .expect("deploy construction failed")
            .confirmations(REQ_CONFIRMATIONS)
            .options(Options::with(|opt| {
                opt.gas = Some(5_700_000.into())
            }))
            .execute(
                kernel_code,
                (proc_key, proc_address, encoded_cap_list),
                conn.sender,
            )
            .expect("Correct parameters are passed to the constructor.")
            .wait()
            .expect("deployment failed");
    println!("Kernel Instance Address: {:?}", kernel_contract.address());
    let web3::types::Bytes(code_vec_kernel)= conn.web3.eth().code(kernel_contract.address(), None).wait().unwrap();
    println!("Kernel Code Length: {:?}", code_vec_kernel.len());
    println!("Kernel Gas Used (Deployment): {:?}", kernel_receipt.gas_used);

    kernel_contract
}

// Deploy a contract
pub fn deploy_contract<T: Transport>(conn:  &EthConn<T>, code: Vec<u8>, interface: &[u8]) -> Contract<T> {
    println!("Deploying contract");
    conn.web3.personal().unlock_account(conn.sender, "user", None).wait().unwrap();
    let bal = conn.web3.eth().balance(conn.sender, None).wait();
    println!("bal: {:?}", bal);
    let (contract, receipt) = Contract::deploy(conn.web3.eth(), interface)
            .expect("deploy construction failed")
            .confirmations(REQ_CONFIRMATIONS)
            .options(Options::with(|opt| {
                opt.gas = Some(5_700_000.into())
            }))
            .execute(
                code,
                ( ),
                conn.sender,
            )
            .expect("Correct parameters are passed to the constructor.")
            .wait()
            .expect("deployment failed");
    println!("Contract Address: {:?}", contract.address());
    let web3::types::Bytes(code_vec_kernel)= conn.web3.eth().code(contract.address(), None).wait().unwrap();
    println!("Code Length: {:?}", code_vec_kernel.len());
    println!("Gas Used (Deployment): {:?}", receipt.gas_used);

    contract
}

// pub fn deploy_proc<T: Transport>(conn:  &EthConn<T>, kernel_address: Address, proc_code_path: String, proc_abi_path: String, name: String) {
//     println!("about to deploy proc");
//     let kernel_abi = include_bytes!("../../Kernel/Kernel.abi");
//     let kernel_contract = match Contract::from_json(conn.web3.eth(), kernel_address, kernel_abi) {
//             Err(r) => {
//                 panic!("unable to build kernel contract: {:?}", r);
//             },
//             Ok(con) => con,
//         };
//     deploy_register_procedure(conn, &kernel_contract, name, vec![]).expect("Procedure deployed successfully");
// }

// pub fn deploy_register_procedure<T: Transport>(conn:  &EthConn<T>, kernel_contract: &Contract<T>, name: String, caps : Vec<Cap>) -> Result<TransactionReceipt,String> {
//     // Deploy the procedure
//     let example_code: Vec<u8> = include_str!("../../Adder/Adder.bin").from_hex().unwrap();
//         // Deploying a contract
//     let (example_contract, example_receipt) = Contract::deploy(conn.web3.eth(), include_bytes!("../../Adder/Adder.abi"))
//             .unwrap()
//             .confirmations(REQ_CONFIRMATIONS)
//             .options(Options::with(|opt| {
//                 opt.gas = Some(3_000_000.into())
//             }))
//             .execute(
//                 example_code,
//                 ( ),
//                 conn.sender,
//             )
//             .expect("Correct parameters are passed to the constructor.")
//             // If we pass this wait to the parent we can do faster batch jobs
//             .wait()
//             .unwrap();
//     println!("Procedure Address: {:?}", example_contract.address());
//     let web3::types::Bytes(code_vec_example)= conn.web3.eth().code(example_contract.address(), None).wait().expect("Procedure code should be retrieved");
//     println!("Procedure Code Length: {:?}", code_vec_example.len());
//     println!("Procedure Gas Used (Deployment): {:?}", example_receipt.gas_used);
//     register_procedure(conn, kernel_contract, example_contract.address(), name, caps).wait().map_err(|_| String::from("ss"))
// }

// pub fn deploy_register_procedure_f<'a, T: Transport>(conn:  &'a EthConn<T>, kernel_contract: &'a Contract<T>, name: String, caps : Vec<Cap>) -> Box<Future<Item = TransactionReceipt, Error = String>+'a> {
//     // Deploy the procedure
//     let example_code: Vec<u8> = include_str!("../../Adder/Adder.bin").from_hex().unwrap();
//         // Deploying a contract
//     Box::new(Contract::deploy(conn.web3.eth(), include_bytes!("../../Adder/Adder.abi"))
//             .unwrap()
//             .confirmations(REQ_CONFIRMATIONS)
//             .options(Options::with(|opt| {
//                 opt.gas = Some(3_000_000.into())
//             }))
//             .execute(
//                 example_code,
//                 ( ),
//                 conn.sender,
//             )
//             .expect("Correct parameters are passed to the constructor.")
//             // If we pass this wait to the parent we can do faster batch jobs
//             .map_err(|_| String::from("ss"))
//             .and_then(move |(example_contract,example_receipt)| {
//                 println!("Procedure Address: {:?}", example_contract.address());
//                 let web3::types::Bytes(code_vec_example)= conn.web3.eth().code(example_contract.address(), None).wait().expect("Procedure code should be retrieved");
//                 println!("Procedure Code Length: {:?}", code_vec_example.len());
//                 println!("Procedure Gas Used (Deployment): {:?}", example_receipt.gas_used);
//                 register_procedure(conn, kernel_contract, example_contract.address(), name, caps)
//             }))
// }

// #[derive(Clone)]
// pub enum Cap {
//     WriteCap {address: U256, add_keys: U256},
//     RegisterCap,
//     CallCap(Vec<U256>),
//     LogCap(Vec<U256>), // vec is of length 0-4
// }

// impl Cap {
//     fn to_u256s(&self) -> Vec<U256> {
//         match self {
//             Cap::WriteCap {address, add_keys} => vec![/* length */ U256::from(3), /* type */ U256::from(7),U256::from(address),U256::from(add_keys)],
//             Cap::RegisterCap => vec![/* length */ U256::from(1), /* type */ U256::from(11)],
//             Cap::LogCap(topics) => {
//                 let mut v = vec![/* length */ U256::from(1+topics.len()), /* type */ U256::from(9)];
//                 v.extend(topics);
//                 v
//                 },
//             Cap::CallCap(keys) => vec![/* length */ U256::from(1), /* type */ U256::from(3)],
//         }
//     }
// }

// fn caps_into_u256s(caps: Vec<Cap>) -> Vec<U256> {
//     concat_vecs(caps.iter().map(|c| {c.to_u256s()}).collect())
// }

// fn concat_vecs(vecs: Vec<Vec<U256>>) -> Vec<U256> {
//     let size = vecs.iter().fold(0, |a, b| a + b.len());
//     vecs.into_iter().fold(Vec::with_capacity(size), |mut acc, v| {
//         acc.extend(v); acc
//     })
// }


// #[cfg(test)]
// mod deploy_tests {

//     use super::*;
//     use web3::futures::Future;
//     use web3::contract::{Contract, Options};
//     use web3::types::{Address, U256};
//     use web3::Transport;
//     use rustc_hex::FromHex;
//     use ethabi::Token::Uint;

//     #[test]
//     fn deploying_kernel() {
//         let (_eloop, transport) = web3::transports::Http::new("http://localhost:8545").expect("Connection built");
//         let web3 = web3::Web3::new(transport);
//         let sender = match web3.eth().accounts().wait() {
//             Err(_r) => {
//                 panic!("No Ethereum network available");
//                 },
//             Ok(x) => x[0],
//         };
//         let conn = &EthConn {
//             web3,
//             sender
//         };
//         // Deploy a kernel instance
//         let kernel_contract = deploy_kernel(conn);
//     }

//     /// Each of these write caps is 4 keys long. As the maximum length of
//     /// the cap table is 128, the most we can have is 32 (32*4=128). Therefore,
//     /// this example with 32 write caps should succeed.
//     #[test]
//     fn deploying_proc_32_caps() {
//         let (_eloop, transport) = web3::transports::Http::new("http://localhost:8545").expect("Connection built");
//         let web3 = web3::Web3::new(transport);
//         let sender = match web3.eth().accounts().wait() {
//             Err(_r) => {
//                 panic!("No Ethereum network available");
//                 },
//             Ok(x) => x[0],
//         };
//         let conn = &EthConn {
//             web3,
//             sender
//         };
//         // Deploy a kernel instance
//         let kernel_contract = deploy_kernel(conn);

//         let x_caps = 32;
//         let those_caps: Vec<Cap> = (0..x_caps).map(|cap_num| Cap::WriteCap{address: U256::from(0x1000+cap_num), add_keys: U256::from(1)}).collect();
//         deploy_register_procedure(conn, &kernel_contract, String::from(format!("Jane's proc #{}",x_caps)), those_caps).expect("Procedure deployed successfully");
//     }

//     /// Each of these write caps is 4 keys long. As the maximum length of
//     /// the cap table is 128, the most we can have is 32 (32*4=128). Therefore,
//     /// this example with 33 write caps should fail.
//     #[test]
//     fn deploying_proc_33_caps() {
//         let (_eloop, transport) = web3::transports::Http::new("http://localhost:8545").expect("Connection built");
//         let web3 = web3::Web3::new(transport);
//         let sender = match web3.eth().accounts().wait() {
//             Err(_r) => {
//                 panic!("No Ethereum network available");
//                 },
//             Ok(x) => x[0],
//         };
//         let conn = &EthConn {
//             web3,
//             sender
//         };
//         // Deploy a kernel instance
//         let kernel_contract = deploy_kernel(conn);

//         let x_caps = 33;
//         let those_caps: Vec<Cap> = (0..x_caps).map(|cap_num| Cap::WriteCap{address: U256::from(0x1000+cap_num), add_keys: U256::from(1)}).collect();
//         deploy_register_procedure(conn, &kernel_contract, String::from(format!("Jane's proc #{}",x_caps)), those_caps).expect_err("Procedure not deployed successfully");
//     }


//     #[test]
//     fn deploying_512_procs() {
//         let (_eloop, transport) = web3::transports::Http::new("http://localhost:8545").unwrap();
//         let web3 = web3::Web3::new(transport);
//         let sender = match web3.eth().accounts().wait() {
//             Err(_r) => {
//                 panic!("No Ethereum network available");
//                 },
//             Ok(x) => x[0],
//         };
//         let conn = &EthConn {
//             web3,
//             sender
//         };
//         // Deploy a kernel instance
//         let kernel_contract = deploy_kernel(conn);

//         let n_procs = 512;
//         for proc_num in 0..n_procs {
//             let n_caps = 1;
//             let these_caps: Vec<Cap> = (0..n_caps).map(|cap_num| Cap::WriteCap{address: U256::from(0x8000+proc_num*n_caps+cap_num), add_keys: U256::from(1)}).collect();
//             deploy_register_procedure(conn, &kernel_contract, String::from(format!("Jane's proc #{}",proc_num)), these_caps).expect("Procedure deployed successfully");
//         }
//         kernel_contract.call("setEntryProcedure", (string_to_proc_key(String::from("member's procedure")),), conn.sender, Options::default()).wait().unwrap();
//     }

//     #[test]
//     fn deploying_512_procs_512_caps() {
//         let (_eloop, transport) = web3::transports::Http::new("http://localhost:8545").unwrap();
//         let web3 = web3::Web3::new(transport);
//         let sender = match web3.eth().accounts().wait() {
//             Err(_r) => {
//                 panic!("No Ethereum network available");
//                 },
//             Ok(x) => x[0],
//         };
//         let conn = &EthConn {
//             web3,
//             sender
//         };
//         // Deploy a kernel instance
//         let kernel_contract = deploy_kernel(conn);
//         let n_procs = 512;
//         for proc_num in 0..n_procs {
//             let n_caps = proc_num;
//             let these_caps: Vec<Cap> = (0..n_caps).map(|cap_num| Cap::WriteCap{address: U256::from(0x8000+proc_num*n_caps+cap_num), add_keys: U256::from(1)}).collect();
//             deploy_register_procedure(conn, &kernel_contract, String::from(format!("Jane's proc #{}",proc_num)), these_caps).expect("Procedure deployed successfully");
//         }
//         kernel_contract.call("setEntryProcedure", (string_to_proc_key(String::from("member's procedure")),), conn.sender, Options::default()).wait().unwrap();
//     }
// }
