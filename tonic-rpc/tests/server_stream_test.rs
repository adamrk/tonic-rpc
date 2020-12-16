use tokio::sync::mpsc;
use tonic_rpc::tonic_rpc;

#[tonic_rpc]
trait Counter {
    #[server_streaming]
    fn count(args: i32) -> i32;
}

type State = ();

#[tonic::async_trait]
impl counter_server::Counter for State {
    type countStream = mpsc::Receiver<Result<i32, tonic::Status>>;

    async fn count(
        &self,
        request: tonic::Request<i32>,
    ) -> Result<tonic::Response<Self::countStream>, tonic::Status> {
        let mut x = request.into_inner();
        let (mut tx, rx) = mpsc::channel(100);
        tokio::spawn(async move {
            loop {
                tx.send(Ok(x)).await.unwrap();
                x += 1;
                tokio::time::delay_for(std::time::Duration::from_millis(1)).await;
            }
        });
        Ok(tonic::Response::new(rx))
    }
}

pub async fn run_server() -> u16 {
    let mut listener = tokio::net::TcpListener::bind("[::1]:0").await.unwrap();
    let port = listener.local_addr().unwrap().port();

    tokio::spawn(async move {
        tonic::transport::Server::builder()
            .add_service(counter_server::CounterServer::new(()))
            .serve_with_incoming(listener.incoming())
            .await
            .unwrap();
    });
    port
}

#[tokio::test]
async fn test_server_streaming() {
    let port = run_server().await;
    // Wait for server to start
    tokio::time::delay_for(std::time::Duration::from_millis(1)).await;
    let mut client = counter_client::CounterClient::connect(format!("http://[::1]:{}", port))
        .await
        .expect("Failed to connect");

    let request = tonic::Request::new(42_i32);
    let mut responses = client
        .count(request)
        .await
        .expect("Failed to send request")
        .into_inner();
    let first = responses.message().await.unwrap().unwrap();
    assert_eq!(42, first);
    let second = responses.message().await.unwrap().unwrap();
    assert_eq!(43, second);
}
