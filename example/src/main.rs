mod example;

#[tokio::main]
async fn main() {
    example::run_server().await
}
