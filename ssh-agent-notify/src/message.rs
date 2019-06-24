use serde::de::{self, Deserializer, EnumAccess, VariantAccess};
use serde::Deserialize;
use std::fmt::{self, Formatter};

#[derive(Debug)]
pub enum Message<'a> {
    RequestIdentites,
    IdentitiesAnswer(Vec<(KeyBlob<'a>, &'a str)>),
    SignRequest((&'a [u8], &'a [u8], u32)),
    Unknown(u8),
}

impl<'de, 'a> Deserialize<'de> for Message<'a>
where
    'de: 'a,
{
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        struct Visitor;

        impl<'de> de::Visitor<'de> for Visitor {
            type Value = Message<'de>;
            fn expecting(&self, formatter: &mut Formatter) -> fmt::Result {
                write!(formatter, "a enum tagged by u32")
            }

            fn visit_enum<A>(self, data: A) -> Result<Self::Value, A::Error>
            where
                A: EnumAccess<'de>,
            {
                let (tag, variant) = data.variant::<u8>()?;
                match tag as _ {
                    ssh_agent_sys::SSH2_AGENTC_REQUEST_IDENTITIES => {
                        variant.unit_variant()?;
                        Ok(Message::RequestIdentites)
                    }
                    ssh_agent_sys::SSH2_AGENT_IDENTITIES_ANSWER => {
                        Ok(Message::IdentitiesAnswer(variant.newtype_variant()?))
                    }
                    ssh_agent_sys::SSH2_AGENTC_SIGN_REQUEST => {
                        Ok(Message::SignRequest(variant.newtype_variant()?))
                    }
                    _ => Ok(Message::Unknown(tag)),
                }
            }
        }

        deserializer.deserialize_enum(
            "",
            &["REQUEST_IDENTITIES", "IDENTITIES_ANSWER", "SIGN_REQUEST"],
            Visitor,
        )
    }
}

#[derive(Debug, Deserialize)]
pub struct Identity<'a> {
    pub key_blob: KeyBlob<'a>,
    pub comment: &'a str,
}

#[derive(Debug)]
pub enum KeyBlob<'a> {
    Rsa((&'a [u8], &'a [u8])),
    Ed25519(&'a [u8]),
    Unknown(&'a [u8]),
}

impl<'de, 'a> Deserialize<'de> for KeyBlob<'a>
where
    'de: 'a,
{
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        struct Visitor;

        impl<'de> de::Visitor<'de> for Visitor {
            type Value = KeyBlob<'de>;
            fn expecting(&self, formatter: &mut Formatter) -> fmt::Result {
                write!(formatter, "a enum tagged by u32")
            }

            fn visit_enum<A>(self, data: A) -> Result<Self::Value, A::Error>
            where
                A: EnumAccess<'de>,
            {
                let (tag, variant) = data.variant::<&[u8]>()?;
                match tag {
                    b"ssh-rsa" => Ok(KeyBlob::Rsa(variant.newtype_variant()?)),
                    b"ssh-ed25519" => Ok(KeyBlob::Ed25519(variant.newtype_variant()?)),
                    _ => Ok(KeyBlob::Unknown(tag)),
                }
            }
        }

        deserializer.deserialize_enum("", &["ssh-rsa", "ssh-ed25519"], Visitor)
    }
}
