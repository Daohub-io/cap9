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
