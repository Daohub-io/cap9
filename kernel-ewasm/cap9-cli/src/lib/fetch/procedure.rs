use web3::futures::Future;
use web3::types::{Address, U256};
use web3::Transport;
use rustc_hex::FromHex;
use rustc_hex::ToHex;
use crate::connection::EthConn;
use cap9_std::proc_table::cap::*;
use std::fmt;
use cap9_std::proc_table::ProcPointer;
use cap9_core::Error;
use cap9_core::Read;
use crate::utils;
use std::collections::HashMap;
use serde::ser::{Serialize, Serializer, SerializeSeq, SerializeStruct};
use serde::de::{self, Deserialize, Deserializer, Visitor, SeqAccess, MapAccess};

#[derive(Clone, Debug)]
pub struct Procedure {
    pub key: [u8; 24],
    pub index: U256,
    pub address: Address,
    pub caps: Caps,
}

impl fmt::Display for Procedure {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let key_hex: String = self.key.to_hex();
        let key_utf8: &str = std::str::from_utf8(&self.key).unwrap().trim_end_matches('\0');
        write!(f, "Procedure[{}]: 0x{} (\"{}\")\n  Address: {:?}\n  Caps({}):\n{}",
            self.index.as_u64(), key_hex, key_utf8, self.address, self.caps.len(), self.caps)
    }
}

#[derive(Clone, Debug)]
pub struct Caps {
    pub proc_call: Vec<Capability>,
    pub proc_register: Vec<Capability>,
    pub proc_delete: Vec<Capability>,
    pub proc_entry: Vec<Capability>,
    pub store_write: Vec<Capability>,
    pub log: Vec<Capability>,
    pub acc_call: Vec<Capability>,
}

impl Caps {
    pub fn len(&self) -> usize {
        self.proc_call.len()
            + self.proc_register.len()
            + self.proc_delete.len()
            + self.proc_entry.len()
            + self.store_write.len()
            + self.log.len()
            + self.acc_call.len()
    }
}


impl fmt::Display for Caps {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        if self.proc_call.len() > 0 {
            write!(f, "    CAP_PROC_CALL({}):\n", self.proc_call.len())?;
            for (i, cap) in self.proc_call.iter().enumerate() {
                write!(f, "        {}: {}\n", i, cap)?;
            }
        }
        if self.proc_register.len() > 0 {
            write!(f, "    CAP_PROC_REGISTER({}):\n", self.proc_register.len())?;
            for (i, cap) in self.proc_register.iter().enumerate() {
                write!(f, "        {}: {}\n", i, cap)?;
            }
        }
        if self.proc_delete.len() > 0 {
            write!(f, "    CAP_PROC_DELETE({}):\n", self.proc_delete.len())?;
            for (i, cap) in self.proc_delete.iter().enumerate() {
                write!(f, "        {}: {}\n", i, cap)?;
            }
        }
        if self.proc_entry.len() > 0 {
            write!(f, "    CAP_PROC_CALL({}):\n", self.proc_entry.len())?;
            for (i, cap) in self.proc_entry.iter().enumerate() {
                write!(f, "        {}: {}\n", i, cap)?;
            }
        }
        if self.store_write.len() > 0 {
            write!(f, "    CAP_STORE_WRITE({}):\n", self.store_write.len())?;
            for (i, cap) in self.store_write.iter().enumerate() {
                write!(f, "        {}: {}\n", i, cap)?;
            }
        }
        if self.log.len() > 0 {
            write!(f, "    CAP_LOG({}):\n", self.log.len())?;
            for (i, cap) in self.log.iter().enumerate() {
                write!(f, "        {}: {}\n", i, cap)?;
            }
        }
        if self.acc_call.len() > 0 {
            write!(f, "    CAP_ACC_CALL({}):\n", self.acc_call.len())?;
            for (i, cap) in self.acc_call.iter().enumerate() {
                write!(f, "        {}: {}\n", i, cap)?;
            }
        }
        write!(f, "")
    }
}

struct CapReader<'a, T> where T: Transport {
    conn: &'a EthConn<T>,
    kernel_address: Address,
    proc_pointer: ProcPointer,
    cap_type: u8,
    cap_index: u8,
    current_val: u8,
}

impl<'a, T: Transport> Read<pwasm_abi::types::U256> for CapReader<'a, T> {
    fn read(&mut self, buf: &mut [pwasm_abi::types::U256]) -> Result<(), Error> {
        for i in 0..buf.len() {
            let next_val_ptr = self.proc_pointer.get_cap_val_ptr(self.cap_type, self.cap_index, self.current_val);
            let next_val = self.conn.web3.eth().storage(self.kernel_address, U256::from_big_endian(&next_val_ptr), None).wait().expect("proc key raw");
            self.current_val += 1;
            buf[i] = pwasm_abi::types::U256::from_big_endian(&next_val.to_fixed_bytes());
        }
        Ok(())
    }

    fn remaining(&self) -> usize {
        1_usize
    }
}


fn get_idx_proc_address(i: u64) -> U256 {
    let idx: u8 = i as u8;
    U256::from_big_endian(&[0xff, 0xff, 0xff, 0xff, 0x01, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, idx, 0x00, 0x00, 0x00])
}

#[derive(Clone, Debug)]
pub struct SerialNewCapList(pub NewCapList);
#[derive(Clone, Debug)]
pub struct SerialNewCap(NewCapability);
#[derive(Clone, Debug)]
pub struct SerialCapability(Capability);
#[derive(Clone, Debug)]
pub struct SerialAddress(Address);

impl Serialize for SerialNewCapList {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let cap_list = &(self.0).0;

        let mut seq = serializer.serialize_seq(Some(cap_list.len()))?;
        for e in cap_list {
            seq.serialize_element(&SerialNewCap(e.clone()))?;
        }
        seq.end()
    }
}

impl Serialize for SerialNewCap {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let cap = &(self.0).cap;
        let parent_index = (self.0).parent_index;

        let mut state = serializer.serialize_struct("NewCapability", 2)?;
        state.serialize_field("cap", &SerialCapability(cap.clone()))?;
        state.serialize_field("parent_index", &parent_index)?;
        state.end()
    }
}

fn key_to_str(key: [u8; 24]) -> String {
    let mut key_hex: String = String::from("0x");;
    let s: String = key.to_hex();
    key_hex.push_str(&s);
    key_hex
}

fn b32_to_str(key: [u8; 32]) -> String {
    let mut key_hex: String = String::from("0x");;
    let s: String = key.to_hex();
    key_hex.push_str(&s);
    key_hex
}

impl Serialize for SerialCapability {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        match &self.0 {
            Capability::ProcedureCall(cap) => {
                let mut state = serializer.serialize_struct("ProcedureCallCap", 3)?;
                state.serialize_field("type", "ProcedureCallCap")?;
                state.serialize_field("prefix", &cap.prefix)?;
                state.serialize_field("key", &key_to_str(cap.key))?;
                state.end()
            },
            Capability::ProcedureRegister(cap) => {
                let mut state = serializer.serialize_struct("ProcedureRegisterCap", 3)?;
                state.serialize_field("type", "ProcedureRegisterCap")?;
                state.serialize_field("prefix", &cap.prefix)?;
                state.serialize_field("key", &key_to_str(cap.key))?;
                state.end()

            },
            Capability::ProcedureDelete(cap) => {
                let mut state = serializer.serialize_struct("ProcedureDeleteCap", 3)?;
                state.serialize_field("type", "ProcedureDeleteCap")?;
                state.serialize_field("prefix", &cap.prefix)?;
                state.serialize_field("key", &key_to_str(cap.key))?;
                state.end()

            },
            Capability::ProcedureEntry(_cap) => {
                let mut state = serializer.serialize_struct("ProcedureEntryCap", 1)?;
                state.serialize_field("type", "ProcedureEntryCap")?;
                state.end()

            },
            Capability::StoreWrite(cap) => {
                let mut state = serializer.serialize_struct("StoreWriteCap", 3)?;
                state.serialize_field("type", "StoreWriteCap")?;
                state.serialize_field("location", &b32_to_str(cap.location))?;
                state.serialize_field("size", &b32_to_str(cap.size))?;
                state.end()

            },
            Capability::Log(cap) => {
                let mut state = serializer.serialize_struct("LogCap", 6)?;
                state.serialize_field("type", "LogCap")?;
                state.serialize_field("topics", &cap.topics)?;
                state.serialize_field("t1", &b32_to_str(cap.t1))?;
                state.serialize_field("t2", &b32_to_str(cap.t2))?;
                state.serialize_field("t3", &b32_to_str(cap.t3))?;
                state.serialize_field("t4", &b32_to_str(cap.t4))?;
                state.end()

            },
            Capability::AccountCall(cap) => {
                let mut state = serializer.serialize_struct("AccountCallCap", 3)?;
                state.serialize_field("type", "AccountCallCap")?;
                state.serialize_field("can_call_any", &cap.can_call_any)?;
                state.serialize_field("can_send", &cap.can_send)?;
                state.serialize_field("address", &SerialAddress(utils::from_common_address(cap.address)))?;
                state.end()

            },
        }
    }
}

impl Serialize for SerialAddress {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let sl: String = (self.0).to_fixed_bytes().to_hex();
        serializer.serialize_str(format!("0x{}",sl).as_ref())
    }
}

#[derive(Clone, Debug)]
struct SerialNewCapListVisitor;

impl<'de> Visitor<'de> for SerialNewCapListVisitor {
    type Value = SerialNewCapList;

    fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        formatter.write_str("a new capability list")
    }

    fn visit_seq<S>(self, mut seq: S) -> Result<Self::Value, S::Error>
    where
        S: SeqAccess<'de>,
    {
        let mut cap_list = Vec::new();

        // Update the max while there are additional values.
        while let Some(value) = seq.next_element()? {
            let v: serde_json::Value = value;
            let SerialNewCap(new_cap) = serde_json::from_value(v).unwrap();
            cap_list.push(new_cap);
        }

        Ok(SerialNewCapList(NewCapList(cap_list)))
    }
}

impl<'de> Deserialize<'de> for SerialNewCapList {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
        where D: Deserializer<'de>
    {
        deserializer.deserialize_seq(SerialNewCapListVisitor)
    }
}



#[derive(Clone, Debug)]
struct SerialNewCapVisitor;

impl<'de> Visitor<'de> for SerialNewCapVisitor {
    type Value = SerialNewCap;

    fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        formatter.write_str("a capability")
    }

    fn visit_map<M>(self, mut access: M) -> Result<Self::Value, M::Error>
    where
        M: MapAccess<'de>,
    {
        // While there are entries remaining in the input, add them
        // into our map.
        let mut cap = None;
        let mut parent_index = None;
        while let Some((key, value)) = access.next_entry()? {
            let k: String = key;
            match k.as_ref() {
                "cap" => {
                    let SerialCapability(cap_s) = serde_json::from_value(value).unwrap();
                    cap = Some(cap_s);
                },
                "parent_index" => {
                    parent_index = Some(serde_json::from_value(value).unwrap());
                },
                _ => (),
            }
        }
        match (cap, parent_index) {
            (Some(cap), Some(parent_index)) => Ok(SerialNewCap(NewCapability {
                cap,
                parent_index,
            })),
            _ => Err(serde::de::Error::custom("missing data")),
        }
    }
}

impl<'de> Deserialize<'de> for SerialNewCap {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
        where D: Deserializer<'de>
    {
        deserializer.deserialize_map(SerialNewCapVisitor)
    }
}


#[derive(Clone, Debug)]
struct SerialCapabilityVisitor;

impl<'de> Visitor<'de> for SerialCapabilityVisitor {
    type Value = SerialCapability;

    fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        formatter.write_str("a capability")
    }

    fn visit_map<M>(self, mut access: M) -> Result<Self::Value, M::Error>
    where
        M: MapAccess<'de>,
    {
        // let mut map = MyMap::with_capacity(access.size_hint().unwrap_or(0));

        // While there are entries remaining in the input, add them
        // into our map.
        let mut map = HashMap::new();
        while let Some((key, value)) = access.next_entry()? {
            let k: String = key;
            let v: serde_json::Value = value;
            map.insert(k, v);
        }

        let type_string: String = serde_json::from_value(map.get("type").unwrap().clone()).unwrap();
        match type_string.as_ref() {
            "ProcedureCallCap" => {
                let prefix: u8 = serde_json::from_value(map.get("prefix").unwrap().clone()).unwrap();
                let key_s: String = serde_json::from_value(map.get("key").unwrap().clone()).unwrap();
                let key = str_to_key(key_s);
                Ok(SerialCapability(Capability::ProcedureCall(ProcedureCallCap {
                        prefix,
                        key,
                })))
            },
            "ProcedureRegisterCap" => {
                let prefix: u8 = serde_json::from_value(map.get("prefix").unwrap().clone()).unwrap();
                let key_s: String = serde_json::from_value(map.get("key").unwrap().clone()).unwrap();
                let key = str_to_key(key_s);
                Ok(SerialCapability(Capability::ProcedureRegister(ProcedureRegisterCap {
                        prefix,
                        key,
                })))
            },
            "ProcedureDeleteCap" => {
                let prefix: u8 = serde_json::from_value(map.get("prefix").unwrap().clone()).unwrap();
                let key_s: String = serde_json::from_value(map.get("key").unwrap().clone()).unwrap();
                let key = str_to_key(key_s);
                Ok(SerialCapability(Capability::ProcedureDelete(ProcedureDeleteCap {
                        prefix,
                        key,
                })))
            },
            "ProcedureEntryCap" => {
                Ok(SerialCapability(Capability::ProcedureEntry(ProcedureEntryCap)))
            },
            "StoreWriteCap" => {
                let location_s: String = serde_json::from_value(map.get("location").unwrap().clone()).unwrap();
                let size_s: String = serde_json::from_value(map.get("size").unwrap().clone()).unwrap();
                let location = str_to_b32(location_s);
                let size = str_to_b32(size_s);
                Ok(SerialCapability(Capability::StoreWrite(StoreWriteCap {
                        location,
                        size,
                })))
            },
            "LogCap" => {
                let topics: u8 = serde_json::from_value(map.get("n_topics").unwrap().clone()).unwrap();
                let t1_s: String = serde_json::from_value(map.get("t1").unwrap().clone()).unwrap();
                let t1 = str_to_b32(t1_s);
                let t2_s: String = serde_json::from_value(map.get("t2").unwrap().clone()).unwrap();
                let t2 = str_to_b32(t2_s);
                let t3_s: String = serde_json::from_value(map.get("t3").unwrap().clone()).unwrap();
                let t3 = str_to_b32(t3_s);
                let t4_s: String = serde_json::from_value(map.get("t4").unwrap().clone()).unwrap();
                let t4 = str_to_b32(t4_s);
                Ok(SerialCapability(Capability::Log(LogCap {
                        topics,
                        t1,
                        t2,
                        t3,
                        t4,
                })))
            },
            "AccountCallCap" => {
                let can_call_any: bool = serde_json::from_value(map.get("can_call_any").unwrap().clone()).unwrap();
                let can_send: bool = serde_json::from_value(map.get("can_send").unwrap().clone()).unwrap();
                let SerialAddress(address): SerialAddress = serde_json::from_value(map.get("address").unwrap().clone()).unwrap();
                Ok(SerialCapability(Capability::AccountCall(AccountCallCap {
                    can_call_any,
                    can_send,
                    address: utils::to_common_address(address),
                })))
            },
            t => Err(serde::de::Error::custom(format!("unrecognised cap type: {}", t))),
        }
    }
}

impl<'de> Deserialize<'de> for SerialCapability {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
        where D: Deserializer<'de>
    {
        deserializer.deserialize_map(SerialCapabilityVisitor)
    }
}

fn str_to_key(s: String) -> [u8; 24] {
    let (_,r) = s.split_at(2);
    let v: Vec<u8> = r.from_hex().unwrap();
    let mut n: [u8; 24] = [0; 24];
    n.copy_from_slice(v.as_slice());
    n
}

fn str_to_b32(s: String) -> [u8; 32] {
    let (_,r) = s.split_at(2);
    let v: Vec<u8> = r.from_hex().unwrap();
    let mut n: [u8; 32] = [0; 32];
    n.copy_from_slice(v.as_slice());
    n
}


struct SerialAddressVisitor;

impl<'de> Visitor<'de> for SerialAddressVisitor {
    type Value = SerialAddress;

    fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        formatter.write_str("an Ethereum address")
    }

    fn visit_str<E>(self, s: &str) -> Result<Self::Value, E>
    where
        E: de::Error,
    {
        let (_,r) = s.split_at(2);
        let b: Vec<u8> = r.from_hex().expect("hex decode");
        println!("b: {:?}", b);
        Ok(SerialAddress(Address::from_slice(&b)))
    }
}

impl<'de> Deserialize<'de> for SerialAddress {
    fn deserialize<D>(deserializer: D) -> Result<SerialAddress, D::Error>
    where
        D: Deserializer<'de>,
    {
        deserializer.deserialize_str(SerialAddressVisitor)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json;

    #[test]
    fn address_serialisation () {
        let s = "\"0xababababababababababababab1111ffffffffff\"";
        let des: SerialAddress = serde_json::from_str(s).unwrap();
        let ser = serde_json::to_string_pretty(&des);
        println!("address: {:?}", des);
        assert_eq!(s, ser.unwrap());
    }

    #[test]
    fn cap_serialisation () {
        let s = "{ \"type\": \"ProcedureCallCap\", \"prefix\": 4, \"key\": \"0x0000000000000ab000000000000000000000000000000000\" }";
        let des: SerialCapability = serde_json::from_str(s).expect("sss");
        println!("cap: {:?}", des);
    }

    #[test]
    fn new_cap_serialisation () {
        let s = "{\"cap\":{ \"type\": \"ProcedureCallCap\", \"prefix\": 4, \"key\": \"0x0000000000000ab000000000000000000000000000000000\" },\"parent_index\": 0}";
        let des: SerialNewCap = serde_json::from_str(s).expect("sss");
        println!("cap: {:?}", des);
    }

    #[test]
    fn new_cap_list_serialisation () {
        let s = "[{\"cap\":{ \"type\": \"ProcedureCallCap\", \"prefix\": 4, \"key\": \"0x0000000000000ab000000000000000000000000000000000\" },\"parent_index\": 0}]";
        let des: SerialNewCapList = serde_json::from_str(s).expect("sss");
        println!("cap: {:?}", des);
    }

    #[test]
    fn cap_serialisation_unknown () {
        let s = "{ \"type\": \"SomeCap\", \"prefix\": 4, \"key\": \"0x0000000000000ab000000000000000000000000000000000\" }";
        let des: Result<SerialCapability, _> = serde_json::from_str(s);
        println!("cap: {:?}", des.unwrap_err());
    }
}
