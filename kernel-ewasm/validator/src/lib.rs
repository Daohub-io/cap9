#![cfg_attr(not(feature = "std"), no_std)]

#[cfg(not(feature="std"))]
#[macro_use]
extern crate alloc;

use pwasm_std;
use pwasm_std::vec::Vec;
use pwasm_std::String;

pub mod instructions;
pub mod func;
mod primitives;
pub mod io;
pub mod serialization;
pub mod import_entry;
pub mod types;
pub use self::io::{Error};
pub use self::serialization::{Deserialize};

pub use self::primitives::{
    VarUint32, VarUint7, Uint8, VarUint1, VarInt7, Uint32, VarInt32, VarInt64,
    Uint64, VarUint64, CountedList
};

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

#[derive(Debug, Clone)]
pub struct ImportEntry {
    index: u32,
    mod_name: String,
    field_name: String,
}

impl Listed for ImportEntry {
    fn listing(&self) -> Listing {
        // Nothing should need to be imported from outside "env", but let's
        // blacklist it just in case.
        if self.mod_name != "env" {
            Listing::Black
        } else {
            // Tehcnically we don't have to list blacklisted items here, but we
            // do just for clarity.
            match self.field_name.as_ref() {
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

/// Be able to determine a contracts validity.
pub trait Validity {
    fn is_valid(&self) -> bool;
}

// Seek does not seem to be implemented in core, so we'll reimplement what we
// need.
#[derive(Debug)]
struct Cursor<'a> {
    current_offset: usize,
    body: &'a [u8],
}

impl<'a> Cursor<'a> {
    // Read the byte at the cusor, and increment the pointer by 1.
    fn read_ref(&mut self) -> &'a u8 {
        let val = &self.body[self.current_offset];
        self.current_offset += 1;
        val
    }

    fn read_ref_n(&mut self, n: usize) -> &'a [u8] {
        let val = &self.body[self.current_offset..(self.current_offset+n)];
        self.current_offset += n;
        val
    }

    fn skip(&mut self, n: usize) {
        self.current_offset += n;
    }
}

/// Implement standard read definition (which clones). This is basically the
/// rust definition of read for slice.
impl<'a> io::Read for Cursor<'a> {
    fn read(&mut self, buf: &mut [u8]) -> io::Result<()> {
        let actual_self = &self.body[self.current_offset..];
        let amt = core::cmp::min(buf.len(), actual_self.len());
        let (a, _) = actual_self.split_at(amt);

        if amt == 1 {
            buf[0] = a[0];
        } else {
            buf[..amt].copy_from_slice(a);
        }

        self.current_offset += amt;
        Ok(())
    }
}

impl Validity for &[u8] {
    fn is_valid(&self) -> bool {
        // Set an index value, which is our offset into the wasm bytes.
        let mut cursor = Cursor {
            current_offset: 0,
            body: self,
        };

        // The first two steps are to take the magic number and version to check
        // that it is valid wasm. This is not strictly necessary, as it is the
        // job of the runtime to ensure the wasm is valid (ad we rely on that
        // fact), however, it's cheap and allows us prevent future versions of
        // wasm code being deployed (for which our assumptions may not hold).

        // Take the magic number, check that it matches
        if cursor.read_ref_n(4) != &[0, 97, 115, 109] {
            panic!("magic number not found");
        }

        // Take the version, check that it matches
        if cursor.read_ref_n(4) != &[1, 0, 0, 0] {
            panic!("proper version number not found");
        }

        // Now we should be at the first section. We care about 4 sections:
        // types, imports, functions, and code. The first thing we want to do is
        // to find the offsets of these 4 sections. We assume the wasm is well
        // formed and there are no duplicate sections and the like. It is also
        // possible some of these sections don't exist.
        let mut type_section_offset: Option<usize> = None;
        let mut import_section_offset: Option<usize> = None;
        let mut function_section_offset: Option<usize> = None;
        let mut code_section_offset: Option<usize> = None;
        while cursor.current_offset < self.len() {
            let section: Section = parse_section(&mut cursor);
            // There are many section types we don't care about, for example,
            // Custom sections generally contain debugging symbols and
            // meaningful function names which are irrelevant to the current
            // process. We care only about types, imports, functions, and code.
            match section.type_ {
                SectionType::Type => {
                    if type_section_offset.is_some() {panic!("multiple type sections");}
                    type_section_offset = Some(section.offset);
                },
                SectionType::Import => {
                    if import_section_offset.is_some() {panic!("multiple import sections");}
                    import_section_offset = Some(section.offset);
                },
                SectionType::Function => {
                    if function_section_offset.is_some() {panic!("multiple function sections");}
                    function_section_offset = Some(section.offset);
                },
                SectionType::Code => {
                    if code_section_offset.is_some() {panic!("multiple code sections");}
                    code_section_offset = Some(section.offset);
                },
                // We ignore any section we are not interested in.
                _ => (),
            }
        }
        if cursor.current_offset != self.len() {
            panic!("mismatched length");
        }

        // Now that we have our hooks into the module, let's iterate over the
        // imports to determine white/grey/black listings. We need to remember
        // where the function and code data starts.

        // There is only one greylisted item (dcall) so we will just reserve a
        // place for that rather than maintain a list.
        let mut dcall_index: Option<usize> = None;
        if let Some(imports_offset) = import_section_offset {
         // Make a new cursor for imports
            let mut imports_cursor = Cursor {current_offset:imports_offset,body:&self};
            let _section_size = parse_varuint_32(&mut imports_cursor);
            // How many imports do we have?
            let n_imports = parse_varuint_32(&mut imports_cursor);
            // println!("n_imports: {}", n_imports);
            for i in 0..n_imports {
                // let mut cursor = Cursor {i:0};

                // Here we parse the names of the import, and its function
                // index.
                let import = parse_import(&mut imports_cursor, i);

                // println!("mod_name: {}, field_name: {}, f_index: {}, listing: {:?}",
                    // import.mod_name, import.field_name, import.index, import.listing());
                match import.listing() {
                    Listing::White => (),
                    Listing::Grey => {
                        if dcall_index.is_some() {panic!("dcall imported multiple times");}
                        // Document here why this is the case
                        dcall_index = Some(import.index as usize);
                    },
                    Listing::Black => {
                        // If we encounter a blacklisted import we can return
                        // early.
                        // println!("{:?} is blacklisted", import);
                        return false;
                    },
                }
            }
        }

        // The functions index into types. In fact the function section is just
        // a vector of type ids. We don't care about types at this stage.
        if let (Some(functions_offset), Some(code_offset)) = (function_section_offset, code_section_offset) {
            // Make a new cursor for functions
            let mut functions_cursor = Cursor {current_offset:functions_offset,body:&self};
            // Make a new cursor for code
            let mut code_cursor = Cursor {current_offset:code_offset,body:&self};
            // We will have to try and update these in parallel
            let _function_section_size = parse_varuint_32(&mut functions_cursor);
            let _code_section_size = parse_varuint_32(&mut code_cursor);
            // println!("functions_offset: {:?}", functions_offset);
            // println!("code_offset: {:?}", code_offset);
            let n_functions = parse_varuint_32(&mut functions_cursor);
            let n_bodies = parse_varuint_32(&mut code_cursor);

            // println!("functions_size: {:?}", function_section_size);
            // println!("code_size: {:?}", code_section_size);

            assert_eq!(n_functions,n_bodies);

            // Next we iterate through the function bodies and check if they
            // violate any of our rules.
            for _i in 0..n_bodies {
                let body_size = parse_varuint_32(&mut code_cursor);
                // First we check if it is a system call
                if is_syscall(&self[(code_cursor.current_offset)..(code_cursor.current_offset+body_size as usize)]) {
                    // If the function is a system call we can continue past it
                    continue;
                }
                // let body = parse_varuint_32(&mut code_cursor, &self);
                // println!("function[{}] is {} bytes", i, body_size);
                code_cursor.skip(body_size as usize);
                // As the function is not a system call, it is not permitted to
                // have a dcall in it, so we iterate through all the
                // instructions. If we encounter a dcall, we return with a
                // false, as this is invalid.


            }

            // // How many imports do we have?
            // let n_imports = parse_varuint_32(&mut imports_cursor, &self);
            // for i in 0..n_imports {
            //     let mut cursor = Cursor {i:0};

            //     // Here we parse the names of the import, and its function
            //     // index.
            //     let import = parse_import(&mut cursor, data, n);

            //     println!("mod_name: {}, field_name: {}, f_index: {}, listing: {:?}",
            //         import.mod_name, import.field_name, import.index, import.listing());
            //     match import.listing() {
            //         Listing::White => (),
            //         Listing::Grey => {
            //             if dcall_index.is_some() {panic!("dcall imported multiple times");}
            //             // Document here why this is the case
            //             dcall_index = Some(import.index);
            //         },
            //         Listing::Black => {
            //             // If we encounter a blacklisted import we can return
            //             // early.
            //             println!("{:?} is blacklisted", import);
            //             return false;
            //         },
            //     }
            // }
        }

            // We now know the location of dcall, if there is one.
            // We need to iterate over every function and read its code. A
            // function can be one of three things:
            //
            //     * A syscall that follows the format
            //     * A function which is not a syscall and features a greylisted call.
            //     * A function which does not contain a greylisted call or a blacklistd call.
            // The possiblities are checked in that order.

            // Let's find the functions:
            // for section in sections {
            // }

            // for function in functions {

            // }


            // for import in greys {
            //     // If the grey test does not pass return early with false.
            //     if !check_grey(&self, import.index) {
            //         return false;
            //     }
            // }

        // All the tests have passed so we can return true.
        true
    }
}

#[derive(Debug)]
enum SectionType {
    Custom,
    Type,
    Import,
    Function,
    Table,
    Memory,
    Global,
    Export,
    Start,
    Element,
    Code,
    Data,
}

#[derive(Debug)]
struct Section {
    type_: SectionType,
    // The offset is the byte offset of the start of this
    // section, i.e. it points directly to the length byte.
    offset: usize,
}

fn parse_section(cursor: &mut Cursor) -> Section {
    let type_n = cursor.read_ref();
    let offset = cursor.current_offset;
    let size_n = parse_varuint_32(cursor);
    let type_ = n_to_section(type_n);
    let section = Section {
        type_,
        offset,
    };
    cursor.current_offset += size_n as usize;
    section
}

fn n_to_section(byte: &u8) -> SectionType {
    match byte {
        0 => SectionType::Custom,
        1 => SectionType::Type,
        2 => SectionType::Import,
        3 => SectionType::Function,
        4 => SectionType::Table,
        5 => SectionType::Memory,
        6 => SectionType::Global,
        7 => SectionType::Export,
        8 => SectionType::Start,
        9 => SectionType::Element,
        10 => SectionType::Code,
        11 => SectionType::Data,
        _ => panic!("invalid section type"),
    }
}

fn parse_varuint_32(cursor: &mut Cursor) -> u32 {
    let mut res = 0;
    let mut shift = 0;
    loop {
        if shift > 31 { panic!("invalid varuint32"); }

        let b = cursor.read_ref().clone() as u32;
        res |= (b & 0x7f).checked_shl(shift).expect("invalid varuint32");
        shift += 7;
        if (b >> 7) == 0 {
            if shift >= 32 && (b as u8).leading_zeros() < 4 {
                panic!("invalid varuint32");
            }
            break;
        }
    }
    res
}

fn parse_import(cursor: &mut Cursor, index: u32) -> ImportEntry {
    let mut reader = Cursor {
        current_offset: cursor.current_offset,
        body: cursor.body,
    };
    let import: import_entry::ImportEntry = import_entry::ImportEntry::deserialize(&mut reader).expect("counted list");
    ImportEntry {
        index,
        mod_name: String::from(import.module()),
        field_name: String::from(import.field()),
    }
}

// fn get_imports(module: &Module) -> Vec<ImportEntry> {
//     if let Some(import_section) = module.import_section() {
//         import_section.entries().to_vec()
//     } else {
//         Vec::new()
//     }
// }

// fn check_grey(module: &Module, grey_index: usize) -> Vec<(u32, u32)> {
//     let mut uses = Vec::new();
//     let code_section = module.code_section().unwrap();
//     let codes = Vec::from(code_section.bodies());
//     // If the instruction Call(grey_index) exists in the body of the function, that is a dangerous function.
//     let this_call = parity_wasm::elements::Instruction::Call(grey_index as u32);
//     for (func_index, func_body) in codes.iter().enumerate() {
//         for (instruction_index, instruction) in func_body.code().elements().iter().enumerate() {
//             if instruction == &this_call && !is_syscall(module, func_index as u32) {
//                 uses.push((func_index as u32, instruction_index as u32));
//             }
//         }
//     }
//     uses
// }

// // Find the function index of an import
// pub fn find_import(module: &Module, mod_name: &str, field_name: &str) -> Option<u32> {
//     let imports = module.import_section().unwrap().entries();
//     for (i,import) in imports.iter().enumerate() {
//         if import.module() == mod_name && import.field() == field_name {
//             return Some(i as u32);
//         }
//     }
//     return None;
// }

// pub fn is_syscall(module: &Module, function_index: u32) -> bool {

//     let function_section = module.function_section().unwrap();
//     let functions = Vec::from(function_section.entries());
//     let function = functions.get(function_index as usize).unwrap();
//     let type_index = function.type_ref();

//     let type_section = module.type_section().unwrap();
//     let types = Vec::from(type_section.types());
//     let this_type = types.get(type_index as usize).unwrap();

//     let code_section = module.code_section().unwrap();
//     let codes = Vec::from(code_section.bodies());
//     let code = codes.get(function_index as usize).unwrap();
//     let instructions = Vec::from(code.code().elements());

//     // First we need to check that the instructions are correct, that is:
//     //   0. call $a
//     //   1. call $b
//     //   2. get_local 0
//     //   3. get_local 1
//     //   4. get_local 2
//     //   5. get_local 3
//     //   6. call $c
//     // $a, $b, and $c will be used later.
//     // First we simply check the length
//     if instructions.len() != 8 {
//         return false;
//     }
//     //   0. call gasleft
//     if let Instruction::Call(f_ind) = instructions[0] {
//         // Check that f_ind is the function index of "gasleft"
//         let gasleft_index = find_import(module, "env", "gasleft");
//         if Some(f_ind) != gasleft_index {
//             return false;
//         }
//     } else {
//         return false;
//     }
//     //   1. call sender
//     if let Instruction::Call(f_ind) = instructions[1] {
//         // Check that f_ind is the function index of "sender"
//         let sender_index = find_import(module, "env", "sender");
//         if Some(f_ind) != sender_index {
//             return false;
//         }
//     } else {
//         return false;
//     }
//     //   2. get_local 0
//     if let Instruction::GetLocal(0) = instructions[2] {
//     } else {
//         return false;
//     }
//     //   3. get_local 1
//     if let Instruction::GetLocal(1) = instructions[3] {
//     } else {
//         return false;
//     }
//     //   4. get_local 2
//     if let Instruction::GetLocal(2) = instructions[4] {
//     } else {
//         return false;
//     }
//     //   5. get_local 3
//     if let Instruction::GetLocal(3) = instructions[5] {
//     } else {
//         return false;
//     }

//     //   6. call dcall
//     if let Instruction::Call(f_ind) = instructions[6] {
//         // Check that f_ind is the function index of "dcall"
//         let dcall_index = find_import(module, "env", "dcall");
//         if Some(f_ind) != dcall_index {
//             return false;
//         }
//     } else {
//         return false;
//     }
//     //   7. END
//     if let Instruction::End = instructions[7] {
//     } else {
//         return false;
//     }

//     // Check that no locals are used
//     if code.locals().len() > 0 {
//         return false;
//     }
//     // Check that the type signature is correct
//     let parity_wasm::elements::Type::Function(f_type) = this_type;
//     if f_type.return_type() != Some(ValueType::I32) {
//         return false;
//     }
//     if f_type.params() != [ ValueType::I32, ValueType::I32, ValueType::I32, ValueType::I32] {
//         return false;
//     }
//     if f_type.form() != 0x60 {
//         return false;
//     }

//     true
// }

/// An iterator which counts from one to five
struct Code<'a> {
    current_offset: usize,
    body: &'a [u8],
}

// we want our count to start at one, so let's add a new() method to help.
// This isn't strictly necessary, but is convenient. Note that we start
// `count` at zero, we'll see why in `next()`'s implementation below.
impl<'a> Code<'a> {
    fn new(body: &'a [u8]) -> Code {
        let mut reader = Cursor {
            current_offset: 0,
            body: body,
        };
        // We currently don't care about locals
        let _locals: Vec<func::Local> = CountedList::<func::Local>::deserialize(&mut reader).expect("counted list").into_inner();
        Code {
            current_offset: reader.current_offset,
            body: body,
        }
    }
}

impl<'a> Iterator for Code<'a> {
    type Item = crate::instructions::Instruction;

    fn next(&mut self) -> Option<Self::Item> {
        if self.current_offset < self.body.len() {
            // We need to parse the code into something meaningful
            let mut reader = Cursor {
                current_offset: self.current_offset,
                body: self.body,
            };
            let val = Some(crate::instructions::Instruction::deserialize(&mut reader).expect("expected valid instruction"));
            self.current_offset = reader.current_offset;
            val
        } else {
            None
        }
    }
}

// TODO: we need to provide the indices of the various necessary functions for
// the system call.
pub fn is_syscall(body: &[u8]) -> bool {
    // println!("body: {:?}", body);
    let code_iter = Code::new(body);
    let mut indexed_iter = code_iter.enumerate();

    // First we need to check that the instructions are correct, that is:
    //   0. call $a
    //   1. call $b
    //   2. get_local 0
    //   3. get_local 1
    //   4. get_local 2
    //   5. get_local 3
    //   6. call $c
    // $a, $b, and $c will be used later.


    //   0. call gasleft
    if let Some((_instr_index, instructions::Instruction::Call(_f_ind))) = indexed_iter.next() {
        // Check that f_ind is the function index of "gasleft"
        // println!("call_index: {}", f_ind);
    }
    // if let Instruction::Call(f_ind) = instructions[0] {
    //     let gasleft_index = find_import(module, "env", "gasleft");
    //     if Some(f_ind) != gasleft_index {
    //         return false;
    //     }
    // } else {
    //     return false;
    // }
    // //   1. call sender
    // if let Instruction::Call(f_ind) = instructions[1] {
    //     // Check that f_ind is the function index of "sender"
    //     let sender_index = find_import(module, "env", "sender");
    //     if Some(f_ind) != sender_index {
    //         return false;
    //     }
    // } else {
    //     return false;
    // }
    // //   2. get_local 0
    // if let Instruction::GetLocal(0) = instructions[2] {
    // } else {
    //     return false;
    // }
    // //   3. get_local 1
    // if let Instruction::GetLocal(1) = instructions[3] {
    // } else {
    //     return false;
    // }
    // //   4. get_local 2
    // if let Instruction::GetLocal(2) = instructions[4] {
    // } else {
    //     return false;
    // }
    // //   5. get_local 3
    // if let Instruction::GetLocal(3) = instructions[5] {
    // } else {
    //     return false;
    // }

    // //   6. call dcall
    // if let Instruction::Call(f_ind) = instructions[6] {
    //     // Check that f_ind is the function index of "dcall"
    //     let dcall_index = find_import(module, "env", "dcall");
    //     if Some(f_ind) != dcall_index {
    //         return false;
    //     }
    // } else {
    //     return false;
    // }
    // //   7. END
    // if let Instruction::End = instructions[7] {
    // } else {
    //     return false;
    // }

    // // Check that no locals are used
    // if code.locals().len() > 0 {
    //     return false;
    // }
    // // Check that the type signature is correct
    // let parity_wasm::elements::Type::Function(f_type) = this_type;
    // if f_type.return_type() != Some(ValueType::I32) {
    //     return false;
    // }
    // if f_type.params() != [ ValueType::I32, ValueType::I32, ValueType::I32, ValueType::I32] {
    //     return false;
    // }
    // if f_type.form() != 0x60 {
    //     return false;
    // }

    // true
    false
}

#[cfg(test)]
mod tests {
    use super::*;
    use wabt::wat2wasm;
    use std::fs::File;
    use std::io::Read;

    #[test]
    fn module_only_pass() {
        let wat = "(module)";
        let wasm = wat2wasm(wat).unwrap();
        // let module: Module = parity_wasm::deserialize_buffer(wasm.as_slice()).expect("deserialise wasm");
        assert!(wasm.as_slice().is_valid());
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
        assert!(wasm.as_slice().is_valid());
    }

    #[test]
    fn example_contract_1_pass() {
        let mut f = File::open("../example_contract_1/target/wasm32-unknown-unknown/release/example_contract_1.wasm").expect("could not open file");
        let mut wasm = Vec::new();
        f.read_to_end(&mut wasm).unwrap();
        assert!(!wasm.as_slice().is_valid());
    }

//     #[test]
//     fn minimal_contract_with_write_fail() {
//         let wat = r#"
// ;; Minimal contract with a single storage write call
// (module
//   (type $t0 (func))
//   (type $t1 (func (param i32 i32)))
//   (import "env" "storage_write" (func $env.storage_write (type $t1)))
//   (func $call (type $t0)
//     i32.const 5
//     i32.const 15
//     call $env.storage_write
//     unreachable)
//   (export "call" (func $call)))
// "#;
//         let wasm: pwasm_std::Vec<u8> = wat2wasm(wat).unwrap();
//         let module: Module = parity_wasm::deserialize_buffer(wasm.as_slice()).expect("deserialise wasm");
//         assert!(!module.is_valid());
//     }

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
