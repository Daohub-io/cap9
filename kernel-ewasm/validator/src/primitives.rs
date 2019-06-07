// This file is based on parity-wasm from parity, MIT & Apache Licensed
// use crate::rust::{vec::Vec, string::String};
use crate::{io};
use crate::{Deserialize};
use pwasm_std::vec::Vec;
use pwasm_std::String;
use crate::serialization::{Error};


macro_rules! buffered_read {
    ($buffer_size: expr, $length: expr, $reader: expr) => {
        {
            let mut vec_buf = Vec::new();
            let mut total_read = 0;
            let mut buf = [0u8; $buffer_size];
            while total_read < $length {
                let next_to_read = if $length - total_read > $buffer_size { $buffer_size } else { $length - total_read };
                $reader.read(&mut buf[0..next_to_read])?;
                vec_buf.extend_from_slice(&buf[0..next_to_read]);
                total_read += next_to_read;
            }
            vec_buf
        }
    }
}



/// Unsigned variable-length integer, limited to 32 bits,
/// represented by at most 5 bytes that may contain padding 0x80 bytes.
#[derive(Debug, Copy, Clone, PartialEq)]
pub struct VarUint32(u32);

impl From<VarUint32> for usize {
    fn from(var: VarUint32) -> usize {
        var.0 as usize
    }
}

impl From<VarUint32> for u32 {
    fn from(var: VarUint32) -> u32 {
        var.0
    }
}

impl From<u32> for VarUint32 {
    fn from(i: u32) -> VarUint32 {
        VarUint32(i)
    }
}

impl From<usize> for VarUint32 {
    fn from(i: usize) -> VarUint32 {
        assert!(i <= u32::max_value() as usize);
        VarUint32(i as u32)
    }
}

impl Deserialize for VarUint32 {
    type Error = Error;

    fn deserialize<R: io::Read>(reader: &mut R) -> Result<Self, Self::Error> {
        let mut res = 0;
        let mut shift = 0;
        let mut u8buf = [0u8; 1];
        loop {
            if shift > 31 { return Err(Error::InvalidVarUint32); }

            reader.read(&mut u8buf)?;
            let b = u8buf[0] as u32;
            res |= (b & 0x7f).checked_shl(shift).ok_or(Error::InvalidVarUint32)?;
            shift += 7;
            if (b >> 7) == 0 {
                if shift >= 32 && (b as u8).leading_zeros() < 4 {
                    return Err(Error::InvalidVarInt32);
                }
                break;
            }
        }
        Ok(VarUint32(res))
    }
}

/// Unsigned variable-length integer, limited to 64 bits,
/// represented by at most 9 bytes that may contain padding 0x80 bytes.
#[derive(Debug, Copy, Clone, PartialEq)]
pub struct VarUint64(u64);

impl From<VarUint64> for u64 {
    fn from(var: VarUint64) -> u64 {
        var.0
    }
}

impl Deserialize for VarUint64 {
    type Error = Error;

    fn deserialize<R: io::Read>(reader: &mut R) -> Result<Self, Self::Error> {
        let mut res = 0;
        let mut shift = 0;
        let mut u8buf = [0u8; 1];
        loop {
            if shift > 63 { return Err(Error::InvalidVarUint64); }

            reader.read(&mut u8buf)?;
            let b = u8buf[0] as u64;
            res |= (b & 0x7f).checked_shl(shift).ok_or(Error::InvalidVarUint64)?;
            shift += 7;
            if (b >> 7) == 0 {
                if shift >= 64 && (b as u8).leading_zeros() < 7 {
                    return Err(Error::InvalidVarInt64);
                }
                break;
            }
        }
        Ok(VarUint64(res))
    }
}

impl From<u64> for VarUint64 {
    fn from(u: u64) -> VarUint64 {
        VarUint64(u)
    }
}

/// 7-bit unsigned integer, encoded in LEB128 (always 1 byte length).
#[derive(Debug, Copy, Clone, PartialEq)]
pub struct VarUint7(u8);

impl From<VarUint7> for u8 {
    fn from(v: VarUint7) -> u8 {
        v.0
    }
}

impl From<u8> for VarUint7 {
    fn from(v: u8) -> Self {
        VarUint7(v)
    }
}

impl Deserialize for VarUint7 {
    type Error = Error;

    fn deserialize<R: io::Read>(reader: &mut R) -> Result<Self, Self::Error> {
        let mut u8buf = [0u8; 1];
        reader.read(&mut u8buf)?;
        Ok(VarUint7(u8buf[0]))
    }
}

/// 7-bit signed integer, encoded in LEB128 (always 1 byte length)
#[derive(Debug, Copy, Clone, PartialEq)]
pub struct VarInt7(i8);

impl From<VarInt7> for i8 {
    fn from(v: VarInt7) -> i8 {
        v.0
    }
}

impl From<i8> for VarInt7 {
    fn from(v: i8) -> VarInt7 {
        VarInt7(v)
    }
}

impl Deserialize for VarInt7 {
    type Error = Error;

    fn deserialize<R: io::Read>(reader: &mut R) -> Result<Self, Self::Error> {
        let mut u8buf = [0u8; 1];
        reader.read(&mut u8buf)?;

        // check if number is not continued!
        if u8buf[0] & 0b1000_0000 != 0 {
            return Err(Error::InvalidVarInt7(u8buf[0]));
        }

        // expand sign
        if u8buf[0] & 0b0100_0000 == 0b0100_0000 { u8buf[0] |= 0b1000_0000 }

        Ok(VarInt7(u8buf[0] as i8))
    }
}

/// 8-bit unsigned integer, NOT encoded in LEB128;
/// it's just a single byte.
#[derive(Debug, Copy, Clone, PartialEq)]
pub struct Uint8(u8);

impl From<Uint8> for u8 {
    fn from(v: Uint8) -> u8 {
        v.0
    }
}

impl From<u8> for Uint8 {
    fn from(v: u8) -> Self {
        Uint8(v)
    }
}

impl Deserialize for Uint8 {
    type Error = Error;

    fn deserialize<R: io::Read>(reader: &mut R) -> Result<Self, Self::Error> {
        let mut u8buf = [0u8; 1];
        reader.read(&mut u8buf)?;
        Ok(Uint8(u8buf[0]))
    }
}

/// 32-bit signed integer, encoded in LEB128 (can be 1-5 bytes length).
#[derive(Debug, Copy, Clone, PartialEq)]
pub struct VarInt32(i32);

impl From<VarInt32> for i32 {
    fn from(v: VarInt32) -> i32 {
        v.0
    }
}

impl From<i32> for VarInt32 {
    fn from(v: i32) -> VarInt32 {
        VarInt32(v)
    }
}

impl Deserialize for VarInt32 {
    type Error = Error;

    fn deserialize<R: io::Read>(reader: &mut R) -> Result<Self, Self::Error> {
        let mut res = 0;
        let mut shift = 0;
        let mut u8buf = [0u8; 1];
        loop {
            if shift > 31 { return Err(Error::InvalidVarInt32); }
            reader.read(&mut u8buf)?;
            let b = u8buf[0];

            res |= ((b & 0x7f) as i32).checked_shl(shift).ok_or(Error::InvalidVarInt32)?;

            shift += 7;
            if (b >> 7) == 0 {
                if shift < 32 && b & 0b0100_0000 == 0b0100_0000 {
                    res |= (1i32 << shift).wrapping_neg();
                } else if shift >= 32 && b & 0b0100_0000 == 0b0100_0000 {
                    if (!(b | 0b1000_0000)).leading_zeros() < 5 {
                        return Err(Error::InvalidVarInt32);
                    }
                } else if shift >= 32 && b & 0b0100_0000 == 0 {
                    if b.leading_zeros() < 5 {
                        return Err(Error::InvalidVarInt32);
                    }
                }
                break;
            }
        }
        Ok(VarInt32(res))
    }
}

/// 64-bit signed integer, encoded in LEB128 (can be 1-9 bytes length).
#[derive(Debug, Copy, Clone, PartialEq)]
pub struct VarInt64(i64);

impl From<VarInt64> for i64 {
    fn from(v: VarInt64) -> i64 {
        v.0
    }
}

impl From<i64> for VarInt64 {
    fn from(v: i64) -> VarInt64 {
        VarInt64(v)
    }
}

impl Deserialize for VarInt64 {
    type Error = Error;

    fn deserialize<R: io::Read>(reader: &mut R) -> Result<Self, Self::Error> {
        let mut res = 0i64;
        let mut shift = 0;
        let mut u8buf = [0u8; 1];

        loop {
            if shift > 63 { return Err(Error::InvalidVarInt64); }
            reader.read(&mut u8buf)?;
            let b = u8buf[0];

            res |= ((b & 0x7f) as i64).checked_shl(shift).ok_or(Error::InvalidVarInt64)?;

            shift += 7;
            if (b >> 7) == 0 {
                if shift < 64 && b & 0b0100_0000 == 0b0100_0000 {
                    res |= (1i64 << shift).wrapping_neg();
                } else if shift >= 64 && b & 0b0100_0000 == 0b0100_0000 {
                    if (b | 0b1000_0000) as i8 != -1 {
                        return Err(Error::InvalidVarInt64);
                    }
                } else if shift >= 64 && b != 0 {
                    return Err(Error::InvalidVarInt64);
                }
                break;
            }
        }
        Ok(VarInt64(res))
    }
}

/// 32-bit unsigned integer, encoded in little endian.
#[derive(Debug, Copy, Clone, PartialEq)]
pub struct Uint32(u32);

impl Deserialize for Uint32 {
    type Error = Error;

    fn deserialize<R: io::Read>(reader: &mut R) -> Result<Self, Self::Error> {
        let mut buf = [0u8; 4];
        reader.read(&mut buf)?;
        // todo check range
        Ok(u32::from_le_bytes(buf).into())
    }
}

impl From<Uint32> for u32 {
    fn from(var: Uint32) -> u32 {
        var.0
    }
}

impl From<u32> for Uint32 {
    fn from(u: u32) -> Self { Uint32(u) }
}

/// 64-bit unsigned integer, encoded in little endian.
#[derive(Debug, Copy, Clone, PartialEq)]
pub struct Uint64(u64);

impl Deserialize for Uint64 {
    type Error = Error;

    fn deserialize<R: io::Read>(reader: &mut R) -> Result<Self, Self::Error> {
        let mut buf = [0u8; 8];
        reader.read(&mut buf)?;
        // todo check range
        Ok(u64::from_le_bytes(buf).into())
    }
}

impl From<u64> for Uint64 {
    fn from(u: u64) -> Self { Uint64(u) }
}

impl From<Uint64> for u64 {
    fn from(var: Uint64) -> u64 {
        var.0
    }
}


/// VarUint1, 1-bit value (0/1).
#[derive(Debug, Copy, Clone, PartialEq)]
pub struct VarUint1(bool);

impl From<VarUint1> for bool {
    fn from(v: VarUint1) -> bool {
        v.0
    }
}

impl From<bool> for VarUint1 {
    fn from(b: bool) -> Self {
        VarUint1(b)
    }
}

impl Deserialize for VarUint1 {
    type Error = Error;

    fn deserialize<R: io::Read>(reader: &mut R) -> Result<Self, Self::Error> {
        let mut u8buf = [0u8; 1];
        reader.read(&mut u8buf)?;
        match u8buf[0] {
            0 => Ok(VarUint1(false)),
            1 => Ok(VarUint1(true)),
            v @ _ => Err(Error::InvalidVarUint1(v)),
        }
    }
}

impl Deserialize for String {
    type Error = Error;

    fn deserialize<R: io::Read>(reader: &mut R) -> Result<Self, Self::Error> {
        let length = u32::from(VarUint32::deserialize(reader)?) as usize;
        if length > 0 {
            String::from_utf8(buffered_read!(1024, length, reader)).map_err(|_| Error::NonUtf8String)
        }
        else {
            Ok(String::new())
        }
    }
}

/// List for reading sequence of elements typed `T`, given
/// they are preceded by length (serialized as VarUint32).
#[derive(Debug, Clone)]
pub struct CountedList<T: Deserialize>(Vec<T>);

impl<T: Deserialize> CountedList<T> {
    /// Destroy counted list returing inner vector.
    pub fn into_inner(self) -> Vec<T> { self.0 }
}

impl<T: Deserialize> Deserialize for CountedList<T> where T::Error: From<Error> {
    type Error = T::Error;

    fn deserialize<R: io::Read>(reader: &mut R) -> Result<Self, Self::Error> {
        let count: usize = VarUint32::deserialize(reader)?.into();
        let mut result = Vec::new();
        for _ in 0..count { result.push(T::deserialize(reader)?); }
        Ok(CountedList(result))
    }
}

// #[cfg(test)]
// mod tests {

// 	use super::super::{deserialize_buffer, Serialize};
// 	use super::{CountedList, VarInt7, VarUint32, VarInt32, VarInt64, VarUint64};
// 	use crate::io::Error;

// 	fn varuint32_ser_test(val: u32, expected: Vec<u8>) {
// 		let mut buf = Vec::new();
// 		let v1: VarUint32 = val.into();
// 		v1.serialize(&mut buf).expect("to be serialized ok");
// 		assert_eq!(expected, buf);
// 	}

// 	fn varuint32_de_test(dt: Vec<u8>, expected: u32) {
// 		let val: VarUint32 = deserialize_buffer(&dt).expect("buf to be serialized");
// 		assert_eq!(expected, val.into());
// 	}

// 	fn varuint32_serde_test(dt: Vec<u8>, val: u32) {
// 		varuint32_de_test(dt.clone(), val);
// 		varuint32_ser_test(val, dt);
// 	}

// 	fn varint32_ser_test(val: i32, expected: Vec<u8>) {
// 		let mut buf = Vec::new();
// 		let v1: VarInt32 = val.into();
// 		v1.serialize(&mut buf).expect("to be serialized ok");
// 		assert_eq!(expected, buf);
// 	}

// 	fn varint32_de_test(dt: Vec<u8>, expected: i32) {
// 		let val: VarInt32 = deserialize_buffer(&dt).expect("buf to be serialized");
// 		assert_eq!(expected, val.into());
// 	}

// 	fn varint32_serde_test(dt: Vec<u8>, val: i32) {
// 		varint32_de_test(dt.clone(), val);
// 		varint32_ser_test(val, dt);
// 	}

// 	fn varuint64_ser_test(val: u64, expected: Vec<u8>) {
// 		let mut buf = Vec::new();
// 		let v1: VarUint64 = val.into();
// 		v1.serialize(&mut buf).expect("to be serialized ok");
// 		assert_eq!(expected, buf);
// 	}

// 	fn varuint64_de_test(dt: Vec<u8>, expected: u64) {
// 		let val: VarUint64 = deserialize_buffer(&dt).expect("buf to be serialized");
// 		assert_eq!(expected, val.into());
// 	}

// 	fn varuint64_serde_test(dt: Vec<u8>, val: u64) {
// 		varuint64_de_test(dt.clone(), val);
// 		varuint64_ser_test(val, dt);
// 	}

// 	fn varint64_ser_test(val: i64, expected: Vec<u8>) {
// 		let mut buf = Vec::new();
// 		let v1: VarInt64 = val.into();
// 		v1.serialize(&mut buf).expect("to be serialized ok");
// 		assert_eq!(expected, buf);
// 	}

// 	fn varint64_de_test(dt: Vec<u8>, expected: i64) {
// 		let val: VarInt64 = deserialize_buffer(&dt).expect("buf to be serialized");
// 		assert_eq!(expected, val.into());
// 	}

// 	fn varint64_serde_test(dt: Vec<u8>, val: i64) {
// 		varint64_de_test(dt.clone(), val);
// 		varint64_ser_test(val, dt);
// 	}

// 	#[test]
// 	fn varuint32_0() {
// 		varuint32_serde_test(vec![0u8; 1], 0);
// 	}

// 	#[test]
// 	fn varuint32_1() {
// 		varuint32_serde_test(vec![1u8; 1], 1);
// 	}

// 	#[test]
// 	fn varuint32_135() {
// 		varuint32_serde_test(vec![135u8, 0x01], 135);
// 	}

// 	#[test]
// 	fn varuint32_8192() {
// 		varuint32_serde_test(vec![0x80, 0x40], 8192);
// 	}

// 	#[test]
// 	fn varint32_8192() {
// 		varint32_serde_test(vec![0x80, 0xc0, 0x00], 8192);
// 	}

// 	#[test]
// 	fn varint32_neg_8192() {
// 		varint32_serde_test(vec![0x80, 0x40], -8192);
// 	}

// 	#[test]
// 	fn varuint64_0() {
// 		varuint64_serde_test(vec![0u8; 1], 0);
// 	}

// 	#[test]
// 	fn varuint64_1() {
// 		varuint64_serde_test(vec![1u8; 1], 1);
// 	}

// 	#[test]
// 	fn varuint64_135() {
// 		varuint64_serde_test(vec![135u8, 0x01], 135);
// 	}

// 	#[test]
// 	fn varuint64_8192() {
// 		varuint64_serde_test(vec![0x80, 0x40], 8192);
// 	}

// 	#[test]
// 	fn varint64_8192() {
// 		varint64_serde_test(vec![0x80, 0xc0, 0x00], 8192);
// 	}

// 	#[test]
// 	fn varint64_neg_8192() {
// 		varint64_serde_test(vec![0x80, 0x40], -8192);
// 	}

// 	#[test]
// 	fn varint64_min() {
// 		varint64_serde_test(
// 			vec![0x80, 0x80, 0x80, 0x80, 0x80, 0x80, 0x80, 0x80, 0x80, 0x7f],
// 			-9223372036854775808,
// 		);
// 	}

// 	#[test]
// 	fn varint64_bad_extended() {
// 		let res = deserialize_buffer::<VarInt64>(&[0x80, 0x80, 0x80, 0x80, 0x80, 0x80, 0x80, 0x80, 0x80, 0x6f][..]);
// 		assert!(res.is_err());
// 	}

// 	#[test]
// 	fn varint32_bad_extended() {
// 		let res = deserialize_buffer::<VarInt32>(&[0x80, 0x80, 0x80, 0x80, 0x6f][..]);
// 		assert!(res.is_err());
// 	}

// 	#[test]
// 	fn varint32_bad_extended2() {
// 		let res = deserialize_buffer::<VarInt32>(&[0x80, 0x80, 0x80, 0x80, 0x41][..]);
// 		assert!(res.is_err());
// 	}

// 	#[test]
// 	fn varint64_max() {
// 		varint64_serde_test(
// 			vec![0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0x00],
// 			9223372036854775807,
// 		);
// 	}

// 	#[test]
// 	fn varint64_too_long() {
// 		assert!(
// 			deserialize_buffer::<VarInt64>(
// 				&[0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0x00][..],
// 			).is_err()
// 		);
// 	}

// 	#[test]
// 	fn varint32_too_long() {
// 		assert!(
// 			deserialize_buffer::<VarInt32>(
// 				&[0xff, 0xff, 0xff, 0xff, 0xff, 0x00][..],
// 			).is_err()
// 		);
// 	}

// 	#[test]
// 	fn varuint64_too_long() {
// 		assert!(
// 			deserialize_buffer::<VarUint64>(
// 				&[0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0x00][..],
// 			).is_err()
// 		);
// 	}

// 	#[test]
// 	fn varuint32_too_long() {
// 		assert!(
// 			deserialize_buffer::<VarUint32>(
// 				&[0xff, 0xff, 0xff, 0xff, 0xff, 0x00][..],
// 			).is_err()
// 		);
// 	}

// 	#[test]
// 	fn varuint32_too_long_trailing() {
// 		assert!(
// 			deserialize_buffer::<VarUint32>(
// 				&[0xff, 0xff, 0xff, 0xff, 0x7f][..],
// 			).is_err()
// 		);
// 	}

// 	#[test]
// 	fn varuint64_too_long_trailing() {
// 		assert!(
// 			deserialize_buffer::<VarUint64>(
// 				&[0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0x04][..],
// 			).is_err()
// 		);
// 	}

// 	#[test]
// 	fn varint32_min() {
// 		varint32_serde_test(
// 			vec![0x80, 0x80, 0x80, 0x80, 0x78],
// 			-2147483648,
// 		);
// 	}

// 	#[test]
// 	fn varint7_invalid() {
// 		match deserialize_buffer::<VarInt7>(&[240]) {
// 			Err(Error::InvalidVarInt7(_)) => {},
// 			_ => panic!("Should be invalid varint7 error!")
// 		}
// 	}

// 	#[test]
// 	fn varint7_neg() {
// 		assert_eq!(-0x10i8, deserialize_buffer::<VarInt7>(&[0x70]).expect("fail").into());
// 	}

// 	#[test]
// 	fn varuint32_too_long_nulled() {
// 		match deserialize_buffer::<VarUint32>(
// 			&[0x80, 0x80, 0x80, 0x80, 0x80, 0x80, 0x80, 0x80, 0x78]
// 		) {
// 			Err(Error::InvalidVarUint32) => {},
// 			_ => panic!("Should be invalid varuint32"),
// 		}
// 	}

// 	#[test]
// 	fn varint32_max() {
// 		varint32_serde_test(
// 			vec![0xff, 0xff, 0xff, 0xff, 0x07],
// 			2147483647,
// 		);
// 	}


// 	#[test]
// 	fn counted_list() {
// 		let payload = [
// 			133u8, //(128+5), length is 5
// 				0x80, 0x80, 0x80, 0x0, // padding
// 			0x01,
// 			0x7d,
// 			0x05,
// 			0x07,
// 			0x09,
// 		];

// 		let list: CountedList<VarInt7> =
// 			deserialize_buffer(&payload).expect("type_section be deserialized");

// 		let vars = list.into_inner();
// 		assert_eq!(5, vars.len());
// 		let v3: i8 = (*vars.get(1).unwrap()).into();
// 		assert_eq!(-0x03i8, v3);
// 	}
// }
