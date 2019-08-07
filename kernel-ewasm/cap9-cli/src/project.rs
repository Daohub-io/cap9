// use web3::Transport;
// use web3::types::{Address};
// use web3::futures::Future;
use serde::{Deserialize, Serialize};
// use serde_json::Result;

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
