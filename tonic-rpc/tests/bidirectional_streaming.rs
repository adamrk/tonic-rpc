use std::{
    collections::HashMap,
    sync::{Arc, Mutex},
};

use tokio::sync::mpsc;
use tokio_stream::wrappers::ReceiverStream;
use tonic_rpc::tonic_rpc;

mod util;

#[tonic_rpc(json)]
trait PubSub {
    #[server_streaming]
    #[client_streaming]
    fn sub(channel: String) -> (String, String);

    fn publish(chanenl: String, value: String) -> ();
}

#[derive(Debug)]
struct State {
    data: Arc<Mutex<HashMap<String, String>>>,
    subscribers:
        Arc<Mutex<HashMap<String, Vec<mpsc::Sender<Result<(String, String), tonic::Status>>>>>>,
}

#[tonic::async_trait]
impl pub_sub_server::PubSub for State {
    type subStream = ReceiverStream<Result<(String, String), tonic::Status>>;

    async fn sub(
        &self,
        channels: tonic::Request<tonic::Streaming<String>>,
    ) -> Result<tonic::Response<Self::subStream>, tonic::Status> {
        let mut channels = channels.into_inner();
        let (tx, rx) = mpsc::channel(20);
        let subscribers = Arc::clone(&self.subscribers);
        let data = Arc::clone(&self.data);
        tokio::spawn(async move {
            while let Some(channel) = channels.message().await.unwrap() {
                let existing_data = data.lock().unwrap().get(&channel).cloned();
                match existing_data {
                    None => {}
                    Some(value) => {
                        tx.send(Ok((channel.clone(), value))).await.unwrap();
                    }
                }
                let mut subscribers = subscribers.lock().unwrap();
                subscribers
                    .entry(channel)
                    .or_insert(vec![])
                    .push(tx.clone());
            }
        });
        Ok(tonic::Response::new(ReceiverStream::new(rx)))
    }

    async fn publish(
        &self,
        kvp: tonic::Request<(String, String)>,
    ) -> Result<tonic::Response<()>, tonic::Status> {
        let (key, value) = kvp.into_inner();
        self.data.lock().unwrap().insert(key.clone(), value.clone());
        let to_send = {
            let subscribers = self.subscribers.lock().unwrap();
            subscribers.get(&key).unwrap_or(&vec![]).clone()
        };
        for subscriber in to_send {
            subscriber
                .send(Ok((key.clone(), value.clone())))
                .await
                .unwrap();
        }
        Ok(tonic::Response::new(()))
    }
}

#[tokio::test]
async fn test_bidirectional() {
    let state = State {
        data: Arc::new(Mutex::new(HashMap::new())),
        subscribers: Arc::new(Mutex::new(HashMap::new())),
    };
    let addr = util::run_server(pub_sub_server::PubSubServer::new(state)).await;
    let mut client = pub_sub_client::PubSubClient::connect(addr)
        .await
        .expect("Error connecting");
    let (tx, rx) = mpsc::channel(10);
    let mut updates = client
        .sub(ReceiverStream::new(rx))
        .await
        .unwrap()
        .into_inner();
    tx.send("foo".to_string()).await.unwrap();
    client
        .publish(("foo".to_string(), "fooval".to_string()))
        .await
        .unwrap();
    client
        .publish(("bar".to_string(), "barval".to_string()))
        .await
        .unwrap();
    assert_eq!(
        ("foo".to_string(), "fooval".to_string()),
        updates.message().await.unwrap().unwrap()
    );
    tx.send("bar".to_string()).await.unwrap();
    assert_eq!(
        ("bar".to_string(), "barval".to_string()),
        updates.message().await.unwrap().unwrap()
    );
    client
        .publish(("foo".to_string(), "fooval2".to_string()))
        .await
        .unwrap();
    assert_eq!(
        ("foo".to_string(), "fooval2".to_string()),
        updates.message().await.unwrap().unwrap()
    );
}
