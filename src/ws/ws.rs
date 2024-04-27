use anyhow::anyhow;
use base64::prelude::*;
use http_body_util::combinators::BoxBody;
use hyper::{
    body::Bytes,
    header::{
        HeaderValue, CONNECTION, SEC_WEBSOCKET_ACCEPT, SEC_WEBSOCKET_KEY, SEC_WEBSOCKET_VERSION,
        UPGRADE, USER_AGENT,
    },
    HeaderMap, Request, Response, StatusCode, Version,
};
use sha1::Digest;

use crate::utils::creators::{empty, full};

//TODO server should keep track of connections, make sure to
// add some kind of map for users connected or smth so we don't handshake the
// same connectionmultiple times. For now implement it so our connection works.

pub async fn connect_ws(
    req: Request<hyper::body::Incoming>,
) -> Result<Response<BoxBody<Bytes, hyper::Error>>, hyper::Error> {
    // todo!("Implement WS Handshake/Upgrade")
    let mut res = Response::new(empty());
    match ClientWebsocketUpgradeHeaders::from_headers(req.headers()) {
        Ok(client_upgrade_headers) => {
            //TODO I don't know how to handle websocket versions yet (I should just stick to the
            //newest)
            let serv_res_headers =
                ServerWebsocketUpgradeHeaders::from_client_headers(&client_upgrade_headers);
            *res.status_mut() = StatusCode::SWITCHING_PROTOCOLS;
            res.headers_mut().insert(
                CONNECTION,
                HeaderValue::from_str(&serv_res_headers.connection).unwrap(),
            );
            res.headers_mut().insert(
                UPGRADE,
                HeaderValue::from_str(&serv_res_headers.upgrade).unwrap(),
            );
            res.headers_mut().insert(
                SEC_WEBSOCKET_ACCEPT,
                HeaderValue::from_str(&serv_res_headers.sec_websocket_accept).unwrap(),
            );
        }
        Err(err) => {
            println!("Failed to upgrade request due to {}", err);
            *res.status_mut() = StatusCode::BAD_REQUEST;
        }
    }

    println!("Connected client");
    //TODO implement frame exchagne on a separate thread
    Ok(res)
}

pub async fn handle_ws_connection(
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
/*
 Upgrade: websocket
Connection: Upgrade
Sec-WebSocket-Key: dGhlIHNhbXBsZSBub25jZQ==
Sec-WebSocket-Version: 13
 */
struct ClientWebsocketUpgradeHeaders {
    //TODO owned strings for now, think about this when ws conn works
    upgrade: String,
    connection: String,
    sec_websocket_key: String,
    _sec_websocket_version: u8,
}
impl ClientWebsocketUpgradeHeaders {
    /// Create a struct containing headers that are required to make a WS handshake
    /// Error if any of headers is not present.
    /// Required headers are:
    /// - Upgrade
    /// - Connection
    /// - Sec-WebSocket-Key
    /// - Sec-WebSocket-Version
    fn from_headers(headers: &HeaderMap<HeaderValue>) -> Result<Self, anyhow::Error> {
        //Maybe just do headers.get() and if err then return Err
        if !headers.contains_key(UPGRADE) {
            return Err(anyhow!("Missing Upgrade header"));
        }
        if !headers.contains_key(CONNECTION) {
            return Err(anyhow!("Missing Connection header"));
        }
        if !headers.contains_key(SEC_WEBSOCKET_KEY) {
            return Err(anyhow!("Missing Sec-WebSocket-Key header"));
        }
        if !headers.contains_key(SEC_WEBSOCKET_VERSION) {
            return Err(anyhow!("Missing Sec-WebSocket-Version header"));
        }
        //TODO yikes
        Ok(Self {
            upgrade: headers.get(UPGRADE).unwrap().to_str().unwrap().to_owned(),
            connection: headers
                .get(CONNECTION)
                .unwrap()
                .to_str()
                .unwrap()
                .to_owned(),
            sec_websocket_key: headers
                .get(SEC_WEBSOCKET_KEY)
                .unwrap()
                .to_str()
                .unwrap()
                .to_owned(),
            _sec_websocket_version: headers
                .get(SEC_WEBSOCKET_VERSION)
                .unwrap()
                .to_str()
                .unwrap()
                .parse()
                .unwrap(),
        })
    }
}
/*
Upgrade: websocket
Connection: Upgrade
Sec-WebSocket-Accept: s3pPLMBiTxaQ9kYGzzhZRbK+xOo=
*/
struct ServerWebsocketUpgradeHeaders {
    //TODO owned strings for now, think about this when ws conn works
    upgrade: String,
    connection: String,
    sec_websocket_accept: String,
}
impl ServerWebsocketUpgradeHeaders {
    /// Returns headers required to be returned by the server
    /// to switch protocol to WebSocket.
    /// Those headers are:
    /// - Upgrade
    /// - Connection
    /// - Sec-WebSocket-Accept
    fn from_client_headers(client_request_headers: &ClientWebsocketUpgradeHeaders) -> Self {
        //TODO gen the magic_str:
        let magic_string = "258EAFA5-E914-47DA-95CA-C5AB0DC85B11";
        //add magic to Sec-WebSocket-Key
        let mut sec_websocket_accept = client_request_headers.sec_websocket_key.clone();
        sec_websocket_accept.push_str(magic_string);
        //sha1 it
        let mut hasher = sha1::Sha1::new();
        hasher.update(sec_websocket_accept);
        let hash_result = hasher.finalize();
        //base64 it
        let sec_websocket_accept = BASE64_STANDARD.encode(hash_result);

        Self {
            upgrade: client_request_headers.upgrade.clone(),
            connection: client_request_headers.connection.clone(),
            sec_websocket_accept,
        }
    }
}
