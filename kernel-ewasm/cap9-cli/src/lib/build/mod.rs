use std::{fs, io};
use std::path::PathBuf;
use clap::{App, Arg, ArgMatches, SubCommand};
use parity_wasm::elements;
use parity_wasm::elements::{Instruction, Instructions};
use parity_wasm::elements::{MemoryType, Module};
use pwasm_utils::{build, BuildError, SourceTarget, TargetRuntime};

#[derive(Debug)]
pub enum Error {
    Io(io::Error),
    FailedToCopy(String),
    Decoding(elements::Error, String),
    Encoding(elements::Error),
    Build(BuildError),
}

impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> Result<(), std::fmt::Error> {
        use self::Error::*;
        match *self {
            Io(ref io) => write!(f, "Generic i/o error: {}", io),
            FailedToCopy(ref msg) => write!(f, "{}. Have you tried to run \"cargo build\"?", msg),
            Decoding(ref err, ref file) => write!(f, "Decoding error ({}). Must be a valid wasm file {}. Pointed wrong file?", err, file),
            Encoding(ref err) => write!(f, "Encoding error ({}). Almost impossible to happen, no free disk space?", err),
            Build(ref err) => write!(f, "Build error: {}", err)
        }
    }
}


pub fn build_commands<'a, 'b>() -> Vec<App<'a, 'b>> {
    let full_command: App = SubCommand::with_name("full")
        .arg(Arg::with_name("INPUT-FILE")
            .index(1)
            .required(true)
            .help("input .wasm file"))
        .arg(Arg::with_name("OUTPUT-FILE")
            .index(2)
            .required(true)
            .help("output .wasm file"))
        .arg(Arg::with_name("target-runtime")
            .help("What runtime we are compiling to")
            .long("target-runtime")
            .takes_value(true)
            .default_value("pwasm")
            .possible_values(&["substrate", "pwasm"]))
        .arg(Arg::with_name("skip_optimization")
            .help("Skip symbol optimization step producing final wasm")
            .long("skip-optimization"))
        .arg(Arg::with_name("enforce_stack_adjustment")
            .help("Enforce stack size adjustment (used for old wasm32-unknown-unknown)")
            .long("enforce-stack-adjustment"))
        .arg(Arg::with_name("runtime_type")
            .help("Injects RUNTIME_TYPE global export")
            .takes_value(true)
            .long("runtime-type"))
        .arg(Arg::with_name("runtime_version")
            .help("Injects RUNTIME_VERSION global export")
            .takes_value(true)
            .long("runtime-version"))
        .arg(Arg::with_name("source_target")
            .help("Cargo target type kind ('wasm32-unknown-unknown' or 'wasm32-unknown-emscripten'")
            .takes_value(true)
            .long("target"))
        .arg(Arg::with_name("final_name")
            .help("Final wasm binary name")
            .takes_value(true)
            .long("final"))
        .arg(Arg::with_name("save_raw")
            .help("Save intermediate raw bytecode to path")
            .takes_value(true)
            .long("save-raw"))
        .arg(Arg::with_name("shrink_stack")
            .help("Shrinks the new stack size for wasm32-unknown-unknown")
            .takes_value(true)
            .long("shrink-stack"))
        .arg(Arg::with_name("public_api")
            .help("Preserves specific imports in the library")
            .takes_value(true)
            .long("public-api"));
    let wasm_build_command: App = SubCommand::with_name("wasm-build")
        .arg(Arg::with_name("INPUT-FILE")
            .index(1)
            .required(true)
            .help("input .wasm file"))
        .arg(Arg::with_name("OUTPUT-FILE")
            .index(2)
            .required(true)
            .help("output .wasm file"))
        .arg(Arg::with_name("target-runtime")
            .help("What runtime we are compiling to")
            .long("target-runtime")
            .takes_value(true)
            .default_value("pwasm")
            .possible_values(&["substrate", "pwasm"]))
        .arg(Arg::with_name("skip_optimization")
            .help("Skip symbol optimization step producing final wasm")
            .long("skip-optimization"))
        .arg(Arg::with_name("enforce_stack_adjustment")
            .help("Enforce stack size adjustment (used for old wasm32-unknown-unknown)")
            .long("enforce-stack-adjustment"))
        .arg(Arg::with_name("runtime_type")
            .help("Injects RUNTIME_TYPE global export")
            .takes_value(true)
            .long("runtime-type"))
        .arg(Arg::with_name("runtime_version")
            .help("Injects RUNTIME_VERSION global export")
            .takes_value(true)
            .long("runtime-version"))
        .arg(Arg::with_name("source_target")
            .help("Cargo target type kind ('wasm32-unknown-unknown' or 'wasm32-unknown-emscripten'")
            .takes_value(true)
            .long("target"))
        .arg(Arg::with_name("final_name")
            .help("Final wasm binary name")
            .takes_value(true)
            .long("final"))
        .arg(Arg::with_name("save_raw")
            .help("Save intermediate raw bytecode to path")
            .takes_value(true)
            .long("save-raw"))
        .arg(Arg::with_name("shrink_stack")
            .help("Shrinks the new stack size for wasm32-unknown-unknown")
            .takes_value(true)
            .long("shrink-stack"))
        .arg(Arg::with_name("public_api")
            .help("Preserves specific imports in the library")
            .takes_value(true)
            .long("public-api"));
    let build_command: App = SubCommand::with_name("build-proc")
        .about("Convert a regular contract into a cap9 procedure.")
        .arg(
            Arg::with_name("INPUT-FILE")
                .required(true)
                .help("input file"),
        )
        .arg(
            Arg::with_name("OUTPUT-FILE")
                .required(true)
                .help("output file"),
        );
    let set_mem_command: App = SubCommand::with_name("set-mem")
        .about("Set the number of memory pages in a procedure.")
        .arg(
            Arg::with_name("INPUT-FILE")
                .required(true)
                .help("input file"),
        )
        .arg(
            Arg::with_name("OUTPUT-FILE")
                .required(true)
                .help("output file"),
        )
        .arg(
            Arg::with_name("pages")
                .short("p")
                .long("pages")
                .value_name("PAGES")
                .required(true)
                .help("Number of pages to set the memory to"),
        );
    vec![build_command, set_mem_command, wasm_build_command, full_command]
}

pub fn execute_build_proc(opts: &ArgMatches) {
    let input_path = opts.value_of("INPUT-FILE").expect("input file is required");
    let output_path = opts
        .value_of("OUTPUT-FILE")
        .expect("output path is required");

    let module = parity_wasm::deserialize_file(input_path).expect("parsing of input failed");
    let new_module = contract_build(module);
    parity_wasm::serialize_to_file(output_path, new_module).expect("serialising to output failed");
}

pub fn execute_set_mem(opts: &ArgMatches) {
    let input_path = opts.value_of("INPUT-FILE").expect("input file is required");
    let output_path = opts
        .value_of("OUTPUT-FILE")
        .expect("output path is required");
    let mem_pages = opts
        .value_of("pages")
        .expect("number of memory pages is required");

    let module = parity_wasm::deserialize_file(input_path).expect("parsing of input failed");
    let new_module = set_mem( module, mem_pages .parse() .expect("expected number for number of pages"), );
    parity_wasm::serialize_to_file(output_path, new_module).expect("serialising to output failed");
}

pub fn execute_wasm_build(opts: &ArgMatches) {
    let input_path = opts.value_of("INPUT-FILE").expect("input file is required");
    let output_path = opts .value_of("OUTPUT-FILE") .expect("output path is required");

    let module = parity_wasm::deserialize_file(&input_path)
        .map_err(|e| Error::Decoding(e, input_path.to_string())).unwrap();
    let new_module = wasm_build(opts, module);
    parity_wasm::serialize_to_file(&output_path, new_module).map_err(Error::Encoding).unwrap();
}

fn wasm_build(opts: &ArgMatches, module: Module) -> Module {
    let runtime_type_version = if let (Some(runtime_type), Some(runtime_version))
         = (opts.value_of("runtime_type"), opts.value_of("runtime_version")) {
        let mut ty: [u8; 4] = Default::default();
        let runtime_bytes = runtime_type.as_bytes();
        if runtime_bytes.len() != 4 {
            panic!("--runtime-type should be equal to 4 bytes");
        }
        ty.copy_from_slice(runtime_bytes);
        let version: u32 = runtime_version.parse()
            .expect("--runtime-version should be a positive integer");
        Some((ty, version))
    } else {
        None
    };

    let public_api_entries = opts.value_of("public_api")
        .map(|val| val.split(",").collect())
        .unwrap_or(Vec::new());

    let target_runtime = match opts.value_of("target-runtime").expect("target-runtime has a default value; qed") {
        "pwasm" => TargetRuntime::pwasm(),
        "substrate" => TargetRuntime::substrate(),
        _ => unreachable!("all possible values are enumerated in clap config; qed"),
    };

    let (module, ctor_module) = build(
        module,
        SourceTarget::Unknown,
        runtime_type_version,
        &public_api_entries,
        opts.is_present("enforce_stack_adjustment"),
        opts.value_of("shrink_stack").unwrap_or_else(|| "49152").parse()
            .expect("New stack size is not valid u32"),
        opts.is_present("skip_optimization"),
        &target_runtime,
    ).map_err(Error::Build).expect("invalid build");

    if let Some(save_raw_path) = opts.value_of("save_raw") {
        parity_wasm::serialize_to_file(save_raw_path, module.clone()).map_err(Error::Encoding).unwrap();
    }

    if let Some(ctor_module) = ctor_module {
        ctor_module
    } else {
        module
    }
}

pub fn execute_full(opts: &ArgMatches) {
    let input_path = opts.value_of("INPUT-FILE").expect("input file is required");
    let output_path = opts .value_of("OUTPUT-FILE") .expect("output path is required");

    let module = parity_wasm::deserialize_file(input_path).expect("parsing of input failed");
    let contract_module = contract_build(module);
    let mem_pages = 4;
    let mem_module = set_mem(contract_module, mem_pages);
    let new_module = wasm_build(opts, mem_module);

    parity_wasm::serialize_to_file(output_path, new_module).expect("serialising to output failed");
}

/// Perform the operations necessary for cap9 procedures.
fn contract_build(module: Module) -> Module {

    // TODO: we need to make sure these values never change between now and when
    // we use them. In the current set up they will not, but it is fragile,
    // there are changes that could be introduced which would change this.
    let syscall_instructions_res = get_syscall_instructions(&module);

    // TODO: what is the index of this newly added function?
    let new_module_builder = parity_wasm::builder::from_module(module);
    // Add the syscall function, if applicable.
    let mut new_module = if let Ok(syscall_instructions) = syscall_instructions_res {
        new_module_builder
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
            .build()
    } else {
        new_module_builder.build()
    };

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
    pwasm_utils::optimize(&mut new_module, vec!["call","deploy"]).unwrap();
    new_module
}

fn set_mem(mut module: Module, num_pages: u32) -> Module {
    // We want to find the single memory section, and change it from its current
    // value to the one we've requested.
    let mem_entry: &mut Vec<MemoryType> = module.memory_section_mut().unwrap().entries_mut();
    mem_entry[0] = parity_wasm::elements::MemoryType::new(num_pages,None);
    module
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

enum SysCallError {
    NoDCall,
    NoGasLeft,
    NoSender,
}

fn get_syscall_instructions(module: &Module) -> Result<Instructions,SysCallError> {
    // If any of these three environments are not pulled in from the
    // environment, we cannot have syscalls.
    let dcall_index = find_import(module, "env", "dcall").ok_or(SysCallError::NoDCall)?;
    let gasleft_index = find_import(module, "env", "gasleft").ok_or(SysCallError::NoGasLeft)?;
    let sender_index = find_import(module, "env", "sender").ok_or(SysCallError::NoSender)?;
    let syscall_instructions = parity_wasm::elements::Instructions::new(vec![
        // Call gas
        Instruction::Call(gasleft_index),
        // TODO: this subtraction is a little hacky
        Instruction::I64Const(10000),
        Instruction::I64Sub,
        // Call sender, this will place the sender somewhere in memory,
        // therefore we need to allocate or something. An address is 160 bits
        // long, and therefore can't fit into a word. We need to place a
        // location here first.
        // TODO: allocate this memory rather than picking a random location.
        //
        // Place a memory location for the "sender" function to place the
        // address.
        Instruction::I32Const(80000),
        // Call the sender to function to place the address in memory.
        // TODO: because of the lack of call code, this will be incorrect.
        Instruction::Call(sender_index),
        // Place the same memory location on the stack again for use by the
        // dcall function.
        Instruction::I32Const(80000),
        Instruction::GetLocal(0),
        Instruction::GetLocal(1),
        Instruction::GetLocal(2),
        Instruction::GetLocal(3),
        // Do the delegate call
        Instruction::Call(dcall_index),
        // End function
        Instruction::End,
        ]);
    Ok(syscall_instructions)
}
