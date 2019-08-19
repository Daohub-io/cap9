use web3::Transport;
use web3::contract::{Contract, Options};
use web3::types::{Address, U256, H256};
use web3::contract::tokens::Tokenize;
// use web3::futures::Future;
use serde::{Deserialize, Serialize, Serializer};
// use serde_json::Result;
use rustc_hex::FromHex;
use rustc_hex::ToHex;

use web3::futures::Future;

use std::fs::create_dir;
use std::fs::File;
use std::path::PathBuf;
use std::io::prelude::*;

use crate::fetch::*;
use crate::connection::*;
use crate::connection;
use crate::deploy::*;
use crate::default_procedures::*;
use crate::utils::*;
use crate::constants::*;
use cap9_std::proc_table::cap::*;

#[derive(Serialize, Deserialize)]
pub struct DeployFile {
    // pub sender: Address,
    pub deploy_spec: DeploySpec,
    pub kernel: ContractSpec,
    pub standard_acl_abi: bool,
}

impl DeployFile {
    pub fn new(kernel_spec: ContractSpec, deploy_spec: DeploySpec) -> Self {
        DeployFile {
            deploy_spec: deploy_spec,
            kernel: kernel_spec,
            standard_acl_abi: false,
        }
    }

    pub fn new_with_acl(kernel_spec: ContractSpec, deploy_spec: DeploySpec) -> Self {
        DeployFile {
            deploy_spec: deploy_spec,
            kernel: kernel_spec,
            standard_acl_abi: true,
        }
    }
}

/// The information defining the structure of a deployed kernel. For example,
/// the initial entry procedure.
#[derive(Serialize, Deserialize)]
pub struct DeploySpec {
    pub initial_entry: ContractSpec,
}

// impl DeploySpec {
//     /// Create a new DeploySpec using the acl bootstrap procedure as a default.
//     pub fn new() -> Self {
//         DeploySpec {
//             initial_entry: ContractSpec::from_default(ACL_BOOTSTRAP, "acl_bootstrap".to_string())
//         }
//     }
// }

#[derive(Serialize, Deserialize)]
pub struct ContractSpec {
    pub code_path: String,
    pub abi_path: String,
}

impl ContractSpec {
    pub fn from_default(def: DefaultProcedure, dir: &PathBuf, name: String) -> Self {
        let code = def.code();
        let abi = def.abi().to_vec();
        let mut code_path = PathBuf::new();
        code_path.push(&dir);
        code_path.push(&name);
        code_path.set_extension("bin");
        let mut code_path_rel = PathBuf::new();
        code_path_rel.push(&name);
        code_path_rel.set_extension("bin");
        let mut code_file = File::create(&code_path).expect(format!("Could not create file: {:?}", code_path).as_str());
        code_file.write_all(code.as_slice()).unwrap();
        let mut abi_path = PathBuf::new();
        abi_path.push(&dir);
        abi_path.push(&name);
        abi_path.set_extension("json");
        let mut abi_path_rel = PathBuf::new();
        abi_path_rel.push(&name);
        abi_path_rel.set_extension("json");
        let mut abi_file = File::create(&abi_path).expect("Could not create file");
        abi_file.write_all(abi.as_slice()).unwrap();
        ContractSpec {
            code_path: code_path_rel.to_str().unwrap().to_string(),
            abi_path: abi_path_rel.to_str().unwrap().to_string(),
        }
    }

    pub fn code(&self) -> Vec<u8> {
        let mut f_code = File::open(&self.code_path).expect("could not open file");
        let mut code: Vec<u8> = Vec::new();
        f_code.read_to_end(&mut code).unwrap();
        code
    }

    pub fn abi(&self) -> Vec<u8> {
        let mut f_abi = File::open(&self.abi_path).expect("could not open file");
        let mut abi: Vec<u8> = Vec::new();
        f_abi.read_to_end(&mut abi).unwrap();
        abi
    }
}

#[derive(Serialize, Deserialize)]
pub struct StatusFile {
    pub kernel_address: Address,
}

impl StatusFile {
    pub fn new(address: Address) -> Self {
        StatusFile {
            kernel_address: address,
        }
    }
}

/// A representation of the local project information. Methods on this struct
/// operate on the local filesystem.
pub struct LocalProject {
    /// The deployment information for this project. This is necessary for the
    /// project to exist.
    deploy_file: DeployFile,
    /// The status file is only present when the project is deployed. When the
    /// status file is present, the project is deployed.
    status_file: Option<StatusFile>,
}

impl LocalProject {
    pub fn create(project_name: &str) -> Self {
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
        // Save the kernel code to file and create a ContractSpec
        let dir = PathBuf::from(project_name);
        let mut path = PathBuf::new();
        path.push(project_name);
        path.push("deploy");
        path.set_extension("json");
        let kernel_spec = ContractSpec::from_default(KERNEL, &dir, "kernel".to_string());
        let init_entry_spec = ContractSpec::from_default(ACL_BOOTSTRAP, &dir, "acl_bootstrap".to_string());
        let deploy_spec = DeploySpec {
            initial_entry: init_entry_spec,
        };
        let deploy_file = DeployFile::new(kernel_spec, deploy_spec);
        let f = File::create(&path).expect("Could not create file");
        serde_json::ser::to_writer_pretty(f, &deploy_file).expect("Could not serialise deploy data");
        LocalProject {
            deploy_file,
            status_file: None,
        }
    }

    pub fn create_with_acl(project_name: &str) -> Self {
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
        // Save the kernel code to file and create a ContractSpec
        let dir = PathBuf::from(project_name);
        let mut path = PathBuf::new();
        path.push(project_name);
        path.push("deploy");
        path.set_extension("json");
        let kernel_spec = ContractSpec::from_default(KERNEL, &dir, "kernel".to_string());
        let init_entry_spec = ContractSpec::from_default(ACL_BOOTSTRAP, &dir, "acl_bootstrap".to_string());
        let deploy_spec = DeploySpec {
            initial_entry: init_entry_spec,
        };
        let deploy_file = DeployFile::new_with_acl(kernel_spec, deploy_spec);
        let f = File::create(&path).expect("Could not create file");
        serde_json::ser::to_writer_pretty(f, &deploy_file).expect("Could not serialise deploy data");
        LocalProject {
            deploy_file,
            status_file: None,
        }
    }

    pub fn read() -> Self {
        let f_deploy = File::open("deploy.json").expect("could not open file");
        let deploy_file = serde_json::from_reader(f_deploy).expect("Could not parse deploy file");
        let f_status = File::open("status.json");
        let status_file = match f_status {
            Ok(f) => Some(serde_json::from_reader(f).expect("Could not parse status file")),
            Err(e) => {
                if e.kind() == std::io::ErrorKind::NotFound {
                    None
                } else {
                    panic!("{:?}", e)
                }
            }
        };
        LocalProject {
            deploy_file,
            status_file,
        }
    }

    pub fn deploy_file(&self) -> &DeployFile {
        &self.deploy_file
    }

    pub fn status_file(&self) -> &Option<StatusFile> {
        &self.status_file
    }

    pub fn add_status_file(&mut self, address: Address) {
        let status_file = StatusFile::new(address);
        let out_file = File::create("status.json").expect("could not create status file");
        serde_json::to_writer_pretty(out_file, &status_file).expect("could not serialise to file");
        self.status_file = Some(status_file);
    }

    pub fn deploy<'a, 'b, T: Transport>(&'b mut self, conn:  &'a EthConn<T>) {
        // Deploy initial procedure
        let _init_contract = deploy_contract(conn, ACL_BOOTSTRAP.code(), ACL_BOOTSTRAP.abi());
        let deploy_file = self.deploy_file();
        if deploy_file.standard_acl_abi {
            self.deploy_with_acl(&conn);
        } else {
            self.deploy_std(&conn);
        }
    }

    pub fn deploy_std<'a, 'b, T: Transport>(&'b mut self, conn:  &'a EthConn<T>) -> DeployedKernel<'a, 'b, T> {
        // Deploy initial procedure
        let init_contract = deploy_contract(conn, ACL_BOOTSTRAP.code(), ACL_BOOTSTRAP.abi());
        let deploy_file = self.deploy_file();
        // Deploying a kernel instance
        let kernel_code: &Vec<u8> = &deploy_file.kernel.code();
        let proc_key = String::from("init");
        let proc_address = init_contract.address();
        let entry_caps: Vec<NewCapability> = DEFAULT_CAPS.to_vec();

        let cap_list: NewCapList = NewCapList(entry_caps.clone());
        let encoded_cap_list: Vec<U256> = from_common_u256_vec(cap_list.to_u256_list());

        let code_hex: String = kernel_code.clone().to_hex();
        let kernel_contract = Contract::deploy(conn.web3.eth(), KERNEL.abi())
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
        self.add_status_file(kernel_contract.address());
        DeployedKernel::new(conn, self)
    }

    pub fn deploy_with_acl<'a, 'b, T: Transport>(&'b mut self, conn:  &'a EthConn<T>) -> DeployedKernelWithACL<'a, 'b, T> {
        let deployed_kernel = self.deploy_std(conn);
        let proxied_init_contract = web3::contract::Contract::from_json(
                conn.web3.eth(),
                deployed_kernel.address(),
                ACL_BOOTSTRAP.abi(),
            ).expect("proxied_init_contract");
        let entry_contract = deploy_contract(conn, ACL_ENTRY.code(), ACL_ENTRY.abi());
        let admin_contract = deploy_contract(conn, ACL_ADMIN.code(), ACL_ADMIN.abi());
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

        let main_account = &conn.sender;

        {
            let entry_proc_address: U256 = U256::from_big_endian(&[0xff, 0xff, 0xff, 0xff, 0x04, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00]);
            // println!("EntryProcAddress: 0x{}", entry_proc_address.to_hex());
            let store_val = conn.web3.eth().storage(deployed_kernel.address(), entry_proc_address, None).wait();
            println!("EntryProc: {:?}", store_val);
        }
        {
            let storage_address: U256 = U256::from_big_endian(&[0xff, 0xff, 0xff, 0xff, 0x00, 0x45, 0x6e, 0x74, 0x72, 0x79, 0x50, 0x72, 0x6f, 0x63, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00]);
            let storage_value = conn.web3.eth().storage(deployed_kernel.address(), storage_address, None).wait();
            println!("EntryProcAddress: {:?}", storage_value);
        }
        {
            let entry_proc_address: U256 = U256::from_big_endian(&[0xff, 0xff, 0xff, 0xff, 0x01, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00]);
            let store_val2 = conn.web3.eth().storage(deployed_kernel.address(), entry_proc_address, None).wait();
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
        let cap_index = 0;

        let contract = deploy_contract(&conn, include_bytes!("acl_group_5.wasm").to_vec(), include_bytes!("ACLGroup5Interface.json"));
        let cap_list: Vec<U256> = vec![];
        // let message = admin_contract.methods.regProc(cap_index, proc_key, contract.address, encodedRequestedCaps).encodeABI();
        // let proxy_message = tester.interface.methods.proxy(message).encodeABI();
        // await web3.eth.sendTransaction({ to: tester.kernel.contract.address, data: proxy_message, gas:2_100_000});
        // regInterface = contract;

        let _proxied_admin_contract = web3::contract::Contract::from_json(
                conn.web3.eth(),
                deployed_kernel.address(),
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
                deployed_kernel.address(),
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
        let test_account = deployed_kernel.address().clone();

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

        let entry_proc_address: U256 = U256::from_big_endian(&[0xff, 0xff, 0xff, 0xff, 0x02, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00]);
        println!("EntryProcAddress: 0x{:x?}", entry_proc_address);
        let store_val = conn.web3.eth().storage(deployed_kernel.address(), entry_proc_address, None).wait();
        println!("EntryProc: {:?}", store_val);

        let keys: Vec<H256> = serde_json::value::from_value(connection::list_storage_keys(deployed_kernel.address()).result.unwrap()).unwrap();
        for key in keys {
            let val = conn.web3.eth().storage(deployed_kernel.address(), key.as_fixed_bytes().into(), None).wait().expect("storage value");
            println!("key: {:?}, val: {:?}", key, val);
        }
        DeployedKernelWithACL::new(deployed_kernel)
    }
}

const EMPTY_KEY: [u8; 24] = [0; 24];

const DEFAULT_CAPS: [NewCapability; 7] = [
    NewCapability {
        cap: Capability::ProcedureRegister(ProcedureRegisterCap {
            prefix: 0,
            key: EMPTY_KEY,
        }),
        parent_index: 0,
    },
    NewCapability {
        cap: Capability::ProcedureRegister(ProcedureRegisterCap {
            prefix: 0,
            key: EMPTY_KEY,
        }),
        parent_index: 0,
    },
    NewCapability {
        cap: Capability::ProcedureCall(ProcedureCallCap {
            prefix: 0,
            key: EMPTY_KEY,
        }),
        parent_index: 0,
    },
    NewCapability {
        cap: Capability::ProcedureDelete(ProcedureDeleteCap {
            prefix: 0,
            key: EMPTY_KEY,
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
