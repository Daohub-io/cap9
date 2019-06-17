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
#[derive(Clone, Debug, PartialEq)]
pub struct SysCall {
    pub cap_index: u8,
    pub action: SysCallAction,
}

impl SysCall {
    pub fn cap_type(&self) -> u8 {
        match &self.action {
            // TODO: use the constants provided elsewhere
            SysCallAction::Call(_)  => 0x3,
            SysCallAction::Write(_) => 0x7,
            SysCallAction::Log(_) => 0x8,
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
            0x3 => {
                Ok(SysCall {
                    cap_index,
                    action: SysCallAction::Call(Call::deserialize(reader)?)
                })
            },
            0x7 => {
                Ok(SysCall {
                    cap_index,
                    action: SysCallAction::Write(WriteCall::deserialize(reader)?)
                })
            },
            0x8 => {
                Ok(SysCall {
                    cap_index,
                    action: SysCallAction::Log(LogCall::deserialize(reader)?)
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
            SysCallAction::Call(_) => writer.write(&[self.cap_type()])?,
            SysCallAction::Write(_) => writer.write(&[self.cap_type()])?,
            SysCallAction::Log(_) => writer.write(&[self.cap_type()])?,
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
#[derive(Clone, Debug, PartialEq)]
pub enum SysCallAction {
    Write(WriteCall),
    Call(Call),
    Log(LogCall),
}

impl SysCallAction {
    pub fn check_cap(&self, cap: Capability) -> bool {
        match self {
            // CALL syscall
            SysCallAction::Call(Call{proc_id,payload:_}) => {
                if let Capability::ProcedureCall(proc_table::cap::ProcedureCallCap {prefix, key}) = cap {
                    // We only want to keep the first $prefix bits of $key, the
                    // rest should be zero. We then XOR this value with the
                    // requested proc id and the value should be zero. TODO:
                    // consider using the unstable BitVec type. For now we will
                    // just a u128 and a u64.
                    let mut mask_a_array = [0;16];
                    mask_a_array.copy_from_slice(&key[0..16]);
                    let mut mask_b_array = [0;8];
                    mask_b_array.copy_from_slice(&key[16..24]);

                    let shift_amt: u32 = 192_u8.checked_sub(prefix).unwrap_or(0) as u32;

                    let prefix_mask_a: u128 = u128::max_value().checked_shl(shift_amt.checked_sub(64).unwrap_or(0)).unwrap_or(0);
                    let prefix_mask_b:u64 = u64::max_value().checked_shl(shift_amt).unwrap_or(0);

                    // mask_a + mask_b is the key we are allowed
                    let mask_a: u128 = u128::from_le_bytes(mask_a_array) & prefix_mask_a;
                    let mask_b: u64 = u64::from_le_bytes(mask_b_array) & prefix_mask_b;

                    // This is the key we are requesting but cleared
                    let mut req_a_array = [0;16];
                    req_a_array.copy_from_slice(&proc_id[0..16]);
                    let mut req_b_array = [0;8];
                    req_b_array.copy_from_slice(&proc_id[16..24]);
                    let req_a: u128 = u128::from_le_bytes(req_a_array) & prefix_mask_a;
                    let req_b: u64 = u64::from_le_bytes(req_b_array) & prefix_mask_b;

                    return (req_a == mask_a) && (req_b == mask_b);
                }
                false
            },
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
            // LOG syscall
            SysCallAction::Log(LogCall{topics,value}) => {
                return true;
                if let Capability::Log(proc_table::cap::LogCap {topics, t1, t2, t3, t4}) = cap {
                    // let location_u256: U256 = location.into();
                    // let size_u256: U256 = size.into();
                    // if (key >= &location_u256) && (key <= &(location_u256 + size_u256)) {
                    //     return true;
                    // }
                    return true;
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
            },
            // LOG syscall
            SysCallAction::Log(LogCall{topics,value}) => {
                pwasm_ethereum::log(&topics.as_slice(), &value.0.as_slice());
            },
            // Call syscall
            SysCallAction::Call(Call{proc_id, payload}) => {
                // Find the address of the procedure we are about to execute
                let proc_id: proc_table::ProcedureKey = proc_table::get_entry_proc_id();
                let proc_address = proc_table::get_proc_addr(proc_id.clone()).expect("No Proc");
                // Remember this procedure which is being executed.
                let this_proc = proc_table::get_current_proc_id();
                // Set the "current_proc" value to the procedure we are
                // about to execute.
                proc_table::set_current_proc_id(proc_id.clone()).unwrap();
                // Execute the procedure
                // We need to subtract some gas from the limit, because there will
                // be instructions in-between that need to be run.
                crate::actual_call_code(pwasm_ethereum::gas_left()-10000, &proc_address, U256::zero(), payload.0.as_slice(), &mut Vec::new()).expect("Invalid Procedure Call");
                // Set the "current_proc" value back to this procedure, as we
                // have returned to it.
                proc_table::set_current_proc_id(this_proc).unwrap();
            }
        }
    }
}

impl Serialize for SysCallAction {
    type Error = io::Error;

    fn serialize<W: io::Write>(self, writer: &mut W) -> Result<(), Self::Error> {
        match self {
            SysCallAction::Call(call) => {
                call.serialize(writer)?;
                Ok(())
            },
            SysCallAction::Write(write_call) => {
                write_call.serialize(writer)?;
                Ok(())
            },
            SysCallAction::Log(log_call) => {
                log_call.serialize(writer)?;
                Ok(())
            },
        }
    }
}


#[derive(Clone, Debug, PartialEq)]
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


#[derive(Clone, Debug, PartialEq)]
pub struct LogCall {
    pub topics: Vec<H256>,
    pub value: Payload,
}

impl Deserialize for LogCall {
    type Error = io::Error;

    fn deserialize<R: io::Read>(reader: &mut R) -> Result<Self, Self::Error> {
        let n_topics = u8::deserialize(reader)?;
        let mut topics : Vec<H256> = Vec::new();
        for _i in 0..(n_topics as usize) {
            topics.push(H256::deserialize(reader)?);
        }
        let value: Payload = Payload::deserialize(reader)?;
        Ok(LogCall{topics, value})
    }
}

impl Serialize for LogCall {
    type Error = io::Error;

    fn serialize<W: io::Write>(self, writer: &mut W) -> Result<(), Self::Error> {
        let n_topics = self.topics.len() as u8;
        n_topics.serialize(writer)?;
        for topic in self.topics {
            topic.serialize(writer)?;
        }
        self.value.serialize(writer)?;
        Ok(())
    }
}


#[derive(Debug, Clone, PartialEq)]
pub struct Payload(pub Vec<u8>);

impl Payload {
    pub fn new() -> Self {
        Payload(Vec::new())
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct Call {
    pub proc_id: proc_table::ProcedureKey,
    pub payload: Payload,
}

impl Deserialize for Call {
    type Error = io::Error;

    fn deserialize<R: io::Read>(reader: &mut R) -> Result<Self, Self::Error> {
        let SysCallProcedureKey(proc_id) = SysCallProcedureKey::deserialize(reader)?;
        let payload = Payload::deserialize(reader)?;
        Ok(Call{proc_id, payload})
    }
}


impl Serialize for Call {
    type Error = io::Error;

    fn serialize<W: io::Write>(self, writer: &mut W) -> Result<(), Self::Error> {
        // Write procedure id
        SysCallProcedureKey(self.proc_id).serialize(writer)?;
        // Write payload
        self.payload.serialize(writer)?;
        Ok(())
    }
}

impl Deserialize for Payload {
    type Error = io::Error;

    fn deserialize<R: io::Read>(reader: &mut R) -> Result<Self, Self::Error> {
        // Here we just need to read all remaining bytes TODO: a buffered read
        // would be better rather than a single byte loop. The Read interface
        // we're currently using isn't flexible enough here, we should change to
        // a Read implementeation with a sized buffer. This is sufficient for
        // correctness.
        let mut payload: Vec<u8> = Vec::new();
        let mut u8buf = [0; 1];
        loop {
            match reader.read(&mut u8buf) {
                Ok(_) => {
                    payload.push(u8buf[0])
                },
                Err(_) => break,
            }
        }
        Ok(Payload(payload))
    }
}

impl Serialize for Payload {
    type Error = io::Error;

    fn serialize<W: io::Write>(self, writer: &mut W) -> Result<(), Self::Error> {
        writer.write(self.0.as_slice())?;
        Ok(())
    }
}

/// Newtype wrapper over procedure keys for interaction with syscalls.
pub struct SysCallProcedureKey(pub proc_table::ProcedureKey);

impl From<H256> for SysCallProcedureKey {
    fn from(h: H256) -> Self {
        let mut proc_id: proc_table::ProcedureKey = [0; 24];
        proc_id.copy_from_slice(&h.as_bytes()[8..32]);
        SysCallProcedureKey(proc_id)
    }
}

impl Into<H256> for SysCallProcedureKey {
    fn into(self) -> H256 {
        let mut proc_id_u256: [u8; 32] = [0; 32];
        proc_id_u256[8..32].copy_from_slice(&self.0);
        proc_id_u256.into()
    }
}

impl Deserialize for SysCallProcedureKey {
    type Error = io::Error;

    fn deserialize<R: io::Read>(reader: &mut R) -> Result<Self, Self::Error> {
        let proc_id_u256: U256 = U256::deserialize(reader)?;
        let mut proc_id_buffer: [u8; 32] = [0; 32];
        proc_id_u256.to_big_endian(&mut proc_id_buffer);
        let mut proc_id: proc_table::ProcedureKey = [0; 24];
        proc_id.copy_from_slice(&proc_id_buffer[8..32]);
        Ok(SysCallProcedureKey(proc_id))
    }
}

impl Serialize for SysCallProcedureKey {
    type Error = io::Error;

    fn serialize<W: io::Write>(self, writer: &mut W) -> Result<(), Self::Error> {
        let mut proc_id_u256: [u8; 32] = [0; 32];
        proc_id_u256[8..32].copy_from_slice(&self.0);
        writer.write(&proc_id_u256)?;
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


    #[test]
    fn deserialise_log_call() {
        let mut input: &[u8] = &[0x08,0x00,0x00,0xab,0xcd,0xab,0xcd];
        let syscall = SysCall::deserialize(&mut input).unwrap();
        assert_eq!(syscall, SysCall{
            cap_index: 0,
            action: SysCallAction::Log(LogCall {
                topics: Vec::new(),
                value: Payload([0xab,0xcd,0xab,0xcd].to_vec()),
            }),
        });
        // assert_eq!(contract.currentProcedure(), [0u8; 24]);
    }
}
