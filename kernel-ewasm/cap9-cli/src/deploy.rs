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
use crate::connection;
use crate::connection::EthConn;
use crate::project::*;
use cap9_std::proc_table::cap::*;
use pwasm_abi;
use std::fs::File;
use crate::deploy::web3::contract::tokens::Tokenize;
use crate::default_procedures::*;
use crate::utils::*;
use crate::constants::*;
use ethabi;

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
