use hyper::service::service_fn;
use std::net::SocketAddr;

use http_body_util::{Empty, Full};
use hyper::{body::Bytes, server::conn::http1, Request, Response};
use hyper_util::rt::TokioIo;
use tokio::net::TcpListener;

use http_body_util::{combinators::BoxBody, BodyExt};
use hyper::body::{Body, Frame};
use hyper::{Method, StatusCode, Version};

async fn echo(
    req: Request<hyper::body::Incoming>,
) -> Result<Response<BoxBody<Bytes, hyper::Error>>, hyper::Error> {
    Ok(Response::new(req.into_body().boxed()))
}

async fn echo_reversed(
    req: Request<hyper::body::Incoming>,
) -> Result<Response<BoxBody<Bytes, hyper::Error>>, hyper::Error> {
    //Set upper body buffer size
    let upper = req.body().size_hint().upper().unwrap_or(u64::MAX);
    if upper > 1024 * 64 {
        let mut resp = Response::new(full("Payload too big"));
        *resp.status_mut() = hyper::StatusCode::PAYLOAD_TOO_LARGE;
        return Ok(resp);
    }
    //Read whole body stream
    let whole_body = req.collect().await?.to_bytes();
    let reversed_body = whole_body.iter().rev().cloned().collect::<Vec<u8>>();
    Ok(Response::new(full(reversed_body)))
}

async fn echo_uppercase(
    req: Request<hyper::body::Incoming>,
) -> Result<Response<BoxBody<Bytes, hyper::Error>>, hyper::Error> {
    let frame_stream = req.into_body().map_frame(|frame| {
        let frame = if let Ok(data) = frame.into_data() {
            data.iter()
                .map(|byte| byte.to_ascii_uppercase())
                .collect::<Bytes>()
        } else {
            Bytes::new()
        };
        Frame::data(frame)
    });

    Ok(Response::new(frame_stream.boxed()))
}

async fn connect_ws(
    _req: Request<hyper::body::Incoming>,
) -> Result<Response<BoxBody<Bytes, hyper::Error>>, hyper::Error> {
    todo!("Implement WS Handshake/Upgrade")
        //TODO implement frame exchagne 
}

async fn handle_ws_connection(
    req: Request<hyper::body::Incoming>,
) -> Result<Response<BoxBody<Bytes, hyper::Error>>, hyper::Error> {
    let http_version = req.version();
    match http_version {
        Version::HTTP_09 | Version::HTTP_10 => {
            let mut res = Response::new(full("Older HTTP version used, please use HTTP/1.1+"));
            *res.status_mut() = hyper::StatusCode::HTTP_VERSION_NOT_SUPPORTED;
            Ok(res)
        }
        //HTTP/1.1 +
        _ => connect_ws(req).await,
    }
}

async fn route(
    req: Request<hyper::body::Incoming>,
) -> Result<Response<BoxBody<Bytes, hyper::Error>>, hyper::Error> {
    match (req.method(), req.uri().path()) {
        (&Method::GET, "/") => Ok(Response::new(full("To connect to chat send GET /chat"))),
        (&Method::GET, "/chat") => handle_ws_connection(req).await,
        (&Method::POST, "/echo") => echo(req).await,
        (&Method::POST, "/echo/reversed") => echo_reversed(req).await,
        (&Method::POST, "/echo/uppercase") => echo_uppercase(req).await,
        _ => {
            let mut not_found = Response::new(empty());
            *not_found.status_mut() = StatusCode::NOT_FOUND;
            Ok(not_found)
        }
    }
}
#[tokio::main]
async fn main() -> Result<(), anyhow::Error> {
    let addr = SocketAddr::from(([127, 0, 0, 1], 8000));
    let listener = TcpListener::bind(addr).await?;
    println!("Bound to address {}", addr);

    loop {
        let (stream, _) = listener.accept().await?;

        let io = TokioIo::new(stream);
        tokio::task::spawn(async move {
            if let Err(err) = http1::Builder::new()
                .serve_connection(io, service_fn(route))
                .await
            {
                println!("Error serving connection: {:?}", err);
            }
        });
    }
}

fn empty() -> BoxBody<Bytes, hyper::Error> {
    Empty::<Bytes>::new()
        .map_err(|never| match never {})
        .boxed()
}

fn full<T: Into<Bytes>>(chunk: T) -> BoxBody<Bytes, hyper::Error> {
    Full::new(chunk.into())
        .map_err(|never| match never {})
        .boxed()
}
