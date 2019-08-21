#![allow(unused_imports)]
#![allow(dead_code)]
#[macro_use] extern crate log;
#[macro_use] extern crate lazy_static;
#[macro_use] extern crate failure_derive;

use clap::{Arg, App, SubCommand, AppSettings};
// use std::process::Command;
// use std::str::FromStr;
// use web3::futures::Future;
use web3::types::{Address};
use web3::contract::{Contract, Options};
use cap9_std::data::{Keyable, Storable};

use std::fs::create_dir;
use std::fs::File;
use std::path::PathBuf;
use rustc_hex::ToHex;

use env_logger;

mod connection;
mod deploy;
mod project;
mod fetch;
mod constants;
mod default_procedures;
mod utils;

use fetch::{DeployedKernel, DeployedKernelWithACL};

fn main() {
    env_logger::init();
    let matches = App::new("Cap9 CLI")
            .setting(AppSettings::ArgRequiredElseHelp)
            .version("0.2.0")
            .author("Daolab <info@daolab.io>")
            .about("A command-line interface for BeakerOS on the Ethereum blockchain.")
            .subcommand(SubCommand::with_name("new")
                .about("Create a new Cap9 project in directory PROJECT-NAME")
                .arg(Arg::with_name("PROJECT-NAME")
                    .required(true)
                    .help("project name"))
                .arg(Arg::with_name("acl")
                    .long("acl")
                    .help("Create with the default acl"))
            )
            .subcommand(SubCommand::with_name("deploy")
                .about("Deploy a project to the chain"))
            .subcommand(SubCommand::with_name("deploy-contract")
                .arg(Arg::with_name("CODE-FILE")
                    .required(true)
                    .help("Binary code file"))
                .arg(Arg::with_name("ABI-FILE")
                    .required(true)
                    .help("JSON ABI file"))
                .about("Deploy a contract to the chain"))
            .subcommand(SubCommand::with_name("new-group")
                .about("Add group 5"))
            .subcommand(SubCommand::with_name("fetch")
                .setting(AppSettings::ArgRequiredElseHelp)
                .about("Query information about the current project")
                .subcommand(SubCommand::with_name("gas")
                    .about("Get the amount of gas held by the kernel"))
                .subcommand(SubCommand::with_name("procedures")
                    .about("List all the registered procedures"))
                .subcommand(SubCommand::with_name("acl")
                    .about("Query information pertaining to a standard ACL")
                    .subcommand(SubCommand::with_name("groups")
                        .about("List the groups in the ACL"))
                    .subcommand(SubCommand::with_name("users")
                        .about("List the users in the ACL"))
                )
            )
            .get_matches();

    if let Some(_deploy_matches) = matches.subcommand_matches("deploy") {
        // Connect to a local network over http.
        let conn: connection::EthConn<web3::transports::Http> = connection::EthConn::new_http();
        // Read the local project from out current directory.
        let mut local_project = project::LocalProject::read();
        // Deploy a kernel with the ACL Bootstrap procedure
        local_project.deploy(&conn).unwrap_or_else(|err| panic!("Deployment failure: {}", err));
    } else if let Some(_new_group_matches) = matches.subcommand_matches("new-group") {
        // Connect to a local network over http.
        let conn: connection::EthConn<web3::transports::Http> = connection::EthConn::new_http();
        // Read the local project from out current directory.
        let local_project = project::LocalProject::read();
        let kernel = DeployedKernel::new(&conn, &local_project);
        let kernel_with_acl = DeployedKernelWithACL::new(kernel);
        let group_5_spec = project::ContractSpec::from_files(&PathBuf::from("acl_group_5.wasm"), &PathBuf::from("ACLGroup5Interface.json"));
        kernel_with_acl.new_group("randomProcName".to_string(), group_5_spec).unwrap();
    } else if let Some(deploy_contract_matches) = matches.subcommand_matches("deploy-contract") {
        let code_file = PathBuf::from(deploy_contract_matches.value_of("CODE-FILE").expect("No code file"));
        let abi_file = PathBuf::from(deploy_contract_matches.value_of("ABI-FILE").expect("No ABI file"));
        // Connect to a local network over http.
        let conn: connection::EthConn<web3::transports::Http> = connection::EthConn::new_http();
        // // Read the local project from out current directory.
        // let local_project = project::LocalProject::read();
        // // The project directory is our current directory.
        // let project_directory = std::env::current_dir().expect("Could not get CWD");
        // Create a contract specification from the given files.
        let contract_spec = project::ContractSpec::from_files(&code_file, &abi_file);
        // Deploy the contract onto the chain.
        match contract_spec.deploy(&conn, ( )) {
            Ok(contract) => println!("Contract deployed to {}", contract.address()),
            Err(err) => println!("Contract not deployed: {:?}", err),
        }
    } else if let Some(new_matches) = matches.subcommand_matches("new") {
        let project_name = new_matches.value_of("PROJECT-NAME").expect("No project name");
        let _local_project = if new_matches.is_present("acl") {
            project::LocalProject::create_with_acl(project_name)
        } else {
            project::LocalProject::create(project_name)
        };
        info!("New Project Created: {}", project_name);
    } else if let Some(fetch_matches) = matches.subcommand_matches("fetch") {
        let network: connection::EthConn<web3::transports::Http> = connection::EthConn::new_http();
        let local_project = project::LocalProject::read();
        let kernel = DeployedKernel::new(&network, &local_project);
        if let Some(_procs_matches) = fetch_matches.subcommand_matches("procedures") {
            // List procedures
            let procs = kernel.procedures();
            for procedure in procs {
                println!("{}", procedure);
            }
        } else if let Some(_gas_matches) = fetch_matches.subcommand_matches("gas") {
            let gas = kernel.gas();
            println!("Gas: {}", gas);
        } else if let Some(acl_matches) = fetch_matches.subcommand_matches("acl") {
            let kernel_with_acl = DeployedKernelWithACL::new(kernel);
            if let Some(_groups_matches) = acl_matches.subcommand_matches("groups") {
                let groups = kernel_with_acl.groups();
                println!("# Groups: {}", groups.len());
                for (k, v) in groups.iter() {
                    let ks = &v.procedure_key.0;
                    let key: String = ks.to_hex();
                    let key_utf8: &str = std::str::from_utf8(ks).unwrap().trim_end_matches('\0');
                    print!("  {}: procedure_key: 0x{} (\"{}\")\n    Users:\n", k, key, key_utf8);
                    for user in v.users.iter() {
                        print!("      {}\n", user);
                    }
                }
            } else if let Some(_users_matches) = acl_matches.subcommand_matches("users") {
                let users = kernel_with_acl.users();
                println!("# Users: {}", users.len());
                for (k, v) in users.iter() {
                    println!("  {}: {}", k, v);
                }
            } else {
                println!("fetching acl stuff");
            }
        }
    }
}
