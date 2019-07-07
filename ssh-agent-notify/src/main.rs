#![feature(async_await)]
#![recursion_limit = "128"]

mod message;

use self::message::{KeyBlob, Message};
use failure::{format_err, Error};
use futures::compat::{Compat01As03, Future01CompatExt, Stream01CompatExt};
use futures::executor::block_on;
use futures::future::ready;
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
use std::path::Path;
use tokio::net::{UnixListener, UnixStream};
use tokio_signal::unix::{Signal, SIGINT, SIGTERM};

fn main() -> Result<(), Error> {
    env_logger::init();
    libnotify::init(env!("CARGO_PKG_NAME")).map_err(|err| format_err!("{}", err))?;

    let signals = select(
        Signal::new(SIGINT)
            .compat()
            .map(|stream| stream.map(Stream01CompatExt::compat))
            .try_flatten_stream(),
        Signal::new(SIGTERM)
            .compat()
            .map(|stream| stream.map(Stream01CompatExt::compat))
            .try_flatten_stream(),
    );

    let ssh_auth_sock = env::var_os("SSH_AUTH_SOCK").unwrap();
    let ssh_auth_sock = &ssh_auth_sock;

    let sock = Path::new("ssh-agent-notify.sock");
    let listener = UnixListener::bind(sock)?.incoming().compat();

    block_on(
        select(
            listener.map(|conn| conn.map(Some)),
            signals.map(|signal| signal.map(|_| None)),
        )
        .take_while(|conn| ready(conn.as_ref().map(|conn| conn.is_some()).unwrap_or(false)))
        .for_each_concurrent(None, async move |conn| {
            let conn = conn.unwrap().unwrap();
            if let Err(err) = proc(ssh_auth_sock, conn).await {
                error!("{}", err);
            }
        }),
    );

    fs::remove_file(sock)?;
    libnotify::uninit();
    info!("Exit");

    Ok(())
}

async fn proc(ssh_auth_sock: &OsStr, conn: UnixStream) -> io::Result<()> {
    let mut server = Compat01As03::new(UnixStream::connect(ssh_auth_sock).compat().await?);
    let mut client = Compat01As03::new(conn);

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
