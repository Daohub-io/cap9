#[macro_use] extern crate log;

use clap::{App, AppSettings, Arg, SubCommand};
use ethabi::token::Tokenizer;

use rustc_hex::ToHex;
use std::fs::File;
use std::path::PathBuf;

use env_logger;

use cap9_cli::build;
use cap9_cli::connection;
use cap9_cli::fetch;
use cap9_cli::project;

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
            .subcommand(SubCommand::with_name("build")
                .about("Create a Cap9 compatible procedure from WASM file")
                .subcommands(build::build_commands()),
            )
            .subcommand(SubCommand::with_name("compile")
                .about("Compile a WASM contract")
                .arg(Arg::with_name("CARGO-PATH")
                    .required(true)
                    .help("path to cargo project")
                )
            )
            // This will understand the ACL
            .subcommand(SubCommand::with_name("call")
                .about("Call a procedure")
                .arg(Arg::with_name("FUNCTION-NAME")
                    .required(true)
                    .help("path to cargo project")
                )
                .arg(Arg::with_name("INPUTS")
                    .multiple(true)
                    .help("function inputs")
                )
            )
            .subcommand(SubCommand::with_name("query")
                .about("Query a procedure")
                .arg(Arg::with_name("FUNCTION-NAME")
                    .required(true)
                    .help("path to cargo project")
                )
                .arg(Arg::with_name("INPUTS")
                    .multiple(true)
                    .help("function inputs")
                )
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
            .subcommand(SubCommand::with_name("deploy-procedure")
                .arg(Arg::with_name("PROCEDURE-NAME")
                    .required(true)
                    .help("Name of the procedure"))
                .arg(Arg::with_name("CODE-FILE")
                    .required(true)
                    .help("Binary code file"))
                .arg(Arg::with_name("ABI-FILE")
                    .required(true)
                    .help("JSON ABI file"))
                .arg(Arg::with_name("CAP-FILE")
                    .required(true)
                    .help("JSON cap file"))
                .about("Deploy a contract to the chain and register it as a procedure"))
            .subcommand(SubCommand::with_name("new-group")
                .arg(Arg::with_name("GROUP-NUMBER")
                    .required(true)
                    .help("Group number/id"))
                .arg(Arg::with_name("PROCEDURE-NAME")
                    .required(true)
                    .help("Name of the group's procedure"))
                .arg(Arg::with_name("CODE-FILE")
                    .required(true)
                    .help("Binary code file of the group's procedure"))
                .arg(Arg::with_name("ABI-FILE")
                    .required(true)
                    .help("JSON ABI file of the group's procedure"))
                .arg(Arg::with_name("CAP-FILE")
                    .required(true)
                    .help("JSON cap file"))
                .about("Add an new group"))
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
                    .subcommand(SubCommand::with_name("abi")
                        .about("List the functions of each procedure"))
                )
            )
            .get_matches();

    if let Some(_deploy_matches) = matches.subcommand_matches("deploy") {
        // Connect to a local network over http.
        let conn: connection::EthConn<web3::transports::Http> = connection::EthConn::new_http();
        // Read the local project from out current directory.
        let local_project = project::LocalProject::read();
        // Deploy a kernel with the ACL Bootstrap procedure
        local_project.deploy(&conn).unwrap_or_else(|err| panic!("Deployment failure: {}", err));
    } else if let Some(_compile_matches) = matches.subcommand_matches("compile") {
        // let cargo_path = PathBuf::from(compile_matches.value_of("CARGO-PATH").expect("No code file"));
        // Read the local project from out current directory.
        // let local_project = project::LocalProject::read();
    } else if let Some(call_matches) = matches.subcommand_matches("call") {
        let function_name = call_matches.value_of("FUNCTION-NAME").expect("No code file");

        let network: connection::EthConn<web3::transports::Http> = connection::EthConn::new_http();
        let local_project = project::LocalProject::read();
        let kernel = DeployedKernel::new(&network, local_project);
        let kernel_with_acl = DeployedKernelWithACL::new(kernel);

        let proc_key = kernel_with_acl.get_group_proc(&kernel_with_acl.kernel.conn.sender);
        let procedure = kernel_with_acl.kernel.procedure(proc_key).unwrap();
        let status_file: &project::StatusFile = kernel_with_acl.kernel.local_project.status_file().as_ref().unwrap();
        let abi_path = status_file.abis.get(&procedure.address).unwrap();
        let abi_file = File::open(abi_path).unwrap();
        let abi = ethabi::Contract::load(abi_file).unwrap();
        let inputs: Vec<ethabi::Token> = match call_matches.values_of("INPUTS") {
            Some(vals) => vals
                .zip(abi.functions.get(function_name).unwrap().inputs.clone())
                .map(|(s, input)|
                    ethabi::token::LenientTokenizer::tokenize(&input.kind, s).expect("input parse failure")
                )
                .collect(),
            None => Vec::new(),
        };
        println!("Sending from: {:?}", kernel_with_acl.kernel.conn.sender);
        println!("Inputs: {:?}", inputs);
        let result: web3::types::TransactionReceipt = kernel_with_acl.call(function_name, &inputs);
        println!("Result: {:?}", result);
    } else if let Some(query_matches) = matches.subcommand_matches("query") {
        let function_name = query_matches.value_of("FUNCTION-NAME").expect("No code file");

        let network: connection::EthConn<web3::transports::Http> = connection::EthConn::new_http();
        let local_project = project::LocalProject::read();
        let kernel = DeployedKernel::new(&network, local_project);
        let kernel_with_acl = DeployedKernelWithACL::new(kernel);

        let proc_key = kernel_with_acl.get_group_proc(&kernel_with_acl.kernel.conn.sender);
        let procedure = kernel_with_acl.kernel.procedure(proc_key).unwrap();
        let status_file: &project::StatusFile = kernel_with_acl.kernel.local_project.status_file().as_ref().unwrap();
        let abi_path = status_file.abis.get(&procedure.address).unwrap();
        let abi_file = File::open(abi_path).unwrap();
        let abi = ethabi::Contract::load(abi_file).unwrap();
        let inputs: Vec<ethabi::Token> = match query_matches.values_of("INPUTS") {
            Some(vals) => vals
                .zip(abi.functions.get(function_name).unwrap().inputs.clone())
                .map(|(s, input)|
                    ethabi::token::LenientTokenizer::tokenize(&input.kind, s).expect("input parse failure")
                )
                .collect(),
            None => Vec::new(),
        };
        println!("Sending from: {:?}", kernel_with_acl.kernel.conn.sender);
        println!("Inputs: {:?}", inputs);
        let result: Vec<ethabi::Token> = kernel_with_acl.query(function_name, &inputs).unwrap();
        println!("Result: {:?}", result);
    } else if let Some(new_group_matches) = matches.subcommand_matches("new-group") {
        let group_number: u8 = new_group_matches.value_of("GROUP-NUMBER").expect("No code file").parse().unwrap();
        let proc_name = new_group_matches.value_of("PROCEDURE-NAME").expect("No code file");
        let code_file = PathBuf::from(new_group_matches.value_of("CODE-FILE").expect("No code file"));
        let abi_file = PathBuf::from(new_group_matches.value_of("ABI-FILE").expect("No ABI file"));
        let cap_file = PathBuf::from(new_group_matches.value_of("CAP-FILE").expect("No ABI file"));
        // Connect to a local network over http.
        let conn: connection::EthConn<web3::transports::Http> = connection::EthConn::new_http();
        // Read the local project from out current directory.
        let local_project = project::LocalProject::read();
        let kernel = DeployedKernel::new(&conn, local_project);
        let kernel_with_acl = DeployedKernelWithACL::new(kernel);
        let group_5_spec = project::ContractSpec::from_files(&code_file, &abi_file);
        let proc_spec = project::ProcSpec {
            contract_spec: group_5_spec,
            cap_path: cap_file,
        };
        kernel_with_acl.new_group(group_number, proc_name.to_string(), proc_spec).unwrap();
    } else if let Some(deploy_procedure_matches) = matches.subcommand_matches("deploy-procedure") {
        let proc_name = deploy_procedure_matches.value_of("PROCEDURE-NAME").expect("No code file");
        let code_file = PathBuf::from(deploy_procedure_matches.value_of("CODE-FILE").expect("No code file"));
        let abi_file = PathBuf::from(deploy_procedure_matches.value_of("ABI-FILE").expect("No ABI file"));
        let cap_file = PathBuf::from(deploy_procedure_matches.value_of("CAP-FILE").expect("No cap file"));
        // Connect to a local network over http.
        let conn: connection::EthConn<web3::transports::Http> = connection::EthConn::new_http();
        // Read the local project from out current directory.
        let local_project = project::LocalProject::read();
        let kernel = DeployedKernel::new(&conn, local_project);
        let mut kernel_with_acl = DeployedKernelWithACL::new(kernel);
        let contract_spec = project::ContractSpec::from_files(&code_file, &abi_file);
        let proc_spec = project::ProcSpec {
            contract_spec,
            cap_path: cap_file,
        };
        kernel_with_acl.deploy_procedure(proc_name.to_string(), proc_spec).unwrap();
    } else if let Some(deploy_contract_matches) = matches.subcommand_matches("deploy-contract") {
        let code_file = PathBuf::from(deploy_contract_matches.value_of("CODE-FILE").expect("No code file"));
        let abi_file = PathBuf::from(deploy_contract_matches.value_of("ABI-FILE").expect("No ABI file"));
        // Connect to a local network over http.
        let conn: connection::EthConn<web3::transports::Http> = connection::EthConn::new_http();
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
    } else if let Some(build_matches) = matches.subcommand_matches("build") {
        if let Some(build_proc_matches) = build_matches.subcommand_matches("build-proc") {
            build::execute_build_proc(build_proc_matches);
        } else if let Some(set_mem_matches) = build_matches.subcommand_matches("set-mem") {
            build::execute_set_mem(set_mem_matches);
        } else if let Some(wasm_build_matches) = build_matches.subcommand_matches("wasm-build") {
            build::execute_wasm_build(wasm_build_matches);
        } else if let Some(full_matches) = build_matches.subcommand_matches("full") {
            build::execute_full(full_matches);
        } else {
            panic!("no build command");
        }
    } else if let Some(fetch_matches) = matches.subcommand_matches("fetch") {
        let network: connection::EthConn<web3::transports::Http> = connection::EthConn::new_http();
        let local_project = project::LocalProject::read();
        let kernel = DeployedKernel::new(&network, local_project);
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
            } else if let Some(_users_matches) = acl_matches.subcommand_matches("abi") {
                // Take the information from the ABI files.
                let procs = kernel_with_acl.kernel.procedures();
                let status_file: &project::StatusFile = kernel_with_acl.kernel.local_project.status_file().as_ref().unwrap();
                // TODO: get proc names
                for procedure in procs {
                    // println!("{}", procedure.address);
                    let path = status_file.abis.get(&procedure.address).unwrap();
                    let ks = procedure.key;
                    let key: String = ks.to_hex();
                    let key_utf8: &str = std::str::from_utf8(&ks).unwrap().trim_end_matches('\0');
                    println!("Procedure: 0x{} (\"{}\")", key, key_utf8);
                    let abi_file = File::open(path).unwrap();
                    let abi = ethabi::Contract::load(abi_file).unwrap();
                    for function in abi.functions() {
                        print!("  ");
                        print_function(function);
                    }
                    println!("");
                }
            } else {
                println!("fetching acl stuff");
            }
        }
    }
}

fn print_function(function: &ethabi::Function) {
    print!("{}: (", function.name);
    for (i, param) in function.inputs.iter().enumerate() {
        print_param(param);
        if i+1 < function.inputs.len() {
            print!(", ");
        }
    }
    print!(") -> (");
    for (i, param) in function.outputs.iter().enumerate() {
        print_param(param);
        if i+1 < function.outputs.len() {
            print!(", ");
        }
    }
    println!(")");
}

fn print_param(param: &ethabi::Param) {
    print!("{}: {:?}", param.name, param.kind);
}
