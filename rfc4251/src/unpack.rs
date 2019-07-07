use crate::Error;
use num_bigint::BigInt;
use std::convert::TryInto;
use std::mem;

pub struct Unpacker<'a>(&'a [u8]);

impl<'a> Unpacker<'a> {
    pub fn new(data: &'a [u8]) -> Self {
        Self(data)
    }

    pub fn unpack<T>(&mut self) -> Result<T, T::Error>
    where
        T: Unpack<'a>,
    {
        T::unpack(self)
    }

    fn read_bytes(&mut self, len: usize) -> Result<&'a [u8], Error> {
        let bytes = self.0.get(..len).ok_or(Error::InsufficientData)?;
        self.0 = &self.0[len..];
        Ok(bytes)
    }
}

pub trait Unpack<'a>: Sized {
    type Error;
    fn unpack(unpacker: &mut Unpacker<'a>) -> Result<Self, Self::Error>;
}

macro_rules! impl_uint {
    ($t:ty) => {
        impl Unpack<'_> for $t {
            type Error = Error;

            fn unpack(unpacker: &mut Unpacker<'_>) -> Result<Self, Self::Error> {
                Ok(Self::from_be_bytes(
                    unpacker
                        .read_bytes(mem::size_of::<$t>())?
                        .try_into()
                        .unwrap(),
                ))
            }
        }
    };
}

impl_uint!(u8);
impl_uint!(u32);
impl_uint!(u64);

impl Unpack<'_> for bool {
    type Error = Error;

    fn unpack(unpacker: &mut Unpacker<'_>) -> Result<Self, Self::Error> {
        Ok(unpacker.unpack::<u8>()? != 0)
    }
}

impl<'a> Unpack<'a> for &'a [u8] {
    type Error = Error;

    fn unpack(unpacker: &mut Unpacker<'a>) -> Result<Self, Self::Error> {
        let len = unpacker.unpack::<u32>()?;
        Ok(unpacker.read_bytes(len as _)?)
    }
}

impl<'a> Unpack<'a> for &'a str {
    type Error = Error;

    fn unpack(unpacker: &mut Unpacker<'a>) -> Result<Self, Self::Error> {
        Ok(std::str::from_utf8(unpacker.unpack()?)?)
    }
}

impl Unpack<'_> for BigInt {
    type Error = Error;

    fn unpack(unpacker: &mut Unpacker<'_>) -> Result<Self, Self::Error> {
        Ok(BigInt::from_signed_bytes_be(unpacker.unpack()?))
    }
}

#[cfg(test)]
mod tests;
