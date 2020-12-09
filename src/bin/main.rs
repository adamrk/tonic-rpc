use structopt::StructOpt;

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
        Args::Server { port } => tonic_trivial::run_server(port).await,
        Args::Inc { port, inc } => {
            let mut client =
                tonic_trivial::counter_from_codegen::count_client::CountClient::connect(format!(
                    "http://[::1]:{}",
                    port
                ))
                .await
                .expect("Failed to connect");

            let request = tonic::Request::new(tonic_trivial::counter_from_codegen::CountRequest {
                amount: inc,
            });
            let response = client.count(request).await.expect("Failed to send request");
            println!("Response: {}", response.into_inner().total)
        }
    }
}
