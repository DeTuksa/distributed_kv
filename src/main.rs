mod server;

#[tokio::main]
async fn main() {
    if let Err(e) = server::start("127.0.0.1:6379").await {
        eprintln!("Server error: {}", e);
    }
}
