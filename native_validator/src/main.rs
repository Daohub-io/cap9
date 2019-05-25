extern crate parity_wasm;

use parity_wasm::elements::{ImportEntry, Module};
use parity_wasm::elements::Instruction;
use parity_wasm::elements::{ValueType};
use std::fs::File;
use std::io::Read;
use native_validator::*;

fn main() {

    let mut f = File::open("../kernel-ewasm/example_contract_1/build/example_contract_1.wasm").expect("open file");

    let mut buffer = Vec::new();
    // read the whole file
    f.read_to_end(&mut buffer).expect("read file");
    let module: Module = parity_wasm::deserialize_buffer(buffer.as_slice()).expect("desrialise wasm");
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
