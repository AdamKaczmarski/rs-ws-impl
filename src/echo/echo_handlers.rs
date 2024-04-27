use crate::full;
use http_body_util::{combinators::BoxBody, BodyExt};
use hyper::body::{Body, Frame};
use hyper::{body::Bytes, Request, Response};

pub async fn echo(
    req: Request<hyper::body::Incoming>,
) -> Result<Response<BoxBody<Bytes, hyper::Error>>, hyper::Error> {
    Ok(Response::new(req.into_body().boxed()))
}

pub async fn echo_reversed(
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

pub async fn echo_uppercase(
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
