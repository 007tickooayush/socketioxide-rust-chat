use std::cell::RefCell;
use mongodb::bson::doc;
use mongodb::{Collection, IndexModel};
use crate::db_model::{MessageCollection, SocketCollection};
use crate::errors::MyError;
use crate::model::Message;

/// override the standard result type for the module
type Result<T> = std::result::Result<T, MyError>;

#[derive(Clone, Debug)]
pub struct DB {
    pub sockets_collection: Option<Collection<SocketCollection>>,
    pub messages_collection: Option<Collection<MessageCollection>>,
}

impl Default for DB {
    fn default() -> DB {
        DB {
            sockets_collection: None,
            messages_collection: None,
        }
    }
}
impl DB {
    pub async fn connect_mongo() -> Result<DB> {
        let client = mongodb::Client::with_uri_str("mongodb://localhost:27017").await?;
        let db = client.database("socketioxide");
        let sockets_collection = Some(db.collection("sockets"));
        let messages_collection = Some(db.collection("messages"));

        let socket_idx = RefCell::new(IndexModel::builder()
            .keys(doc! {"_id": 1})
            .build());
        // sockets_collection.create_index(doc! {"id": 1}, None).await?;
        if let Some(sockets_collection) = &sockets_collection {
            sockets_collection.create_index(socket_idx.borrow().clone(), None).await?;
        }
        if let Some(messages_collection) = &messages_collection {
            messages_collection.create_index(socket_idx.borrow().clone(), None).await?;
        }

        Ok(DB {
            sockets_collection,
            messages_collection,
        })
    }
    pub fn message_to_doc(&self, message: Message) -> MessageCollection {
        MessageCollection {
            id: mongodb::bson::oid::ObjectId::new(),
            room: message.room,
            message: message.message,
            created_at: message.date_time,
            updated_at: chrono::Utc::now(),
        }
    }
}

impl DB {
    pub async fn insert_message(&self, room: &str, message: Message) -> Result<()> {
        if let Some(messages_collection) = &self.messages_collection {
            let message = self.message_to_doc(message);
            messages_collection.insert_one(message, None).await?;
        }
        Ok(())
    }
}