use super::*;
use serde::Deserialize;

#[test]
fn test_byte() {
    assert_eq!(from_slice::<u8>(&[42]).unwrap(), 42);
}

#[test]
fn test_boolean() {
    assert_eq!(from_slice::<bool>(&[0]).unwrap(), false);
    assert_eq!(from_slice::<bool>(&[1]).unwrap(), true);
    assert_eq!(from_slice::<bool>(&[2]).unwrap(), true);
}

#[test]
fn test_uint32() {
    assert_eq!(from_slice::<u32>(&[0, 0, 7, 227]).unwrap(), 2019);
}

#[test]
fn test_uint64() {
    assert_eq!(
        from_slice::<u64>(&[1, 2, 3, 4, 5, 6, 7, 8]).unwrap(),
        0x0102_0304_0506_0708
    );
}

#[test]
fn test_string_binary() {
    assert_eq!(
        from_slice::<&[u8]>(&[0, 0, 0, 3, b'f', b'o', b'o']).unwrap(),
        b"foo"
    );
}

#[test]
fn test_string_text() {
    assert_eq!(
        from_slice::<&str>(&[0, 0, 0, 3, b'f', b'o', b'o']).unwrap(),
        "foo"
    );
}

#[test]
fn test_seq() {
    assert_eq!(
        from_slice::<Vec<&[u8]>>(&[
            0, 0, 0, 2, 0, 0, 0, 3, b'f', b'o', b'o', 0, 0, 0, 3, b'b', b'a', b'r'
        ])
        .unwrap(),
        &[&b"foo"[..], b"bar"]
    );
}

#[test]
fn test_tuple() {
    assert_eq!(
        from_slice::<(u8, &[u8])>(&[42, 0, 0, 0, 3, b'f', b'o', b'o']).unwrap(),
        (42, &b"foo"[..])
    );
}

#[test]
fn test_struct() {
    #[derive(Debug, Deserialize, PartialEq)]
    struct S<'a> {
        byte: u8,
        string: &'a [u8],
    }

    assert_eq!(
        from_slice::<S>(&[42, 0, 0, 0, 3, b'f', b'o', b'o']).unwrap(),
        S {
            byte: 42,
            string: b"foo"
        }
    );
}

#[test]
fn test_enum() {
    #[derive(Debug, Deserialize, PartialEq)]
    enum E<'a> {
        Foo(u8),
        Bar(&'a [u8]),
    }

    assert_eq!(from_slice::<E>(&[0, 42]).unwrap(), E::Foo(42));
    assert_eq!(
        from_slice::<E>(&[1, 0, 0, 0, 3, b'b', b'a', b'r']).unwrap(),
        E::Bar(b"bar")
    );
}

#[test]
#[should_panic(expected = "InsufficientData")]
fn test_insufficient_data() {
    from_slice::<u8>(&[]).unwrap();
}

#[test]
#[should_panic(expected = "RemainingData")]
fn test_remaining_data() {
    from_slice::<u8>(&[0, 1]).unwrap();
}
