pub mod handler;

use tokio::net::TcpListener;
use std::{error::Error, sync::Arc};

use crate::kv_store::store::KeyValueStore;

pub async fn start(address: &str) -> Result<(), Box<dyn Error>> {
    let listener = TcpListener::bind(&address).await?;
    println!("Server listening on {}", address);

    let store = Arc::new(KeyValueStore::new());
    loop {
        let (socket, _) = listener.accept().await?;
        let store_clone = store.clone();
        tokio::spawn(async move {
            handler::handle_connection(socket, &store_clone).await;
        });
    }
}