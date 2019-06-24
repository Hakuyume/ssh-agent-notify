#![feature(async_await)]

mod message;

use self::message::Message;
use futures::compat::{Compat01As03, Future01CompatExt, Stream01CompatExt};
use futures::executor::block_on;
use futures::prelude::*;
use std::convert::TryInto;
use std::env;
use std::error::Error;
use std::ffi::OsStr;
use std::io;
use tempfile;
use tokio::net::{UnixListener, UnixStream};

fn main() -> Result<(), Box<dyn Error>> {
    let ssh_auth_sock = env::var_os("SSH_AUTH_SOCK").unwrap();
    let ssh_auth_sock = &ssh_auth_sock;

    // let temp = tempfile::tempdir()?;
    // let sock = temp.path().join("ssh-agent");
    let sock = std::path::Path::new("ssh-agent-notify.sock");
    if sock.exists() {
        std::fs::remove_file(&sock)?;
    }

    let listener = UnixListener::bind(&sock)?;
    println!("export SSH_AUTH_SOCK={}", sock.display());
    block_on(
        listener
            .incoming()
            .compat()
            .for_each_concurrent(None, async move |conn| {
                if let Err(err) = proc(ssh_auth_sock, conn).await {
                    eprintln!("{}", err);
                }
            }),
    );
    Ok(())
}

async fn proc(ssh_auth_sock: &OsStr, conn: io::Result<UnixStream>) -> io::Result<()> {
    let mut server = Compat01As03::new(UnixStream::connect(ssh_auth_sock).compat().await?);
    let mut client = Compat01As03::new(conn?);

    while let Ok(request) = read(&mut client).await {
        server.write_all(&request).await?;
        if let Ok(request) = rfc4251::from_slice::<Message>(&request) {
            println!("request: {:?}", request);
        }

        let response = read(&mut server).await?;
        client.write_all(&response).await?;
        if let Ok(response) = rfc4251::from_slice::<Message>(&response) {
            println!("response: {:?}", response);
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
