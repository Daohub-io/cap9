/// This module holds the default procedures that are compiled into the CLI
/// binary.
use std::path::PathBuf;
use std::io::prelude::*;
use crate::project::ContractSpec;
use std::convert::TryInto;

pub struct DefaultProcedure {
    code: &'static [u8],
    abi: &'static [u8],
    name: &'static str,
}

impl DefaultProcedure {
    pub fn code(&self) -> Vec<u8> {
        self.code.to_vec()
    }

    pub fn abi(&self) -> &'static [u8] {
        self.abi
    }

    pub fn write_abi(&self, dir: &PathBuf) -> PathBuf {
        let mut rel_path = PathBuf::new();
        rel_path.push(format!("{}-abi", self.name));
        rel_path.set_extension("json");
        let mut path = PathBuf::new();
        path.push(dir);
        path.push(rel_path.clone());
        let mut abi_file = std::fs::File::create(&path).expect(format!("Could not create file: {:?}", path).as_str());
        abi_file.write_all(self.abi()).unwrap();
        rel_path
    }

    pub fn write_code(&self, dir: &PathBuf) -> PathBuf {
        let mut rel_path = PathBuf::new();
        rel_path.push(format!("{}-code", self.name));
        rel_path.set_extension("wasm");
        let mut path = PathBuf::new();
        path.push(dir);
        path.push(rel_path.clone());
        let mut abi_file = std::fs::File::create(&path).expect(format!("Could not create file: {:?}", path).as_str());
        abi_file.write_all(&self.code()).unwrap();
        rel_path
    }

    pub fn contract_spec(self, dir: &PathBuf) -> ContractSpec {
        let code_path: String = self.write_code(dir).to_str().unwrap().to_string();
        let abi_path: String = self.write_abi(dir).to_str().unwrap().to_string();
        ContractSpec {
            code_path,
            abi_path,
        }
    }
}

pub const KERNEL: DefaultProcedure = DefaultProcedure {
    code: include_bytes!("cap9_kernel.wasm"),
    abi: include_bytes!("KernelInterface.json"),
    name: "cap9-kernel",
};

pub const ACL_BOOTSTRAP: DefaultProcedure = DefaultProcedure {
    code: include_bytes!("acl_bootstrap.wasm"),
    abi: include_bytes!("ACLBootstrapInterface.json"),
    name: "acl-bootstrap",
};

pub const ACL_ENTRY: DefaultProcedure = DefaultProcedure {
    code: include_bytes!("acl_entry.wasm"),
    abi: include_bytes!("ACLEntryInterface.json"),
    name: "acl-entry",
};

pub const ACL_ADMIN: DefaultProcedure = DefaultProcedure {
    code: include_bytes!("acl_admin.wasm"),
    abi: include_bytes!("ACLAdminInterface.json"),
    name: "acl-admin",
};
