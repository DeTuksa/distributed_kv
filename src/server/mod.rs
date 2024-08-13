pub mod handler;

use tokio::net::TcpListener;
use std::error::Error;

pub async fn start(address: &str) -> Result<(), Box<dyn Error>> {
    let listener = TcpListener::bind(&address).await?;
    println!("Server listening on {}", address);

    loop {
        let (socket, _) = listener.accept().await?;
        tokio::spawn(async move {
            handler::handle_connection(socket).await;
        });
    }
}