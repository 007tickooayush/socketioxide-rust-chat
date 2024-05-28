use std::collections::{HashMap, VecDeque};
use tokio::sync::RwLock;
use crate::db::DB;
use crate::model::Message;

pub type RoomStore = HashMap<String, VecDeque<Message>>;

/// Utilizing the RwLock to store the messages in the room and using the DB instance as well to store messages for longer durations
/// *This is a shared state between the WebSocket handlers*
/// *The **tokio::sync::RwLock is used** to ensure that the messages are not accessed concurrently* and we have not used the std::sync::RwLock because it is not async
/// and also implements the system default mechanism for priority of the threads
#[derive(Debug)]
pub struct SocketState {
    pub db: DB,
    pub messages: RwLock<RoomStore>
}

impl SocketState {

    /// Create a new instance of the SocketState
    pub fn new(db: DB) -> Self {
        Self {
            db,
            messages: RwLock::new(RoomStore::new())
        }
    }

    /// push the messages to top of the queue and insert the message to the database
    pub async fn insert(&self, room: &String, message: Message) {
        let mut _messages = self.messages.write().await;
        let _room = _messages.entry(room.clone()).or_default();
        _room.push_front(message.clone());
        self.db.insert_message(room, message).await.unwrap();
    }

    /// get the messages from the room but not read from the db
    pub async fn get_messages(&self, room: &str) -> Vec<Message> {
        let _messages = self.messages.read().await;
        let _room = _messages.get(room).cloned().unwrap_or_default();
        _room.into_iter().rev().collect()
    }
}