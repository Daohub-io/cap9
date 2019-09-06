#[macro_use]
extern crate log;

use clap::{App, AppSettings, Arg, SubCommand};
use ethabi::token::Tokenizer;

use rustc_hex::ToHex;
use std::fs::create_dir;
use std::fs::File;
use std::io::prelude::*;
use std::path::PathBuf;

use env_logger;

use cap9_cli::build;
use cap9_cli::connection;
use cap9_cli::fetch;
use cap9_cli::project;

use cap9_cli::utils::string_to_proc_key;
use cap9_std::proc_table::cap::*;
use fetch::{DeployedKernel, DeployedKernelWithACL, SerialNewCapList};

fn main() {
    env_logger::init();
    let matches = App::new("Cap9 CLI")
        .setting(AppSettings::ArgRequiredElseHelp)
        .version("0.2.0")
        .author("Daolab <info@daolab.io>")
        .about("A command-line interface for BeakerOS on the Ethereum blockchain.")
        .subcommand(
            SubCommand::with_name("new")
                .about("Create a new Cap9 project in directory PROJECT-NAME")
                .arg(
                    Arg::with_name("PROJECT-NAME")
                        .required(true)
                        .help("project name"),
                )
                .arg(
                    Arg::with_name("acl")
                        .long("acl")
                        .help("Create with the default acl"),
                ),
        )
        .subcommand(
            SubCommand::with_name("build")
                .about("Create a Cap9 compatible procedure from WASM file")
                .subcommands(build::build_commands()),
        )
        .subcommand(
            SubCommand::with_name("compile")
                .about("Compile a WASM contract")
                .arg(
                    Arg::with_name("CARGO-PATH")
                        .required(true)
                        .help("path to cargo project"),
                ),
        )
        // This will understand the ACL
        .subcommand(
            SubCommand::with_name("call")
                .about("Call a procedure")
                .arg(
                    Arg::with_name("FUNCTION-NAME")
                        .required(true)
                        .help("path to cargo project"),
                )
                .arg(
                    Arg::with_name("INPUTS")
                        .multiple(true)
                        .help("function inputs"),
                ),
        )
        // This will understand the ACL
        .subcommand(
            SubCommand::with_name("call-any")
                .about("Call a procedure using the admin procs special support for calling any procedure.")
                .arg(
                    Arg::with_name("PROC-NAME")
                        .required(true)
                        .help("Name of the procedure to call"),
                )
                .arg(
                    Arg::with_name("FUNCTION-NAME")
                        .required(true)
                        .help("Name of the function to call"),
                )
                .arg(
                    Arg::with_name("INPUTS")
                        .multiple(true)
                        .help("function inputs"),
                ),
        )
        .subcommand(
            SubCommand::with_name("query")
                .about("Query a procedure")
                .arg(
                    Arg::with_name("FUNCTION-NAME")
                        .required(true)
                        .help("path to cargo project"),
                )
                .arg(
                    Arg::with_name("INPUTS")
                        .multiple(true)
                        .help("function inputs"),
                ),
        )
        .subcommand(
            SubCommand::with_name("query-any")
                .about("Query a procedure using the admin procs special support for calling any procedure.")
                .arg(
                    Arg::with_name("PROC-NAME")
                        .required(true)
                        .help("Name of the procedure to call"),
                )
                .arg(
                    Arg::with_name("FUNCTION-NAME")
                        .required(true)
                        .help("Name of the function to call"),
                )
                .arg(
                    Arg::with_name("INPUTS")
                        .multiple(true)
                        .help("function inputs"),
                ),
        )
        .subcommand(SubCommand::with_name("deploy").about("Deploy a project to the chain"))
        .subcommand(
            SubCommand::with_name("deploy-contract")
                .arg(
                    Arg::with_name("CODE-FILE")
                        .required(true)
                        .help("Binary code file"),
                )
                .arg(
                    Arg::with_name("ABI-FILE")
                        .required(true)
                        .help("JSON ABI file"),
                )
                .about("Deploy a contract to the chain"),
        )
        .subcommand(
            SubCommand::with_name("deploy-procedure")
                .arg(
                    Arg::with_name("PROCEDURE-NAME")
                        .required(true)
                        .help("Name of the procedure"),
                )
                .arg(
                    Arg::with_name("CODE-FILE")
                        .required(false)
                        .help("Binary code file"),
                )
                .arg(
                    Arg::with_name("ABI-FILE")
                        .required(false)
                        .help("JSON ABI file"),
                )
                .arg(
                    Arg::with_name("CAP-FILE")
                        .required(false)
                        .help("JSON cap file"),
                )
                .about("Deploy a contract to the chain and register it as a procedure"),
        )
        .subcommand(
            SubCommand::with_name("new-procedure")
                .arg(
                    Arg::with_name("PROCEDURE-NAME")
                        .required(true)
                        .help("Name of the procedure"),
                )
                // .arg(
                //     Arg::with_name("CODE-FILE")
                //         .required(true)
                //         .help("Binary code file"),
                // )
                // .arg(
                //     Arg::with_name("ABI-FILE")
                //         .required(true)
                //         .help("JSON ABI file"),
                // )
                // .arg(
                //     Arg::with_name("CAP-FILE")
                //         .required(true)
                //         .help("JSON cap file"),
                // )
                .about("Create a new procedure"),
        )
        .subcommand(
            SubCommand::with_name("delete-procedure")
                .arg(
                    Arg::with_name("PROCEDURE-NAME")
                        .required(true)
                        .help("Name of the procedure"),
                )
                .about("Delete a procedure from the kernel (does not reap contract)"),
        )
        .subcommand(
            SubCommand::with_name("new-group")
                .arg(
                    Arg::with_name("GROUP-NUMBER")
                        .required(true)
                        .help("Group number/id"),
                )
                .arg(
                    Arg::with_name("PROCEDURE-NAME")
                        .required(true)
                        .help("Name of the group's procedure"),
                )
                .arg(
                    Arg::with_name("CODE-FILE")
                        .required(true)
                        .help("Binary code file of the group's procedure"),
                )
                .arg(
                    Arg::with_name("ABI-FILE")
                        .required(true)
                        .help("JSON ABI file of the group's procedure"),
                )
                .arg(
                    Arg::with_name("CAP-FILE")
                        .required(true)
                        .help("JSON cap file"),
                )
                .about("Add an new group"),
        )
        .subcommand(
            SubCommand::with_name("fetch")
                .setting(AppSettings::ArgRequiredElseHelp)
                .about("Query information about the current project")
                .subcommand(
                    SubCommand::with_name("gas").about("Get the amount of gas held by the kernel"),
                )
                .subcommand(
                    SubCommand::with_name("procedures").about("List all the registered procedures"),
                )
                .subcommand(
                    SubCommand::with_name("acl")
                        .about("Query information pertaining to a standard ACL")
                        .subcommand(
                            SubCommand::with_name("groups").about("List the groups in the ACL"),
                        )
                        .subcommand(
                            SubCommand::with_name("users").about("List the users in the ACL"),
                        )
                        .subcommand(
                            SubCommand::with_name("abi")
                                .about("List the functions of each procedure"),
                        ),
                ),
        )
        .get_matches();

    if let Some(_deploy_matches) = matches.subcommand_matches("deploy") {
        // Connect to a local network over http.
        let conn: connection::EthConn<web3::transports::Http> = connection::EthConn::new_http();
        // Read the local project from out current directory.
        let local_project = project::LocalProject::read();
        // Deploy a kernel with the ACL Bootstrap procedure
        local_project
            .deploy(&conn)
            .unwrap_or_else(|err| panic!("Deployment failure: {}", err));
    } else if let Some(_compile_matches) = matches.subcommand_matches("compile") {
        // let cargo_path = PathBuf::from(compile_matches.value_of("CARGO-PATH").expect("No code file"));
        // Read the local project from out current directory.
        // let local_project = project::LocalProject::read();
    } else if let Some(call_matches) = matches.subcommand_matches("call") {
        let function_name = call_matches
            .value_of("FUNCTION-NAME")
            .expect("No code file");

        let network: connection::EthConn<web3::transports::Http> = connection::EthConn::new_http();
        let local_project = project::LocalProject::read();
        let kernel = DeployedKernel::new(&network, local_project);
        let kernel_with_acl = DeployedKernelWithACL::new(kernel);

        let proc_key = kernel_with_acl.get_group_proc(&kernel_with_acl.kernel.conn.sender);
        let procedure = kernel_with_acl.kernel.procedure(proc_key).unwrap();
        let status_file: &project::StatusFile = kernel_with_acl
            .kernel
            .local_project
            .status_file()
            .as_ref()
            .unwrap();
        let abi_path = status_file.abis.get(&procedure.address).unwrap();
        let abi_file = File::open(abi_path).unwrap();
        let abi = ethabi::Contract::load(abi_file).unwrap();
        let inputs: Vec<ethabi::Token> = match call_matches.values_of("INPUTS") {
            Some(vals) => vals
                .zip(abi.functions.get(function_name).unwrap().inputs.clone())
                .map(|(s, input)| {
                    ethabi::token::LenientTokenizer::tokenize(&input.kind, s)
                        .expect("input parse failure")
                })
                .collect(),
            None => Vec::new(),
        };
        println!("Sending from: {:?}", kernel_with_acl.kernel.conn.sender);
        println!("Inputs: {:?}", inputs);
        let result: web3::types::TransactionReceipt = kernel_with_acl.call(function_name, &inputs);
        println!("Result: {:?}", result);
    } else if let Some(call_any_matches) = matches.subcommand_matches("call-any") {
        let proc_name = call_any_matches
            .value_of("PROC-NAME")
            .expect("No code file");
        let function_name = call_any_matches
            .value_of("FUNCTION-NAME")
            .expect("No code file");

        // Here we assume the proc_name is simply ascii.
        let proc_key = cap9_std::SysCallProcedureKey(string_to_proc_key(proc_name.to_string()));


        let network: connection::EthConn<web3::transports::Http> = connection::EthConn::new_http();
        let local_project = project::LocalProject::read();
        let kernel = DeployedKernel::new(&network, local_project);
        let kernel_with_acl = DeployedKernelWithACL::new(kernel);

        // First we need to encode the message to the final procedure
        // let proc_key = kernel_with_acl.get_group_proc(&kernel_with_acl.kernel.conn.sender);
        // This will need to be the admin proc
        let procedure = kernel_with_acl.kernel.procedure(proc_key.clone()).unwrap();
        let status_file: &project::StatusFile = kernel_with_acl
            .kernel
            .local_project
            .status_file()
            .as_ref()
            .expect("could not get status file");
        let abi_path = status_file.abis.get(&procedure.address).expect("could not find ABI");
        println!("ABI Path: {:?}", abi_path);
        let abi_file = File::open(abi_path).unwrap();
        let abi = ethabi::Contract::load(abi_file).unwrap();
        println!("ABI: {:?}", abi);
        let inputs: Vec<ethabi::Token> = match call_any_matches.values_of("INPUTS") {
            Some(vals) => vals
                .zip(abi.functions.get(function_name).unwrap().inputs.clone())
                .map(|(s, input)| {
                    ethabi::token::LenientTokenizer::tokenize(&input.kind, s)
                        .expect("input parse failure")
                })
                .collect(),
            None => Vec::new(),
        };


        //  // First we need to encode the message to the final procedure
        // let proc_key = kernel_with_acl.get_group_proc(&kernel_with_acl.kernel.conn.sender);
        // // This will need to be the admin proc
        // let procedure = kernel_with_acl.kernel.procedure(user_proc_key).unwrap();
        // let status_file: &project::StatusFile = kernel_with_acl
        //     .kernel
        //     .local_project
        //     .status_file()
        //     .as_ref()
        //     .unwrap();
        // let abi_path = status_file.abis.get(&procedure.address).unwrap();
        // let abi_file = File::open(abi_path).unwrap();
        // let abi = ethabi::Contract::load(abi_file).unwrap();
        // let inputs: Vec<ethabi::Token> = match call_any_matches.values_of("INPUTS") {
        //     Some(vals) => vals
        //         .zip(abi.functions.get(function_name).unwrap().inputs.clone())
        //         .map(|(s, input)| {
        //             ethabi::token::LenientTokenizer::tokenize(&input.kind, s)
        //                 .expect("input parse failure")
        //         })
        //         .collect(),
        //     None => Vec::new(),
        // };


        println!("Sending from: {:?}", kernel_with_acl.kernel.conn.sender);
        println!("Inputs: {:?}", inputs);
        let result: web3::types::TransactionReceipt = kernel_with_acl.call_any(proc_key, function_name, &inputs);
        println!("Result: {:?}", result);
    } else if let Some(call_any_matches) = matches.subcommand_matches("query-any") {
        let proc_name = call_any_matches
            .value_of("PROC-NAME")
            .expect("No code file");
        let function_name = call_any_matches
            .value_of("FUNCTION-NAME")
            .expect("No code file");

        // Here we assume the proc_name is simply ascii.
        let proc_key = cap9_std::SysCallProcedureKey(string_to_proc_key(proc_name.to_string()));


        let network: connection::EthConn<web3::transports::Http> = connection::EthConn::new_http();
        let local_project = project::LocalProject::read();
        let kernel = DeployedKernel::new(&network, local_project);
        let kernel_with_acl = DeployedKernelWithACL::new(kernel);

        // First we need to encode the message to the final procedure
        // let proc_key = kernel_with_acl.get_group_proc(&kernel_with_acl.kernel.conn.sender);
        // This will need to be the admin proc
        let procedure = kernel_with_acl.kernel.procedure(proc_key.clone()).unwrap();
        let status_file: &project::StatusFile = kernel_with_acl
            .kernel
            .local_project
            .status_file()
            .as_ref()
            .expect("could not get status file");
        let abi_path = status_file.abis.get(&procedure.address).expect("could not find ABI");
        let abi_file = File::open(abi_path).unwrap();
        let abi = ethabi::Contract::load(abi_file).unwrap();
        let inputs: Vec<ethabi::Token> = match call_any_matches.values_of("INPUTS") {
            Some(vals) => vals
                .zip(abi.functions.get(function_name).unwrap().inputs.clone())
                .map(|(s, input)| {
                    ethabi::token::LenientTokenizer::tokenize(&input.kind, s)
                        .expect("input parse failure")
                })
                .collect(),
            None => Vec::new(),
        };
        let result = kernel_with_acl.query_any(proc_key, function_name, &inputs);
        println!("Result: {:?}", result);
    } else if let Some(query_matches) = matches.subcommand_matches("query") {
        let function_name = query_matches
            .value_of("FUNCTION-NAME")
            .expect("No code file");

        let network: connection::EthConn<web3::transports::Http> = connection::EthConn::new_http();
        let local_project = project::LocalProject::read();
        let kernel = DeployedKernel::new(&network, local_project);
        let kernel_with_acl = DeployedKernelWithACL::new(kernel);

        let proc_key = kernel_with_acl.get_group_proc(&kernel_with_acl.kernel.conn.sender);
        let procedure = kernel_with_acl.kernel.procedure(proc_key).unwrap();
        let status_file: &project::StatusFile = kernel_with_acl
            .kernel
            .local_project
            .status_file()
            .as_ref()
            .unwrap();
        let abi_path = status_file.abis.get(&procedure.address).unwrap();
        let abi_file = File::open(abi_path).unwrap();
        let abi = ethabi::Contract::load(abi_file).unwrap();
        let inputs: Vec<ethabi::Token> = match query_matches.values_of("INPUTS") {
            Some(vals) => vals
                .zip(abi.functions.get(function_name).unwrap().inputs.clone())
                .map(|(s, input)| {
                    ethabi::token::LenientTokenizer::tokenize(&input.kind, s)
                        .expect("input parse failure")
                })
                .collect(),
            None => Vec::new(),
        };
        println!("Sending from: {:?}", kernel_with_acl.kernel.conn.sender);
        println!("Inputs: {:?}", inputs);
        let result: Vec<ethabi::Token> = kernel_with_acl.query(function_name, &inputs).unwrap();
        println!("Result: {:?}", result);
    } else if let Some(new_group_matches) = matches.subcommand_matches("new-group") {
        let group_number: u8 = new_group_matches
            .value_of("GROUP-NUMBER")
            .expect("No code file")
            .parse()
            .unwrap();
        let proc_name = new_group_matches
            .value_of("PROCEDURE-NAME")
            .expect("No code file");
        let code_file = PathBuf::from(
            new_group_matches
                .value_of("CODE-FILE")
                .expect("No code file"),
        );
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
        kernel_with_acl
            .new_group(group_number, proc_name.to_string(), proc_spec)
            .unwrap();
    } else if let Some(deploy_procedure_matches) = matches.subcommand_matches("new-procedure") {
        let proc_name = deploy_procedure_matches
            .value_of("PROCEDURE-NAME")
            .expect("No code file");
        // Read the local project from out current directory. We do this to
        // ensure we are in a project.
        let _local_project = project::LocalProject::read();
        // Create a new directory in the local project.
        // Create a new directory, throw an error if the directory exists.
        let proc_path = PathBuf::from(proc_name);
        let creation_result = create_dir(&proc_path);
        // Check that the directory was correctly created.
        match creation_result {
            Ok(_) => (),
            Err(ref err) if err.kind() == std::io::ErrorKind::AlreadyExists => {
                println!("The directory {:?} already exists.", proc_path);
                std::process::exit(1);
            }
            e => e.unwrap(),
        }
        // Create a source directory within that directory
        let mut src_path = PathBuf::from(proc_name);
        src_path.push("src");
        let creation_result_src = create_dir(&src_path);
        // Check that the directory was correctly created.
        match creation_result_src {
            Ok(_) => (),
            Err(ref err) if err.kind() == std::io::ErrorKind::AlreadyExists => {
                println!("The directory {:?} already exists.", src_path);
                std::process::exit(1);
            }
            e => e.unwrap(),
        }
        let mut main_path = src_path.clone();
        main_path.push("main.rs");
        let mut main_file = std::fs::File::create(&main_path).unwrap();
        main_file
            .write_all(include_bytes!("example_proc.rs.example"))
            .unwrap();

        let mut toml_path = proc_path.clone();
        toml_path.push("Cargo.toml");
        let mut toml_file = std::fs::File::create(&toml_path).unwrap();
        let mut toml_data: toml::Value =
            toml::from_slice(include_bytes!("example_Cargo.toml.example")).unwrap();
        // println!("toml-data: {:?}", toml_data);
        let package_table = toml_data
            .get_mut("package")
            .unwrap()
            .as_table_mut()
            .unwrap();
        let package_name: toml::Value = toml::Value::String(proc_name.to_string());
        package_table.insert("name".to_string(), package_name);
        toml_file
            .write_all(toml::to_string_pretty(&toml_data).unwrap().as_bytes())
            .unwrap();

        let config_str = "[target.wasm32-unknown-unknown]\n
rustflags = [
  \"-C\", \"link-args=-z stack-size=65536\",
]";
        // Create a source directory within that directory
        let mut config_dir_path = proc_path.clone();
        config_dir_path.push(".cargo");
        let creation_result_config = create_dir(&config_dir_path);
        // Check that the directory was correctly created.
        match creation_result_config {
            Ok(_) => (),
            Err(ref err) if err.kind() == std::io::ErrorKind::AlreadyExists => {
                println!("The directory {:?} already exists.", config_dir_path);
                std::process::exit(1);
            }
            e => e.unwrap(),
        }
        let mut config_file_path = config_dir_path.clone();
        config_file_path.push("config");
        let mut config_file = std::fs::File::create(&config_file_path).unwrap();
        config_file.write_all(config_str.as_bytes()).unwrap();

        let mut caps_path = proc_path.clone();
        caps_path.push("caps.json");
        let prefix = 0;
        let cap_key = string_to_proc_key("".to_string());
        let caps_file = std::fs::File::create(&caps_path).unwrap();
        let caps_data: Vec<NewCapability> = vec![
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
                cap: Capability::Log(LogCap {
                    topics: 0_u8,
                    t1: [0_u8; 32],
                    t2: [0_u8; 32],
                    t3: [0_u8; 32],
                    t4: [0_u8; 32],
                }),
                parent_index: 0,
            },
            NewCapability {
                cap: Capability::StoreWrite(StoreWriteCap {
                    location: [
                        0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
                        0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
                        0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
                    ],
                    size: [
                        0x10, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
                        0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
                        0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
                    ],
                }),
                parent_index: 0,
            },
            NewCapability {
                cap: Capability::StoreWrite(StoreWriteCap {
                    location: [
                        0x30, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
                        0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
                        0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
                    ],
                    size: [
                        0x10, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
                        0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
                        0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
                    ],
                }),
                parent_index: 1,
            },
            NewCapability {
                cap: Capability::ProcedureEntry(ProcedureEntryCap),
                parent_index: 0,
            },
        ];
        let serial_cap_list = SerialNewCapList(NewCapList(caps_data));
        serde_json::ser::to_writer_pretty(caps_file, &serial_cap_list)
            .expect("Could not serialise caps data");
    } else if let Some(deploy_procedure_matches) = matches.subcommand_matches("deploy-procedure") {
        let proc_name = deploy_procedure_matches
            .value_of("PROCEDURE-NAME")
            .expect("No pricedure name");
        let code_file_opt = deploy_procedure_matches
            .value_of("CODE-FILE")
            .map(PathBuf::from);
        let abi_file_opt = deploy_procedure_matches
            .value_of("ABI-FILE")
            .map(PathBuf::from);
        let cap_file_opt = deploy_procedure_matches
            .value_of("CAP-FILE")
            .map(PathBuf::from);
        let (code_file, abi_file, cap_file) = match (code_file_opt, abi_file_opt, cap_file_opt) {
            (Some(code_file), Some(abi_file), Some(cap_file)) => (code_file, abi_file, cap_file),
            (None, None, None) => get_proc_paths(&proc_name.to_string()),
            _ => panic!("You must specify either just a procedure name, or a procedure name with CODE-FILE, ABI-FILE, and CAP-FILE arguments."),
        };
        // println!("paths: {:?}, {:?}. {:?}", code_file, abi_file, cap_file);
        // Check that the code file exists
        if !code_file.as_path().exists() {
            println!("The compiled WASM code does not exist. Be sure to compile with --target wasm32-unknown-unknown --release.");
            std::process::exit(1);
        }
        // Check that the abi file exists
        if !abi_file.as_path().exists() {
            println!("The ABI file does not exist. Be sure that the code is compiled and that there is only one *.json file in the <proc-name>/target/json directory.");
            std::process::exit(1);
        }
        // Check that the cap file exists
        if !cap_file.as_path().exists() {
            println!("The capabilities JSON file does not exist. The root of the procedure directory should contain a file called \"caps.json\" which contains JSON formatted capability information.");
            std::process::exit(1);
        }

        // Connect to a local network over http.
        let conn: connection::EthConn<web3::transports::Http> = connection::EthConn::new_http();
        // Read the local project from out current directory. We don't need it,
        // we just want to make sure we are in one.
        let local_project = project::LocalProject::read();
        let kernel = DeployedKernel::new(&conn, local_project);
        let mut kernel_with_acl = DeployedKernelWithACL::new(kernel);
        // Check that the procedure doesn't already exist.
        let procedures = kernel_with_acl.kernel.procedures();
        for procedure in procedures {
            if procedure.key == string_to_proc_key(proc_name.to_string()) {
                println!("The procedure \"{}\" is already registered in the kernel. You need to delete the procedure first if you'd like to re-register this procedure. Use `cap9-cli delete-procedure {}` to delete this procedure.", proc_name, proc_name);
                std::process::exit(1);
            }
        }
        let contract_spec = project::ContractSpec::from_files(&code_file, &abi_file);
        let proc_spec = project::ProcSpec {
            contract_spec,
            cap_path: cap_file,
        };
        kernel_with_acl
            .deploy_procedure(proc_name.to_string(), proc_spec)
            .unwrap();
    } else if let Some(delete_procedure_matches) = matches.subcommand_matches("delete-procedure") {
        let proc_name = delete_procedure_matches
            .value_of("PROCEDURE-NAME")
            .expect("No code file");
        // Connect to a local network over http.
        let conn: connection::EthConn<web3::transports::Http> = connection::EthConn::new_http();
        // Read the local project from out current directory.
        let local_project = project::LocalProject::read();
        let kernel = DeployedKernel::new(&conn, local_project);
        let mut kernel_with_acl = DeployedKernelWithACL::new(kernel);
        kernel_with_acl
            .delete_procedure(proc_name.to_string())
            .unwrap();
    } else if let Some(deploy_contract_matches) = matches.subcommand_matches("deploy-contract") {
        let code_file = PathBuf::from(
            deploy_contract_matches
                .value_of("CODE-FILE")
                .expect("No code file"),
        );
        let abi_file = PathBuf::from(
            deploy_contract_matches
                .value_of("ABI-FILE")
                .expect("No ABI file"),
        );
        // Connect to a local network over http.
        let conn: connection::EthConn<web3::transports::Http> = connection::EthConn::new_http();
        // Create a contract specification from the given files.
        let contract_spec = project::ContractSpec::from_files(&code_file, &abi_file);
        // Deploy the contract onto the chain.
        match contract_spec.deploy(&conn, ()) {
            Ok(contract) => println!("Contract deployed to {}", contract.address()),
            Err(err) => println!("Contract not deployed: {:?}", err),
        }
    } else if let Some(new_matches) = matches.subcommand_matches("new") {
        let project_name = new_matches
            .value_of("PROJECT-NAME")
            .expect("No project name");
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
                    print!(
                        "  {}: procedure_key: 0x{} (\"{}\")\n    Users:\n",
                        k, key, key_utf8
                    );
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
                let status_file: &project::StatusFile = kernel_with_acl
                    .kernel
                    .local_project
                    .status_file()
                    .as_ref()
                    .unwrap();
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
            }
        }
    }
}

fn print_function(function: &ethabi::Function) {
    print!("{}: (", function.name);
    for (i, param) in function.inputs.iter().enumerate() {
        print_param(param);
        if i + 1 < function.inputs.len() {
            print!(", ");
        }
    }
    print!(") -> (");
    for (i, param) in function.outputs.iter().enumerate() {
        print_param(param);
        if i + 1 < function.outputs.len() {
            print!(", ");
        }
    }
    println!(")");
}

fn print_param(param: &ethabi::Param) {
    print!("{}: {:?}", param.name, param.kind);
}

fn get_proc_paths(proc_name: &String) -> (PathBuf, PathBuf, PathBuf) {
    let proc_dir = PathBuf::from(proc_name);
    if !proc_dir.as_path().exists() {
        println!("The directory for {} does not exist. Are you sure you created such a procedure? Try using `cap9-cli new-procedure {}`.",proc_name, proc_name);
        std::process::exit(1);
    }
    let mut code_path = proc_dir.clone();
    code_path.push("target");
    code_path.push("wasm32-unknown-unknown");
    code_path.push("release");
    code_path.push(proc_name);
    code_path.set_extension("wasm");

    // For now we will assume there is only one JSON abi file.
    let mut abi_path = proc_dir.clone();
    abi_path.push("target");
    abi_path.push("json");
    // list dir contents
    if !abi_path.as_path().exists() {
        println!("The abi directory for {} does not exist. Are you sure you compiled the code for this procedure?",proc_name);
        std::process::exit(1);
    }
    let paths = std::fs::read_dir(&abi_path).unwrap();
    let mut abi_name = None;
    for path in paths {
        let p = path.unwrap().path();
        if p.extension() == Some(std::ffi::OsStr::new("json")) {
            abi_name = Some(p);
            break;
        }
    }
    if let Some(name) = abi_name {
        abi_path = PathBuf::from(name);
    } else {
        panic!("No ABI files");
    }

    let mut cap_path = proc_dir.clone();
    cap_path.push("caps");
    cap_path.set_extension("json");

    (code_path, abi_path, cap_path)
}
