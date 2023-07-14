### Run server
```
cd server
cargo run -- --data 127.0.0.1:42424 --http 127.0.0.1:8080 --public 127.0.0.1:42424
```

### Run client
```
cd client
cargo run -- --server http://127.0.0.1:8080/new_rtc_session
```