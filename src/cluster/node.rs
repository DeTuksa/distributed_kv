use crate::kv_store::store::KeyValueStore;
use std::collections::HashMap;
use std::error::Error;
use std::sync::Arc;
use tokio::{
    io::{AsyncReadExt, AsyncWriteExt},
    net::{TcpListener, TcpStream},
    sync::Mutex
};
use serde_json;

use super::message::Message;

pub struct Node {
    pub id: String,
    pub addr: String,
    pub is_leader: Arc<Mutex<bool>>,
    store: Arc<KeyValueStore>,
    known_nodes: Arc<Mutex<HashMap<String, String>>>,
    leader_id: Arc<Mutex<Option<String>>>,
    votes: Arc<Mutex<i32>>,
    election_in_progress: Arc<Mutex<bool>>
}

impl Node {
    pub fn new(id: &str, addr: &str) -> Self {
        Node {
            id: id.to_string(),
            addr: addr.to_string(),
            is_leader: Arc::new(Mutex::new(false)),
            store: Arc::new(KeyValueStore::new()),
            known_nodes: Arc::new(Mutex::new(HashMap::new())),
            leader_id: Arc::new(Mutex::new(None)),
            votes: Arc::new(Mutex::new(0)),
            election_in_progress: Arc::new(Mutex::new(false))
        }
    }

    pub async fn start(self: Arc<Self>) -> Result<(), Box<dyn Error>> {
        let listener = TcpListener::bind(&self.addr).await?;
        println!("Node {} listening on {}", self.id, self.addr);

        loop {
            let (stream, _) = listener.accept().await?;
            let self_clone = Arc::clone(&self);
            tokio::spawn(async move {
                self_clone.connect(stream).await;
            });
        }
    }

    pub async fn connect_to_node(&self, addr: &str, id: &str) -> Result<(), Box<dyn Error>> {
        self.known_nodes.lock().await.insert(id.to_string(), addr.to_string());
        let message = Message::Ping(self.id.clone());
        self.send_message(addr,&message).await?;
        Ok(())
    }

    pub async fn send_message(&self, addr: &str, message: &Message) -> Result<(), Box<dyn Error>> {
        let mut stream = TcpStream::connect(addr).await?;
        let message = serde_json::to_string(message)?;
        stream.write_all(message.as_bytes()).await?;
        Ok(())
    }

    // pub async fn set_leader(&mut self, is_leader: bool) {
    //     self.is_leader = Arc::new(Mutex::new(is_leader));
    // }

    // pub fn is_leader(&self) -> bool {
    //     tokio::task::block_in_place(|| *self.is_leader.lock())
    // }

    pub async fn start_election(&self)-> Result<(), Box<dyn Error>> {
        {
            let mut in_progress = self.election_in_progress.lock().await;
            if *in_progress {
                return Ok(());
            }
            *in_progress = true;
            println!("Node {} starting election", self.id);
        }

        {
            let mut votes = self.votes.lock().await;
            *votes = 1;
        }

        let known_nodes = self.known_nodes.lock().await;

        for (_, node_addr) in known_nodes.iter() {
            let vote_request = Message::VoteRequest(self.id.clone());
            self.send_message(node_addr, &vote_request).await?;
        }
        // sleep(Duration::from_secs(2)).await;

        let votes = self.votes.lock().await;
        let known_nodes_count = known_nodes.len();

        if *votes > known_nodes_count as i32/2 {
            let mut is_leader = self.is_leader.lock().await;
            *is_leader = true;
            println!("Node {} is now the leader", self.id);

            let mut leader_id = self.leader_id.lock().await;
            *leader_id = Some(self.id.clone());

            for (_, node_addr) in self.known_nodes.lock().await.iter() {
                let leader_announcement = Message::LeaderAnnouncement(self.id.clone());
                self.send_message(node_addr, &leader_announcement).await?;
            }
        } else {
            println!("Node {} did not receive enough votes to become leader", self.id);
        }
        {
            let mut in_progress = self.election_in_progress.lock().await;
            *in_progress = false;
        }
        Ok(())
    }

    pub async fn handle_vote_request(self: Arc<Self>, candidate_id: String) -> Result<(), Box<dyn Error>> {
        let is_leader = self.is_leader.lock().await;
        if !*is_leader {
            drop(is_leader);

            let sender_addr = self.known_nodes.lock().await.get(&candidate_id).unwrap().clone();
            let vote = Message::Vote(self.id.clone());
            self.send_message(&sender_addr, &vote).await?;

            println!("Node {} voted for {}", self.id, candidate_id);
        }
        Ok(())
    }

    pub async fn handle_vote(&self, _sender_id: String) -> Result<(), Box<dyn Error>> {
        println!("Node {} received a vote request", self.id);
        let mut votes = self.votes.lock().await;
        *votes += 1;
        

        Ok(())
    }

    async fn connect(
        self: Arc<Self>,
        mut stream: TcpStream,
    ) {
        let mut buf = [0; 1024];
        loop {
            match stream.read(&mut buf).await {
                Ok(0) => return,
                Ok(n) => {
                    let req = String::from_utf8_lossy(&buf[0..n]);
                    println!("Received request: {}", req);
    
                    let message: Result<Message, _> = serde_json::from_str(&req);
                    if let Ok(msg) = message {
                        match msg {
                            Message::Ping(node_id) => {
                                println!("Received PING from {}", node_id);
                            }
                            Message::VoteRequest(candidate_id) => {
                                println!("Received VOTE REQUEST from {}", candidate_id);
                                
                                let self_clone = Arc::clone(&self);
                                tokio::spawn(async move {
                                    self_clone.handle_vote_request(candidate_id).await.unwrap();
                                });
                            }
                            Message::LeaderAnnouncement(new_leader) => {
                                println!("Received LEADER ANNOUNCEMENT from {}", new_leader);
                                let mut is_leader = self.is_leader.lock().await;
                                *is_leader = false;
                                let mut leader_id = self.leader_id.lock().await;
                                *leader_id = Some(new_leader);
                            }
                            Message::Vote(vote_id) => {
                                println!("Received VOTE from {}", vote_id);
                                let self_clone = Arc::clone(&self);
                                tokio::spawn(async move {
                                    self_clone.handle_vote(vote_id).await.unwrap();
                                });
                            }
                            Message::Set(key, val) => {
                                println!("Received SET operation for key {}", key);
                                let self_clone = Arc::clone(&self);
                                tokio::spawn(async move {
                                    self_clone.handle_replication_message(Message::Set(key, val)).await.unwrap();
                                });
                            }
                            Message::Delete(key) => {
                                println!("Received DELETE operation for key {}", key);
                                let self_clone = Arc::clone(&self);
                                tokio::spawn(async move {
                                    self_clone.handle_replication_message(Message::Delete(key)).await.unwrap();
                                });
                            }
                            Message::Get(key) => {
                                println!("Received GET operation for key {}", key);
                                let self_clone = Arc::clone(&self);
                                tokio::spawn(async move {
                                    self_clone.handle_replication_message(Message::Get(key)).await.unwrap();
                                });
                            }
                            Message::Error(error) => {
                                println!("Error occured {}" , error);
                            }
                        }
                    } else {
                        println!("Unknown message format: {}", req);
                    }
                }
                Err(e) => {
                    eprintln!("Failed to read from stream: {:?}", e);
                    return;
                }
            }
        }
    }

    pub async fn handle_write_request(
        &self, key: String, value: String
    ) -> Result<(), Box<dyn Error>> {
        self.store.set(key.clone(), value.clone()).await;

        let known_nodes = self.known_nodes.lock().await;
        for (_, node_addr) in known_nodes.iter()  {
            let message = Message::Set(key.clone(), value.clone());
            self.send_message(&node_addr, &message).await?;
        }

        Ok(())
    }
    async fn handle_replication_message(
        &self, message: Message
    ) -> Result<(), Box<dyn Error>> {

        match message {
            Message::Set(key, value) => {
                self.store.set(key, value).await;
            }
            Message::Delete(key) => {
                self.store.delete(&key).await;
            }
            Message::Get(key) => {
                self.store.get(key).await;
            }
            _ => {
                return Err("Unknown message type".into());
            }
        }
        Ok(())
    }
}

// impl Node {
//     async fn send_message_to(addr: &str, message: &Message) -> Result<(), Box<dyn Error>> {
//         let mut stream = TcpStream::connect(addr).await?;
//         let message = serde_json::to_string(message)?;
//         stream.write_all(message.as_bytes()).await?;
//         Ok(())
//     }
// }