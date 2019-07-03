use super::*;
use std::fmt::Debug;

fn check<'a, T>(data: &'a [u8], expected: T)
where
    T: Debug + PartialEq + Data<'a>,
{
    let mut reader = Reader::new(data);
    assert_eq!(reader.read::<T>().unwrap(), expected);
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
fn test_string() {
    check::<&[u8]>(&[0, 0, 0, 3, b'f', b'o', b'o'], b"foo");
}

#[test]
fn test_string_text() {
    check(&[0, 0, 0, 3, b'f', b'o', b'o'], "foo");
}

#[test]
#[should_panic(expected = "InsufficientData")]
fn test_insufficient_data() {
    check::<u8>(b"", 0);
}
