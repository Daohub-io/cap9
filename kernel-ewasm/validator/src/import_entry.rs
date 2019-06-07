use pwasm_std;
use pwasm_std::String;

use crate::io;
use crate::{Deserialize, Uint8, VarUint32, VarUint1, VarUint7};
use crate::types::{TableElementType, ValueType};
use crate::serialization::{Error};

const FLAG_HAS_MAX: u8 = 0x01;
const FLAG_SHARED: u8 = 0x02;

/// Global definition struct
#[derive(Debug, Copy, Clone, PartialEq)]
pub struct GlobalType {
    content_type: ValueType,
    is_mutable: bool,
}

impl GlobalType {
    /// New global type
    pub fn new(content_type: ValueType, is_mutable: bool) -> Self {
        GlobalType {
            content_type: content_type,
            is_mutable: is_mutable,
        }
    }

    /// Type of the global entry
    pub fn content_type(&self) -> ValueType { self.content_type }

    /// Is global entry is declared as mutable
    pub fn is_mutable(&self) -> bool { self.is_mutable }
}

impl Deserialize for GlobalType {
    type Error = Error;

    fn deserialize<R: io::Read>(reader: &mut R) -> Result<Self, Self::Error> {
        let content_type = ValueType::deserialize(reader)?;
        let is_mutable = VarUint1::deserialize(reader)?;
        Ok(GlobalType {
            content_type: content_type,
            is_mutable: is_mutable.into(),
        })
    }
}

/// Table entry
#[derive(Debug, Copy, Clone, PartialEq)]
pub struct TableType {
    elem_type: TableElementType,
    limits: ResizableLimits,
}

impl TableType {
    /// New table definition
    pub fn new(min: u32, max: Option<u32>) -> Self {
        TableType {
            elem_type: TableElementType::AnyFunc,
            limits: ResizableLimits::new(min, max),
        }
    }

    /// Table memory specification
    pub fn limits(&self) -> &ResizableLimits { &self.limits }

    /// Table element type
    pub fn elem_type(&self) -> TableElementType { self.elem_type }
}

impl Deserialize for TableType {
    type Error = Error;

    fn deserialize<R: io::Read>(reader: &mut R) -> Result<Self, Self::Error> {
        let elem_type = TableElementType::deserialize(reader)?;
        let limits = ResizableLimits::deserialize(reader)?;
        Ok(TableType {
            elem_type: elem_type,
            limits: limits,
        })
    }
}

/// Memory and table limits.
#[derive(Debug, Copy, Clone, PartialEq)]
pub struct ResizableLimits {
    initial: u32,
    maximum: Option<u32>,
    shared: bool,
}

impl ResizableLimits {
    /// New memory limits definition.
    pub fn new(min: u32, max: Option<u32>) -> Self {
        ResizableLimits {
            initial: min,
            maximum: max,
            shared: false,
        }
    }
    /// Initial size.
    pub fn initial(&self) -> u32 { self.initial }
    /// Maximum size.
    pub fn maximum(&self) -> Option<u32> { self.maximum }
    /// Whether or not this is a shared array buffer.
    pub fn shared(&self) -> bool { self.shared }
}

impl Deserialize for ResizableLimits {
    type Error = Error;

    fn deserialize<R: io::Read>(reader: &mut R) -> Result<Self, Self::Error> {
        let flags: u8 = Uint8::deserialize(reader)?.into();
        match flags {
            0x00 | 0x01 | 0x03 => {},
            _ => return Err(Error::InvalidLimitsFlags(flags)),
        }

        let initial = VarUint32::deserialize(reader)?;
        let maximum = if flags & FLAG_HAS_MAX != 0 {
            Some(VarUint32::deserialize(reader)?.into())
        } else {
            None
        };
        let shared = flags & FLAG_SHARED != 0;

        Ok(ResizableLimits {
            initial: initial.into(),
            maximum: maximum,
            shared,
        })
    }
}

/// Memory entry.
#[derive(Debug, Copy, Clone, PartialEq)]
pub struct MemoryType(ResizableLimits);

impl MemoryType {
    /// New memory definition
    pub fn new(min: u32, max: Option<u32>, shared: bool) -> Self {
        let mut r = ResizableLimits::new(min, max);
        r.shared = shared;
        MemoryType(r)
    }

    /// Limits of the memory entry.
    pub fn limits(&self) -> &ResizableLimits {
        &self.0
    }
}

impl Deserialize for MemoryType {
    type Error = Error;

    fn deserialize<R: io::Read>(reader: &mut R) -> Result<Self, Self::Error> {
        Ok(MemoryType(ResizableLimits::deserialize(reader)?))
    }
}

/// External to local binding.
#[derive(Debug, Copy, Clone, PartialEq)]
pub enum External {
    /// Binds to a function whose type is associated with the given index in the
    /// type section.
    Function(u32),
    /// Describes local table definition to be imported as.
    Table(TableType),
    /// Describes local memory definition to be imported as.
    Memory(MemoryType),
    /// Describes local global entry to be imported as.
    Global(GlobalType),
}

impl Deserialize for External {
    type Error = Error;

    fn deserialize<R: io::Read>(reader: &mut R) -> Result<Self, Self::Error> {
        let kind = VarUint7::deserialize(reader)?;
        match kind.into() {
            0x00 => Ok(External::Function(VarUint32::deserialize(reader)?.into())),
            0x01 => Ok(External::Table(TableType::deserialize(reader)?)),
            0x02 => Ok(External::Memory(MemoryType::deserialize(reader)?)),
            0x03 => Ok(External::Global(GlobalType::deserialize(reader)?)),
            _ => Err(Error::UnknownExternalKind(kind.into())),
        }
    }
}

/// Import entry.
#[derive(Debug, Clone, PartialEq)]
pub struct ImportEntry {
    module_str: String,
    field_str: String,
    external: External,
}

impl ImportEntry {
    /// New import entry.
    pub fn new(module_str: String, field_str: String, external: External) -> Self {
        ImportEntry {
            module_str: module_str,
            field_str: field_str,
            external: external,
        }
    }

    /// Module reference of the import entry.
    pub fn module(&self) -> &str { &self.module_str }

    /// Module reference of the import entry (mutable).
    pub fn module_mut(&mut self) -> &mut String {
        &mut self.module_str
    }

    /// Field reference of the import entry.
    pub fn field(&self) -> &str { &self.field_str }

    /// Field reference of the import entry (mutable)
    pub fn field_mut(&mut self) -> &mut String {
        &mut self.field_str
    }

    /// Local binidng of the import entry.
    pub fn external(&self) -> &External { &self.external }

    /// Local binidng of the import entry (mutable)
    pub fn external_mut(&mut self) -> &mut External { &mut self.external }
}

impl Deserialize for ImportEntry {
    type Error = Error;

    fn deserialize<R: io::Read>(reader: &mut R) -> Result<Self, Self::Error> {
        let module_str = String::deserialize(reader)?;
        let field_str = String::deserialize(reader)?;
        let external = External::deserialize(reader)?;

        Ok(ImportEntry {
            module_str: module_str,
            field_str: field_str,
            external: external,
        })
    }
}
