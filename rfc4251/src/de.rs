use crate::Error;
use serde::de::{
    self, Deserialize, DeserializeSeed, Deserializer as _, EnumAccess, SeqAccess, VariantAccess,
    Visitor,
};
use std::convert::TryInto;
use std::mem;

pub struct Deserializer<'de>(&'de [u8]);

impl<'de> Deserializer<'de> {
    pub fn new(data: &'de [u8]) -> Self {
        Self(data)
    }
}

pub fn from_slice<'de, T>(data: &'de [u8]) -> Result<T, Error>
    where T: Deserialize<'de>
{
    let mut deserializer = Deserializer::new(data);
    T::deserialize(&mut deserializer)
}

macro_rules! impl_uint {
    ($t:ty, $f:ident, $v:ident) => {
        fn $f<V>(self, visitor: V) -> Result<V::Value, Self::Error>
        where
            V: Visitor<'de>,
        {
            let v = visitor.$v(<$t>::from_be_bytes(
                self.0
                    .get(..mem::size_of::<$t>())
                    .ok_or(Error::InsufficientData)?
                    .try_into()
                    .unwrap(),
            ))?;
            self.0 = &self.0[mem::size_of::<$t>()..];
            Ok(v)
        }
    };
}

macro_rules! impl_not_supported {
    ($f:ident$(, $t:ty)*) => {
        fn $f<V>(self$(, _: $t)*, _: V) -> Result<V::Value, Self::Error>
        where
            V: Visitor<'de>,
        {
            Err(Error::NotSupported)
        }
    };
}

impl<'de> de::Deserializer<'de> for &mut Deserializer<'de> {
    type Error = Error;

    impl_not_supported!(deserialize_any);

    fn deserialize_bool<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        let v = visitor.visit_bool(*self.0.get(0).ok_or(Error::InsufficientData)? != 0)?;
        self.0 = &self.0[1..];
        Ok(v)
    }

    impl_not_supported!(deserialize_i8);
    impl_not_supported!(deserialize_i16);
    impl_not_supported!(deserialize_i32);
    impl_not_supported!(deserialize_i64);

    impl_uint!(u8, deserialize_u8, visit_u8);
    impl_not_supported!(deserialize_u16);
    impl_uint!(u32, deserialize_u32, visit_u32);
    impl_uint!(u64, deserialize_u64, visit_u64);

    impl_not_supported!(deserialize_f32);
    impl_not_supported!(deserialize_f64);
    impl_not_supported!(deserialize_char);

    fn deserialize_str<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        self.deserialize_bytes(visitor)
    }

    fn deserialize_string<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        self.deserialize_bytes(visitor)
    }

    fn deserialize_bytes<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        let len = u32::deserialize(&mut *self)? as usize;
        let v = visitor.visit_borrowed_bytes(self.0.get(..len).ok_or(Error::InsufficientData)?)?;
        self.0 = &self.0[len..];
        Ok(v)
    }

    fn deserialize_byte_buf<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        self.deserialize_bytes(visitor)
    }

    impl_not_supported!(deserialize_option);
    impl_not_supported!(deserialize_unit);
    impl_not_supported!(deserialize_unit_struct, &'static str);
    impl_not_supported!(deserialize_newtype_struct, &'static str);

    fn deserialize_seq<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        let len = u32::deserialize(&mut *self)? as usize;
        self.deserialize_tuple(len, visitor)
    }

    fn deserialize_tuple<V>(self, len: usize, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        struct Access<'a, 'de>(&'a mut Deserializer<'de>, usize);

        impl<'a, 'de> SeqAccess<'de> for Access<'a, 'de> {
            type Error = Error;

            fn next_element_seed<T>(&mut self, seed: T) -> Result<Option<T::Value>, Self::Error>
            where
                T: DeserializeSeed<'de>,
            {
                if self.1 > 0 {
                    let v = seed.deserialize(&mut *self.0)?;
                    self.1 -= 1;
                    Ok(Some(v))
                } else {
                    Ok(None)
                }
            }
        }

        visitor.visit_seq(Access(self, len))
    }

    fn deserialize_tuple_struct<V>(
        self,
        _name: &'static str,
        len: usize,
        visitor: V,
    ) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        self.deserialize_tuple(len, visitor)
    }

    impl_not_supported!(deserialize_map);

    fn deserialize_struct<V>(
        self,
        _name: &'static str,
        fields: &'static [&'static str],
        visitor: V,
    ) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        self.deserialize_tuple(fields.len(), visitor)
    }

    fn deserialize_enum<V>(
        self,
        _name: &'static str,
        _variants: &'static [&'static str],
        visitor: V,
    ) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        let mut deserializer = Deserializer(<&'de [u8]>::deserialize(self)?);
        visitor.visit_enum(&mut deserializer)
    }

    impl_not_supported!(deserialize_identifier);
    impl_not_supported!(deserialize_ignored_any);
}

impl<'de> VariantAccess<'de> for &mut Deserializer<'de> {
    type Error = Error;

    fn unit_variant(self) -> Result<(), Self::Error> {
        Ok(())
    }

    fn newtype_variant_seed<T>(self, seed: T) -> Result<T::Value, Self::Error>
    where
        T: DeserializeSeed<'de>,
    {
        seed.deserialize(self)
    }

    fn tuple_variant<V>(self, len: usize, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        self.deserialize_tuple(len, visitor)
    }

    fn struct_variant<V>(
        self,
        fields: &'static [&'static str],
        visitor: V,
    ) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        self.deserialize_struct("", fields, visitor)
    }
}

impl<'de> EnumAccess<'de> for &mut Deserializer<'de> {
    type Error = Error;
    type Variant = Self;

    fn variant_seed<V>(self, seed: V) -> Result<(V::Value, Self::Variant), Self::Error>
    where
        V: DeserializeSeed<'de>,
    {
        Ok((seed.deserialize(&mut *self)?, self))
    }
}

#[cfg(test)]
mod tests;
