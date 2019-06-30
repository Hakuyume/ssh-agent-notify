use super::{Message, KeyBlob};
use serde::de::{self, Deserialize, Deserializer, EnumAccess, Error as _, SeqAccess, VariantAccess};
use std::fmt::{self, Formatter};

impl<'de, 'a> Deserialize<'de> for Message<'a>
where
    'de: 'a,
{
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        const VARIANTS: &[&str] = &[
            "REQUEST_IDENTITIES",
            "IDENTITIES_ANSWER",
            "SIGN_REQUEST",
            "SIGN_RESPONSE",
        ];

        struct Visitor;

        impl<'de> de::Visitor<'de> for Visitor {
            type Value = Message<'de>;

            fn expecting(&self, formatter: &mut Formatter) -> fmt::Result {
                write!(formatter, "a enum tagged by u8")
            }

            fn visit_enum<A>(self, data: A) -> Result<Self::Value, A::Error>
            where
                A: EnumAccess<'de>,
            {
                let (tag, variant) = data.variant::<u8>()?;

                match u32::from(tag) {
                    ssh_agent_sys::SSH2_AGENTC_REQUEST_IDENTITIES => Ok(Message::RequestIdentites),
                    ssh_agent_sys::SSH2_AGENT_IDENTITIES_ANSWER => {
                        Ok(Message::IdentitiesAnswer(variant.newtype_variant()?))
                    }
                    ssh_agent_sys::SSH2_AGENTC_SIGN_REQUEST => {
                        struct Visitor;

                        impl<'de> de::Visitor<'de> for Visitor {
                            type Value = (KeyBlob<'de>, &'de [u8], u32);

                            fn expecting(&self, formatter: &mut Formatter) -> fmt::Result {
                                write!(formatter, "a tuple of key blob, byte array, and u32")
                            }

                            fn visit_seq<A>(self, mut seq: A) -> Result<Self::Value, A::Error>
                            where
                                A: SeqAccess<'de>,
                            {
                                Ok((
                                    seq.next_element()?.ok_or_else(|| {
                                        A::Error::invalid_length(
                                            0,
                                            &"a tuple of size 3 is expected",
                                        )
                                    })?,
                                    seq.next_element()?.ok_or_else(|| {
                                        A::Error::invalid_length(
                                            1,
                                            &"a tuple of size 3 is expected",
                                        )
                                    })?,
                                    seq.next_element()?.ok_or_else(|| {
                                        A::Error::invalid_length(
                                            2,
                                            &"a tuple of size 3 is expected",
                                        )
                                    })?,
                                ))
                            }
                        }

                        let (key, data, flag) =
                            variant.struct_variant(&["key", "data", "flag"], Visitor)?;
                        Ok(Message::SignRequest { key, data, flag })
                    }
                    ssh_agent_sys::SSH2_AGENT_SIGN_RESPONSE => {
                        Ok(Message::SignResponse(variant.newtype_variant()?))
                    }
                    _ => Err(A::Error::unknown_variant(&format!("{}", tag), VARIANTS)),
                }
            }
        }

        deserializer.deserialize_enum("Message", VARIANTS, Visitor)
    }
}

impl<'de, 'a> Deserialize<'de> for KeyBlob<'a>
where
    'de: 'a,
{
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        const VARIANTS: &[&str] = &["ssh-rsa", "ssh-ed25519"];

        struct Visitor;

        impl<'de> de::Visitor<'de> for Visitor {
            type Value = KeyBlob<'de>;

            fn expecting(&self, formatter: &mut Formatter) -> fmt::Result {
                write!(formatter, "a enum tagged by byte array")
            }

            fn visit_enum<A>(self, data: A) -> Result<Self::Value, A::Error>
            where
                A: EnumAccess<'de>,
            {
                let (tag, variant) = data.variant::<&'de [u8]>()?;

                match tag {
                    b"ssh-rsa" => {
                        struct Visitor;

                        impl<'de> de::Visitor<'de> for Visitor {
                            type Value = (&'de [u8], &'de [u8]);

                            fn expecting(&self, formatter: &mut Formatter) -> fmt::Result {
                                write!(formatter, "a tuple of byte array and byte array")
                            }

                            fn visit_seq<A>(self, mut seq: A) -> Result<Self::Value, A::Error>
                            where
                                A: SeqAccess<'de>,
                            {
                                Ok((
                                    seq.next_element()?.ok_or_else(|| {
                                        A::Error::invalid_length(
                                            0,
                                            &"a tuple of size 2 is expected",
                                        )
                                    })?,
                                    seq.next_element()?.ok_or_else(|| {
                                        A::Error::invalid_length(
                                            1,
                                            &"a tuple of size 2 is expected",
                                        )
                                    })?,
                                ))
                            }
                        }

                        let (e, p) = variant.struct_variant(&["e", "p"], Visitor)?;
                        Ok(KeyBlob::Rsa { e, p })
                    }
                    b"ssh-ed25519" => Ok(KeyBlob::Ed25519(variant.newtype_variant()?)),
                    _ => Err(A::Error::unknown_variant(
                        &String::from_utf8_lossy(tag),
                        VARIANTS,
                    )),
                }
            }
        }

        deserializer.deserialize_enum("KeyBlob", VARIANTS, Visitor)
    }
}
