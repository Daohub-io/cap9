/// This module holds the default procedures that are compiled into the CLI
/// binary.


pub struct DefaultProcedure {
    code: &'static [u8],
    abi: &'static [u8],
}

impl DefaultProcedure {
    pub fn code(&self) -> Vec<u8> {
        self.code.to_vec()
    }

    pub fn abi(&self) -> &'static [u8] {
        self.abi
    }
}

pub const KERNEL: DefaultProcedure = DefaultProcedure {
    code: include_bytes!("cap9-kernel.wasm"),
    abi: include_bytes!("KernelInterface.json"),
};

pub const ACL_BOOTSTRAP: DefaultProcedure = DefaultProcedure {
    code: include_bytes!("acl_bootstrap.wasm"),
    abi: include_bytes!("ACLBootstrapInterface.json"),
};

pub const ACL_ENTRY: DefaultProcedure = DefaultProcedure {
    code: include_bytes!("acl_entry.wasm"),
    abi: include_bytes!("ACLEntryInterface.json"),
};

pub const ACL_ADMIN: DefaultProcedure = DefaultProcedure {
    code: include_bytes!("acl_admin.wasm"),
    abi: include_bytes!("ACLAdminInterface.json"),
};
