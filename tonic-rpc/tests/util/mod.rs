use std::error::Error;

use tokio::net::TcpListener;
use tokio_stream::wrappers::TcpListenerStream;
use tonic::{
    body::BoxBody,
    codegen::{
        http::{Request, Response},
        Service,
    },
    transport::{Body, NamedService, Server},
};

/// Returns the address to connect to.
pub async fn run_server<S>(svc: S) -> String
where
    S: Service<Request<Body>, Response = Response<BoxBody>> + NamedService + Clone + Send + 'static,
    S::Future: Send + 'static,
    S::Error: Error + Send + Sync,
{
    let listener = TcpListener::bind("[::1]:0").await.unwrap();
    let port = listener.local_addr().unwrap().port();

    tokio::spawn(async move {
        Server::builder()
            .add_service(svc)
            .serve_with_incoming(TcpListenerStream::new(listener))
            .await
            .unwrap();
    });

    format!("http://[::1]:{}", port)
}
