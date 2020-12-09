use std::sync::Mutex;

use tonic::{transport::Server, Request, Response, Status};

pub mod counter_from_codegen;
pub mod json_codec;

/* pub mod counter_our_codegen {
    use serde::{Deserialize, Serialize};

    #[derive(Clone, PartialEq, Serialize, Deserialize)]
    pub struct CountRequest {
        pub amount: i32,
    }
    #[derive(Clone, PartialEq, Serialize, Deserialize)]
    pub struct CountResponse {
        pub total: i32,
    }

    generate_macro::generate_code!();
} */

struct State(Mutex<i32>);

#[tonic::async_trait]
impl counter_from_codegen::count_server::Count for State {
    async fn count(
        &self,
        request: Request<counter_from_codegen::CountRequest>,
    ) -> Result<Response<counter_from_codegen::CountResponse>, Status> {
        let mut locked = self.0.lock().unwrap();
        *locked += request.into_inner().amount;
        let reply = counter_from_codegen::CountResponse { total: *locked };
        Ok(Response::new(reply))
    }
}

/* #[tonic::async_trait]
impl counter_our_codegen::count_server::Count for State {
    async fn count(
        &self,
        request: Request<counter_our_codegen::CountRequest>,
    ) -> Result<Response<counter_our_codegen::CountResponse>, Status> {
        let mut locked = self.0.lock().unwrap();
        *locked += request.into_inner().amount;
        let reply = counter_our_codegen::CountResponse { total: *locked };
        Ok(Response::new(reply))
    }
}
 */
pub async fn run_server(port: u32) {
    let addr = format!("[::1]:{}", port).parse().unwrap();

    Server::builder()
        .add_service(counter_from_codegen::count_server::CountServer::new(State(
            Mutex::new(0),
        )))
        .serve(addr)
        .await
        .expect("Error serving")
}

/* pub async fn run_our_server(port: u32) {
    let addr = format!("[::1]:{}", port).parse().unwrap();

    Server::builder()
        .add_service(counter_our_codegen::count_server::CountServer::new(State(
            Mutex::new(0),
        )))
        .serve(addr)
        .await
        .expect("Error serving")
}
 */
