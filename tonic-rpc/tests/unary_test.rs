use tonic_rpc::tonic_rpc;

#[tonic_rpc]
trait Increment {
    fn increment(x: i32) -> i32;
}

type State = ();

#[tonic::async_trait]
impl increment_server::Increment for State {
    async fn increment(
        &self,
        request: tonic::Request<i32>,
    ) -> Result<tonic::Response<i32>, tonic::Status> {
        Ok(tonic::Response::new(request.into_inner() + 1))
    }
}

pub async fn run_server() -> u16 {
    let mut listener = tokio::net::TcpListener::bind("[::1]:0").await.unwrap();
    let port = listener.local_addr().unwrap().port();

    tokio::spawn( async move {
        tonic::transport::Server::builder()
            .add_service(increment_server::IncrementServer::new(()))
            .serve_with_incoming(listener.incoming()).await.unwrap();
        }
    );
    port
}

#[tokio::test]
async fn test() {
    let port = run_server().await;
    // Wait for server to start
    tokio::time::delay_for(std::time::Duration::from_millis(1)).await;
    let mut client =
        increment_client::IncrementClient::connect(format!(
            "http://[::1]:{}",
            port 
        ))
        .await
        .expect("Failed to connect");

    let request = tonic::Request::new(5);
    let response = client.increment(request).await.expect("Failed to send request");
    assert_eq!(6, response.into_inner())
}