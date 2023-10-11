use bytes::Bytes;
use http_body_util::Full;
use hyper::body::Incoming as IncomingBody;
use hyper::http::StatusCode;
use hyper::service::Service;
use hyper::{Request, Response};

use std::future::Future;
use std::pin::Pin;
use std::sync::Arc;

use crate::fs::FsNode;

pub struct FsSvc {
    pub fs: Arc<FsNode>,
}

impl Service<Request<IncomingBody>> for FsSvc {
    type Response = Response<Full<Bytes>>;
    type Error = hyper::Error;
    type Future = Pin<Box<dyn Future<Output = Result<Self::Response, Self::Error>> + Send>>;

    fn call(&self, req: Request<IncomingBody>) -> Self::Future {
        fn mk_response(v: &[u8]) -> Result<Response<Full<Bytes>>, hyper::Error> {
            Ok(Response::builder()
                .body(Full::new(Bytes::from(v.to_owned())))
                .unwrap())
        }
        fn mk_error(status: StatusCode) -> Result<Response<Full<Bytes>>, hyper::Error> {
            Ok(Response::builder()
                .status(status)
                .body(Full::new(Bytes::from(format!("{}", status))))
                .unwrap())
        }

        let path = req.uri().path();
        let resp = match self.fs.get(&path.split('/').skip(1).collect::<Vec<&str>>()) {
            Some(content) => mk_response(content),
            _ => mk_error(StatusCode::NOT_FOUND),
        };
        Box::pin(async { resp })
    }
}
