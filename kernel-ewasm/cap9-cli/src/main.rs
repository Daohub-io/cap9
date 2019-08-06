use clap::{Arg, App, SubCommand};
use std::process::Command;
use std::str::FromStr;
use web3::futures::Future;
use web3::types::{Address};

use std::fs::create_dir;

mod conn;

fn main() {
    let matches = App::new("Cap9 CLI")
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

    if let Some(deploy_matches) = matches.subcommand_matches("deploy") {
        unimplemented!();
    } else if let Some(new_matches) = matches.subcommand_matches("new") {
        let project_name = new_matches.value_of("PROJECT-NAME").expect("No project name");
        // Create a new directory, throw an error if the directory exists
        let creation_result = create_dir(project_name);
        match creation_result {
            Ok(_) => (),
            Err(ref err) if err.kind() == std::io::ErrorKind::AlreadyExists => {
                println!("The directory {} already exists.", project_name);
                std::process::exit(1);
            },
            e => e.unwrap(),
        }
    }
}
