//! # Cap9 Core
//!
//! This crate contains some of the base types and mechanism used throughout the
//! kernel and contracts. Most critically at this stage it contains the Cursor
//! and Serialization types.
//!

#![no_std]
pub use pwasm_std::types::{U256,H256, Address};

#[cfg(not(feature = "std"))]
use pwasm_std::vec::Vec;

/// IO specific error.
#[derive(Debug)]
pub enum Error {
    /// Some unexpected data left in the buffer after reading all data.
    TrailingData,

    /// Unexpected End-Of-File
    UnexpectedEof,

    /// Invalid data is encountered.
    InvalidData,
}


pub trait Write {
    /// Write a buffer of data into this write.
    ///
    /// All data is written at once.
    fn write(&mut self, buf: &[u8]) -> Result<(), Error>;
}

pub trait Read<T> {
    /// Read a data from this read to a buffer.
    ///
    /// If there is not enough data in this read then `UnexpectedEof` will be returned.
    fn read(&mut self, buf: &mut [T]) -> Result<(), Error>;
}


// Seek does not seem to be implemented in core, so we'll reimplement what we
// need.
#[derive(Debug)]
pub struct Cursor<'a, T> {
    pub current_offset: usize,
    pub body: &'a [T],
}

impl<'a, T> Cursor<'a, T> {

    pub fn new(body: &'a [T]) -> Cursor<'a, T> {
        Cursor {
            body,
            current_offset: 0,
        }
    }

    // Read the byte at the cusor, and increment the pointer by 1.
    pub fn read_ref(&mut self) -> Option<&'a T> {
        if self.current_offset < self.body.len() {
            let val = &self.body[self.current_offset];
            self.current_offset += 1;
            Some(val)
        } else {
            None
        }
    }

    pub fn read_ref_n(&mut self, n: usize) -> &'a [T] {
        let val = &self.body[self.current_offset..(self.current_offset + n)];
        self.current_offset += n;
        val
    }

    pub fn skip(&mut self, n: usize) {
        self.current_offset += n;
    }

    pub fn remaining(&self) -> usize {
        self.body.len() - self.current_offset
    }

    pub fn len(&self) -> usize {
        self.body.len()
    }

    pub fn position(&self) -> usize {
        self.current_offset
    }

    pub fn inner(&self) -> &'a [T] {
        self.body
    }
}

/// Implement standard read definition (which clones). This is basically the
/// rust definition of read for slice.
impl<'a, T: Copy> Read<T> for Cursor<'a, T> {
    fn read(&mut self, buf: &mut [T]) -> Result<(), Error> {
        if self.remaining() < buf.len() {
            return Err(Error::UnexpectedEof);
        }
        let actual_self = &self.body[self.current_offset..];
        let amt = core::cmp::min(buf.len(), actual_self.len());
        let (a, _) = actual_self.split_at(amt);

        if amt == 1 {
            buf[0] = a[0];
        } else {
            buf[..amt].copy_from_slice(a);
        }

        self.current_offset += amt;
        Ok(())
    }
}

impl Write for Vec<u8> {
    fn write(&mut self, buf: &[u8]) -> Result<(), Error> {
        self.extend(buf);
        Ok(())
    }
}



/// Deserialization from serial i/o.
pub trait Deserialize<T> : Sized {
    /// Serialization error produced by deserialization routine.
    type Error: From<Error>;
    /// Deserialize type from serial i/o
    fn deserialize<R: Read<T>>(reader: &mut R) -> Result<Self, Self::Error>;
}

/// Serialization to serial i/o. Takes self by value to consume less memory
/// (parity-wasm IR is being partially freed by filling the result buffer).
pub trait Serialize {
    /// Serialization error produced by serialization routine.
    type Error: From<Error>;
    /// Serialize type to serial i/o
    fn serialize<W: Write>(self, writer: &mut W) -> Result<(), Self::Error>;
}

impl Deserialize<u8> for u8 {
    type Error = Error;

    fn deserialize<R: Read<u8>>(reader: &mut R) -> Result<Self, Self::Error> {
        let mut u8buf = [0u8; 1];
        reader.read(&mut u8buf)?;
        Ok(u8buf[0])
    }
}

impl Deserialize<U256> for u8 {
    type Error = Error;

    fn deserialize<R: Read<U256>>(reader: &mut R) -> Result<Self, Self::Error> {
        let mut buf = [U256::zero(); 1];
        reader.read(&mut buf)?;
        Ok(buf[0].as_u32() as u8)
    }
}

impl Serialize for u8 {
    type Error = Error;

    fn serialize<W: Write>(self, writer: &mut W) -> Result<(), Self::Error> {
        writer.write(&[self])?;
        Ok(())
    }
}

impl Deserialize<u8> for U256 {
    type Error = Error;

    fn deserialize<R: Read<u8>>(reader: &mut R) -> Result<Self, Self::Error> {
        let mut u8buf = [0u8; 32];
        // TODO: check that enough bytes were read
        reader.read(&mut u8buf)?;
        Ok(u8buf.into())
    }
}


impl Serialize for U256 {
    type Error = Error;

    fn serialize<W: Write>(self, writer: &mut W) -> Result<(), Self::Error> {
        let mut bytes: Vec<u8> = Vec::new();
        bytes.resize(32,0);
        self.to_big_endian(bytes.as_mut_slice());
        writer.write(&bytes)?;
        Ok(())
    }
}


impl Deserialize<u8> for H256 {
    type Error = Error;

    fn deserialize<R: Read<u8>>(reader: &mut R) -> Result<Self, Self::Error> {
        let mut u8buf = [0u8; 32];
        reader.read(&mut u8buf)?;
        Ok(u8buf.into())
    }
}


impl Serialize for H256 {
    type Error = Error;

    fn serialize<W: Write>(self, writer: &mut W) -> Result<(), Self::Error> {
        let bytes = self.to_fixed_bytes();
        writer.write(&bytes)?;
        Ok(())
    }
}


impl Deserialize<u8> for Address {
    type Error = Error;

    fn deserialize<R: Read<u8>>(reader: &mut R) -> Result<Self, Self::Error> {
        let mut u8buf = [0u8; 32];
        // TODO: check that enough bytes were read
        reader.read(&mut u8buf)?;
        let h: H256 = u8buf.into();
        Ok(h.into())
    }
}

impl Serialize for Address {
    type Error = Error;

    fn serialize<W: Write>(self, writer: &mut W) -> Result<(), Self::Error> {
        let h: H256 = self.into();
        writer.write(&h.to_fixed_bytes())?;
        Ok(())
    }
}
