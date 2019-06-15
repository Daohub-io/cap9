#[cfg(feature="std")]
use std::io;

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

    #[cfg(feature = "std")]
    IoError(io::Error),
}

/// IO specific Result.
pub type Result<T> = core::result::Result<T, Error>;

pub trait Write {
    /// Write a buffer of data into this write.
    ///
    /// All data is written at once.
    fn write(&mut self, buf: &[u8]) -> Result<()>;
}

pub trait Read {
    /// Read a data from this read to a buffer.
    ///
    /// If there is not enough data in this read then `UnexpectedEof` will be returned.
    fn read(&mut self, buf: &mut [u8]) -> Result<()>;
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

impl<T: AsRef<[u8]>> Read for Cursor<T> {
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

#[cfg(not(feature = "std"))]
impl Write for Vec<u8> {
    fn write(&mut self, buf: &[u8]) -> Result<()> {
        self.extend(buf);
        Ok(())
    }
}

#[cfg(feature = "std")]
impl<T: io::Read> Read for T {
    fn read(&mut self, buf: &mut [u8]) -> Result<()> {
        self.read_exact(buf)
            .map_err(Error::IoError)
    }
}

#[cfg(not(feature = "std"))]
impl Read for &[u8] {
    #[inline]
    fn read(&mut self, buf: &mut [u8]) -> Result<()> {
        let amt = core::cmp::min(buf.len(), self.len());
        let (a, b) = self.split_at(amt);

        // First check if the amount of bytes we want to read is small:
        // `copy_from_slice` will generally expand to a call to `memcpy`, and
        // for a single byte the overhead is significant.
        if amt == 1 {
            buf[0] = a[0];
        } else {
            buf[..amt].copy_from_slice(a);
        }

        *self = b;
        Ok(())
    }
}

#[cfg(feature = "std")]
impl<T: io::Write> Write for T {
    fn write(&mut self, buf: &[u8]) -> Result<()> {
        self.write_all(buf).map_err(Error::IoError)
    }
}
