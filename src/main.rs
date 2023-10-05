use bytes::Bytes;
use http_body_util::Full;
use hyper::body::Incoming as IncomingBody;
use hyper::http::StatusCode;
use hyper::server::conn::http1;
use hyper::service::Service;
use hyper::{Request, Response};
use hyper_util::rt::TokioIo;
use tokio::net::TcpListener;

use std::env;
use std::future::Future;
use std::net::SocketAddr;
use std::pin::Pin;
use std::sync::Arc;

use light_server::FsNode;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let args: Vec<_> = env::args().collect();
    let dir_path = args[1].clone();
    let addr: SocketAddr = ([127, 0, 0, 1], 3000).into();
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

struct FsSvc {
    fs: Arc<FsNode>,
}

impl Service<Request<IncomingBody>> for FsSvc {
    type Response = Response<Full<Bytes>>;
    type Error = hyper::Error;
    type Future = Pin<Box<dyn Future<Output = Result<Self::Response, Self::Error>> + Send>>;

    fn call(&self, req: Request<IncomingBody>) -> Self::Future {
        fn mk_response(s: Vec<u8>) -> Result<Response<Full<Bytes>>, hyper::Error> {
            Ok(Response::builder().body(Full::new(Bytes::from(s))).unwrap())
        }
        fn mk_error() -> Result<Response<Full<Bytes>>, hyper::Error> {
            Ok(Response::builder()
                .status(StatusCode::NOT_FOUND)
                .body(Full::new(Bytes::from("not found")))
                .unwrap())
        }

        let path = req.uri().path();
        println!("{:?}", path);
        for part in path.split('/') {
            println!("{}", part);
        }
        let resp = mk_error();
        Box::pin(async { resp })
    }
}
