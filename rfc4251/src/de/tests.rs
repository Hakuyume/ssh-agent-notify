use super::*;
use serde::Deserialize;
use std::fmt::{self, Debug, Formatter};

fn check<'de, T>(data: &'de [u8], expected: T)
where
    T: Debug + PartialEq + Deserialize<'de>,
{
    assert_eq!(from_slice::<T>(data).unwrap(), expected);
}

#[test]
fn test_byte() {
    check::<u8>(&[42], 42);
}

#[test]
fn test_boolean() {
    check(&[0], false);
    check(&[1], true);
    check(&[2], true);
}

#[test]
fn test_uint32() {
    check::<u32>(&[0, 0, 7, 227], 2019);
}

#[test]
fn test_uint64() {
    check::<u64>(&[1, 2, 3, 4, 5, 6, 7, 8], 0x0102_0304_0506_0708);
}

#[test]
fn test_string_binary() {
    check::<&[u8]>(&[0, 0, 0, 3, b'f', b'o', b'o'], b"foo");
}

#[test]
fn test_string_text() {
    check(&[0, 0, 0, 3, b'f', b'o', b'o'], "foo");
}

#[test]
fn test_seq() {
    check::<&[u8]>(&[0, 0, 0, 4, 2, 0, 1, 9], &[2, 0, 1, 9]);
}

#[test]
fn test_tuple() {
    check::<(u8, &[u8])>(&[42, 0, 0, 0, 3, b'f', b'o', b'o'], (42, b"foo"));
}

#[test]
fn test_struct() {
    #[derive(Debug, Deserialize, PartialEq)]
    struct S<'a> {
        byte: u8,
        string: &'a [u8],
    }

    check(
        &[42, 0, 0, 0, 3, b'f', b'o', b'o'],
        S {
            byte: 42,
            string: b"foo",
        },
    );
}

#[test]
fn test_enum() {
    #[derive(Debug, PartialEq)]
    enum E<'a> {
        Foo(u8),
        Bar(&'a [u8]),
    }

    impl<'de> Deserialize<'de> for E<'de> {
        fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
        where
            D: de::Deserializer<'de>,
        {
            struct Visitor;

            impl<'de> de::Visitor<'de> for Visitor {
                type Value = E<'de>;
                fn expecting(&self, formatter: &mut Formatter) -> fmt::Result {
                    write!(formatter, "a enum tagged by u32")
                }

                fn visit_enum<A>(self, data: A) -> Result<Self::Value, A::Error>
                where
                    A: EnumAccess<'de>,
                {
                    let (tag, variant) = data.variant::<u32>()?;
                    match tag {
                        0 => Ok(E::Foo(variant.newtype_variant()?)),
                        1 => Ok(E::Bar(variant.newtype_variant()?)),
                        _ => unreachable!(),
                    }
                }
            }

            deserializer.deserialize_enum("E", &["Foo", "Bar"], Visitor)
        }
    }

    check(&[0, 0, 0, 5, 0, 0, 0, 0, 42], E::Foo(42));
    check(
        &[0, 0, 0, 11, 0, 0, 0, 1, 0, 0, 0, 3, b'b', b'a', b'r'],
        E::Bar(b"bar"),
    );
}

#[test]
#[should_panic(expected = "InsufficientData")]
fn test_insufficient_data() {
    check::<u8>(&[], 0);
}

#[test]
#[should_panic(expected = "RemainingData")]
fn test_remaining_data() {
    check::<u8>(&[0, 1], 0);
}
