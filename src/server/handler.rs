use tokio::{io::{AsyncReadExt, AsyncWriteExt}, net::TcpStream};
use std::sync::Arc;
use crate::kv_store::store::KeyValueStore;

pub async fn handle_connection(
    mut socket: TcpStream,
    store: &Arc<KeyValueStore>
) {
    let mut buf = [0; 1024];
    loop {
        match socket.read(&mut buf).await {
            Ok(0) => return ,
            Ok(n) => {
                let request = String::from_utf8_lossy(&buf[0..n]);
                let response = handle_request(request.to_string(), &store).await;

                if let Err(e) = socket.write_all(response.as_bytes()).await {
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

async fn handle_request(
    request: String,
    store: &Arc<KeyValueStore>
) -> String {
    let parts: Vec<&str> = request.trim().split_whitespace().collect();
    match parts.as_slice() {
        ["SET", key, value] => {
            store.set(key.to_string(), value.to_string()).await;
            "OK\n".to_string()
        }
        ["GET", key] => {
            if let Some(value) = store.get(key.to_string()).await {
                format!("Value: {}\n", value)
            } else {
                "None\n".to_string()
            }
        }
        ["DELETE", key] => {
            store.delete(key).await;
            "OK\n".to_string()
        }
        _ => "Invalid request\n".to_string()
    }
}