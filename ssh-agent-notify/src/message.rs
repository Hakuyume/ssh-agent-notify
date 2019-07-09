use failure::{format_err, Error};
use rfc4251::num_bigint::BigInt;
use rfc4251::{Pack, Packer, Unpack, Unpacker};
use sha2::digest::generic_array::GenericArray;
use sha2::digest::Digest;
use std::convert::TryInto;

#[derive(Debug)]
pub enum Message<'a> {
    RequestIdentites,
    IdentitiesAnswer(Vec<Identity<'a>>),
    SignRequest {
        key: KeyBlob,
        data: &'a [u8],
        flag: u32,
    },
    SignResponse(&'a [u8]),
}

impl<'a> Unpack<'a> for Message<'a> {
    type Error = Error;

    fn unpack(unpacker: &mut Unpacker<'a>) -> Result<Self, Self::Error> {
        let mut unpacker = Unpacker::new(unpacker.unpack()?);
        match u32::from(unpacker.unpack::<u8>()?) {
            ssh_agent_sys::SSH2_AGENTC_REQUEST_IDENTITIES => Ok(Message::RequestIdentites),
            ssh_agent_sys::SSH2_AGENT_IDENTITIES_ANSWER => Ok(Message::IdentitiesAnswer(
                (0..unpacker.unpack::<u32>()?)
                    .map(|_| unpacker.unpack())
                    .collect::<Result<_, _>>()?,
            )),
            ssh_agent_sys::SSH2_AGENTC_SIGN_REQUEST => {
                let key = Unpacker::new(unpacker.unpack()?).unpack()?;
                let data = unpacker.unpack()?;
                let flag = unpacker.unpack()?;
                Ok(Message::SignRequest { key, data, flag })
            }
            ssh_agent_sys::SSH2_AGENT_SIGN_RESPONSE => {
                Ok(Message::SignResponse(unpacker.unpack()?))
            }
            type_ => Err(format_err!("Unknown message type: {}", type_)),
        }
    }
}

#[derive(Debug)]
pub struct Identity<'a> {
    pub key: KeyBlob,
    pub comment: &'a str,
}

impl<'a> Unpack<'a> for Identity<'a> {
    type Error = Error;

    fn unpack(unpacker: &mut Unpacker<'a>) -> Result<Self, Self::Error> {
        let key = Unpacker::new(unpacker.unpack()?).unpack()?;
        let comment = unpacker.unpack()?;
        Ok(Self { key, comment })
    }
}

#[derive(Debug, Eq, Hash, PartialEq)]
pub enum KeyBlob {
    Rsa { e: BigInt, p: BigInt },
    Ed25519([u8; 32]),
}

impl Pack for &KeyBlob {
    fn pack(self, packer: &mut Packer) {
        match self {
            KeyBlob::Rsa { e, p } => {
                packer.pack(b"ssh-rsa".as_ref());
                packer.pack(e);
                packer.pack(p);
            }
            KeyBlob::Ed25519(key) => {
                packer.pack(b"ssh-ed25519".as_ref());
                packer.pack(key.as_ref());
            }
        }
    }
}

impl Unpack<'_> for KeyBlob {
    type Error = Error;

    fn unpack(unpacker: &mut Unpacker<'_>) -> Result<Self, Self::Error> {
        match unpacker.unpack::<&[u8]>()? {
            b"ssh-rsa" => {
                let e = unpacker.unpack()?;
                let p = unpacker.unpack()?;
                Ok(KeyBlob::Rsa { e, p })
            }
            b"ssh-ed25519" => Ok(KeyBlob::Ed25519(unpacker.unpack::<&[u8]>()?.try_into()?)),
            type_ => Err(format_err!(
                "Unkown key type: {}",
                String::from_utf8_lossy(type_)
            )),
        }
    }
}

impl KeyBlob {
    pub fn bits(&self) -> usize {
        match self {
            KeyBlob::Rsa { p, .. } => p.bits(),
            KeyBlob::Ed25519(key) => key.len() * 8,
        }
    }

    pub fn digest<D>(&self) -> GenericArray<u8, D::OutputSize>
    where
        D: Digest,
    {
        let mut packer = Packer::default();
        packer.pack(self);
        D::digest(&packer.inner())
    }
}
