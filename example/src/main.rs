use tokio::net::TcpListener;
use tokio_stream::wrappers::TcpListenerStream;
use tonic::transport::Server;
use tonic_rpc::tonic_rpc;

/// The `tonic_rpc` attribute says that we want to build an RPC defined by this trait.
/// The `json` option says that we should use the `tokio-serde` Json codec for serialization.
#[tonic_rpc(json)]
trait Increment {
    /// Our service will have a single endpoint.
    fn increment(arg: i32) -> i32;
}

/// Our server doesn't need any state.
struct State;

#[tonic::async_trait]
impl increment_server::Increment for State {
    /// The request type gets wrapped in a `tonic::Request`.
    /// The response type gets wrapped in a `Result<tonic::Response<_>, tonic::Status>`.
    async fn increment(
        &self,
        request: tonic::Request<i32>,
    ) -> Result<tonic::Response<i32>, tonic::Status> {
        let arg = request.into_inner();
        Ok(tonic::Response::new(arg + 1))
    }
}

/// Run the server.
#[tokio::main]
async fn main() {
    let listener = TcpListener::bind("[::1]:8080").await.unwrap();
    let addr = listener.local_addr().unwrap();
    tokio::spawn(async move {
        Server::builder()
            .add_service(increment_server::IncrementServer::new(State))
            .serve_with_incoming(TcpListenerStream::new(listener))
            .await
    });
    let mut client = increment_client::IncrementClient::connect(format!("http://{}", addr))
        .await
        .unwrap();
    let response = client.increment(32).await.unwrap().into_inner();
    println!("Got {}", response);
    assert_eq!(33, response);
}
