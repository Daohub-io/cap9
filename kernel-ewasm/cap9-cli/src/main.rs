#[macro_use] extern crate log;

use clap::{Arg, App, SubCommand, AppSettings};
// use std::process::Command;
// use std::str::FromStr;
use web3::futures::Future;
// use web3::types::{Address};
use web3::contract::{Contract, Options};

use std::fs::create_dir;
use std::fs::File;
use std::path::PathBuf;

use env_logger;

mod conn;
mod deploy;
mod project;

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
                    .help("project name")))
            .subcommand(SubCommand::with_name("deploy")
                .about("Deploy a project to the chain"))
            .get_matches();

    if let Some(_deploy_matches) = matches.subcommand_matches("deploy") {
        // Connect to a local network over http.
        let network: conn::EthConn<web3::transports::Http> = conn::EthConn::new_http();
        // Read the local project from out current directory.
        let mut local_project = project::LocalProject::read();
        // Deploy a kernel with the ACL Bootstrap procedure
        let (init_contract, kernel_contract) = deploy::deploy_kernel(&network, &mut local_project);

    } else if let Some(new_matches) = matches.subcommand_matches("new") {
        let project_name = new_matches.value_of("PROJECT-NAME").expect("No project name");
        let local_project = project::LocalProject::create(project_name);
        info!("New Project Created: {}", project_name);
    }
}
