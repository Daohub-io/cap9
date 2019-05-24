extern crate parity_wasm;

use parity_wasm::elements::{ImportEntry, Module};
use clap::{Arg, App, SubCommand};
use parity_wasm::elements::Instruction;

fn main() {
    let matches = App::new("cap9-build")
            .version("0.2.0")
            .author("Cap9 <info@daohub.io>")
            .about("A command-line interface for linking Cap9 procedures.")
            .arg(Arg::with_name("INPUT-FILE")
                .required(true)
                .help("input file"))
            .arg(Arg::with_name("OUTPUT-FILE")
                .required(false)
                .help("input file"))
            .get_matches();

    let input_path = matches.value_of("INPUT-FILE").expect("input file is required");
    let output_path = matches.value_of("OUTPUT-FILE").expect("output path is required");

    let module = parity_wasm::deserialize_file(input_path).expect("parsing of input failed");
    // println!("Names {:?}", module);
    // println!("Names {:?}", module.clone().parse_names());
    // println!("Names {:?}", module.names_section());
    assert!(module.code_section().is_some());

    // TODO: we need to make sure these values never change between now and when
    // we use them. In the current set up they will not, but it is fragile,
    // there are changes that could be introduced which would change this.
    let dcall_index = find_import(&module, "env", "dcall").expect("No dcall import found");
    let gasleft_index = find_import(&module, "env", "gasleft").expect("No gasleft import found");
    let sender_index = find_import(&module, "env", "sender").expect("No sender import found");
    let syscall_instructions = parity_wasm::elements::Instructions::new(vec![
        // Call gas
        Instruction::Call(gasleft_index),
        // Call sender
        Instruction::Call(sender_index),
        Instruction::GetLocal(0),
        Instruction::GetLocal(1),
        Instruction::GetLocal(2),
        Instruction::GetLocal(3),
        // Do the delegate call
        Instruction::Call(dcall_index),
        // End function
        Instruction::End,
        ]);
    let new_module = parity_wasm::builder::from_module(module)
            .function()
                .signature()
                    .with_param(parity_wasm::elements::ValueType::I32)
                    .with_param(parity_wasm::elements::ValueType::I32)
                    .with_param(parity_wasm::elements::ValueType::I32)
                    .with_param(parity_wasm::elements::ValueType::I32)
                    .with_return_type(Some(parity_wasm::elements::ValueType::I32))
                    .build()
                .body()
                    .with_instructions(syscall_instructions)
                    .build()
                .build()
            .build();

    parity_wasm::serialize_to_file(output_path, new_module).expect("serialising to output failed");
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
