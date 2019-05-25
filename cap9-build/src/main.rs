extern crate parity_wasm;
extern crate pwasm_utils;

use parity_wasm::elements::{Module};
use clap::{Arg, App};
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

    // TODO: what is the index of this newly added function?
    let mut new_module = parity_wasm::builder::from_module(module)
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

    // TODO: robustly determine the function index of the function we just
    // added. I think at this point it's simply the last funciton added, thereby
    // functions_space - 1, but this is not guaranteed anywhere.
    let added_syscall_index = new_module.functions_space() - 1;

    // If we find cap9_syscall_low as an import, we need to replace all
    // references to it with a reference to this newly added function, and
    // remove the import. Once we replace the internal references and run optimize, it will be removed anyway.
    let cap9_syscall_low_index = find_import(&new_module, "env", "cap9_syscall_low");
    match cap9_syscall_low_index {
        None => (),
        Some(syscall_index) => {
            // Search though the code of each function, if we encounter a
            // Call(syscall_index), replace it with Call(added_syscall_index).
            // TODO: investigate the use of CallIndirect
            for f in new_module.code_section_mut().unwrap().bodies_mut().iter_mut() {
                for i in 0..f.code().elements().len() {
                    let instruction = &f.code().elements()[i];
                    if instruction == &Instruction::Call(syscall_index) {
                        f.code_mut().elements_mut()[i] = Instruction::Call(added_syscall_index as u32);
                    }
                }
            }
        }
    }

    // Next we want to delete dummy_syscall if it exists. First we find it among
    // the exports (if it doesn't exist we don't need to do anything). We take
    // the reference of the export (i.e. the function it exports) and delete
    // both that function and the export. One way to do this would be to delete
    // the export and run the parity's optimizer again.
    // 1. Get the index of the export
    if let Some(dummy_syscall_export_index) = find_export(&new_module, "dummy_syscall") {
        // println!("dummy_syscall_export_index: {}", dummy_syscall_export_index);
        // 2. Delete the export
        new_module.export_section_mut().unwrap().entries_mut().remove(dummy_syscall_export_index as usize);
    }
    // 3. At this stage the dummy_syscall function still exists internally. We
    //    can't use the same remove procedure without screwing up the internal
    //    references, so we will just run the parity optmizer again for now to
    //    let it deal with that.
    pwasm_utils::optimize(&mut new_module, vec!["call"]).unwrap();

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

// Find the function index of an export
fn find_export(module: &Module, field_name: &str) -> Option<u32> {
    let exports = module.export_section().unwrap().entries();
    for (i,export) in exports.iter().enumerate() {
        if export.field() == field_name {
            return Some(i as u32);
        }
    }
    return None;
}
