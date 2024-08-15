use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug)]
pub enum Message {
    VoteRequest(String),
    LeaderAnnouncement(String),
    Vote(String),
    Error(String),
    Ping(String),
    Set(String, String),
    Delete(String),
    Get(String),
}