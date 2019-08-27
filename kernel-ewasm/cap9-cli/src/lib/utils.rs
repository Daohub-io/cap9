use web3::types::{Address, U256, H256};
use pwasm_abi;

pub fn string_to_proc_key(mut name: String) -> [u8; 24] {
    if !name.is_ascii() {
        panic!("name ({}) is not ascii", name);
    }
    if name.len() > 24 {
        panic!("name ({}) is greater than 24 characters, it is {} characters", name, name.len());
    }
    name.push_str("\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0");
    name.truncate(24);
    let mut procedure_key : [u8; 24] = [0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0];
    let byte_name = name.into_bytes();
    procedure_key.clone_from_slice(&byte_name[..24]);
    procedure_key
}

pub fn proc_key_to_string<'a>(key: &'a [u8]) -> &'a str {
    std::str::from_utf8(key).unwrap().trim_end_matches('\0')
}

pub fn proc_key_to_32_bytes(proc_key: &[u8; 24]) -> [u8; 32] {
    let mut buf = [0; 32];
    buf[8..].copy_from_slice(proc_key);
    buf
}



/// Convert one U256 of the ABI type a U256 of the web3 library type (the web3
/// library and the ABI use different U256s).
pub fn from_common_u256(u: pwasm_abi::types::U256) -> U256 {
    let mut buf = [0; 32];
    u.to_little_endian(&mut buf);
    U256::from_little_endian(&buf)
}

pub fn to_common_u256(u: U256) -> pwasm_abi::types::U256 {
    let mut buf = [0; 32];
    u.to_big_endian(&mut buf);
    pwasm_abi::types::U256::from_big_endian(&buf)
}

pub fn to_common_h256(h: H256) -> pwasm_abi::types::H256 {
    let buf = h.as_fixed_bytes();
    pwasm_abi::types::H256::from_slice(buf)
}

pub fn from_common_address(a: pwasm_abi::types::Address) -> Address {
    let buf = a.as_fixed_bytes();
    Address::from_slice(buf)
}

pub fn to_common_address(a: Address) -> pwasm_abi::types::Address {
    let buf = a.as_fixed_bytes();
    pwasm_abi::types::Address::from_slice(buf)
}

/// Convert a vector of U256 of the ABI type a U256 of the web3 library type
/// (the web3 library and the ABI use different U256s).
pub fn from_common_u256_vec(v: Vec<pwasm_abi::types::U256>) -> Vec<U256> {
    let mut new_v = Vec::new();
    for n in v {
        new_v.push(from_common_u256(n))
    }
    new_v
}
