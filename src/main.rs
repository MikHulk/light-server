use hyper::server::conn::http1;
use hyper_util::rt::TokioIo;
use std::env;
use std::net::SocketAddr;
use std::sync::Arc;
use std::time::SystemTime;
use tokio::io::{ErrorKind, Interest};
use tokio::net::{TcpListener, TcpStream};

use chrono::offset::Utc;
use chrono::DateTime;

use light_server::fs::FsNode;
use light_server::services::FsSvc;

macro_rules! print_error {
    ( $err:ident ) => {{
        let now: DateTime<Utc> = SystemTime::now().into();
        eprintln!(
            "{} | Failed to serve connection: {}",
            now.format("%Y-%m-%d %T"),
            $err,
        );
    }};
}

async fn server_loop() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let args: Vec<_> = env::args().collect();
    let dir_path = args[1].clone();
    let addr: SocketAddr = args[2].parse()?;
    let listener = TcpListener::bind(addr).await?;
    println!("Listening on http://{}, serving {}", addr, dir_path);

    let root_dir =
        Arc::new(tokio::task::spawn_blocking(move || FsNode::from_fs(&dir_path)).await??);
    loop {
        match listener.accept().await {
            Ok((stream, _)) => {
                let io = TokioIo::new(stream);
                let new_ref = root_dir.clone();
                tokio::task::spawn(async move {
                    if let Err(err) = http1::Builder::new()
                        .serve_connection(io, FsSvc { fs: new_ref })
                        .await
                    {
                        print_error!(err);
                    }
                });
            }
            Err(err) => {
                print_error!(err);
            }
        }
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let mut handle = tokio::spawn(server_loop());
    let listener = TcpListener::bind("127.0.0.1:9999").await?;
    'outer: loop {
        match listener.accept().await {
            Ok((stream, addr)) => {
                let ready = stream.ready(Interest::READABLE).await?;
                if ready.is_readable() {
                    let mut data = vec![0; 1024];
                    match stream.try_read(&mut data) {
                        Ok(n) => {
                            data.truncate(n);
                            let msg = String::from_utf8(data)?.trim().to_string();
                            println!("{} sent {}", addr, msg);
                            async fn send_msg(
                                msg: &str,
                                stream: &TcpStream,
                            ) -> Result<(), Box<dyn std::error::Error + Send + Sync>>
                            {
                                let ready = stream.ready(Interest::WRITABLE).await?;
                                if ready.is_writable() {
                                    _ = stream.try_write(msg.as_bytes());
                                }
                                Ok(())
                            }
                            match msg.as_str() {
                                "abort" => {
                                    println!("server ending");
                                    handle.abort();
                                    send_msg("server ending\r\n", &stream).await?;
                                    break 'outer;
                                }
                                "reload" => {
                                    println!("content reloading");
                                    handle.abort();
                                    handle = tokio::spawn(server_loop());
                                    send_msg("content reloaded\r\n", &stream).await?;
                                }
                                _ => send_msg("unkown message\r\n", &stream).await?,
                            }
                        }
                        Err(ref e) if e.kind() == ErrorKind::WouldBlock => {
                            continue;
                        }
                        Err(e) => {
                            return Err(e.into());
                        }
                    }
                }
            }
            Err(err) => {
                print_error!(err);
            }
        }
    }
    println!("bye!");
    Ok(())
}
