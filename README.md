## Str0m
### Run server
The server is modified from the chat demo of the Str0m library.
```
cd str0m/server
cargo run
```
The HTTP server for ICE will run on `127.0.0.1:8080`.

### Run client
The client uses webrtc-rs.
```
cd str0m/client
cargo run -- --server http://127.0.0.1:8080/new_rtc_session
```

## webrtc-unreliable
### Run server
The server uses webrtc-unreliable library.
```
cd webrtc-unreliable/server
cargo run -- --data 127.0.0.1:42424 --http 127.0.0.1:8080 --public 127.0.0.1:42424
```

### Run client
The client uses webrtc-rs.
```
cd webrtc-unreliable/client
cargo run -- --server http://127.0.0.1:8080/new_rtc_session
```
The `on_message` callback of data channel some how cannot be triggered, while the SCTP assocation does receive message from server:
```
[2023-07-20T08:54:29Z DEBUG webrtc_sctp::association] [] recving 44 bytes
```
