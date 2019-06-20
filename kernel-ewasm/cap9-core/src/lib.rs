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

/// IO specific Result.
pub type Result<T> = core::result::Result<T, Error>;

pub trait Write {
    /// Write a buffer of data into this write.
    ///
    /// All data is written at once.
    fn write(&mut self, buf: &[u8]) -> Result<()>;
}

pub trait Read<T> {
    /// Read a data from this read to a buffer.
    ///
    /// If there is not enough data in this read then `UnexpectedEof` will be returned.
    fn read(&mut self, buf: &mut [T]) -> Result<()>;
}

/// Reader that saves the last position.
pub struct Cursor<T> {
    inner: T,
    pos: usize,
}

impl<T> Cursor<T> {
    pub fn new(inner: T) -> Cursor<T> {
        Cursor {
            inner,
            pos: 0,
        }
    }

    pub fn position(&self) -> usize {
        self.pos
    }
}

impl<T: AsRef<[u8]>> Read<u8> for Cursor<T> {
    fn read(&mut self, buf: &mut [u8]) -> Result<()> {
        let slice = self.inner.as_ref();
        let remainder = slice.len() - self.pos;
        let requested = buf.len();
        if requested > remainder {
            return Err(Error::UnexpectedEof);
        }
        buf.copy_from_slice(&slice[self.pos..(self.pos + requested)]);
        self.pos += requested;
        Ok(())
    }
}

impl<T: AsRef<[U256]>> Read<U256> for Cursor<T> {
    fn read(&mut self, buf: &mut [U256]) -> Result<()> {
        let slice = self.inner.as_ref();
        let remainder = slice.len() - self.pos;
        let requested = buf.len();
        if requested > remainder {
            return Err(Error::UnexpectedEof);
        }
        buf.copy_from_slice(&slice[self.pos..(self.pos + requested)]);
        self.pos += requested;
        Ok(())
    }
}

impl Write for Vec<u8> {
    fn write(&mut self, buf: &[u8]) -> Result<()> {
        self.extend(buf);
        Ok(())
    }
}
