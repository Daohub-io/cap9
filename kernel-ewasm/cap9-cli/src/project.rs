// use web3::Transport;
use web3::types::{Address};
// use web3::futures::Future;
use serde::{Deserialize, Serialize};
// use serde_json::Result;

use std::fs::create_dir;
use std::fs::File;
use std::path::PathBuf;

const ACL_BOOTSTRAP: &[u8] = include_bytes!("acl_bootstrap.wasm");
const KERNEL_CODE: &[u8] = include_bytes!("cap9-kernel.wasm");


#[derive(Serialize, Deserialize)]
pub struct DeployFile {
    // pub sender: Address,
    pub deploy_spec: DeploySpec,
    pub kernel_code: ContractSpec,
}

impl DeployFile {
    pub fn new() -> Self {
        DeployFile {
            deploy_spec: DeploySpec::new(),
            kernel_code: ContractSpec {
                bytes: KERNEL_CODE.clone().to_vec()
            },
        }
    }
}

/// The information defining the structure of a deployed kernel. For example,
/// the initial entry procedure.
#[derive(Serialize, Deserialize)]
pub struct DeploySpec {
    pub initial_entry: ContractSpec,
}

impl DeploySpec {
    /// Create a new DeploySpec using the acl bootstrap procedure as a default.
    pub fn new() -> Self {
        DeploySpec {
            initial_entry: ContractSpec {
                bytes: ACL_BOOTSTRAP.clone().to_vec(),
            }
        }
    }
}

#[derive(Serialize, Deserialize)]
pub struct ContractSpec {
    #[serde(with = "serde_bytes")]
    pub bytes: Vec<u8>,
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
        let deploy_file = DeployFile::new();
        let mut path = PathBuf::new();
        path.push(project_name);
        path.push("deploy");
        path.set_extension("json");
        let f = File::create(path).expect("Could not create file");
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
}
