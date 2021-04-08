#[cfg(feature = "json")]
#[tokio::main]
async fn main() {
    example::run_server()
}

#[cfg(not(feature = "json"))]
fn main() {}
