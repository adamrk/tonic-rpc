# Tonic-RPC

A macro that allows you to use [`tonic`](https://crates.io/crates/tonic) by defining your RPC interfaces in pure Rust code instead of using [`protobuf`](https://developers.google.com/protocol-buffers). When your service doesn't have clients written in other languages this allows you use the full power of the Rust type system in you RPC interface.

# Examples
Further examples can be found in the [`tests`](https://github.com/adamrk/tonic-rpc/tree/main/tonic-rpc/tests) folder.

The following is an example where clients send an `i32` and the server responds by incrementing the `i32`.

### Define the RPC Interface
```rust
use serde::{Deserialize, Serialize};
use tonic_rpc::tonic_rpc;

/// The type sent by clients in an RPC request.
/// It must implement `Serialize`/`Deserialize`.
#[derive(Debug, Serialize, Deserialize)]
pub struct IncRequest {
    num: i32,
}

/// The response returned by the server.
/// It must also implement `Serialize`/`Deserialize`.
/// We can use an enum to show that the calculation might fail due to overflow.
#[derive(Debug, PartialEq, Serialize, Deserialize)]
pub enum IncResult {
    Overflow,
    Incremented(i32),
}

/// The `tonic_rpc` attribute says that we want to build an RPC defined by this trait.
/// The `json` option says that we should use the `tokio-serde` Json codec for serialization.
#[tonic_rpc(json)]
trait Increment {
    /// Our service will have a single endpoint which responds to an `IncRequest` with an `IncResult`.
    fn increment(arg: IncRequest) -> IncResult;
}
```

### Implement the Server

```rust
/// Our server doesn't need any state.
type State = ();

/// The implementation of our service is done just as for a `tonic` service that was defined using `protobuf`.
#[tonic::async_trait]
impl increment_server::Increment for State {
    /// The request type gets wrapped in a `tonic::Request`.
    /// The response type gets wrapped in a `Result<tonic::Response<_>, tonic::Status>`.
    async fn increment(
        &self,
        request: tonic::Request<IncRequest>,
    ) -> Result<tonic::Response<IncResult>, tonic::Status> {
        let arg = request.into_inner().num;
        let result = match arg.checked_add(1) {
            Some(result) => IncResult::Incremented(result),
            None => IncResult::Overflow,
        };
        Ok(tonic::Response::new(result))
    }
}

/// Run the server.
#[tokio::main]
async fn main() {
    let mut listener = tokio::net::TcpListener::bind("[::1]:8080").await.unwrap();
    tonic::transport::Server::builder()
        .add_service(increment_server::IncrementServer::new(()))
        .serve_with_incoming(listener.incoming())
        .await
        .unwrap();
}
```

### Send a Request
```rust
    /// Create a client.
    let mut client = increment_client::IncrementClient::connect("[::1]:8080")
        .await
        .expect("Failed to connect");

    /// Send a request.
    let request = IncRequest { num: 5 };
    let response = client
        .increment(request)
        .await
        .expect("Failed to send request");
    assert_eq!(IncResult::Incremented(6), response.into_inner());
```