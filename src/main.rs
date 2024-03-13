use std::io::{prelude::*, BufReader};
use std::net::{TcpListener, TcpStream};
use std::str::FromStr;

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
    let buf_reader = BufReader::new(&mut stream);
    let http_request: Vec<_> = buf_reader
        .lines()
        .map(|result| result.unwrap())
        .take_while(|line| !line.is_empty())
        .collect();
    //TODO maybe http parser to get header n stuff?
    let request_line = http_request.first();
    match request_line {
        Some(req) => {
            if let Ok(http_uri) = HttpURILine::from_str(req) {
                route(&http_uri, stream);
            }
        }
        None => return_error(stream),
    }
}

fn route(http_uri: &HttpURILine, stream: TcpStream) {
    if http_uri.route == "/chat" && http_uri.method == "GET" && http_uri.protocol_ver >= 1.1 {
        handle_chat(stream);
    } else {
        println!("{:?}",http_uri);
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

#[derive(Debug, PartialEq)]
struct HttpURILine {
    method: String,
    route: String,
    protocol: String,
    protocol_ver: f32,
}
#[derive(Debug, PartialEq, Eq)]
struct ParseHttpURIError;

impl FromStr for HttpURILine {
    type Err = ParseHttpURIError;
    //TODO error handling
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let parts: Vec<_> = s.split(" ").map(|x| x.trim()).collect();
        if parts.len() != 3 {
            return Err(ParseHttpURIError);
        }
        let method = String::from(parts.get(0).unwrap().to_owned());
        let route = String::from(parts.get(1).unwrap().to_owned());
        let prot: Vec<_> = parts.get(2).unwrap().split("/").collect();
        let protocol = String::from(prot.get(0).unwrap().to_owned());
        let protocol_ver = prot.get(1).unwrap().parse::<f32>().unwrap();
        let res = HttpURILine {
            method,
            route,
            protocol,
            protocol_ver,
        };
        return Ok(res);
    }
}
