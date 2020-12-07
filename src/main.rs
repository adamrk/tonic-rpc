use std::sync::Mutex;

use structopt::StructOpt;
use tonic::{transport::Server, Request, Response, Status};

mod counter_from_codegen;
mod json_codec;

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

async fn run_server(port: u32) {
    let addr = format!("[::1]:{}", port).parse().unwrap();

    Server::builder()
        .add_service(counter_from_codegen::count_server::CountServer::new(State(
            Mutex::new(0),
        )))
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
            let mut client = counter_from_codegen::count_client::CountClient::connect(format!(
                "http://[::1]:{}",
                port
            ))
            .await
            .expect("Failed to connect");

            let request = tonic::Request::new(counter_from_codegen::CountRequest { amount: inc });
            let response = client.count(request).await.expect("Failed to send request");
            println!("Response: {}", response.into_inner().total)
        }
    }
}
