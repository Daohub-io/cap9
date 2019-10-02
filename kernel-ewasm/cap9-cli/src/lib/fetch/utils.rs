use web3::types::{H256, U256};

pub fn h256_to_u256(h: H256) -> U256 {
    U256::from_big_endian(&h.to_fixed_bytes())
}

pub fn u256_to_h256(u: U256) -> H256 {
    let mut buf: [u8; 32] = [0; 32];
    u.to_big_endian(&mut buf);
    H256::from_slice(&buf)
}
