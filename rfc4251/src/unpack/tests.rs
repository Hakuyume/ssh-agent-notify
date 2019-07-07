use super::*;
use std::fmt::Debug;

fn check<'a, T>(data: &'a [u8], expected: T)
where
    T: Debug + PartialEq + Unpack<'a>,
    T::Error: Debug,
{
    assert_eq!(Unpacker::new(data).unpack::<T>().unwrap(), expected);
}

#[test]
fn test_byte() {
    check(&[42], 42_u8);
}

#[test]
fn test_boolean() {
    check(&[0], false);
    check(&[1], true);
    check(&[2], true);
}

#[test]
fn test_uint32() {
    check(&[0, 0, 7, 227], 2019_u32);
}

#[test]
fn test_uint64() {
    check(&[1, 2, 3, 4, 5, 6, 7, 8], 0x0102_0304_0506_0708_u64);
}

#[test]
fn test_string() {
    check(&[0, 0, 0, 3, b'f', b'o', b'o'], b"foo".as_ref());
    check(&[0, 0, 0, 3, b'f', b'o', b'o'], "foo");
}

#[test]
fn test_mpint() {
    check(&[0, 0, 0, 0], BigInt::from(0x0));
    check(
        &[0, 0, 0, 8, 0x09, 0xa3, 0x78, 0xf9, 0xb2, 0xe3, 0x32, 0xa7],
        BigInt::from(0x9a3_78f9_b2e3_32a7_i64),
    );
    check(&[0, 0, 0, 2, 0x00, 0x80], BigInt::from(0x80));
    check(&[0, 0, 0, 2, 0xed, 0xcc], BigInt::from(-0x1234));
    check(
        &[0, 0, 0, 5, 0xff, 0x21, 0x52, 0x41, 0x11],
        BigInt::from(-0xdead_beef_i64),
    )
}

#[test]
#[should_panic(expected = "InsufficientData")]
fn test_insufficient_data() {
    check(b"", 0_u8);
}
