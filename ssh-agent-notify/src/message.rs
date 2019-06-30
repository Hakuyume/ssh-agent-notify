mod de;

use serde::Deserialize;

#[derive(Debug)]
pub enum Message<'a> {
    RequestIdentites,
    IdentitiesAnswer(Vec<Identity<'a>>),
    SignRequest {
        key: KeyBlob<'a>,
        data: &'a [u8],
        flag: u32,
    },
    SignResponse(&'a [u8]),
}

#[derive(Debug, Deserialize)]
pub struct Identity<'a> {
    pub key: KeyBlob<'a>,
    pub comment: &'a str,
}

#[derive(Debug)]
pub enum KeyBlob<'a> {
    Rsa { e: &'a [u8], p: &'a [u8] },
    Ed25519(&'a [u8]),
}
