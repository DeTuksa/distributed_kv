mod server;
mod kv_store;
mod cluster;

use cluster::node::Node;
use tokio::time::{sleep, Duration};

#[tokio::main]
async fn main() {

    let node1 = Node::new("node1", "127.0.0.1:6379");
    let node2 = Node::new("node2", "127.0.0.1:6380");

    tokio::spawn(async move {
        node1.start().await.unwrap();
    });

    sleep(Duration::from_secs(1)).await;

    tokio::spawn(async move {
        node2.connect_to_node("127.0.0.1:6379").await.unwrap();
        node2.start().await.unwrap();
    });

    loop {
        sleep(Duration::from_secs(60)).await;
    }
}
