# WebSocket server implementation in Rust

I just wanted to do one for fun.

Sources I used:
- [Mozilla - Writing WebSocket servers](https://developer.mozilla.org/en-US/docs/Web/API/WebSockets_API/Writing_WebSocket_servers)
- [WebSocket Protocol RFC](https://datatracker.ietf.org/doc/rfc6455)
- [Rust Book](https://doc.rust-lang.org/book/ch20-01-single-threaded.html)
- [Hyper](https://hyper.rs)

## testing curls 
Upgrade from client
```shell
curl -v localhost:8000/chat \
    -H "Upgrade: websocket" \
    -H "Connection: Upgrade" \
    -H "Sec-Websocket-Key: dGhlIHNhbXBsZSBub25jZQ==" \
    -H "Sec-WebSocket-Version: 13"
```
