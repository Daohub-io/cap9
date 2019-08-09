use clap::{Arg, App, SubCommand, AppSettings};
// use std::process::Command;
// use std::str::FromStr;
use web3::futures::Future;
// use web3::types::{Address};
use web3::contract::{Contract, Options};

use std::fs::create_dir;
use std::fs::File;
use std::path::PathBuf;

mod conn;
mod deploy;
mod project;

fn main() {
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
        let f = File::open("deploy.json").expect("could not open file");
        let deploy_file = serde_json::from_reader(f).expect("Could not parse deploy file");
        // Deploy a kernel with the ACL Bootstrap procedure
        let (init_contract, kernel_contract) = deploy::deploy_kernel(&network, deploy_file);

    } else if let Some(new_matches) = matches.subcommand_matches("new") {
        let project_name = new_matches.value_of("PROJECT-NAME").expect("No project name");
        // Create a new directory, throw an error if the directory exists.
        let creation_result = create_dir(project_name);
        // Check that the directory was correctly created.
        match creation_result {
            Ok(_) => (),
            Err(ref err) if err.kind() == std::io::ErrorKind::AlreadyExists => {
                println!("The directory {} already exists.", project_name);
                std::process::exit(1);
            },
            e => e.unwrap(),
        }
        let deploy_file = project::DeployFile::new();
        let mut path = PathBuf::new();
        path.push(project_name);
        path.push("deploy");
        path.set_extension("json");
        let f = File::create(path).expect("Could not create file");
        serde_json::ser::to_writer_pretty(f, &deploy_file).expect("Could not serialise deploy data");
    }
}
