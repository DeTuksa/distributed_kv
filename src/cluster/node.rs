use tokio::{io::{AsyncReadExt, AsyncWriteExt}, net::{TcpListener, TcpStream}};
use std::sync::Arc;
use crate::kv_store::store::KeyValueStore;
use std::error::Error;

pub struct Node {
    pub id: String,
    pub addr: String,
    store: Arc<KeyValueStore>
}

impl Node {
    pub fn new(id: &str, addr: &str) -> Self {
        Node {
            id: id.to_string(),
            addr: addr.to_string(),
            store: Arc::new(KeyValueStore::new())
        }
    }

    pub async fn start(&self) -> Result<(), Box<dyn Error>> {
        let listener = TcpListener::bind(&self.addr).await?;
        println!("Node {} listening on {}", self.id, self.addr);

        loop {
            let (stream, _) = listener.accept().await?;
            let store = Arc::clone(&self.store);
            tokio::spawn(async move {
                connect(stream, store).await;
            });
        }
    }

    pub async fn connect_to_node(
        &self, addr: &str
    ) -> Result<(), Box<dyn Error>> {
        let mut stream = TcpStream::connect(addr).await?;
        let message = format!("Pinging you from node {}", self.id);
        stream.write_all(message.as_bytes()).await?;
        Ok(())
    }
}

async fn connect(
    mut stream: TcpStream,
    store: Arc<KeyValueStore>
) {
    let mut buf = [0; 1024];
    loop {
        match stream.read(&mut buf).await {
            Ok(0) => return ,
            Ok(n) => {
                let req = String::from_utf8_lossy(&buf[0..n]);
                println!("Received request: {}", req);
            }
            Err(e) => {
                eprintln!("Failed to read from stream: {:?}", e);
                return;
            }
        }
    }
}