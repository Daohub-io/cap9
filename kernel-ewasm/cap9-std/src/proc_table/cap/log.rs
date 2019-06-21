use super::{AsCap,ProcedureKey,matching_keys};
use cap9_core::{Serialize, Deserialize};
use pwasm_abi::types::*;
pub const CAP_LOG: u8 = 8;
pub const CAP_LOG_SIZE: u8 = 5;

#[derive(Clone, Debug, PartialEq)]
pub struct LogCap {
    pub topics: u8,
    pub t1: [u8; 32],
    pub t2: [u8; 32],
    pub t3: [u8; 32],
    pub t4: [u8; 32],
}

impl AsCap for LogCap {
    fn is_subset_of(&self, parent_cap: &Self) -> bool {
        // First we check the number of required topics. The number of
        // required topics of the requested cap must be equal to or greater
        // than the number of required topics for the current cap.
        if self.topics < parent_cap.topics {
            return false;
        }
        // Next we check that the topics required by the parent cap are
        // also required by the requested cap.
        if parent_cap.topics >= 1 {
            if parent_cap.t1 != self.t1 {
                return false;
            }
        }
        if parent_cap.topics >= 2 {
            if parent_cap.t2 != self.t2 {
                return false;
            }
        }
        if parent_cap.topics >= 3 {
            if parent_cap.t3 != self.t3 {
                return false;
            }
        }
        if parent_cap.topics >= 4 {
            if parent_cap.t4 != self.t4 {
                return false;
            }
        }
        true
    }
}

impl Deserialize<U256> for LogCap {
    type Error = cap9_core::Error;

    fn deserialize<R: cap9_core::Read<U256>>(reader: &mut R) -> Result<Self, Self::Error> {
        let mut buf = [U256::zero(); 5];
        reader.read(&mut buf).unwrap();

        let topics_len: usize = buf[0].byte(0) as usize;
        let mut topics = [[0; 32]; 4];
        if topics_len != 0 {
            match topics_len {
                1..=4 => {
                    for i in 0..topics_len {
                        topics[i] = buf[i+1].into()
                    }
                }
                _ => return Err(cap9_core::Error::InvalidData)
                // _ => return Err(CapDecodeErr::InvalidCapLen(topics_len as u8)),
            }
        }

        Ok(LogCap {
            topics: topics_len as u8,
            t1: topics[0],
            t2: topics[1],
            t3: topics[2],
            t4: topics[3],
        })
    }
}

impl Serialize<U256> for LogCap {
    type Error = cap9_core::Error;

    fn serialize<W: cap9_core::Write<U256>>(&self, writer: &mut W) -> Result<(), Self::Error> {
        let topics_len = U256::from(self.topics);
        let t1 = U256::from(self.t1);
        let t2 = U256::from(self.t2);
        let t3 = U256::from(self.t3);
        let t4 = U256::from(self.t4);

        writer.write(&[topics_len, t1, t2, t3, t4])?;
        Ok(())
    }
}
