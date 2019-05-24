extern crate parity_wasm;

use parity_wasm::elements::{ImportEntry, Module};

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
            println!("{}: {}", i, show_validation(ve));
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
                "storage_write" => Listing::Black,
                "ccall" => Listing::Grey,
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
                    match check_grey(self, import_index) {
                        None => (),
                        Some(_) => return false,
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
                    match check_grey(self, import_index) {
                        None => (),
                        Some((function_index,instruction_index)) => {
                            report.validation_errors.push(ValidityError::UnsafeGreylistedCall {
                                import: import.clone(),
                                function_index,
                                instruction_index,
                            });

                        }
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

fn check_grey(module: &Module, grey_index: usize) -> Option<(u32, u32)> {
    let code_section = module.code_section().unwrap();
    let codes = Vec::from(code_section.bodies());
    // If the instruction Call(grey_index) exists in the body of the function, that is a dangerous function.
    let this_call = parity_wasm::elements::Instruction::Call(grey_index as u32);
    for (func_index, func_body) in codes.iter().enumerate() {
        for (instruction_index, instruction) in func_body.code().elements().iter().enumerate() {
            if instruction == &this_call {
                return Some((func_index as u32, instruction_index as u32))
            }
        }
    }
    None
}
