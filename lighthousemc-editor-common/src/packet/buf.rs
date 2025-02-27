use crate::packet::{
    PacketEncode,
    PrefixedPacketEncode,
    PacketDecode,
    DecodeError
};
use std::{ fmt, iter, slice };
use std::borrow::Cow;


#[derive(Clone, Hash, Eq, PartialEq)]
pub struct PacketBuf<'l> {
    inner    : Cow<'l, [u8]>,
    read_idx : usize
}


/// Constructors
impl<'l> PacketBuf<'l> {

    pub fn new() -> Self {
        PacketBuf {
            inner    : Cow::Borrowed(&[]),
            read_idx : 0,
        }
    }

    pub fn of_encode<T : PacketEncode>(encodable : T) -> Self {
        let mut buf = Self::new();
        encodable.encode(&mut buf);
        buf
    }

    pub fn of_encode_prefixed<T : PrefixedPacketEncode>(encodable : T) -> Self {
        let mut buf = Self::new();
        encodable.encode_prefixed(&mut buf);
        buf
    }

}
impl<'l> From<&'l [u8]> for PacketBuf<'l> {
    fn from(value : &'l [u8]) -> Self { Self {
        inner    : Cow::Borrowed(value),
        read_idx : 0
    } }
}
impl<'l> From<Vec<u8>> for PacketBuf<'l> {
    fn from(value : Vec<u8>) -> Self { Self {
        inner    : Cow::Owned(value),
        read_idx : 0
    } }
}


/// Deconstructors
impl<'l> PacketBuf<'l> {

    pub fn to_vec(self) -> Vec<u8> {
        self.inner.into_owned()
    }

    pub fn as_slice(&self) -> &[u8] {
        (&*self.inner).get(self.read_idx..).unwrap_or(&[])
    }

}


/// Basic Operations
impl<'l> PacketBuf<'l> {

    #[track_caller]
    pub fn seek(&mut self, idx : usize) {
        assert!(idx <= self.inner.len(), "Seek index exceeded packet size");
        self.read_idx = idx;
    }

    #[track_caller]
    pub fn skip(&mut self, count : usize) {
        self.seek(self.read_idx + count);
    }

    pub fn remaining(&self) -> usize {
        self.inner.len() - self.read_idx
    }

    pub fn write_u8(&mut self, byte : u8) -> () {
        match (&mut self.inner) {
            Cow::Owned(inner) => inner.push(byte),
            Cow::Borrowed(inner) => {
                let mut inner = inner.to_vec();
                inner.push(byte);
                self.inner = Cow::Owned(inner);
            }
        }
    }

    pub fn write_u8s(&mut self, data : &[u8]) -> () {
        match (&mut self.inner) {
            Cow::Owned(inner) => inner.extend_from_slice(data),
            Cow::Borrowed(inner) => {
                let mut inner = inner.to_vec();
                inner.extend_from_slice(data);
                self.inner = Cow::Owned(inner);
            }
        }
    }

    pub fn read_u8(&mut self) -> Result<u8, DecodeError> {
        let byte = self.inner
            .get(self.read_idx).ok_or(DecodeError::EndOfBuffer)?;
        self.read_idx += 1;
        Ok(*byte)
    }

    pub fn read_u8s_const<const BYTES : usize>(&mut self) -> Result<[u8; BYTES], DecodeError> {
        if (BYTES == 0) { return Ok([0; BYTES]); }
        let out = self.inner.iter().skip(self.read_idx).cloned().array_chunks()
            .next().ok_or(DecodeError::EndOfBuffer)?;
        self.read_idx += BYTES;
        Ok(out)
    }

    pub fn read_u8s(&mut self, bytes : usize) -> Result<Vec<u8>, DecodeError> {
        if (bytes == 0) { return Ok(Vec::new()) }
        let mut out  = Vec::with_capacity(bytes);
        let mut data = self.inner.iter().skip(self.read_idx);
        for _ in 0..bytes {
            out.push(*data.next().ok_or(DecodeError::EndOfBuffer)?);
        }
        self.read_idx += bytes;
        Ok(out)
    }

}


impl<'l, 'k : 'l> IntoIterator for &'k PacketBuf<'l> {
    type Item     = u8;
    type IntoIter = iter::Cloned<iter::Skip<slice::Iter<'l, u8>>>;
    fn into_iter(self) -> Self::IntoIter {
        self.inner.iter().skip(self.read_idx).cloned()
    }
}
/// Iterator
impl<'l> PacketBuf<'l> {
    pub fn iter(&self) -> impl Iterator<Item = u8> { (&self).into_iter() }
}


/// Encode & Decode
impl<'l> PacketBuf<'l> {

    pub fn encode_write<T : PacketEncode>(&mut self, encodable : T) -> () {
        encodable.encode(self);
    }

    pub fn read_decode<T : PacketDecode>(&mut self) -> Result<T, DecodeError> {
        T::decode(self)
    }

}


impl<'l> fmt::Debug for PacketBuf<'l> {
    fn fmt(&self, f : &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "PacketBuf(0x")?;
        for byte in self.inner.iter().skip(self.read_idx) {
            write!(f, "{:X}", byte)?;
        }
        write!(f, ")")
    }
}
