use web3::Transport;
use web3::contract::{Contract, Options};
use web3::types::{Address, U256, H256};
use web3::contract::tokens::Tokenize;
use serde::{Deserialize, Serialize};

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
use cap9_std::proc_table::cap::*;

use std::collections::HashMap;

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
    pub initial_entry: ProcSpec,
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
pub struct ProcSpec {
    pub contract_spec: ContractSpec,
    pub cap_path: PathBuf,
}

impl ProcSpec {
    /// Deploy the underlying contract to the blockchain.
    pub fn deploy<T: Transport, P: Tokenize>(&self, conn: &EthConn<T>, params: P) -> Result<Contract<T>, ContractDeploymentError> {
        // Before a procedure contract is deployed, it must go through
        // "proc-build".
        let module = parity_wasm::deserialize_file(&self.contract_spec.code_path).expect("ProcSpec::deploy() - parsing of input failed");
        let new_module = crate::build::contract_build(module);
        let code: Vec<u8> = parity_wasm::serialize(new_module).expect("serialising to output failed");
        deploy_contract(conn, code, &self.contract_spec.abi(), params)
    }
}

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

    /// Get a ['ContractSpec'] from files. The paths are relative to the project
    /// directory.
    pub fn from_files(code_file: &PathBuf, abi_file: &PathBuf) -> Self {
        ContractSpec {
            code_path: code_file.to_str().unwrap().to_string(),
            abi_path: abi_file.to_str().unwrap().to_string(),
        }
    }

    pub fn deploy<T: Transport, P: Tokenize>(&self, conn: &EthConn<T>, params: P) -> Result<Contract<T>, ContractDeploymentError> {
        let code: Vec<u8> = self.code();
        let abi: Vec<u8> = self.abi();
        let deploy_result = deploy_contract(conn, code, &abi, params);
        deploy_result
    }

    pub fn code(&self) -> Vec<u8> {
        let mut f_code = File::open(&self.code_path).expect("could not open file");
        let mut code: Vec<u8> = Vec::new();
        f_code.read_to_end(&mut code).unwrap();
        code
    }

    pub fn code_reader(&self) -> File {
        File::open(&self.code_path).expect("could not open file")
    }

    pub fn abi(&self) -> Vec<u8> {
        let mut f_abi = File::open(&self.abi_path).expect("could not open file");
        let mut abi: Vec<u8> = Vec::new();
        f_abi.read_to_end(&mut abi).unwrap();
        abi
    }

    pub fn abi_reader(&self) -> File {
        File::open(&self.abi_path).expect("could not open file")
    }
}

#[derive(Serialize, Deserialize)]
pub struct StatusFile {
    pub kernel_address: Address,
    /// A map from Contract addresses to ABI files.
    pub abis: HashMap<Address, PathBuf>,
}

impl StatusFile {
    pub fn new(address: Address) -> Self {
        StatusFile {
            kernel_address: address,
            abis: HashMap::new(),
        }
    }

    pub fn add_abi(&mut self, contract_address: Address, abi_path: PathBuf) {
        self.abis.insert(contract_address, abi_path);
    }
}

#[derive(Debug, Fail)]
pub enum ProjectDeploymentError {
    #[fail(display = "failed to deploy a contract \"{}\" which is necessary for project, due to: {}", contract_name, error)]
    ContractDeploymentError {
        contract_name: String,
        error: String,
    },
    #[fail(display = "incorrect parameters passed to constructor: {}", err)]
    BadParameters {
        err: String,
    },
    #[fail(display = "Could not form a proxied contract: {}", err)]
    ProxiedProcedureError {
        err: String,
    },
}

/// A representation of the local project information. Methods on this struct
/// operate on the local filesystem.
pub struct LocalProject {
    /// The location of the project.
    abs_path: PathBuf,
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
        let init_entry_spec = ProcSpec {
            contract_spec: ContractSpec::from_default(ACL_BOOTSTRAP, &dir, "acl_bootstrap".to_string()),
            // TODO: fix cap path
            cap_path: PathBuf::from("example_caps.json"),
        };
        let deploy_spec = DeploySpec {
            initial_entry: init_entry_spec,
        };
        let deploy_file = DeployFile::new(kernel_spec, deploy_spec);
        let f = File::create(&path).expect("Could not create file");
        serde_json::ser::to_writer_pretty(f, &deploy_file).expect("Could not serialise deploy data");
        let abs_path = PathBuf::from(".").canonicalize().unwrap();
        LocalProject {
            abs_path,
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
        let init_entry_spec = ProcSpec {
            contract_spec: ContractSpec::from_default(ACL_BOOTSTRAP, &dir, "acl_bootstrap".to_string()),
            cap_path: PathBuf::from("example_caps.json"),
        };
        let deploy_spec = DeploySpec {
            initial_entry: init_entry_spec,
        };
        let deploy_file = DeployFile::new_with_acl(kernel_spec, deploy_spec);
        let f = File::create(&path).expect("Could not create file");
        serde_json::ser::to_writer_pretty(f, &deploy_file).expect("Could not serialise deploy data");
        let abs_path = PathBuf::from(".").canonicalize().unwrap();
        LocalProject {
            abs_path,
            deploy_file,
            status_file: None,
        }
    }

    pub fn read() -> Self {
        Self::read_dir(&PathBuf::from("."))
    }

    pub fn read_dir(dir: &PathBuf) -> Self {
        let deploy_file_path: PathBuf = [dir, &PathBuf::from("deploy.json")].iter().collect();
        let status_file_path: PathBuf = [dir, &PathBuf::from("status.json")].iter().collect();
        let f_deploy = File::open(deploy_file_path).expect("could not open file");
        let deploy_file = serde_json::from_reader(f_deploy).expect("Could not parse deploy file");
        let f_status = File::open(status_file_path);
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
        let abs_path = dir.canonicalize().unwrap();
        LocalProject {
            abs_path,
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

    pub fn status_file_mut(&mut self) -> &mut Option<StatusFile> {
        &mut self.status_file
    }

    /// Write out a status file.
    pub fn add_status_file(&mut self, address: Address) {
        let status_file = StatusFile::new(address);
        let status_file_path: PathBuf = [&self.abs_path, &PathBuf::from("status.json")].iter().collect();
        let out_file = File::create(status_file_path).expect("could not create status file");
        serde_json::to_writer_pretty(out_file, &status_file).expect("could not serialise to file");
        self.status_file = Some(status_file);
    }

    pub fn write_status_file(&self) {
        let status_file: &StatusFile = self.status_file().as_ref().unwrap();
        let status_file_path: PathBuf = [&self.abs_path, &PathBuf::from("status.json")].iter().collect();
        let out_file = File::create(status_file_path).expect("could not create status file");
        serde_json::to_writer_pretty(out_file, &status_file).expect("could not serialise to file");
    }

    pub fn deploy<'a, 'b, T: Transport>(self, conn:  &'a EthConn<T>) -> Result<(),ProjectDeploymentError> {
        // Deploy initial procedure
        let deploy_file = self.deploy_file();
        if deploy_file.standard_acl_abi {
            self.deploy_with_acl(&conn).map(|_| ())
        } else {
            self.deploy_std(&conn).map(|_| ())
        }
    }

    /// Convert a path to be relative to the location of the project.
    pub fn rel_path(&self, path: &PathBuf) -> PathBuf {
        [&self.abs_path, path].iter().collect()
    }

    pub fn deploy_std<'a, T: Transport>(mut self, conn:  &'a EthConn<T>) -> Result<DeployedKernel<'a, T>, ProjectDeploymentError> {
        let deploy_file = self.deploy_file();
        // Deploy initial procedure
        // TODO: does the initial procedure need contructor parameters?
        let init_contract = &deploy_file.deploy_spec.initial_entry.deploy(&conn, ( ))
            .map_err(|err| ProjectDeploymentError::ContractDeploymentError {contract_name: "Init contract".to_string(), error: format!("{:?}", err)})?;
        // Setup some parameters for the the kernel constructor
        let proc_key = String::from("init");
        let proc_address = init_contract.address();
        let entry_caps: Vec<NewCapability> = DEFAULT_CAPS.to_vec();
        let serial_cap_list = SerialNewCapList(NewCapList(entry_caps.clone()));
        let json = serde_json::to_string_pretty(&serial_cap_list);
        println!("json_cap_list: {}", json.unwrap());

        let cap_list: NewCapList = NewCapList(entry_caps.clone());
        let encoded_cap_list: Vec<U256> = from_common_u256_vec(cap_list.to_u256_list());

        let kernel_constructor_params = (proc_key, proc_address, encoded_cap_list);
        let kernel_contract = &deploy_file.kernel.deploy(&conn, kernel_constructor_params)
            .map_err(|err| ProjectDeploymentError::ContractDeploymentError {contract_name: "Kernel contract".to_string(), error: format!("{:?}", err)})?;

        self.add_status_file(kernel_contract.address());
        Ok(DeployedKernel::new(conn, self))
    }

    pub fn deploy_with_acl<'a, T: Transport>(self, conn:  &'a EthConn<T>) -> Result<DeployedKernelWithACL<'a, T>, ProjectDeploymentError> {
        let mut deployed_kernel = self.deploy_std(conn)?;
        let proxied_init_contract = web3::contract::Contract::from_json(
                conn.web3.eth(),
                deployed_kernel.address(),
                ACL_BOOTSTRAP.abi(),
            )
            .map_err(|err| ProjectDeploymentError::ProxiedProcedureError {err: format!("{:?}", err)})?;
        let entry_contract_spec = ACL_ENTRY.contract_spec(&deployed_kernel.local_project.abs_path);
        let admin_contract_spec = ACL_ADMIN.contract_spec(&deployed_kernel.local_project.abs_path);

        let entry_proc_spec = ProcSpec {
            contract_spec: entry_contract_spec,
            cap_path: PathBuf::from(""),
        };
        let admin_proc_spec = ProcSpec {
            contract_spec: admin_contract_spec,
            cap_path: PathBuf::from(""),
        };

        let entry_contract = entry_proc_spec.deploy(conn, ( )).unwrap();
        let admin_contract = admin_proc_spec.deploy(conn, ( )).unwrap();
        // let entry_path = ACL_ENTRY.write_abi(&deployed_kernel.local_project.abs_path);
        // let admin_path = ACL_ADMIN.write_abi(&deployed_kernel.local_project.abs_path);
        // let entry_contract = deploy_contract(conn, ACL_ENTRY.code(), ACL_ENTRY.abi(), ( ))
            // .map_err(|err| ProjectDeploymentError::ContractDeploymentError {contract_name: "ACL entry contract".to_string(), error: format!("{:?}", err)})?;
        {
            let local_project: &mut LocalProject = &mut deployed_kernel.local_project;
            let status_file: &mut StatusFile = local_project.status_file.as_mut().unwrap();
            status_file.add_abi(entry_contract.address(), PathBuf::from(entry_proc_spec.contract_spec.abi_path));
        }
        // let admin_contract = deploy_contract(conn, ACL_ADMIN.code(), ACL_ADMIN.abi(), ( ))
            // .map_err(|err| ProjectDeploymentError::ContractDeploymentError {contract_name: "ACL admin contract".to_string(), error: format!("{:?}", err)})?;
        {
            let local_project: &mut LocalProject = &mut deployed_kernel.local_project;
            let status_file: &mut StatusFile = local_project.status_file.as_mut().unwrap();
            status_file.add_abi(admin_contract.address(), PathBuf::from(admin_proc_spec.contract_spec.abi_path));
        }
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

        let keys: Vec<H256> = serde_json::value::from_value(connection::list_storage_keys(deployed_kernel.address()).result.unwrap()).unwrap();
        for key in keys {
            let val = conn.web3.eth().storage(deployed_kernel.address(), key.as_fixed_bytes().into(), None).wait().expect("storage value");
            println!("key: {:?}, val: {:?}", key, val);
        }
        deployed_kernel.local_project.write_status_file();
        Ok(DeployedKernelWithACL::new(deployed_kernel))
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
