use hyper::server::conn::http1;
use hyper_util::rt::TokioIo;
use tokio::net::TcpListener;

use std::env;
use std::net::SocketAddr;
use std::sync::Arc;

use light_server::fs::FsNode;
use light_server::services::FsSvc;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let args: Vec<_> = env::args().collect();
    let dir_path = args[1].clone();
    let addr: SocketAddr = args[2].parse()?;
    let listener = TcpListener::bind(addr).await?;
    println!("Listening on http://{}, serving {}", addr, dir_path);

    let root_dir =
        Arc::new(tokio::task::spawn_blocking(move || FsNode::from_fs(&dir_path)).await??);
    loop {
        let (stream, _) = listener.accept().await?;
        let io = TokioIo::new(stream);
        let new_ref = root_dir.clone();
        tokio::task::spawn(async move {
            if let Err(err) = http1::Builder::new()
                .serve_connection(io, FsSvc { fs: new_ref })
                .await
            {
                println!("Failed to serve connection: {:?}", err);
            }
        });
    }
}
