use super::*;
use serde::Deserialize;
use std::fmt::Debug;

fn check<'de, T>(data: &'de [u8], expected: T)
where
    T: Debug + PartialEq + Deserialize<'de>,
{
    let mut deserializer = Deserializer::new(data);
    assert_eq!(T::deserialize(&mut deserializer).unwrap(), expected);
    assert_eq!(deserializer.is_empty(), true);
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
    #[derive(Debug, Deserialize, PartialEq)]
    enum E<'a> {
        Foo(u8),
        Bar(&'a [u8]),
    }

    check(&[0, 42], E::Foo(42));
    check(&[1, 0, 0, 0, 3, b'b', b'a', b'r'], E::Bar(b"bar"));
}

#[test]
#[should_panic(expected = "InsufficientData")]
fn test_insufficient_data() {
    check::<u8>(&[], 0);
}
