use std::collections::{HashMap, VecDeque};
use socketioxide::extract::SocketRef;
use tokio::sync::RwLock;
use crate::db::DB;
use crate::db_model::{PrivateMessageCollection};
use crate::model::{InPrivate, Message, PrivateMessage, PrivateMessageReq, User, UserResp};

pub type RoomStore = HashMap<String, VecDeque<Message>>;
// pub type SocketMap = HashMap<String, String>;

/// Utilizing the RwLock to store the messages in the room and using the DB instance as well to store messages for longer durations
/// *This is a shared state between the WebSocket handlers*
/// *The **tokio::sync::RwLock is used** to ensure that the messages are not accessed concurrently* and we have not used the std::sync::RwLock because it is not async
/// and also implements the system default mechanism for priority of the threads
#[derive(Debug)]
pub struct SocketState {
    pub db: DB,
    pub messages: RwLock<RoomStore>,
    // pub socket_map: RwLock<SocketMap>,
}

impl SocketState {
    /// Create a new instance of the SocketState
    pub fn new(db: DB) -> Self {
        Self {
            db,
            messages: RwLock::new(RoomStore::new()),
            // socket_map: RwLock::new(SocketMap::new()),
        }
    }

    /// Remove the connected user from all the other rooms expect the room created with the username<br/>
    /// That room is used for receiving messages specifically from the other users,
    /// that have the knowledge of the user with the specific username being active and connected to server
    pub async fn leave_all_expect_one(&self, _socket: SocketRef, room_to_stay: String) {
        let rooms = _socket.rooms().ok();
        if let Some(rooms) = rooms {
            for room in rooms {
                if room.ne(&room_to_stay) {
                    _socket.leave(room).ok();
                }
            }
        }
    }

    /// Remove the socket from memory and DB once the socket disconnects from the server
    pub async fn remove_socket(&self, user: User) {
        // let mut _socket_map = self.socket_map.write().await;
        // _socket_map.retain(|_, v| v.as_str().ne(&socket_id));

        // info!("socket_map: {:?}", _socket_map);
        self.db.remove_socket(user).await;
    }

    // /// Insert the socket id and the name into the socket map into `sockets_collection`
    // /// ALSO Maintain a HashMap<String,String> for the socket id and the name
    // pub async fn insert_socket_name(&self, socket_id: String) -> (String, String) {
    //     // GENERATE A RANDOM NAME FOR THE SOCKET ID AND INSERT INTO THE DB
    //     let name = names::Generator::default().next().unwrap();
    //
    //     let mut _socket_map = self.socket_map.write().await;
    //     _socket_map.insert(socket_id.clone(), name.clone());
    //     // self.socket_map.write().await.insert(name.clone(), socket_id.clone());
    //
    //     self.db.insert_socket_name(name.clone(), socket_id.clone()).await.unwrap();
    //
    //     (name, socket_id)
    // }

    /// push the messages to top of the queue and insert the message to the database
    pub async fn insert(&self, room: &String, message: Message) {
        let mut _messages = self.messages.write().await;
        let _room = _messages.entry(room.clone()).or_default();
        _room.push_front(message.clone());
        self.db.insert_message(message).await.unwrap();
    }

    /// get the messages from the room but not read from the db <br/>
    /// if the DB is empty then fetch the messages from the stored state <br/>
    /// Give first priority to the messages stored in DB but return in the same truncated limit (20) format
    pub async fn get_messages(&self, room: &str) -> Vec<Message> {
        let _messages = self.messages.read().await;
        let messages_db = self.db.get_messages(Some(String::from(room))).await.unwrap();

        // let messages_db;
        // if !room.is_empty() {
        //     messages_db = self.db.get_messages(Some(String::from(room))).await?;
        // } else {
        //     messages_db = self.db.get_messages(None).await.unwrap();
        // }

        return if let Some(messages) = messages_db {
            messages.clone()
        } else {
            let _room = _messages.get(room).cloned().unwrap_or_default();
            _room.into_iter().rev().collect()
        };
    }

    pub async fn insert_private_messages(&self, message: PrivateMessageReq) -> PrivateMessage {
        let sender = message.sender.clone().unwrap_or_else(|| "".to_string());

        let private_msg = PrivateMessageCollection {
            id: bson::oid::ObjectId::new(),
            sender,
            message: message.message.clone(),
            receiver: message.receiver.clone(),
            created_at: chrono::Utc::now(),
            updated_at: chrono::Utc::now(),
        };
        let resp = self.db.insert_private_message(private_msg).await.unwrap_or_else(|_| {
            PrivateMessageCollection {
                id: bson::oid::ObjectId::new(),
                sender: "".to_string(),
                receiver: "".to_string(),
                message: String::from("Error: Message not sent!"),
                created_at: chrono::Utc::now(),
                updated_at: chrono::Utc::now(),
            }
        });

        PrivateMessage {
            message: resp.message,
            sender: resp.sender,
            receiver: resp.receiver,
            date_time: resp.created_at,
        }
    }

    pub async fn handle_user(&self, user: User) -> UserResp {
        let resp = self.db.handle_user(user).await.unwrap();

        UserResp {
            owned_uname: resp.owned_uname,
            cur_gen_uname: resp.cur_gen_uname,
            updated_at: resp.updated_at,
            created_at: resp.created_at,
        }
    }

    /// Implementation for handling the event when the user is in the private window for chat
    pub async fn handle_private_joined(&self, user: InPrivate) -> InPrivate {
        let res = self.db.handle_private_joined(user).await.unwrap();
        InPrivate {
            in_private: res.in_private,
            username: res.owned_username,
        }
    }

    /// Implementation for handling the event when the user is in the private window for chat
    pub async fn handle_private_left(&self, user: InPrivate) -> InPrivate {
        let res = self.db.handle_private_left(user).await.unwrap();
        InPrivate {
            in_private: res.in_private,
            username: res.owned_username,
        }
    }
}