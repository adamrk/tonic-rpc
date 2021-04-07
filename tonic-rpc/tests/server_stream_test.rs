use std::time::Duration;

use tokio::{net::TcpListener, sync::mpsc};
use tokio_stream::wrappers::{ReceiverStream, TcpListenerStream};
use tonic::transport::Server;
use tonic_rpc::tonic_rpc;

mod util;

#[tonic_rpc(json)]
trait Counter {
    #[server_streaming]
    fn count(start: i32) -> i32;
    #[server_streaming]
    fn count_n(start: i32, values_count: usize) -> i32;
}

type State = ();

#[tonic::async_trait]
impl counter_server::Counter for State {
    type CountStream = ReceiverStream<Result<i32, tonic::Status>>;

    async fn count(
        &self,
        request: tonic::Request<i32>,
    ) -> Result<tonic::Response<Self::CountStream>, tonic::Status> {
        let mut x = request.into_inner();
        let (tx, rx) = mpsc::channel(100);
        tokio::spawn(async move {
            loop {
                tx.send(Ok(x)).await.unwrap();
                x += 1;
                tokio::time::sleep(Duration::from_millis(1)).await;
            }
        });
        Ok(tonic::Response::new(ReceiverStream::new(rx)))
    }

    type CountNStream = ReceiverStream<Result<i32, tonic::Status>>;

    async fn count_n(
        &self,
        request: tonic::Request<(i32, usize)>,
    ) -> Result<tonic::Response<Self::CountNStream>, tonic::Status> {
        let (start, count) = request.into_inner();
        let (tx, rx) = mpsc::channel(1);
        tokio::spawn(async move {
            for i in 0..count {
                tx.send(Ok(start + (i as i32))).await.unwrap();
            }
        });
        Ok(tonic::Response::new(ReceiverStream::new(rx)))
    }
}

pub async fn run_server() -> u16 {
    let listener = TcpListener::bind("[::1]:0").await.unwrap();
    let port = listener.local_addr().unwrap().port();

    tokio::spawn(async move {
        Server::builder()
            .add_service(counter_server::CounterServer::new(()))
            .serve_with_incoming(TcpListenerStream::new(listener))
            .await
            .unwrap();
    });
    port
}

#[tokio::test]
async fn test_server_streaming() {
    let addr = util::run_server(counter_server::CounterServer::new(())).await;
    let mut client = counter_client::CounterClient::connect(addr)
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
    let third = responses.message().await.unwrap().unwrap();
    assert_eq!(44, third);
}

#[tokio::test]
async fn test_server_stream_ends() {
    let addr = util::run_server(counter_server::CounterServer::new(())).await;
    let mut client = counter_client::CounterClient::connect(addr)
        .await
        .expect("Failed to connect");

    let request = (100_i32, 3_usize);
    let mut responses = client
        .count_n(request)
        .await
        .expect("Failed to send request")
        .into_inner();
    for _ in 0..3 {
        assert!(responses.message().await.unwrap().is_some());
    }
    assert_eq!(None, responses.message().await.unwrap());
}
