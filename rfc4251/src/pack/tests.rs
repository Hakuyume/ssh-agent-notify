use super::*;
use std::fmt::Debug;

fn check<'a, T>(value: T, expected: &[u8])
where
    T: Pack,
    T::Error: Debug,
{
    let mut packer = Packer::new();
    packer.pack(value).unwrap();
    assert_eq!(packer.inner(), expected);
}

#[test]
fn test_byte() {
    check(42_u8, &[42]);
}

#[test]
fn test_boolean() {
    check(false, &[0]);
    check(true, &[1]);
}

#[test]
fn test_uint32() {
    check(2019_u32, &[0, 0, 7, 227]);
}

#[test]
fn test_uint64() {
    check(0x0102_0304_0506_0708_u64, &[1, 2, 3, 4, 5, 6, 7, 8]);
}

#[test]
fn test_string() {
    check(b"foo".as_ref(), &[0, 0, 0, 3, b'f', b'o', b'o']);
    check("foo", &[0, 0, 0, 3, b'f', b'o', b'o']);
}

#[test]
fn test_mpint() {
    check(&BigInt::from(0x0), &[0, 0, 0, 0]);
    check(
        &BigInt::from(0x9a3_78f9_b2e3_32a7_i64),
        &[0, 0, 0, 8, 0x09, 0xa3, 0x78, 0xf9, 0xb2, 0xe3, 0x32, 0xa7],
    );
    check(&BigInt::from(0x80), &[0, 0, 0, 2, 0x00, 0x80]);
    check(&BigInt::from(-0x1234), &[0, 0, 0, 2, 0xed, 0xcc]);
    check(
        &BigInt::from(-0xdead_beef_i64),
        &[0, 0, 0, 5, 0xff, 0x21, 0x52, 0x41, 0x11],
    )
}
