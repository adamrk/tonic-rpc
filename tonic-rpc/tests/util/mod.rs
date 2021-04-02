use tokio_stream::wrappers::TcpListenerStream;

/// Returns the address to connect to.
pub async fn run_server<S>(svc: S) -> String
where
    S: tonic::codegen::Service<
            tonic::codegen::http::Request<tonic::transport::Body>,
            Response = tonic::codegen::http::Response<tonic::body::BoxBody>,
        > + tonic::transport::NamedService
        + Clone
        + Send
        + 'static,
    S::Future: Send + 'static,
    S::Error: std::error::Error + Send + Sync,
{
    let listener = tokio::net::TcpListener::bind("[::1]:0").await.unwrap();
    let port = listener.local_addr().unwrap().port();

    tokio::spawn(async move {
        tonic::transport::Server::builder()
            .add_service(svc)
            .serve_with_incoming(TcpListenerStream::new(listener))
            .await
            .unwrap();
    });

    // Wait for server to start
    tokio::time::sleep(std::time::Duration::from_millis(1)).await;

    format!("http://[::1]:{}", port)
}
