use tokio::{io::{AsyncReadExt, AsyncWriteExt}, net::TcpStream};

pub async fn handle_connection(
    mut socket: TcpStream
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