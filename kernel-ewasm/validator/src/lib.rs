//! # Validator
//!
//! Crate for parsing WASM modules and validating pwasm contracts on-chain
//! according to the cap9 spec. This validates the contract in a buffer rather
//! than parsing into native data structure.
#![cfg_attr(not(feature = "std"), no_std)]

#[cfg(not(feature = "std"))]
#[macro_use]
extern crate alloc;

mod func;
mod import_entry;
mod instructions;
mod io;
mod primitives;
mod serialization;
mod types;
use self::serialization::Deserialize;

use self::primitives::{
    CountedList, Uint32, Uint64, Uint8, VarInt32, VarInt64, VarInt7, VarUint1, VarUint32, VarUint7,
};

mod listing;
pub mod modules;
pub use modules::Module;
pub use modules::Function;
use listing::*;

/// A trait for types which can be validated against the cap9 spec.
pub trait Validity {
    /// Tests the object for validity.
    fn is_valid(&self) -> bool;
}

impl<'a> Validity for modules::Module<'a> {
    fn is_valid(&self) -> bool {
        // Now that we have our hooks into the module, let's iterate over the
        // imports to determine white/grey/black listings. We need to remember
        // where the function and code data starts.

        // There is only one greylisted item (dcall) so we will just reserve a
        // place for that rather than maintain a list.
        let mut dcall_index: Option<usize> = None;
        let mut gasleft_index: Option<usize> = None;
        let mut sender_index: Option<usize> = None;
        if let Some(imports) = self.imports() {
            for (index, import) in imports.enumerate() {
                if import.mod_name == "env" && import.field_name == "sender" {
                    if sender_index.is_some() {
                        panic!("sender imported multiple times");
                    }
                    sender_index = Some(index as usize);
                }

                if import.mod_name == "env" && import.field_name == "gasleft" {
                    if gasleft_index.is_some() {
                        panic!("gasleft imported multiple times");
                    }
                    gasleft_index = Some(index as usize);
                }

                // println!("mod_name: {}, field_name: {}, f_index: {}, listing: {:?}",
                // import.mod_name, import.field_name, import.index, import.listing());
                match import.listing() {
                    Listing::White => (),
                    Listing::Grey => {
                        if dcall_index.is_some() {
                            panic!("dcall imported multiple times");
                        }
                        // Document here why this is the case
                        dcall_index = Some(index as usize);
                    }
                    Listing::Black => {
                        // If we encounter a blacklisted import we can return
                        // early.
                        // println!("{:?} is blacklisted", import);
                        return false;
                    }
                }
            }
        } else {
            // println!("no imports",);
        }

        if let Some(funcs) = self.functions() {
            for (_i, func) in funcs.enumerate() {
                // println!("func[{}]: {:?}", i, func);
                if let (Some(dcall_i), Some(gasleft_i), Some(sender_i)) =
                    (dcall_index, gasleft_index, sender_index)
                {
                    if func.is_syscall(dcall_i as u32, gasleft_i as u32, sender_i as u32) {
                        // If the function is a system call we can continue past
                        // it
                        continue;
                    }
                    // At this point we know that the function is not a syscall.
                    // We must now check that it has no black or grey listed
                    // calls. We only care about calls here. We only need to do
                    // this if dcall is imported in the first place.
                    if let Some(dcall_i) = dcall_index {
                        if func.contains_grey_call(dcall_i as u32) {
                            // This function contains a greylisted call (i.e.
                            // dcall), so we must return with false as the
                            // contract is invalid.
                            return false;
                        }
                    }
                }
            }
        }
        // All the tests have passed so we can return true.
        true
    }
}

fn parse_varuint_32(cursor: &mut Cursor) -> u32 {
    let mut res = 0;
    let mut shift = 0;
    loop {
        if shift > 31 {
            panic!("invalid varuint32");
        }

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
        let val = &self.body[self.current_offset..(self.current_offset + n)];
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

#[cfg(test)]
mod tests {
    use super::*;
    use modules::Module;
    use std::fs::File;
    use std::io::Read;
    use wabt::wat2wasm;

    #[test]
    fn module_only_pass() {
        let wat = "(module)";
        let wasm = wat2wasm(wat).unwrap();
        let validation_result = Module::new(wasm.as_slice()).is_valid();
        assert_eq!(validation_result, true);
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
        let validation_result = Module::new(wasm.as_slice()).is_valid();
        assert_eq!(validation_result, true);
    }

    #[test]
    fn example_contract_1_pass() {
        let mut f = File::open(
            "../example_contract_1/target/wasm32-unknown-unknown/release/example_contract_1.wasm",
        )
        .expect("could not open file");
        let mut wasm = Vec::new();
        f.read_to_end(&mut wasm).unwrap();
        let validation_result = Module::new(wasm.as_slice()).is_valid();
        assert_eq!(validation_result, false);
    }

    #[test]
    fn with_syscall_compliant_pass() {
        let mut f =
            File::open("test_files/with_syscall_compliant.wat").expect("could not open file");
        let mut wat = Vec::new();
        f.read_to_end(&mut wat).unwrap();
        let wasm = wat2wasm(wat).unwrap();
        let validation_result = Module::new(wasm.as_slice()).is_valid();
        assert_eq!(validation_result, true);
    }

    #[test]
    fn with_syscall_noncompliant_notpass() {
        let mut f =
            File::open("test_files/with_syscall_noncompliant.wat").expect("could not open file");
        let mut wat = Vec::new();
        f.read_to_end(&mut wat).unwrap();
        let wasm = wat2wasm(wat).unwrap();
        let validation_result = Module::new(wasm.as_slice()).is_valid();
        assert_eq!(validation_result, false);
    }

    #[test]
    fn with_syscall_noncompliant_locals_notpass() {
        let mut f = File::open("test_files/with_syscall_noncompliant_locals.wat")
            .expect("could not open file");
        let mut wat = Vec::new();
        f.read_to_end(&mut wat).unwrap();
        let wasm = wat2wasm(wat).unwrap();
        let validation_result = Module::new(wasm.as_slice()).is_valid();
        assert_eq!(validation_result, false);
    }

    #[test]
    fn with_syscall_extra_dcall_notpass() {
        let mut f =
            File::open("test_files/with_syscall_extra_dcall.wat").expect("could not open file");
        let mut wat = Vec::new();
        f.read_to_end(&mut wat).unwrap();
        let wasm = wat2wasm(wat).unwrap();
        let validation_result = Module::new(wasm.as_slice()).is_valid();
        assert_eq!(validation_result, false);
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
        let wasm = wat2wasm(wat).unwrap();
        let validation_result = Module::new(wasm.as_slice()).is_valid();
        assert_eq!(validation_result, false);
    }
}
