use std::sync::{Arc, Mutex};

use tokio::sync::mpsc;
use tokio_stream::wrappers::ReceiverStream;
use tonic_rpc::tonic_rpc;

mod util;

#[tonic_rpc(json)]
trait Accumulate {
    #[client_streaming]
    fn sum(input: u32) -> u32;
}

type State = ();

#[tonic::async_trait]
impl accumulate_server::Accumulate for State {
    async fn sum(
        &self,
        args: tonic::Request<tonic::Streaming<u32>>,
    ) -> Result<tonic::Response<u32>, tonic::Status> {
        let mut args = args.into_inner();
        let mut total = 0;
        while let Some(arg) = args.message().await.unwrap() {
            total += arg;
        }

        Ok(tonic::Response::new(total))
    }
}

#[tokio::test]
async fn test_client_streaming() {
    let addr = util::run_server(accumulate_server::AccumulateServer::new(())).await;
    let mut client = accumulate_client::AccumulateClient::connect(addr)
        .await
        .unwrap();
    assert_eq!(
        45,
        client
            .sum(tonic::Request::new(futures::stream::iter(0..10)))
            .await
            .unwrap()
            .into_inner()
    );
}

#[tonic_rpc(json)]
trait Store {
    #[client_streaming]
    fn store(req: String) -> (); // Change this to handle no return type?
}

#[derive(Clone)]
struct StoreState {
    store: Arc<Mutex<Vec<String>>>,
    finished: mpsc::Sender<()>,
}

#[tonic::async_trait]
impl store_server::Store for StoreState {
    async fn store(
        &self,
        args: tonic::Request<tonic::Streaming<String>>,
    ) -> Result<tonic::Response<()>, tonic::Status> {
        let mut requests = args.into_inner();
        let store_copy = self.clone();
        tokio::spawn(async move {
            let mut count = 0;
            while let Some(request) = requests.message().await.unwrap() {
                store_copy.store.lock().unwrap().push(request);
                count += 1;
                if count == 3 {
                    store_copy.finished.send(()).await.unwrap();
                }
            }
        });

        Ok(tonic::Response::new(()))
    }
}

#[tokio::test]
async fn test_client_stream_immediate_response() {
    let store = Arc::new(Mutex::new(vec![]));
    let (finished_tx, mut finished_rx) = mpsc::channel(1);
    let addr = util::run_server(store_server::StoreServer::new(StoreState {
        store: Arc::clone(&store),
        finished: finished_tx,
    }))
    .await;
    let mut client = store_client::StoreClient::connect(addr).await.unwrap();
    let (tx, rx) = mpsc::channel(1);
    client.store(ReceiverStream::new(rx)).await.unwrap();
    tx.send("foo".to_string()).await.unwrap();
    tx.send("bar".to_string()).await.unwrap();
    tx.send("baz".to_string()).await.unwrap();
    finished_rx.recv().await.unwrap();
    assert_eq!(
        vec!["foo".to_string(), "bar".to_string(), "baz".to_string()],
        *store.lock().unwrap()
    );
}
