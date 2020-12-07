use std::marker::PhantomData;
use std::pin::Pin;
use std::sync::Mutex;

use bytes::{Buf, BufMut};
use counter::count_client::CountClient;
use counter::count_server::{Count, CountServer};
use counter::{CountRequest, CountResponse};
use serde::{Deserialize, Serialize};
use structopt::StructOpt;
use tokio_serde::{Deserializer, Serializer};
use tonic::{transport::Server, Request, Response, Status};

#[derive(Default, Clone, Copy)]
struct MyEncoder<T> {
    _pd: PhantomData<T>,
}

impl<T> tonic::codec::Encoder for MyEncoder<T>
where
    T: Serialize + Unpin,
{
    type Item = T;
    type Error = tonic::Status;
    fn encode(
        &mut self,
        item: Self::Item,
        dst: &mut tonic::codec::EncodeBuf<'_>,
    ) -> Result<(), Self::Error> {
        let mut serializer: tokio_serde::formats::Json<T, T> =
            tokio_serde::formats::Json::default();
        let bytes = Pin::new(&mut serializer)
            .serialize(&item)
            .map_err(|serde_err| {
                Status::internal(format!("Error in serde deserialize {}", serde_err))
            })?;
        Ok(dst.put(bytes))
    }
}

#[derive(Default, Clone, Copy)]
struct MyDecoder<T> {
    _pd: PhantomData<T>,
}

impl<T> tonic::codec::Decoder for MyDecoder<T>
where
    T: for<'a> Deserialize<'a> + Unpin,
{
    type Item = T;
    type Error = tonic::Status;
    fn decode(
        &mut self,
        src: &mut tonic::codec::DecodeBuf<'_>,
    ) -> Result<Option<Self::Item>, Self::Error> {
        let mut deserializer: tokio_serde::formats::Json<T, T> =
            tokio_serde::formats::Json::default();
        println!("{:?}", src);
        let mut bytes = bytes::BytesMut::new();
        bytes.extend_from_slice(&src.to_bytes());
        println!("{:?}", bytes);
        let result = Pin::new(&mut deserializer)
            .deserialize(&bytes)
            .map_err(|serde_err| Status::internal(format!("Error deserializing {}", serde_err)))?;
        Ok(Some(result))
    }
}

struct MyCodec<T, U> {
    _pd: PhantomData<(T, U)>,
}

impl<T, U> Default for MyCodec<T, U> {
    fn default() -> Self {
        Self { _pd: PhantomData }
    }
}

impl<T, U> tonic::codec::Codec for MyCodec<T, U>
where
    T: Send + Sync + Serialize + Unpin + 'static,
    U: Send + Sync + for<'a> Deserialize<'a> + Unpin + 'static,
{
    type Encode = T;
    type Decode = U;
    type Encoder = MyEncoder<T>;
    type Decoder = MyDecoder<U>;
    fn encoder(&mut self) -> MyEncoder<T> {
        MyEncoder { _pd: PhantomData }
    }
    fn decoder(&mut self) -> MyDecoder<U> {
        MyDecoder { _pd: PhantomData }
    }
}

pub mod counter {
    use serde::{Deserialize, Serialize};

    #[derive(Clone, PartialEq, Serialize, Deserialize)]
    pub struct CountRequest {
        pub amount: i32,
    }
    #[derive(Clone, PartialEq, Serialize, Deserialize)]
    pub struct CountResponse {
        pub total: i32,
    }
    #[doc = r" Generated client implementations."]
    pub mod count_client {
        #![allow(unused_variables, dead_code, missing_docs)]
        use tonic::codegen::*;
        pub struct CountClient<T> {
            inner: tonic::client::Grpc<T>,
        }
        impl CountClient<tonic::transport::Channel> {
            #[doc = r" Attempt to create a new client by connecting to a given endpoint."]
            pub async fn connect<D>(dst: D) -> Result<Self, tonic::transport::Error>
            where
                D: std::convert::TryInto<tonic::transport::Endpoint>,
                D::Error: Into<StdError>,
            {
                let conn = tonic::transport::Endpoint::new(dst)?.connect().await?;
                Ok(Self::new(conn))
            }
        }
        impl<T> CountClient<T>
        where
            T: tonic::client::GrpcService<tonic::body::BoxBody>,
            T::ResponseBody: Body + HttpBody + Send + 'static,
            T::Error: Into<StdError>,
            <T::ResponseBody as HttpBody>::Error: Into<StdError> + Send,
        {
            pub fn new(inner: T) -> Self {
                let inner = tonic::client::Grpc::new(inner);
                Self { inner }
            }
            pub fn with_interceptor(inner: T, interceptor: impl Into<tonic::Interceptor>) -> Self {
                let inner = tonic::client::Grpc::with_interceptor(inner, interceptor);
                Self { inner }
            }
            pub async fn count(
                &mut self,
                request: impl tonic::IntoRequest<super::CountRequest>,
            ) -> Result<tonic::Response<super::CountResponse>, tonic::Status> {
                self.inner.ready().await.map_err(|e| {
                    tonic::Status::new(
                        tonic::Code::Unknown,
                        format!("Service was not ready: {}", e.into()),
                    )
                })?;
                let codec = crate::MyCodec::default();
                let path = http::uri::PathAndQuery::from_static("/counter.Count/Count");
                self.inner.unary(request.into_request(), path, codec).await
            }
        }
        impl<T: Clone> Clone for CountClient<T> {
            fn clone(&self) -> Self {
                Self {
                    inner: self.inner.clone(),
                }
            }
        }
        impl<T> std::fmt::Debug for CountClient<T> {
            fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                write!(f, "CountClient {{ ... }}")
            }
        }
    }
    #[doc = r" Generated server implementations."]
    pub mod count_server {
        #![allow(unused_variables, dead_code, missing_docs)]
        use tonic::codegen::*;
        #[doc = "Generated trait containing gRPC methods that should be implemented for use with CountServer."]
        #[async_trait]
        pub trait Count: Send + Sync + 'static {
            async fn count(
                &self,
                request: tonic::Request<super::CountRequest>,
            ) -> Result<tonic::Response<super::CountResponse>, tonic::Status>;
        }
        #[derive(Debug)]
        pub struct CountServer<T: Count> {
            inner: _Inner<T>,
        }
        struct _Inner<T>(Arc<T>, Option<tonic::Interceptor>);
        impl<T: Count> CountServer<T> {
            pub fn new(inner: T) -> Self {
                let inner = Arc::new(inner);
                let inner = _Inner(inner, None);
                Self { inner }
            }
            pub fn with_interceptor(inner: T, interceptor: impl Into<tonic::Interceptor>) -> Self {
                let inner = Arc::new(inner);
                let inner = _Inner(inner, Some(interceptor.into()));
                Self { inner }
            }
        }
        impl<T, B> Service<http::Request<B>> for CountServer<T>
        where
            T: Count,
            B: HttpBody + Send + Sync + 'static,
            B::Error: Into<StdError> + Send + 'static,
        {
            type Response = http::Response<tonic::body::BoxBody>;
            type Error = Never;
            type Future = BoxFuture<Self::Response, Self::Error>;
            fn poll_ready(&mut self, _cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
                Poll::Ready(Ok(()))
            }
            fn call(&mut self, req: http::Request<B>) -> Self::Future {
                let inner = self.inner.clone();
                match req.uri().path() {
                    "/counter.Count/Count" => {
                        #[allow(non_camel_case_types)]
                        struct CountSvc<T: Count>(pub Arc<T>);
                        impl<T: Count> tonic::server::UnaryService<super::CountRequest> for CountSvc<T> {
                            type Response = super::CountResponse;
                            type Future = BoxFuture<tonic::Response<Self::Response>, tonic::Status>;
                            fn call(
                                &mut self,
                                request: tonic::Request<super::CountRequest>,
                            ) -> Self::Future {
                                let inner = self.0.clone();
                                let fut = async move { (*inner).count(request).await };
                                Box::pin(fut)
                            }
                        }
                        let inner = self.inner.clone();
                        let fut = async move {
                            let interceptor = inner.1.clone();
                            let inner = inner.0;
                            let method = CountSvc(inner);
                            let codec = crate::MyCodec::default();
                            let mut grpc = if let Some(interceptor) = interceptor {
                                tonic::server::Grpc::with_interceptor(codec, interceptor)
                            } else {
                                tonic::server::Grpc::new(codec)
                            };
                            let res = grpc.unary(method, req).await;
                            Ok(res)
                        };
                        Box::pin(fut)
                    }
                    _ => Box::pin(async move {
                        Ok(http::Response::builder()
                            .status(200)
                            .header("grpc-status", "12")
                            .body(tonic::body::BoxBody::empty())
                            .unwrap())
                    }),
                }
            }
        }
        impl<T: Count> Clone for CountServer<T> {
            fn clone(&self) -> Self {
                let inner = self.inner.clone();
                Self { inner }
            }
        }
        impl<T: Count> Clone for _Inner<T> {
            fn clone(&self) -> Self {
                Self(self.0.clone(), self.1.clone())
            }
        }
        impl<T: std::fmt::Debug> std::fmt::Debug for _Inner<T> {
            fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                write!(f, "{:?}", self.0)
            }
        }
        impl<T: Count> tonic::transport::NamedService for CountServer<T> {
            const NAME: &'static str = "counter.Count";
        }
    }
}

struct State(Mutex<i32>);

#[tonic::async_trait]
impl Count for State {
    async fn count(
        &self,
        request: Request<CountRequest>,
    ) -> Result<Response<CountResponse>, Status> {
        let mut locked = self.0.lock().unwrap();
        *locked += request.into_inner().amount;
        let reply = CountResponse { total: *locked };
        Ok(Response::new(reply))
    }
}

async fn run_server(port: u32) {
    let addr = format!("[::1]:{}", port).parse().unwrap();

    Server::builder()
        .add_service(CountServer::new(State(Mutex::new(0))))
        .serve(addr)
        .await
        .expect("Error serving")
}

#[derive(Debug, StructOpt)]
enum Args {
    Server {
        #[structopt(long = "port", default_value = "50051")]
        port: u32,
    },
    Inc {
        #[structopt(long = "port", default_value = "50051")]
        port: u32,
        #[structopt(long = "inc", default_value = "1")]
        inc: i32,
    },
}

#[tokio::main]
async fn main() {
    let args = Args::from_args();
    match args {
        Args::Server { port } => run_server(port).await,
        Args::Inc { port, inc } => {
            let mut client = CountClient::connect(format!("http://[::1]:{}", port))
                .await
                .expect("Failed to connect");

            let request = tonic::Request::new(CountRequest { amount: inc });
            let response = client.count(request).await.expect("Failed to send request");
            println!("Response: {}", response.into_inner().total)
        }
    }
}
