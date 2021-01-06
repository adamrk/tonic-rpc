use tonic_rpc::tonic_rpc;

mod util;

#[tonic_rpc(cbor)]
trait Units {
    fn send(arg: ());
    fn heartbeat();
}

struct State;

#[tonic::async_trait]
impl units_server::Units for State {
    async fn send(&self, _: tonic::Request<()>) -> Result<tonic::Response<()>, tonic::Status> {
        Ok(tonic::Response::new(()))
    }

    async fn heartbeat(&self, _: tonic::Request<()>) -> Result<tonic::Response<()>, tonic::Status> {
        Ok(tonic::Response::new(()))
    }
}

#[tokio::test]
async fn test_unit_in_sig() {
    let addr = util::run_server(units_server::UnitsServer::new(State)).await;
    let mut client = units_client::UnitsClient::connect(addr)
        .await
        .expect("Failed to connect");

    assert_eq!((), client.send(()).await.unwrap().into_inner());
    assert_eq!((), client.heartbeat(()).await.unwrap().into_inner());
}
