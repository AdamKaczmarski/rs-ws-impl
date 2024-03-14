use std::io::prelude::*;
use std::net::{TcpListener, TcpStream};

use anyhow::{anyhow, Error};
use httparse::Request;

fn listen_for_connection(listener: &TcpListener) {
    for stream in listener.incoming() {
        match stream {
            Ok(stream) => {
                handle_connection(stream);
            }
            Err(e) => {
                println!("Connection failed: {}", e)
            }
        }
    }
}

fn handle_connection(mut stream: TcpStream) {
    let mut buffer = [0; 1024];
    stream.read(&mut buffer).unwrap();

    let mut headers = [httparse::EMPTY_HEADER; 16];
    let mut req = httparse::Request::new(&mut headers);
    match req.parse(&buffer) {
        Ok(_) => match verify_http_request(&req) {
            Ok(_) => {
                route(&req, stream);
            }
            Err(err) => {
                println!("{}", err);
                return_error(stream)
            }
        },
        Err(err) => {
            println!("{}", err);
            return_error(stream)
        }
    }
}

fn verify_http_request(req: &Request<'_, '_>) -> Result<(), Error> {
    match req.version {
        Some(_) => {}
        None => {
            return Err(anyhow!("Missing HTTP Version"));
        }
    }
    match req.method {
        Some(_) => {}
        None => {
            return Err(anyhow!("Missing HTTP Method"));
        }
    }
    match req.path {
        Some(_) => {}
        None => {
            return Err(anyhow!("Missing HTTP Path"));
        }
    }
    return Ok(());
}

fn route(request: &Request, stream: TcpStream) {
    if request.path.unwrap() == "/chat"
        && request.method.unwrap() == "GET"
        && request.version.unwrap() >= 1
    {
        handle_chat(stream);
    } else {
        return_error(stream);
    }
}

fn handle_chat(stream: TcpStream) {
    let response = "HTTP/1.1 200 OK\r\n\r\n".as_bytes();
    send_response(response, stream)
}

fn return_error(stream: TcpStream) {
    let status_line = "HTTP/1.1 400 Bad Request\r\n\r\n";
    let contents = "Bad request";
    let len = contents.len();
    let response = &format!("{status_line}\r\nContent-Length: {len}\r\n\r\n{contents}");
    let response_bytes = response.as_bytes();
    send_response(response_bytes, stream)
}

fn send_response(response: &[u8], mut stream: TcpStream) {
    match stream.write_all(response) {
        Ok(_) => {}
        Err(e) => println!("Failed to write a response! {}", e),
    }
}
fn main() {
    let addr = "127.0.0.1:8000";
    let listener = TcpListener::bind(addr).expect(format!("Couldn't bind port {}", addr).as_str());
    println!("Bound to address {}", addr);
    listen_for_connection(&listener);
}
