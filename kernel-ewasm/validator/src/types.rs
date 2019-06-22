// use crate::rust::{fmt, vec::Vec};
pub use core::fmt;
use pwasm_std::vec::Vec;
use cap9_core;
use crate::serialization::{Error, WASMDeserialize};
use super::{VarUint7, VarInt7, CountedList, VarUint32};
pub use pwasm_std::types::{U256,H256, Address};

/// Type definition in types section. Currently can be only of the function type.
#[derive(Debug, Clone, PartialEq, Hash, Eq)]
pub enum Type {
    /// Function type.
    Function(FunctionType),
}

impl WASMDeserialize for Type {
    type Error = Error;

    fn deserialize<R: cap9_core::Read<u8>>(reader: &mut R) -> Result<Self, Self::Error> {
        Ok(Type::Function(FunctionType::deserialize(reader)?))
    }
}

/// Value type.
#[derive(Clone, Copy, Debug, PartialEq, Hash, Eq)]
pub enum ValueType {
    /// 32-bit signed integer
    I32,
    /// 64-bit signed integer
    I64,
    /// 32-bit float
    F32,
    /// 64-bit float
    F64,
    /// 128-bit SIMD register
    V128,
}

impl WASMDeserialize for ValueType {
    type Error = Error;

    fn deserialize<R: cap9_core::Read<u8>>(reader: &mut R) -> Result<Self, Self::Error> {
        let val = VarInt7::deserialize(reader)?;

        match val.into() {
            -0x01 => Ok(ValueType::I32),
            -0x02 => Ok(ValueType::I64),
            -0x03 => Ok(ValueType::F32),
            -0x04 => Ok(ValueType::F64),
            -0x05 => Ok(ValueType::V128),
            // _ => Err(Error::UnknownValueType(val.into())),
            _ => panic!("unknown value type")
        }
    }
}

/// Block type which is basically `ValueType` + NoResult (to define blocks that have no return type)
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum BlockType {
    /// Value-type specified block type
    Value(ValueType),
    /// No specified block type
    NoResult,
}

impl WASMDeserialize for BlockType {
    type Error = Error;

    fn deserialize<R: cap9_core::Read<u8>>(reader: &mut R) -> Result<Self, Self::Error> {
        let val = VarInt7::deserialize(reader)?;

        match val.into() {
            -0x01 => Ok(BlockType::Value(ValueType::I32)),
            -0x02 => Ok(BlockType::Value(ValueType::I64)),
            -0x03 => Ok(BlockType::Value(ValueType::F32)),
            -0x04 => Ok(BlockType::Value(ValueType::F64)),
            0x7b => Ok(BlockType::Value(ValueType::V128)),
            -0x40 => Ok(BlockType::NoResult),
            // _ => Err(Error::UnknownValueType(val.into())),
            _ => panic!("unknown value type")
        }
    }
}

/// Function signature type.
#[derive(Debug, Clone, PartialEq, Hash, Eq)]
pub struct FunctionType {
    form: u8,
    params: Vec<ValueType>,
    return_type: Option<ValueType>,
}

impl Default for FunctionType {
    fn default() -> Self {
        FunctionType {
            form: 0x60,
            params: Vec::new(),
            return_type: None,
        }
    }
}

impl FunctionType {
    /// New function type given the signature in-params(`params`) and return type (`return_type`)
    pub fn new(params: Vec<ValueType>, return_type: Option<ValueType>) -> Self {
        FunctionType {
            params: params,
            return_type: return_type,
            ..Default::default()
        }
    }
    /// Function form (currently only valid value is `0x60`)
    pub fn form(&self) -> u8 { self.form }
    /// Parameters in the function signature.
    pub fn params(&self) -> &[ValueType] { &self.params }
    /// Mutable parameters in the function signature.
    pub fn params_mut(&mut self) -> &mut Vec<ValueType> { &mut self.params }
    /// Return type in the function signature, if any.
    pub fn return_type(&self) -> Option<ValueType> { self.return_type }
    /// Mutable type in the function signature, if any.
    pub fn return_type_mut(&mut self) -> &mut Option<ValueType> { &mut self.return_type }
}

impl WASMDeserialize for FunctionType {
    type Error = Error;

    fn deserialize<R: cap9_core::Read<u8>>(reader: &mut R) -> Result<Self, Self::Error> {
        let form: u8 = VarUint7::deserialize(reader)?.into();

        if form != 0x60 {
            return Err(Error::UnknownFunctionForm(form));
        }

        let params: Vec<ValueType> = CountedList::deserialize(reader)?.into_inner();

        let return_types: u32 = VarUint32::deserialize(reader)?.into();

        let return_type = if return_types == 1 {
            Some(ValueType::deserialize(reader)?)
        } else if return_types == 0 {
            None
        } else {
            return Err(Error::Other("Return types length should be 0 or 1"));
        };

        Ok(FunctionType {
            form: form,
            params: params,
            return_type: return_type,
        })
    }
}

/// Table element type.
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum TableElementType {
    /// A reference to a function with any signature.
    AnyFunc,
}

impl WASMDeserialize for TableElementType {
    type Error = Error;

    fn deserialize<R: cap9_core::Read<u8>>(reader: &mut R) -> Result<Self, Self::Error> {
        let val = VarInt7::deserialize(reader)?;

        match val.into() {
            -0x10 => Ok(TableElementType::AnyFunc),
            _ => Err(Error::UnknownTableElementType(val.into())),
        }
    }
}
