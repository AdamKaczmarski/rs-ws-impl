mod echo;
mod utils;
mod ws;

use crate::echo::*;
use http_body_util::combinators::BoxBody;
use hyper::service::service_fn;
use hyper::{body::Bytes, server::conn::http1, Request, Response};
use hyper::{Method, StatusCode};
use hyper_util::rt::TokioIo;
use std::net::SocketAddr;
use tokio::net::TcpListener;
use utils::creators::{empty, full};
use ws::handle_ws_connection;

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
            println!("Unknown request {} {}", req.method(), req.uri().path());
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
                .with_upgrades() //Hyper upgrade module
                .await
            {
                println!("Error serving connection: {:?}", err);
            }
        });
    }
}
