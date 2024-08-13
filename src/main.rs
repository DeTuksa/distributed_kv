use tokio::{io::{AsyncReadExt, AsyncWriteExt}, net::TcpListener};
use std::error::Error;

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    let address = "127.0.0.1:6379".to_string();
    let listener = TcpListener::bind(&address).await?;
    println!("Server listening on {}", address);

    loop {
        let (socket, _) = listener.accept().await?;
        tokio::spawn(async move {
            handle_connection(socket).await;
        });
    }
}


async fn handle_connection(
    mut socket: tokio::net::TcpStream
) {
    let mut buf = [0; 1024];
    loop {
        match socket.read(&mut buf).await {
            Ok(0) => return ,
            Ok(n) => {
                if let Err(e) = socket.write_all(&buf[0..n]).await {
                    eprintln!("Failed to write to socket; err = {:?}", e);
                    return;
                }
            },
            Err(e) => {
                eprintln!("Failed to read from socket; error = {:?}", e);
                return;
            }
        }
    }
}