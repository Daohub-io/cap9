#![no_std]

extern crate pwasm_abi;
use pwasm_abi::types::*;
use validator::io;
use validator::serialization::{Deserialize, Serialize};

/// Generic wasm error
#[derive(Debug)]
pub struct Error;

use crate::proc_table;
use proc_table::cap::Capability;


/// A full system call request, including the cap_index. This is permitted to
/// access the procedure table as part of the environment.
#[derive(Clone, Debug)]
pub struct SysCall {
    pub cap_index: u8,
    pub action: SysCallAction,
}

impl SysCall {
    pub fn cap_type(&self) -> u8 {
        match &self.action {
            SysCallAction::Write(_) => 0x7,
        }
    }

    pub fn execute(&self) {
        self.action.execute()
    }

    /// Given a syscall, get the relevant Capability for the current procedure
    /// and check that it is sufficient for the given syscall.
    pub fn check_cap(&self) -> bool {
        let current_proc_key = proc_table::get_current_proc_id();
        if let Some(cap) = proc_table::get_proc_cap(current_proc_key, self.cap_type(), self.cap_index) {
            return self.action.check_cap(cap);
        }
        false
    }
}


impl Deserialize for SysCall {
    type Error = io::Error;

    fn deserialize<R: io::Read>(reader: &mut R) -> Result<Self, Self::Error> {
        let syscall_type = u8::deserialize(reader)?;
        let cap_index = u8::deserialize(reader)?;
        match syscall_type {
            0x7 => {
                Ok(SysCall {
                    cap_index,
                    action: SysCallAction::Write(WriteCall::deserialize(reader)?)
                })
            },
            _ => panic!("unknown syscall"),
        }
    }
}


impl Serialize for SysCall {
    type Error = io::Error;

    fn serialize<W: io::Write>(self, writer: &mut W) -> Result<(), Self::Error> {
        // Write syscall type
        match self.action {
            SysCallAction::Write(_) => writer.write(&[0x07])?
        }
        // Write cap index
        writer.write(&[self.cap_index])?;
        self.action.serialize(writer)?;
        Ok(())
    }
}

/// The action portion of a SysCall, i.e. WRITE, LOG, etc. without the
/// permissions information. This is the type where the capability checking and
/// execution logic is written.
#[derive(Clone, Debug)]
pub enum SysCallAction {
    Write(WriteCall),
}

impl SysCallAction {
    pub fn check_cap(&self, cap: Capability) -> bool {
        match self {
            // WRITE syscall
            SysCallAction::Write(WriteCall{key,value:_}) => {
                if let Capability::StoreWrite(proc_table::cap::StoreWriteCap {location, size}) = cap {
                    let location_u256: U256 = location.into();
                    let size_u256: U256 = size.into();
                    if (key >= &location_u256) && (key <= &(location_u256 + size_u256)) {
                        return true;
                    }
                }
                false
            },
        }
    }

    pub fn execute(&self) {
        match self {
            // WRITE syscall
            SysCallAction::Write(WriteCall{key,value}) => {
                let value_h256: H256 = value.into();
                pwasm_ethereum::write(&key.into(), &value_h256.as_fixed_bytes());
                pwasm_ethereum::ret(&[]);
            },
        }
    }
}

#[derive(Clone, Debug)]
pub struct WriteCall {
    pub key: U256,
    pub value: U256,
}

impl Deserialize for WriteCall {
    type Error = io::Error;

    fn deserialize<R: io::Read>(reader: &mut R) -> Result<Self, Self::Error> {
        let key: U256 = U256::deserialize(reader)?;
        let value: U256 = U256::deserialize(reader)?;
        Ok(WriteCall{key, value})
    }
}


impl Serialize for SysCallAction {
    type Error = io::Error;

    fn serialize<W: io::Write>(self, writer: &mut W) -> Result<(), Self::Error> {
        match self {
            SysCallAction::Write(write_call) => {
                write_call.serialize(writer)?;
                Ok(())
            }
        }
    }
}

impl Serialize for WriteCall {
    type Error = io::Error;

    fn serialize<W: io::Write>(self, writer: &mut W) -> Result<(), Self::Error> {
        // Write key
        self.key.serialize(writer)?;
        // Write value
        self.value.serialize(writer)?;
        Ok(())
    }
}


#[cfg(test)]
mod tests {
    use super::*;
    use pwasm_abi::types::*;
    use validator::io;
    use validator::serialization::{Deserialize, Serialize};

    #[test]
    fn serialize_write() {
        let key: U256 = U256::zero();
        let value: U256 = U256::zero();
        let mut buffer = Vec::with_capacity(1 + 1 + 32 + 32);

        let syscall = SysCall {
            cap_index: 0,
            action: SysCallAction::Write(WriteCall{key: key.into(), value: value.into()})
        };
        syscall.serialize(&mut buffer).unwrap();
        let expected: &[u8] = &[0x7, 0x00,
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,0x00,0x00,
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,0x00];
        assert_eq!(buffer, expected);
    }
}
