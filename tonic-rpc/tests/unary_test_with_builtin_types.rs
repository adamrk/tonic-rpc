use tonic_rpc::tonic_rpc;

#[tonic_rpc(json)]
trait Math {
    fn add(args: (i32, i32)) -> i32;
    fn geq(args: (f64, f64)) -> bool;
}

type State = ();

#[tonic::async_trait]
impl math_server::Math for State {
    async fn add(
        &self,
        request: tonic::Request<(i32, i32)>,
    ) -> Result<tonic::Response<i32>, tonic::Status> {
        let (x, y) = request.into_inner();
        Ok(tonic::Response::new(x + y))
    }

    async fn geq(
        &self,
        request: tonic::Request<(f64, f64)>,
    ) -> Result<tonic::Response<bool>, tonic::Status> {
        let (x, y) = request.into_inner();
        Ok(tonic::Response::new(x > y))
    }
}

pub async fn run_server() -> u16 {
    let mut listener = tokio::net::TcpListener::bind("[::1]:0").await.unwrap();
    let port = listener.local_addr().unwrap().port();

    tokio::spawn(async move {
        tonic::transport::Server::builder()
            .add_service(math_server::MathServer::new(()))
            .serve_with_incoming(listener.incoming())
            .await
            .unwrap();
    });
    port
}

#[tokio::test]
async fn test_math_with_builtins() {
    let port = run_server().await;
    // Wait for server to start
    tokio::time::delay_for(std::time::Duration::from_millis(1)).await;
    let mut client = math_client::MathClient::connect(format!("http://[::1]:{}", port))
        .await
        .expect("Failed to connect");

    let request = tonic::Request::new((42i32, 35i32));
    let response = client.add(request).await.expect("Failed to send request");
    assert_eq!(77, response.into_inner());
    let request = tonic::Request::new((23.1, 0.01));
    let response = client.geq(request).await.expect("Failed to send request");
    assert_eq!(true, response.into_inner());
}
