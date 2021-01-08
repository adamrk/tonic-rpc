`tonic-rpc` is a macro that generates the traits and stubs used by [`tonic`](https://crates.io/crates/tonic)
from Rust definitions instead of `proto` files.

This means that you can get all the [benefits](https://github.com/hyperium/tonic#features)
of `tonic` while using regular Rust types and without needing to use `proto` files or build scripts.
Of course, this comes at the sacrifice of interoporability.

# Alternatives
[`tarpc`](https://crates.io/crates/tarpc) is an excellent RPC library that also defines services using
as a Rust trait.

# Required dependencies
```toml
tonic = <tonic-version>
tonic-rpc = <tonic-rpc-version>
```

# Example
Instead of defining a `proto`, define a service as a trait:
```rust
#[tonic_rpc::tonic_rpc(json)]
trait Increment {
    fn increment(arg: i32) -> i32;
}
```
The attribute `#[tonic_rpc(json)]` indicates that this service
will serialize the requests and responses using `json`.
The arguments and return values for each function must implement
`serde::Serialize` and `serde::Deserialize`.

The service can be implemented by defining and `impl`:
```rust
struct State;

#[tonic::async_trait]
impl increment_server::Increment for State {
    async fn increment(
        &self,
        request: tonic::Request<i32>,
    ) -> Result<tonic::Response<i32>, tonic::Status> {
        Ok(tonic::Response::new(request.into_inner() + 1))
    }
}
```

And a server and client can be run:
```rust
async fn run_client_server() {
    let mut listener = tokio::net::TcpListener::bind("[::1]:8080").await.unwrap();
    let addr = listener.local_addr().unwrap();
    tokio::spawn(async move {
        tonic::transport::Server::builder()
            .add_service(increment_server::IncrementServer::new(State))
            .serve_with_incoming(listener.incoming())
            .await
    });
    let mut client = increment_client::IncrementClient::connect(format!("http://{}", addr))
        .await
        .unwrap();
    let response = client.increment(32).await.unwrap().into_inner();
    assert_eq!(33, response);
}
```

The full example is available [here](https://github.com/adamrk/tonic-rpc/tree/main/example).
Further examples are available in the [tests folder](https://github.com/adamrk/tonic-rpc/tree/main/tonic-rpc/tests).

# Encodings
The available encodings are:
- `bincode` - using [`bincode`](https://crates.io/crates/bincode)
- `cbor` - using [`serde_cbor`](https://crates.io/crates/serde_cbor)
- `json` - using [`serde_json`](https://crates.io/crates/serde_json)
- `messagepack` - using [`rmp-serde`](https://crates.io/crates/rmp-serde)

# Streaming
Streaming can be added on the client or server side by adding the attributes
`#[client_streaming]` or `#[server_streaming]` to a function in the service trait.
These behave the same as if the `stream` keyword were added to a `proto` definition.

Examples that use streaming can be found in the [tests folder](https://github.com/adamrk/tonic-rpc/tree/main/tonic-rpc/tests).