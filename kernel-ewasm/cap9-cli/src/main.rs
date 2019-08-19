#![allow(unused_imports)]
// #![allow(dead_code)]
#[macro_use] extern crate log;
#[macro_use] extern crate lazy_static;

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

    if let Some(deploy_matches) = matches.subcommand_matches("deploy") {
        // Connect to a local network over http.
        let conn: connection::EthConn<web3::transports::Http> = connection::EthConn::new_http();
        // Read the local project from out current directory.
        let mut local_project = project::LocalProject::read();
        // Deploy a kernel with the ACL Bootstrap procedure
        local_project.deploy(&conn);

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
