extern crate parity_wasm;

use parity_wasm::elements::{ImportEntry, Module};
use parity_wasm::elements::Instruction;
use parity_wasm::elements::{FunctionType, ValueType};

fn main() {
    let module = parity_wasm::deserialize_file("../kernel-ewasm/example_contract_1/build/example_contract_1.wasm").unwrap();
    assert!(module.code_section().is_some());

    // We have now located the bad imports, but this does not establish if they
    // are used. It does not check that it is actually used. For now we will
    // assumed that if it is imported it is used. This could cause false
    // positives where code imports it but does not use it, however, this is not
    // expected to be common as most compilers would optimise that out fairly
    // trivially, and it also makes it much easier and cheaper for us.

    let validity =  module.validity();
    if validity.validation_errors.len() != 0 {
        println!("Module is not valid, the following validation errors were found:");
        for (i, ve) in validity.validation_errors.iter().enumerate() {
            println!("  {}: {}", i, show_validation(ve));
        }
    } else {
        println!("Module is valid");
    }
}

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
///  * Greylisted: Functions that _do_ perform dangerous operations, by that we
///      need for the operation of syscalls etc. These calls need to be
///      surrounded by the correct protections. These are permitted to be
///      imported, but must be checked for safety.
///  * Blacklisted: Everything else. These cannot even be imported. If they are
///      imported the contract is not valid.
#[derive(Debug)]
enum Listing {
    White,
    Grey,
    Black,
}

trait Listed {
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
                "ret" => Listing::White,
                "memory" => Listing::White,
                "gasleft" => Listing::White,
                "sender" => Listing::White,
                "storage_write" => Listing::Black,
                "ccall" => Listing::Black,
                "dcall" => Listing::Grey,
                _ => Listing::Black,
            }
        }
    }
}

/// Information on why the contract was considered invalid.
#[derive(Debug)]
struct ValidityReport {
    validation_errors: Vec<ValidityError>,
}

#[derive(Debug)]
enum ValidityError {
    BlacklistedImport(ImportEntry),
    UnsafeGreylistedCall {
        import: ImportEntry,
        function_index: u32,
        instruction_index: u32,
    },
}

fn show_validation(ve: &ValidityError) -> String {
    match ve {
        ValidityError::BlacklistedImport(import) => format!("A blacklisted import ({}.{}) was found", import.module(), import.field()),
        ValidityError::UnsafeGreylistedCall {
            import,
            function_index,
            instruction_index,
        } => format!("A greylisted import ({}.{}) was called unsafely in function {} at instruction {}", import.module(), import.field(), function_index, instruction_index),
    }
}

/// Be able to determine a contracts validity.
trait Validity {
    fn is_valid(&self) -> bool;
    fn validity(&self) -> ValidityReport;
}

impl Validity for Module {
    fn is_valid(&self) -> bool {
        let import_section = self.import_section().unwrap();
        let imports = Vec::from(import_section.entries());
        // TODO: this i value needs to be checked to ensure it is as defined by
        // the standard.
        for (import_index, import) in imports.iter().enumerate() {
            match import.listing() {
                Listing::White => (),
                Listing::Grey => {
                    // Check that this grey import is called safely, wherever is
                    // is called.

                    if check_grey(self, import_index).len() > 0 {
                        return false;
                    }
                },
                Listing::Black => return false,
            }
        }
        true
    }

    fn validity(&self) -> ValidityReport {
        let import_section = self.import_section().unwrap();
        let imports = Vec::from(import_section.entries());
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

fn check_grey(module: &Module, grey_index: usize) -> Vec<(u32, u32)> {
    let mut uses = Vec::new();
    let code_section = module.code_section().unwrap();
    let codes = Vec::from(code_section.bodies());
    // If the instruction Call(grey_index) exists in the body of the function, that is a dangerous function.
    let this_call = parity_wasm::elements::Instruction::Call(grey_index as u32);
    for (func_index, func_body) in codes.iter().enumerate() {
        for (instruction_index, instruction) in func_body.code().elements().iter().enumerate() {
            if instruction == &this_call && is_syscall(module, func_index as u32) {
                uses.push((func_index as u32, instruction_index as u32));
            }
        }
    }
    uses
}

// Find the function index of an import
fn find_import(module: &Module, mod_name: &str, field_name: &str) -> Option<u32> {
    let imports = module.import_section().unwrap().entries();
    for (i,import) in imports.iter().enumerate() {
        if import.module() == mod_name && import.field() == field_name {
            return Some(i as u32);
        }
    }
    return None;
}

fn is_syscall(module: &Module, function_index: u32) -> bool {

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
        println!("wrong number of instructions");
        return false;
    }
    //   0. call gasleft
    if let Instruction::Call(f_ind) = instructions[0] {
        // Check that f_ind is the function index of "gasleft"
        let gasleft_index = find_import(module, "env", "gasleft");
        if Some(f_ind) != gasleft_index {
            println!("not gasleft");
            return false;
        }
    } else {
        println!("not call1");
        return false;
    }
    //   1. call sender
    if let Instruction::Call(f_ind) = instructions[1] {
        // Check that f_ind is the function index of "sender"
        let sender_index = find_import(module, "env", "sender");
        if Some(f_ind) != sender_index {
            println!("not sender");
            return false;
        }
    } else {
        println!("not call2");
        return false;
    }
    //   2. get_local 0
    if let Instruction::GetLocal(0) = instructions[2] {
    } else {
        println!("not get_local 0");
        return false;
    }
    //   3. get_local 1
    if let Instruction::GetLocal(1) = instructions[3] {
    } else {
        println!("not get_local 1");
        return false;
    }
    //   4. get_local 2
    if let Instruction::GetLocal(2) = instructions[4] {
    } else {
        println!("not get_local 2");
        return false;
    }
    //   5. get_local 3
    if let Instruction::GetLocal(3) = instructions[5] {
    } else {
        println!("not get_local 3");
        return false;
    }

    //   6. call dcall
    if let Instruction::Call(f_ind) = instructions[6] {
        // Check that f_ind is the function index of "dcall"
        let dcall_index = find_import(module, "env", "dcall");
        if Some(f_ind) != dcall_index {
            println!("not dcall");
            return false;
        }
    } else {
        println!("not call3");
        return false;
    }
    //   7. END
    if let Instruction::End = instructions[7] {
    } else {
        println!("not end");
        return false;
    }

    // Check that no locals are used
    if code.locals().len() > 0 {
        println!("locals used");
        return false;
    }
    // Check that the type signature is correct
    if let parity_wasm::elements::Type::Function(f_type) = this_type {
        if f_type.return_type() != Some(ValueType::I32) {
            println!("incorrect return type");
            return false;
        }
        if Vec::from(f_type.params()) != vec![ ValueType::I32, ValueType::I32, ValueType::I32, ValueType::I32] {
            println!("incorrect params");
            return false;
        }
        if f_type.form() != 0x60 {
            println!("incorrect form");
            return false;
        }
    } else {
        println!("not function type");
        return false;
    }

    true
}
