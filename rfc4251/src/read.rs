use crate::Error;
use std::convert::TryInto;
use std::mem;

pub struct Reader<'a>(&'a [u8]);

impl<'a> Reader<'a> {
    pub fn new(data: &'a [u8]) -> Self {
        Self(data)
    }

    pub fn read<T>(&mut self) -> Result<T, Error>
    where
        T: Data<'a>,
    {
        T::read(self)
    }

    fn read_bytes(&mut self, len: usize) -> Result<&'a [u8], Error> {
        let bytes = self.0.get(..len).ok_or(Error::InsufficientData)?;
        self.0 = &self.0[len..];
        Ok(bytes)
    }
}

pub trait Data<'a>: Sized {
    fn read(reader: &mut Reader<'a>) -> Result<Self, Error>;
}

macro_rules! impl_uint {
    ($t:ty) => {
        impl Data<'_> for $t {
            fn read(reader: &mut Reader<'_>) -> Result<Self, Error> {
                Ok(Self::from_be_bytes(
                    reader.read_bytes(mem::size_of::<$t>())?.try_into().unwrap(),
                ))
            }
        }
    };
}

impl_uint!(u8);
impl_uint!(u32);
impl_uint!(u64);

impl Data<'_> for bool {
    fn read(reader: &mut Reader<'_>) -> Result<Self, Error> {
        Ok(reader.read::<u8>()? != 0)
    }
}

impl<'a> Data<'a> for &'a [u8] {
    fn read(reader: &mut Reader<'a>) -> Result<Self, Error> {
        let len = reader.read::<u32>()?;
        reader.read_bytes(len as _)
    }
}

impl<'a> Data<'a> for &'a str {
    fn read(reader: &mut Reader<'a>) -> Result<Self, Error> {
        Ok(std::str::from_utf8(reader.read()?)?)
    }
}

#[cfg(test)]
mod tests;
