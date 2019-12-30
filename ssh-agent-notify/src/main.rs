mod message;

use self::message::{KeyBlob, Message};
use clap::{App, Arg};
use failure::{format_err, Error};
use futures::future::Either;
use futures::pin_mut;
use futures::prelude::*;
use futures::stream::select;
use log::{error, info, warn};
use rfc4251::Unpacker;
use sha2::Sha256;
use std::collections::HashMap;
use std::convert::TryInto;
use std::env;
use std::ffi::OsStr;
use std::fs;
use std::io;
use tokio::io::{AsyncRead, AsyncReadExt, AsyncWriteExt};
use tokio::net::{UnixListener, UnixStream};
use tokio::signal;

#[tokio::main]
async fn main() -> Result<(), Error> {
    env_logger::init();
    let _libnotify = Libnotify::init().map_err(|err| format_err!("{}", err))?;

    let ssh_auth_sock = env::var_os("SSH_AUTH_SOCK").unwrap();

    let matches = App::new(env!("CARGO_PKG_NAME"))
        .arg(Arg::with_name("PROXY_SOCK").required(true).index(1))
        .get_matches();
    let proxy_sock = matches.value_of("PROXY_SOCK").unwrap();

    let (_sock_path, mut listener) = SockPath::bind(proxy_sock)?;
    let listener = listener.incoming();

    let stream = select(
        listener.map(Either::Left),
        stream::once(signal::ctrl_c()).map(Either::Right),
    );
    pin_mut!(stream);
    while let Either::Left(conn) = stream.select_next_some().await {
        let ssh_auth_sock = ssh_auth_sock.clone();
        tokio::spawn(async move {
            if let Err(err) = async { proc(&ssh_auth_sock, conn?).await }.await {
                error!("{}", err);
            }
        });
    }
    info!("Exit");
    Ok(())
}

async fn proc(ssh_auth_sock: &OsStr, mut client: UnixStream) -> io::Result<()> {
    let mut server = UnixStream::connect(ssh_auth_sock).await?;

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
    let len = u32::from_be_bytes((&buf as &[_]).try_into().unwrap()) as usize;

    unsafe {
        buf.reserve(len);
        buf.set_len(4 + len);
    }
    r.read_exact(&mut buf[4..]).await?;

    Ok(buf)
}

struct Libnotify;

impl Libnotify {
    fn init() -> Result<Self, String> {
        libnotify::init(env!("CARGO_PKG_NAME"))?;
        Ok(Self)
    }
}

impl Drop for Libnotify {
    fn drop(&mut self) {
        libnotify::uninit();
    }
}

struct SockPath<'a>(&'a str);

impl<'a> SockPath<'a> {
    fn bind(path: &'a str) -> io::Result<(Self, UnixListener)> {
        let listener = UnixListener::bind(path)?;
        Ok((Self(path), listener))
    }
}

impl<'a> Drop for SockPath<'a> {
    fn drop(&mut self) {
        let _ = fs::remove_file(self.0);
    }
}
