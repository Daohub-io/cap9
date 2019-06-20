use super::{AsCap,ProcedureKey,matching_keys};
use cap9_core::{Serialize, Deserialize};
use pwasm_abi::types::*;

pub const CAP_ACC_CALL: u8 = 9;
pub const CAP_ACC_CALL_SIZE: u8 = 1;

#[derive(Clone, Debug, PartialEq)]
pub struct AccountCallCap {
    pub can_call_any: bool,
    pub can_send: bool,
    pub address: Address,
}

impl AsCap for AccountCallCap {
    fn is_subset_of(&self, parent_cap: &Self) -> bool {
        // If the requested value of callAny is true, then the parent cap
        // value of callAny must be true.
        if self.can_call_any {
            if !parent_cap.can_call_any {
                return false;
            }
        } else {
            // if the parent_cap value is callAny, we don't care about the value
            // of ethAddress. If the requested value of callAny is false we must
            // check that the addresses are the same
            if !parent_cap.can_call_any {
                // the addresses must match
                if self.address != parent_cap.address {
                    return false;
                }
            }
        }

        // if the requested sendValue flag is true, the parent sendValue flag
        // must also be true.
        if self.can_send && !parent_cap.can_send {
            return false;
        }

        // Othwerwise we can consider it a subset
        true
    }
}

impl Deserialize<U256> for AccountCallCap {
    type Error = cap9_core::Error;

    fn deserialize<R: cap9_core::Read<U256>>(reader: &mut R) -> Result<Self, Self::Error> {
        let mut buf = [U256::zero(); 1];
        reader.read(&mut buf).unwrap();
        let val: U256 = buf[0];

        let can_call_any = val.bit(255);
        let can_send = val.bit(254);

        let mut address = [0u8; 20];
        address.copy_from_slice(&<[u8; 32]>::from(val)[12..]);
        let address = H160::from(address);

        let account_call_cap = AccountCallCap {
            can_call_any: can_call_any,
            can_send: can_send,
            address: address,
        };

        Ok(account_call_cap)
    }
}

impl From<U256> for AccountCallCap {
    fn from(val: U256) -> Self {
        let can_call_any = val.bit(255);
        let can_send = val.bit(254);

        let mut address = [0u8; 20];
        address.copy_from_slice(&<[u8; 32]>::from(val)[12..]);
        let address = H160::from(address);

        AccountCallCap {
            can_call_any: can_call_any,
            can_send: can_send,
            address: address,
        }
    }
}
