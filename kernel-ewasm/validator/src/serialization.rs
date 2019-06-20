use crate::primitives::*;
pub use core::fmt;
use pwasm_std::vec::{Vec};

use pwasm_std::String;


/// Deserialization from serial i/o.
pub trait WASMDeserialize : Sized {
    /// Serialization error produced by deserialization routine.
    type Error: From<cap9_core::Error>;
    /// Deserialize type from serial i/o
    fn deserialize<R: cap9_core::Read<u8>>(reader: &mut R) -> Result<Self, Self::Error>;
}

/// Serialization to serial i/o. Takes self by value to consume less memory
/// (parity-wasm IR is being partially freed by filling the result buffer).
pub trait WASMSerialize {
    /// Serialization error produced by serialization routine.
    type Error: From<cap9_core::Error>;
    /// Serialize type to serial i/o
    fn serialize<W: cap9_core::Write<u8>>(self, writer: &mut W) -> Result<(), Self::Error>;
}


/// Deserialization/serialization error
#[derive(Debug, Clone)]
pub enum Error {
    /// Unexpected end of input.
    UnexpectedEof,
    /// Invalid magic.
    InvalidMagic,
    /// Unsupported version.
    UnsupportedVersion(u32),
    /// Inconsistence between declared and actual length.
    InconsistentLength {
        /// Expected length of the definition.
        expected: usize,
        /// Actual length of the definition.
        actual: usize
    },
    /// Other static error.
    Other(&'static str),
    /// Other allocated error.
    HeapOther(String),
    /// Invalid/unknown value type declaration.
    UnknownValueType(i8),
    /// Invalid/unknown table element type declaration.
    UnknownTableElementType(i8),
    /// Non-utf8 string.
    NonUtf8String,
    /// Unknown external kind code.
    UnknownExternalKind(u8),
    /// Unknown internal kind code.
    UnknownInternalKind(u8),
    /// Unknown opcode encountered.
    UnknownOpcode(u8),
    /// Unknown SIMD opcode encountered.
    UnknownSimdOpcode(u32),
    /// Invalid VarUint1 value.
    InvalidVarUint1(u8),
    /// Invalid VarInt32 value.
    InvalidVarInt32,
    /// Invalid VarInt64 value.
    InvalidVarInt64,
    /// Invalid VarUint32 value.
    InvalidVarUint32,
    /// Invalid VarUint64 value.
    InvalidVarUint64,
    /// Inconsistent metadata.
    InconsistentMetadata,
    /// Invalid section id.
    InvalidSectionId(u8),
    /// Sections are out of order.
    SectionsOutOfOrder,
    /// Duplicated sections.
    DuplicatedSections(u8),
    /// Invalid memory reference (should be 0).
    InvalidMemoryReference(u8),
    /// Invalid table reference (should be 0).
    InvalidTableReference(u8),
    /// Invalid value used for flags in limits type.
    InvalidLimitsFlags(u8),
    /// Unknown function form (should be 0x60).
    UnknownFunctionForm(u8),
    /// Invalid varint7 (should be in -64..63 range).
    InvalidVarInt7(u8),
    /// Number of function body entries and signatures does not match.
    InconsistentCode,
    /// Only flags 0, 1, and 2 are accepted on segments.
    InvalidSegmentFlags(u32),
    /// Sum of counts of locals is greater than 2^32.
    TooManyLocals,
    /// Duplicated name subsections.
    DuplicatedNameSubsections(u8),
    /// Unknown name subsection type.
    UnknownNameSubsectionType(u8),
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            Error::UnexpectedEof => write!(f, "Unexpected end of input"),
            Error::InvalidMagic => write!(f, "Invalid magic number at start of file"),
            Error::UnsupportedVersion(v) => write!(f, "Unsupported wasm version {}", v),
            Error::InconsistentLength { expected, actual } => {
                write!(f, "Expected length {}, found {}", expected, actual)
            }
            Error::Other(msg) => write!(f, "{}", msg),
            Error::HeapOther(ref msg) => write!(f, "{}", msg),
            Error::UnknownValueType(ty) => write!(f, "Invalid or unknown value type {}", ty),
            Error::UnknownTableElementType(ty) => write!(f, "Unknown table element type {}", ty),
            Error::NonUtf8String => write!(f, "Non-UTF-8 string"),
            Error::UnknownExternalKind(kind) => write!(f, "Unknown external kind {}", kind),
            Error::UnknownInternalKind(kind) => write!(f, "Unknown internal kind {}", kind),
            Error::UnknownOpcode(opcode) => write!(f, "Unknown opcode {}", opcode),
            Error::UnknownSimdOpcode(opcode) => write!(f, "Unknown SIMD opcode {}", opcode),
            Error::InvalidVarUint1(val) => write!(f, "Not an unsigned 1-bit integer: {}", val),
            Error::InvalidVarInt7(val) => write!(f, "Not a signed 7-bit integer: {}", val),
            Error::InvalidVarInt32 => write!(f, "Not a signed 32-bit integer"),
            Error::InvalidVarUint32 => write!(f, "Not an unsigned 32-bit integer"),
            Error::InvalidVarInt64 => write!(f, "Not a signed 64-bit integer"),
            Error::InvalidVarUint64 => write!(f, "Not an unsigned 64-bit integer"),
            Error::InconsistentMetadata =>  write!(f, "Inconsistent metadata"),
            Error::InvalidSectionId(ref id) =>  write!(f, "Invalid section id: {}", id),
            Error::SectionsOutOfOrder =>  write!(f, "Sections out of order"),
            Error::DuplicatedSections(ref id) =>  write!(f, "Duplicated sections ({})", id),
            Error::InvalidMemoryReference(ref mem_ref) =>  write!(f, "Invalid memory reference ({})", mem_ref),
            Error::InvalidTableReference(ref table_ref) =>  write!(f, "Invalid table reference ({})", table_ref),
            Error::InvalidLimitsFlags(ref flags) =>  write!(f, "Invalid limits flags ({})", flags),
            Error::UnknownFunctionForm(ref form) =>  write!(f, "Unknown function form ({})", form),
            Error::InconsistentCode =>  write!(f, "Number of function body entries and signatures does not match"),
            Error::InvalidSegmentFlags(n) =>  write!(f, "Invalid segment flags: {}", n),
            Error::TooManyLocals => write!(f, "Too many locals"),
            Error::DuplicatedNameSubsections(n) =>  write!(f, "Duplicated name subsections: {}", n),
            Error::UnknownNameSubsectionType(n) => write!(f, "Unknown subsection type: {}", n),
        }
    }
}

#[cfg(feature = "std")]
impl ::std::error::Error for Error {
    fn description(&self) -> &str {
        match *self {
            Error::UnexpectedEof => "Unexpected end of input",
            Error::InvalidMagic => "Invalid magic number at start of file",
            Error::UnsupportedVersion(_) => "Unsupported wasm version",
            Error::InconsistentLength { .. } => "Inconsistent length",
            Error::Other(msg) => msg,
            Error::HeapOther(ref msg) => &msg[..],
            Error::UnknownValueType(_) => "Invalid or unknown value type",
            Error::UnknownTableElementType(_) => "Unknown table element type",
            Error::NonUtf8String => "Non-UTF-8 string",
            Error::UnknownExternalKind(_) => "Unknown external kind",
            Error::UnknownInternalKind(_) => "Unknown internal kind",
            Error::UnknownOpcode(_) => "Unknown opcode",
            Error::UnknownSimdOpcode(_) => "Unknown SIMD opcode",
            Error::InvalidVarUint1(_) => "Not an unsigned 1-bit integer",
            Error::InvalidVarInt32 => "Not a signed 32-bit integer",
            Error::InvalidVarInt7(_) => "Not a signed 7-bit integer",
            Error::InvalidVarUint32 => "Not an unsigned 32-bit integer",
            Error::InvalidVarInt64 => "Not a signed 64-bit integer",
            Error::InvalidVarUint64 => "Not an unsigned 64-bit integer",
            Error::InconsistentMetadata => "Inconsistent metadata",
            Error::InvalidSectionId(_) =>  "Invalid section id",
            Error::SectionsOutOfOrder =>  "Sections out of order",
            Error::DuplicatedSections(_) =>  "Duplicated section",
            Error::InvalidMemoryReference(_) =>  "Invalid memory reference",
            Error::InvalidTableReference(_) =>  "Invalid table reference",
            Error::InvalidLimitsFlags(_) => "Invalid limits flags",
            Error::UnknownFunctionForm(_) =>  "Unknown function form",
            Error::InconsistentCode =>  "Number of function body entries and signatures does not match",
            Error::InvalidSegmentFlags(_) =>  "Invalid segment flags",
            Error::TooManyLocals => "Too many locals",
            Error::DuplicatedNameSubsections(_) =>  "Duplicated name subsections",
            Error::UnknownNameSubsectionType(_) => "Unknown name subsections type",
        }
    }
}

impl From<cap9_core::Error> for Error {
    fn from(err: cap9_core::Error) -> Self {
        Error::HeapOther(format!("I/O Error: {:?}", err))
    }
}

/// Unparsed part of the module/section.
pub struct Unparsed(pub Vec<u8>);

impl WASMDeserialize for Unparsed {
    type Error = Error;

    fn deserialize<R: cap9_core::Read<u8>>(reader: &mut R) -> Result<Self, Self::Error> {
        let len = VarUint32::deserialize(reader)?.into();
        let mut vec = vec![0u8; len];
        reader.read(&mut vec[..])?;
        Ok(Unparsed(vec))
    }
}

impl From<Unparsed> for Vec<u8> {
    fn from(u: Unparsed) -> Vec<u8> {
        u.0
    }
}

/// Deserialize deserializable type from buffer.
pub fn deserialize_buffer<T: WASMDeserialize>(contents: &[u8]) -> Result<T, T::Error> {
    let mut reader = cap9_core::Cursor::new(contents);
    let result = T::deserialize(&mut reader)?;
    if reader.position() != contents.len() {
        // It's a TrailingData, since if there is not enough data then
        // UnexpectedEof must have been returned earlier in T::deserialize.
        return Err(cap9_core::Error::TrailingData.into())
    }
    Ok(result)
}
