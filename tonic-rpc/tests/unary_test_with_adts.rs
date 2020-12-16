use tonic_rpc::tonic_rpc;
use serde::{Serialize, Deserialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct IncRequest {
    num: i32
}

#[derive(Debug, PartialEq, Serialize, Deserialize)]
pub enum IncResult {
    Overflow,
    Incremented(i32)
}

#[tonic_rpc]
trait Increment {
    fn increment(arg: IncRequest) -> IncResult;
}

type State = ();

#[tonic::async_trait]
impl increment_server::Increment for State {
    async fn increment(
        &self,
        request: tonic::Request<IncRequest>,
    ) -> Result<tonic::Response<IncResult>, tonic::Status> {
        let arg = request.into_inner().num;
        let result = if arg == i32::MAX {
            IncResult::Overflow
        } else {
            IncResult::Incremented(arg + 1)
        };
        Ok(tonic::Response::new(result))
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
async fn test_increment_with_adts() {
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

    let request = tonic::Request::new(IncRequest{num: 5});
    let response = client.increment(request).await.expect("Failed to send request");
    assert_eq!(IncResult::Incremented(6), response.into_inner());
    let request = tonic::Request::new(IncRequest{num: i32::MAX});
    let response = client.increment(request).await.expect("Failed to send request");
    assert_eq!(IncResult::Overflow, response.into_inner());
}