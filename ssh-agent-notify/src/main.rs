#![feature(async_await)]

mod message;

use self::message::{KeyBlob, Message};
use failure::{format_err, Error};
use futures::compat::{Compat01As03, Future01CompatExt, Stream01CompatExt};
use futures::executor::block_on;
use futures::prelude::*;
use log::{error, warn};
use rfc4251::Unpacker;
use sha2::Sha256;
use std::collections::HashMap;
use std::convert::TryInto;
use std::env;
use std::ffi::OsStr;
use std::io;
use tokio::net::{UnixListener, UnixStream};

fn main() -> Result<(), Error> {
    let _ctx = Context::new()?;

    let ssh_auth_sock = env::var_os("SSH_AUTH_SOCK").unwrap();
    let ssh_auth_sock = &ssh_auth_sock;

    let sock = std::path::Path::new("ssh-agent-notify.sock");
    if sock.exists() {
        std::fs::remove_file(&sock)?;
    }

    let listener = UnixListener::bind(&sock)?;
    block_on(
        listener
            .incoming()
            .compat()
            .for_each_concurrent(None, async move |conn| {
                if let Err(err) = proc(ssh_auth_sock, conn).await {
                    error!("{}", err);
                }
            }),
    );
    Ok(())
}

struct Context;

impl Context {
    fn new() -> Result<Self, Error> {
        env_logger::init();
        libnotify::init(env!("CARGO_PKG_NAME")).map_err(|err| format_err!("{}", err))?;
        Ok(Self)
    }
}
impl Drop for Context {
    fn drop(&mut self) {
        libnotify::uninit();
    }
}

async fn proc(ssh_auth_sock: &OsStr, conn: io::Result<UnixStream>) -> io::Result<()> {
    let mut server = Compat01As03::new(UnixStream::connect(ssh_auth_sock).compat().await?);
    let mut client = Compat01As03::new(conn?);

    let mut comments = HashMap::new();

    while let Ok(request) = read(&mut client).await {
        server.write_all(&request).await?;
        match Unpacker::new(&request).unpack() {
            Ok(Message::SignRequest { key, .. }) => {
                let notify = libnotify::Notification::new(
                    &format!("ssh-agent {}", comments.get(&key).unwrap_or(&"".to_owned()),),
                    format!(
                        "{} {} bits\nSHA256:{}",
                        match key {
                            KeyBlob::Rsa { .. } => "RSA",
                            KeyBlob::Ed25519(..) => "ED25519",
                        },
                        key.bits(),
                        base64::encode_config(
                            &key.digest::<Sha256>(),
                            base64::Config::new(base64::CharacterSet::Standard, false)
                        ),
                    )
                    .as_ref(),
                    None,
                );
                let _ = notify.show();
            }
            Ok(_) => (),
            Err(err) => warn!("{}", err),
        }

        let response = read(&mut server).await?;
        client.write_all(&response).await?;
        match Unpacker::new(&response).unpack() {
            Ok(Message::IdentitiesAnswer(identities)) => {
                for identity in identities {
                    comments.insert(identity.key, identity.comment.to_owned());
                }
            }
            Ok(_) => (),
            Err(err) => warn!("{}", err),
        }
    }
    Ok(())
}

async fn read<R>(r: &mut R) -> io::Result<Vec<u8>>
where
    R: Unpin + AsyncRead,
{
    let mut buf = Vec::new();

    unsafe {
        buf.reserve(4);
        buf.set_len(4);
    }
    r.read_exact(&mut buf).await?;
    let len = u32::from_be_bytes(buf[..].try_into().unwrap()) as usize;

    unsafe {
        buf.reserve(len);
        buf.set_len(4 + len);
    }
    r.read_exact(&mut buf[4..]).await?;

    Ok(buf)
}
