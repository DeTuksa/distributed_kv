mod server;
mod kv_store;
mod cluster;

use cluster::node::Node;
use std::sync::Arc;
use tokio::time::{sleep, Duration};
use tokio::task;

#[tokio::main]
async fn main() {
    let node1 = Arc::new(Node::new("node1", "127.0.0.1:8080"));
    let node2 = Arc::new(Node::new("node2", "127.0.0.1:8081"));
    let node3 = Arc::new(Node::new("node3", "127.0.0.1:8082"));

    let node1_clone = Arc::clone(&node1);
    task::spawn(async move {
        node1_clone.start().await.unwrap();
    });

    let node2_clone = Arc::clone(&node2);
    task::spawn(async move {
        node2_clone.start().await.unwrap();
    });

    let node3_clone = Arc::clone(&node3);
    task::spawn(async move {
        node3_clone.start().await.unwrap();
    });

    sleep(Duration::from_secs(1)).await;

    node1.connect_to_node("127.0.0.1:8081", "node2").await.unwrap();
    node1.connect_to_node("127.0.0.1:8082", "node3").await.unwrap();

    node2.connect_to_node("127.0.0.1:8080", "node1").await.unwrap();
    node2.connect_to_node("127.0.0.1:8082", "node3").await.unwrap();

    node3.connect_to_node("127.0.0.1:8080", "node1").await.unwrap();
    node3.connect_to_node("127.0.0.1:8081", "node2").await.unwrap();

    let node1_clone = Arc::clone(&node1);
    task::spawn(async move {
        node1_clone.start_election().await.unwrap();
    });

    sleep(Duration::from_secs(2)).await;
    
    let key = "key1".to_string();
    let value = "value1".to_string();
    let node1_clone = Arc::clone(&node1);
    task::spawn(async move {
        node1_clone.handle_write_request(key, value).await.unwrap();
    });

    loop {
        sleep(Duration::from_secs(10)).await;
    }
}
