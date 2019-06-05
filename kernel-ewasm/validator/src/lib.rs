#![no_std]

use pwasm_std;
use parity_wasm;

pub use parity_wasm::elements::{ImportEntry, Module};
use parity_wasm::elements::Instruction;
use parity_wasm::elements::{ValueType};

use pwasm_std::vec::Vec;

pub use parity_wasm::deserialize_buffer;

/// As per the wasm spec:
///
/// Function Index Space
///
/// The function index space indexes all imported and internally-defined
/// function definitions, assigning monotonically-increasing indices based on
/// the order of definition in the module (as defined by the binary encoding).
/// Thus, the index space starts at zero with the function imports (if any)
/// followed by the functions defined within the module.
// fn get_function_indices() {
//     // First get the imports
//     // Second get the functions
// }

/// A listing is a category of import. There are 3 types of imports whitelisted,
/// greylisted, and blacklisted. There is no blacklist, everything that is not
/// whitlisted or greylisted is blacklisted, even if we don't recognise it.
///
///  * Whitelisted: Functions which can be run with no state effects and we
///      don't care about them. Examples include getting addresses, returning,
///      reverting etc.
///  * Greylisted: Functions that _do_ perform dangerous operations, but that we
///      need for the operation of syscalls etc. These calls need to be
///      surrounded by the correct protections. These are permitted to be
///      imported, but must be checked for safety.
///  * Blacklisted: Everything else. These cannot even be imported. If they are
///      imported the contract is not valid.
#[derive(Debug)]
pub enum Listing {
    White,
    Grey,
    Black,
}

pub trait Listed {
    fn listing(&self) -> Listing;
}

impl Listed for ImportEntry {
    fn listing(&self) -> Listing {
        // Nothing should need to be imported from outside "env", but let's
        // blacklist it just in case.
        if self.module() != "env" {
            Listing::Black
        } else {
            // Tehcnically we don't have to list blacklisted items here, but we
            // do just for clarity.
            match self.field() {
                "memory" => Listing::White,
                "storage_read" => Listing::White,
                "storage_write" => Listing::Black,
                "ret" => Listing::White,
                "gas" => Listing::White,
                "input_length" => Listing::White,
                "fetch_input" => Listing::White,
                "panic" => Listing::White,
                "debug" => Listing::White,
                "ccall" => Listing::Black,
                "dcall" => Listing::Grey,
                "scall" => Listing::White,
                "value" => Listing::White,
                "create" => Listing::Black,
                "suicide" => Listing::White,
                "blockhash" => Listing::White,
                "blocknumber" => Listing::White,
                "coinbase" => Listing::White,
                "difficulty" => Listing::White,
                "gaslimit" => Listing::White,
                "timestamp" => Listing::White,
                "address" => Listing::White,
                "sender" => Listing::White,
                "origin" => Listing::White,
                "elog" => Listing::Black,
                "extcodesize" => Listing::White,
                "extcodecopy" => Listing::White,
                "create2" => Listing::Black,
                "gasleft" => Listing::White,
                _ => Listing::Black,
            }
        }
    }
}

/// Information on why the contract was considered invalid.
#[derive(Debug)]
pub struct ValidityReport {
    pub validation_errors: Vec<ValidityError>,
}

#[derive(Debug)]
pub enum ValidityError {
    BlacklistedImport(ImportEntry),
    UnsafeGreylistedCall {
        import: ImportEntry,
        function_index: u32,
        instruction_index: u32,
    },
}

/// Be able to determine a contracts validity.
pub trait Validity {
    fn is_valid(&self) -> bool;
    fn validity(&self) -> ValidityReport;
}

impl Validity for Module {
    fn is_valid(&self) -> bool {
        self.validity().validation_errors.len() == 0
    }

    fn validity(&self) -> ValidityReport {
        let imports = get_imports(self);
        let mut report = ValidityReport {
            validation_errors: Vec::new()
        };
        // TODO: this i value needs to be checked to ensure it is as defined by
        // the standard.
        for (import_index, import) in imports.iter().enumerate() {
            match import.listing() {
                Listing::White => (),
                Listing::Grey => {
                    // Check that this grey import is called safely, wherever is
                    // is called.
                    for (function_index,instruction_index) in check_grey(self, import_index) {
                        report.validation_errors.push(ValidityError::UnsafeGreylistedCall {
                            import: import.clone(),
                            function_index,
                            instruction_index,
                        });
                    }
                },
                Listing::Black => {
                    report.validation_errors.push(ValidityError::BlacklistedImport(import.clone()));
                },
            }
        }
        report
    }
}

fn get_imports(module: &Module) -> Vec<ImportEntry> {
    if let Some(import_section) = module.import_section() {
        import_section.entries().to_vec()
    } else {
        Vec::new()
    }
}

fn check_grey(module: &Module, grey_index: usize) -> Vec<(u32, u32)> {
    let mut uses = Vec::new();
    let code_section = module.code_section().unwrap();
    let codes = Vec::from(code_section.bodies());
    // If the instruction Call(grey_index) exists in the body of the function, that is a dangerous function.
    let this_call = parity_wasm::elements::Instruction::Call(grey_index as u32);
    for (func_index, func_body) in codes.iter().enumerate() {
        for (instruction_index, instruction) in func_body.code().elements().iter().enumerate() {
            if instruction == &this_call && !is_syscall(module, func_index as u32) {
                uses.push((func_index as u32, instruction_index as u32));
            }
        }
    }
    uses
}

// Find the function index of an import
pub fn find_import(module: &Module, mod_name: &str, field_name: &str) -> Option<u32> {
    let imports = module.import_section().unwrap().entries();
    for (i,import) in imports.iter().enumerate() {
        if import.module() == mod_name && import.field() == field_name {
            return Some(i as u32);
        }
    }
    return None;
}

pub fn is_syscall(module: &Module, function_index: u32) -> bool {

    let function_section = module.function_section().unwrap();
    let functions = Vec::from(function_section.entries());
    let function = functions.get(function_index as usize).unwrap();
    let type_index = function.type_ref();

    let type_section = module.type_section().unwrap();
    let types = Vec::from(type_section.types());
    let this_type = types.get(type_index as usize).unwrap();

    let code_section = module.code_section().unwrap();
    let codes = Vec::from(code_section.bodies());
    let code = codes.get(function_index as usize).unwrap();
    let instructions = Vec::from(code.code().elements());

    // First we need to check that the instructions are correct, that is:
    //   0. call $a
    //   1. call $b
    //   2. get_local 0
    //   3. get_local 1
    //   4. get_local 2
    //   5. get_local 3
    //   6. call $c
    // $a, $b, and $c will be used later.
    // First we simply check the length
    if instructions.len() != 8 {
        return false;
    }
    //   0. call gasleft
    if let Instruction::Call(f_ind) = instructions[0] {
        // Check that f_ind is the function index of "gasleft"
        let gasleft_index = find_import(module, "env", "gasleft");
        if Some(f_ind) != gasleft_index {
            return false;
        }
    } else {
        return false;
    }
    //   1. call sender
    if let Instruction::Call(f_ind) = instructions[1] {
        // Check that f_ind is the function index of "sender"
        let sender_index = find_import(module, "env", "sender");
        if Some(f_ind) != sender_index {
            return false;
        }
    } else {
        return false;
    }
    //   2. get_local 0
    if let Instruction::GetLocal(0) = instructions[2] {
    } else {
        return false;
    }
    //   3. get_local 1
    if let Instruction::GetLocal(1) = instructions[3] {
    } else {
        return false;
    }
    //   4. get_local 2
    if let Instruction::GetLocal(2) = instructions[4] {
    } else {
        return false;
    }
    //   5. get_local 3
    if let Instruction::GetLocal(3) = instructions[5] {
    } else {
        return false;
    }

    //   6. call dcall
    if let Instruction::Call(f_ind) = instructions[6] {
        // Check that f_ind is the function index of "dcall"
        let dcall_index = find_import(module, "env", "dcall");
        if Some(f_ind) != dcall_index {
            return false;
        }
    } else {
        return false;
    }
    //   7. END
    if let Instruction::End = instructions[7] {
    } else {
        return false;
    }

    // Check that no locals are used
    if code.locals().len() > 0 {
        return false;
    }
    // Check that the type signature is correct
    let parity_wasm::elements::Type::Function(f_type) = this_type;
    if f_type.return_type() != Some(ValueType::I32) {
        return false;
    }
    if f_type.params() != [ ValueType::I32, ValueType::I32, ValueType::I32, ValueType::I32] {
        return false;
    }
    if f_type.form() != 0x60 {
        return false;
    }

    true
}

#[cfg(test)]
mod tests {
    // extern crate pwasm_test;
    // use std;
    use super::*;
    use wabt::wat2wasm;
    // use core::str::FromStr;
    // use pwasm_abi::types::*;
    // use self::pwasm_test::{ext_reset, ext_get};
    // use token::TokenInterface;

    #[test]
    fn module_only_pass() {
        let wat = "(module)";
        let wasm = wat2wasm(wat).unwrap();
        let module: Module = parity_wasm::deserialize_buffer(wasm.as_slice()).expect("deserialise wasm");
        assert!(module.is_valid());
    }

    #[test]
    fn minimal_contract_pass() {
        let wat = r#"
;; Minimal contract
(module
  (type $t0 (func))
  (func $call (type $t0)
    unreachable)
  (export "call" (func $call)))
"#;
        let wasm = wat2wasm(wat).unwrap();
        let module: Module = parity_wasm::deserialize_buffer(wasm.as_slice()).expect("deserialise wasm");
        assert!(module.is_valid());
    }

    #[test]
    fn minimal_contract_with_write_fail() {
        let wat = r#"
;; Minimal contract with a single storage write call
(module
  (type $t0 (func))
  (type $t1 (func (param i32 i32)))
  (import "env" "storage_write" (func $env.storage_write (type $t1)))
  (func $call (type $t0)
    i32.const 5
    i32.const 15
    call $env.storage_write
    unreachable)
  (export "call" (func $call)))
"#;
        let wasm: pwasm_std::Vec<u8> = wat2wasm(wat).unwrap();
        let module: Module = parity_wasm::deserialize_buffer(wasm.as_slice()).expect("deserialise wasm");
        assert!(!module.is_valid());
    }

    // #[test]
    // fn should_reject_invalid_address() {
    //     let mut contract = contract::ValidatorContract {};
    //     let owner_address = Address::from_str("ea674fdde714fd979de3edf0f56aa9716b898ec8").unwrap();
    //     let invalid_address = Address::from_str("0").unwrap();

    //     // Here we're creating an External context using ExternalBuilder and set the `sender` to the `owner_address`
    //     // so `pwasm_ethereum::sender()` in TokenInterface::constructor() will return that `owner_address`
    //     ext_reset(|e| e.sender(owner_address.clone()));
    //     assert_eq!(contract.check_contract(invalid_address), false);
    // }
}
