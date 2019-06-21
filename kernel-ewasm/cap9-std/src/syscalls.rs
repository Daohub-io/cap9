extern crate pwasm_abi;
use pwasm_abi::types::*;
use cap9_core;
use cap9_core::{Deserialize, Serialize};

/// Generic wasm error
#[derive(Debug)]
pub struct Error;

use crate::proc_table;
use proc_table::cap::Capability;
use proc_table::cap::*;
use proc_table::ProcedureKey;

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
            SysCallAction::Call(_)  => CAP_PROC_CALL,
            SysCallAction::Write(_) => CAP_STORE_WRITE,
            SysCallAction::Log(_) => CAP_LOG,
            SysCallAction::Register(_) => CAP_PROC_REGISTER,
            SysCallAction::Delete(_) => CAP_PROC_DELETE,
            SysCallAction::SetEntry(_) => CAP_PROC_ENTRY,
            SysCallAction::AccountCall(_) => CAP_ACC_CALL,
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


impl Deserialize<u8> for SysCall {
    type Error = cap9_core::Error;

    fn deserialize<R: cap9_core::Read<u8>>(reader: &mut R) -> Result<Self, Self::Error> {
        let syscall_type = u8::deserialize(reader)?;
        let cap_index = u8::deserialize(reader)?;
        match syscall_type {
            CAP_PROC_CALL => {
                Ok(SysCall {
                    cap_index,
                    action: SysCallAction::Call(Call::deserialize(reader)?)
                })
            },
            CAP_STORE_WRITE => {
                Ok(SysCall {
                    cap_index,
                    action: SysCallAction::Write(WriteCall::deserialize(reader)?)
                })
            },
            CAP_LOG => {
                Ok(SysCall {
                    cap_index,
                    action: SysCallAction::Log(LogCall::deserialize(reader)?)
                })
            },
            CAP_PROC_REGISTER => {
                Ok(SysCall {
                    cap_index,
                    action: SysCallAction::Register(RegisterProc::deserialize(reader)?)
                })
            },
            CAP_PROC_DELETE => {
                Ok(SysCall {
                    cap_index,
                    action: SysCallAction::Delete(DeleteProc::deserialize(reader)?)
                })
            },
            CAP_PROC_ENTRY => {
                Ok(SysCall {
                    cap_index,
                    action: SysCallAction::SetEntry(SetEntry::deserialize(reader)?)
                })
            },
            CAP_ACC_CALL => {
                Ok(SysCall {
                    cap_index,
                    action: SysCallAction::AccountCall(AccountCall::deserialize(reader)?)
                })
            },
            _ => panic!("unknown syscall"),
        }
    }
}


impl Serialize<u8> for SysCall {
    type Error = cap9_core::Error;

    fn serialize<W: cap9_core::Write<u8>>(&self, writer: &mut W) -> Result<(), Self::Error> {
        // Write syscall type
        writer.write(&[self.cap_type()])?;
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
    Register(RegisterProc),
    Delete(DeleteProc),
    SetEntry(SetEntry),
    AccountCall(AccountCall),
}

impl SysCallAction {
    pub fn check_cap(&self, cap: Capability) -> bool {
        match self {
            // CALL syscall
            SysCallAction::Call(Call{proc_id,payload:_}) => {
                if let Capability::ProcedureCall(proc_table::cap::ProcedureCallCap {prefix, key}) = cap {
                    return matching_keys(prefix, &key, proc_id);
                }
                false
            },
            // Delete Procedure syscall
            SysCallAction::Delete(DeleteProc{proc_id}) => {
                if let Capability::ProcedureDelete(proc_table::cap::ProcedureDeleteCap {prefix, key}) = cap {
                    return matching_keys(prefix, &key, proc_id);
                }
                false
            },
            // Set Entry syscall
            SysCallAction::SetEntry(SetEntry{proc_id:_}) => {
                if let Capability::ProcedureEntry(_) = cap {
                    return true;
                }
                false
            },
            // Register Procedure syscall
            SysCallAction::Register(RegisterProc{proc_id,address:_, cap_list}) => {
                // Check that this procedure has the correct capability to
                // register a procedure of the given key.
                if let Capability::ProcedureRegister(proc_table::cap::ProcedureRegisterCap {prefix, key}) = cap {
                    if !matching_keys(prefix, &key, proc_id) {
                        return false;
                    }
                    let this_key: proc_table::ProcedureKey = proc_table::get_current_proc_id();
                    // Check that this procedure has sufficent capabilities to
                    // delegate to the new procedure.
                    let caps = &cap_list.0;
                    for cap in caps {
                        // Retrieve the parent cap that this cap has requested.
                        let parent_cap: Capability = match proc_table::get_proc_cap(this_key, cap.cap.cap_type(), cap.parent_index) {
                            None => return false,
                            Some(cap) => cap,
                        };
                        if !cap.cap.is_subset_of(&parent_cap) {
                            return false;
                        }
                    }
                    return true;
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
            SysCallAction::Log(LogCall{topics,value:_}) => {
                if let Capability::Log(proc_table::cap::LogCap {topics: n_required_topics, t1, t2, t3, t4}) = cap {
                    // Check that all of the topics required by the cap are
                    // satisfied. That is, for every topic in the capability,
                    // the corresponding exists in the system call and is set to
                    // that exact value. First we check that there are enough
                    // topics in the request.
                    if topics.len() < n_required_topics as usize {
                        // The system call specifies an insufficient number of
                        // topics
                        return false;
                    }

                    if topics.len() >= 1 {
                        if topics[0] != t1.into() {
                            return false;
                        }
                    }
                    if topics.len() >= 2 {
                        if topics[1] != t2.into() {
                            return false;
                        }
                    }
                    if topics.len() >= 3 {
                        if topics[2] != t3.into() {
                            return false;
                        }
                    }
                    if topics.len() >= 4 {
                        if topics[3] != t4.into() {
                            return false;
                        }
                    }
                    return true;
                }
                false
            },
            // Account Call syscall
            SysCallAction::AccountCall(AccountCall{address,value,payload:_}) => {
                if let Capability::AccountCall(proc_table::cap::AccountCallCap {can_call_any, can_send, address: cap_address}) = cap {
                    // If can_call_any is false and address does not match the
                    // capability address, return false.
                    if !can_call_any && (address != &cap_address) {
                        return false;
                    }

                    // If can_send is false and amount is non-zero, return false
                    if !can_send && (value != &U256::zero()) {
                        return false;
                    }
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
            // Register Procedure
            SysCallAction::Register(RegisterProc{proc_id, address, cap_list}) => {
                proc_table::insert_proc(proc_id.clone(), address.clone(), cap_list.clone()).unwrap();
            }
            // Delete Procedure
            SysCallAction::Delete(DeleteProc{proc_id}) => {
                proc_table::remove_proc(proc_id.clone()).unwrap();
            }
            // Set Entry
            SysCallAction::SetEntry(SetEntry{proc_id}) => {
                proc_table::set_entry_proc_id(*proc_id);
            }
            // Account Call
            SysCallAction::AccountCall(AccountCall{address,value,payload}) => {
                pwasm_ethereum::call(pwasm_ethereum::gas_left()-10000, &address, *value, payload.0.as_slice(), &mut Vec::new());
            }
        }
    }
}

impl Serialize<u8> for SysCallAction {
    type Error = cap9_core::Error;

    fn serialize<W: cap9_core::Write<u8>>(&self, writer: &mut W) -> Result<(), Self::Error> {
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
            SysCallAction::Register(register_call) => {
                register_call.serialize(writer)?;
                Ok(())
            },
            SysCallAction::Delete(delete_call) => {
                delete_call.serialize(writer)?;
                Ok(())
            },
            SysCallAction::SetEntry(set_entry_call) => {
                set_entry_call.serialize(writer)?;
                Ok(())
            },
            SysCallAction::AccountCall(account_call) => {
                account_call.serialize(writer)?;
                Ok(())
            },
        }
    }
}


#[derive(Clone, Debug, PartialEq)]
pub struct AccountCall {
    pub address: Address,
    pub value: U256,
    pub payload: Payload,
}

impl Deserialize<u8> for AccountCall {
    type Error = cap9_core::Error;

    fn deserialize<R: cap9_core::Read<u8>>(reader: &mut R) -> Result<Self, Self::Error> {
        let address: Address = Address::deserialize(reader)?;
        let value: U256 = U256::deserialize(reader)?;
        let payload = Payload::deserialize(reader)?;
        Ok(AccountCall{address, value, payload})
    }
}

impl Serialize<u8> for AccountCall {
    type Error = cap9_core::Error;

    fn serialize<W: cap9_core::Write<u8>>(&self, writer: &mut W) -> Result<(), Self::Error> {
        self.address.serialize(writer)?;
        self.value.serialize(writer)?;
        self.payload.serialize(writer)?;
        Ok(())
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct WriteCall {
    pub key: U256,
    pub value: U256,
}

impl Deserialize<u8> for WriteCall {
    type Error = cap9_core::Error;

    fn deserialize<R: cap9_core::Read<u8>>(reader: &mut R) -> Result<Self, Self::Error> {
        let key: U256 = U256::deserialize(reader)?;
        let value: U256 = U256::deserialize(reader)?;
        Ok(WriteCall{key, value})
    }
}

impl Serialize<u8> for WriteCall {
    type Error = cap9_core::Error;

    fn serialize<W: cap9_core::Write<u8>>(&self, writer: &mut W) -> Result<(), Self::Error> {
        // Write key
        self.key.serialize(writer)?;
        // Write value
        self.value.serialize(writer)?;
        Ok(())
    }
}


#[derive(Clone, Debug, PartialEq)]
pub struct SetEntry {
    pub proc_id: proc_table::ProcedureKey,
}

impl Deserialize<u8> for SetEntry {
    type Error = cap9_core::Error;

    fn deserialize<R: cap9_core::Read<u8>>(reader: &mut R) -> Result<Self, Self::Error> {
        let SysCallProcedureKey(proc_id) = SysCallProcedureKey::deserialize(reader)?;
        Ok(SetEntry{proc_id})
    }
}

impl Serialize<u8> for SetEntry {
    type Error = cap9_core::Error;

    fn serialize<W: cap9_core::Write<u8>>(&self, writer: &mut W) -> Result<(), Self::Error> {
        // Write procedure id
        SysCallProcedureKey(self.proc_id).serialize(writer)?;
        Ok(())
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct LogCall {
    pub topics: Vec<H256>,
    pub value: Payload,
}

impl Deserialize<u8> for LogCall {
    type Error = cap9_core::Error;

    fn deserialize<R: cap9_core::Read<u8>>(reader: &mut R) -> Result<Self, Self::Error> {
        let n_topics = u8::deserialize(reader)?;
        let mut topics : Vec<H256> = Vec::new();
        for _i in 0..(n_topics as usize) {
            topics.push(H256::deserialize(reader)?);
        }
        let value: Payload = Payload::deserialize(reader)?;
        Ok(LogCall{topics, value})
    }
}

impl Serialize<u8> for LogCall {
    type Error = cap9_core::Error;

    fn serialize<W: cap9_core::Write<u8>>(&self, writer: &mut W) -> Result<(), Self::Error> {
        let n_topics = self.topics.len() as u8;
        n_topics.serialize(writer)?;
        for topic in &self.topics {
            topic.serialize(writer)?;
        }
        self.value.serialize(writer)?;
        Ok(())
    }
}


#[derive(Clone, Debug, PartialEq)]
pub struct RegisterProc {
    pub proc_id: proc_table::ProcedureKey,
    pub address: Address,
    pub cap_list: NewCapList,
}

impl Deserialize<u8> for RegisterProc {
    type Error = cap9_core::Error;

    fn deserialize<R: cap9_core::Read<u8>>(reader: &mut R) -> Result<Self, Self::Error> {
        let SysCallProcedureKey(proc_id) = SysCallProcedureKey::deserialize(reader)?;
        let address = Address::deserialize(reader)?;
        let cap_list = NewCapList::deserialize(reader)?;
        Ok(RegisterProc{proc_id, address, cap_list})
    }
}


impl Serialize<u8> for RegisterProc {
    type Error = cap9_core::Error;

    fn serialize<W: cap9_core::Write<u8>>(&self, writer: &mut W) -> Result<(), Self::Error> {
        // Write procedure id
        SysCallProcedureKey(self.proc_id).serialize(writer)?;
        // Write the address of the contract
        self.address.serialize(writer)?;
        // Write the caps out as 32-byte values, as per the spec
        self.cap_list.serialize(writer)?;
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
pub struct DeleteProc {
    pub proc_id: proc_table::ProcedureKey,
}

impl Deserialize<u8> for DeleteProc {
    type Error = cap9_core::Error;

    fn deserialize<R: cap9_core::Read<u8>>(reader: &mut R) -> Result<Self, Self::Error> {
        let SysCallProcedureKey(proc_id) = SysCallProcedureKey::deserialize(reader)?;
        Ok(DeleteProc{proc_id})
    }
}


impl Serialize<u8> for DeleteProc {
    type Error = cap9_core::Error;

    fn serialize<W: cap9_core::Write<u8>>(&self, writer: &mut W) -> Result<(), Self::Error> {
        // Write procedure id
        SysCallProcedureKey(self.proc_id).serialize(writer)?;
        Ok(())
    }
}


#[derive(Clone, Debug, PartialEq)]
pub struct Call {
    pub proc_id: proc_table::ProcedureKey,
    pub payload: Payload,
}

impl Deserialize<u8> for Call {
    type Error = cap9_core::Error;

    fn deserialize<R: cap9_core::Read<u8>>(reader: &mut R) -> Result<Self, Self::Error> {
        let SysCallProcedureKey(proc_id) = SysCallProcedureKey::deserialize(reader)?;
        let payload = Payload::deserialize(reader)?;
        Ok(Call{proc_id, payload})
    }
}


impl Serialize<u8> for Call {
    type Error = cap9_core::Error;

    fn serialize<W: cap9_core::Write<u8>>(&self, writer: &mut W) -> Result<(), Self::Error> {
        // Write procedure id
        SysCallProcedureKey(self.proc_id).serialize(writer)?;
        // Write payload
        self.payload.serialize(writer)?;
        Ok(())
    }
}

impl Deserialize<u8> for Payload {
    type Error = cap9_core::Error;

    fn deserialize<R: cap9_core::Read<u8>>(reader: &mut R) -> Result<Self, Self::Error> {
        // Read all the remaining bytes in the buffer.
        let mut payload: Vec<u8> = Vec::new();
        payload.resize(reader.remaining(), 0_u8);
        reader.read(&mut payload)?;
        Ok(Payload(payload))
    }
}

impl Serialize<u8> for Payload {
    type Error = cap9_core::Error;

    fn serialize<W: cap9_core::Write<u8>>(&self, writer: &mut W) -> Result<(), Self::Error> {
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


impl From<ProcedureKey> for SysCallProcedureKey {
    fn from(proc_id: ProcedureKey) -> Self {
        SysCallProcedureKey(proc_id)
    }
}

impl Into<ProcedureKey> for SysCallProcedureKey {
    fn into(self) -> ProcedureKey {
        self.0
    }
}

impl Deserialize<u8> for SysCallProcedureKey {
    type Error = cap9_core::Error;

    fn deserialize<R: cap9_core::Read<u8>>(reader: &mut R) -> Result<Self, Self::Error> {
        let proc_id_u256: U256 = U256::deserialize(reader)?;
        let mut proc_id_buffer: [u8; 32] = [0; 32];
        proc_id_u256.to_big_endian(&mut proc_id_buffer);
        let mut proc_id: proc_table::ProcedureKey = [0; 24];
        proc_id.copy_from_slice(&proc_id_buffer[8..32]);
        Ok(SysCallProcedureKey(proc_id))
    }
}

impl Serialize<u8> for SysCallProcedureKey {
    type Error = cap9_core::Error;

    fn serialize<W: cap9_core::Write<u8>>(&self, writer: &mut W) -> Result<(), Self::Error> {
        let mut proc_id_u256: [u8; 32] = [0; 32];
        proc_id_u256[8..32].copy_from_slice(&self.0);
        writer.write(&proc_id_u256)?;
        Ok(())
    }
}


impl Deserialize<u8> for NewCapList {
    type Error = cap9_core::Error;

    fn deserialize<R: cap9_core::Read<u8>>(reader: &mut R) -> Result<Self, Self::Error> {
        let mut cap_list_raw: Vec<U256> = Vec::new();
        loop {
            if let Ok(cap_val) = H256::deserialize(reader) {
                cap_list_raw.push(cap_val.into());
            } else {
                break;
            }
        }
        let cap_list = NewCapList::from_u256_list(cap_list_raw.as_slice()).unwrap();
        Ok(cap_list)
    }
}

impl Serialize<u8> for NewCapList {
    type Error = cap9_core::Error;

    fn serialize<W: cap9_core::Write<u8>>(&self, writer: &mut W) -> Result<(), Self::Error> {
        for val in &self.to_u256_list() {
            val.serialize(writer)?;
        }
        Ok(())
    }
}


#[cfg(test)]
mod tests {
    use super::*;
    use pwasm_abi::types::*;
    use cap9_core;
    use cap9_core::{Deserialize, Serialize};

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
    fn matching_keys_test_1() {
        let prefix = 0;
        let required_key = &[0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0];
        let requested_key = &[0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0];
        let result = matching_keys(prefix, required_key, requested_key);
        assert_eq!(result,true);
    }

    #[test]
    fn matching_keys_test_2() {
        let prefix = 0;
        let required_key = &[1,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0];
        let requested_key = &[2,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0];
        let result = matching_keys(prefix, required_key, requested_key);
        assert_eq!(result,true);
    }

    #[test]
    fn matching_keys_test_3() {
        let prefix = 8;
        let required_key = &[1,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0];
        let requested_key = &[2,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0];
        let result = matching_keys(prefix, required_key, requested_key);
        assert_eq!(result,false);
    }

    #[test]
    fn matching_keys_test_4() {
        let prefix = 128;
        let required_key  = &[0xff,0xff,0xff,0xff,0xff,0xff,0xff,0xff,0xff,0xff,0xff,0xff,0xff,0xfe,0xff,0xff,0xff,0xff,0xff,0xff,0xff,0xff,0xff,0xff];
        let requested_key = &[0xff,0xff,0xff,0xff,0xff,0xff,0xff,0xff,0xff,0xff,0xff,0xff,0xff,0xff,0xff,0xff,0xff,0xff,0xff,0xff,0xff,0xff,0xff,0xfe];
        let result = matching_keys(prefix, required_key, requested_key);
        assert_eq!(result,false);
    }

    #[test]
    fn matching_keys_test_5() {
        let prefix = 189;
        let required_key  = &[0xff,0xff,0xff,0xff,0xff,0xff,0xff,0xff,0xff,0xff,0xff,0xff,0xff,0xff,0xff,0xff,0xff,0xff,0xff,0xff,0xff,0xff,0xff,0xff];
        let requested_key = &[0xff,0xff,0xff,0xff,0xff,0xff,0xff,0xff,0xff,0xff,0xff,0xff,0xff,0xff,0xff,0xff,0xff,0xff,0xff,0xff,0xff,0xff,0xff,0xfe];
        let result = matching_keys(prefix, required_key, requested_key);
        assert_eq!(result,true);
    }

    #[test]
    fn matching_keys_test_6() {
        let prefix = 191;
        let required_key  = &[0xff,0xff,0xff,0xff,0xff,0xff,0xff,0xff,0xff,0xff,0xff,0xff,0xff,0xff,0xff,0xff,0xff,0xff,0xff,0xff,0xff,0xff,0xff,0xff];
        let requested_key = &[0xff,0xff,0xff,0xff,0xff,0xff,0xff,0xff,0xff,0xff,0xff,0xff,0xff,0xff,0xff,0xff,0xff,0xff,0xff,0xff,0xff,0xff,0xff,0xfe];
        let result = matching_keys(prefix, required_key, requested_key);
        assert_eq!(result,true);
    }

    #[test]
    fn matching_keys_test_7() {
        let prefix = 5*8;
        let required_key  = &[0x61,0x62,0x63,0x64,0x65,0x00,0x00,0x00,0x00,0x00,0x00,0x00,0x00,0x00,0x00,0x00,0x00,0x00,0x00,0x00,0x00,0x00,0x00,0x00];
        let requested_key = &[0x61,0x78,0x63,0x64,0x65,0x00,0x00,0x00,0x00,0x00,0x00,0x00,0x00,0x00,0x00,0x00,0x00,0x00,0x00,0x00,0x00,0x00,0x00,0x00];
        let result = matching_keys(prefix, required_key, requested_key);
        assert_eq!(result,false);
    }

    #[test]
    fn deserialise_log_call() {
        let input: &[u8] = &[0x08,0x00,0x00,0xab,0xcd,0xab,0xcd];
        let mut reader = cap9_core::Cursor::new(input);
        let syscall = SysCall::deserialize(&mut reader).unwrap();
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
