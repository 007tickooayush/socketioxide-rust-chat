use std::cell::RefCell;
use bson::oid::ObjectId;
use mongodb::bson::{doc, Document};
use mongodb::{bson, Collection, IndexModel};
use serde::de::DeserializeOwned;
use crate::db_model::{MessageCollection, PrivateMessageCollection, SocketCollection};
use crate::errors::MyError;
use crate::model::{Message, PrivateMessageReq};

/// override the standard result type for the module
type Result<T> = std::result::Result<T, MyError>;

#[derive(Clone, Debug)]
pub struct DB {
    pub sockets_collection: Option<Collection<SocketCollection>>,
    pub messages_collection: Option<Collection<MessageCollection>>,
    pub private_messages_collection: Option<Collection<PrivateMessageCollection>>,
}

impl DB {
    pub async fn connect_mongo() -> Result<DB> {
        let client = mongodb::Client::with_uri_str("mongodb://localhost:27017").await?;
        let db = client.database("socketioxide");
        let sockets_collection = Some(db.collection("sockets"));
        let messages_collection = Some(db.collection("messages"));
        let private_messages_collection = Some(db.collection("private_messages"));

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

        if let Some(private_messages_collection) = &private_messages_collection {
            private_messages_collection.create_index(socket_idx.borrow().clone(), None).await?;
        }

        Ok(DB {
            sockets_collection,
            messages_collection,
            private_messages_collection
        })
    }
    pub fn message_to_doc(&self, message: &Message) -> Result<MessageCollection> {
        Ok(MessageCollection {
            id: mongodb::bson::oid::ObjectId::new(),
            room: String::from(&message.room),
            message: String::from(&message.message),
            created_at: message.date_time,
            updated_at: chrono::Utc::now(),
        })
    }

    #[allow(dead_code)]
    pub fn doc_to_message(&self, doc: Document) -> Result<Message> {
        Ok(Message {
            room: doc.get_str("room").unwrap().to_string(),
            message: doc.get_str("message").unwrap().to_string(),
            date_time: doc.get_datetime("created_at").unwrap().to_chrono(),
        })
    }
}

impl DB {
    pub async fn insert_message(&self, message: Message) -> Result<MessageCollection> {
        if let Some(collection) = &self.messages_collection {
            let doc = self.message_to_doc(&message).unwrap();
            let insert_res = match collection.insert_one(&doc, None).await {
                Ok(res) => res,
                Err(e) => return Err(MyError::MongoError(e))
            };

            let oid = insert_res.inserted_id.as_object_id().unwrap();

            let resp = self.find_message_oid(oid).await?;
            Ok(resp)
        } else {
            Err(MyError::OwnError(String::from("Messages collection not found")))
        }
    }

    pub async fn insert_private_message(&self, message_collection: PrivateMessageCollection) -> Result<PrivateMessageCollection> {
        if let Some(collection) = &self.private_messages_collection {
            let insert_res = match collection.insert_one(&message_collection, None).await {
                Ok(res) => res,
                Err(e) => return Err(MyError::MongoError(e))
            };
            let oid = insert_res.inserted_id.as_object_id().unwrap();

            let verified = self.find_private_message_oid(oid).await?;

            Ok(verified)
        } else {
            Err(MyError::OwnError(String::from("Messages collection not found")))
        }
    }

    pub async fn find_message_oid(&self, oid:ObjectId) -> Result<MessageCollection> {
        self.find_msg_by_oid(&self.messages_collection, oid).await
    }
    pub async fn find_private_message_oid(&self, oid: ObjectId) -> Result<PrivateMessageCollection> {
        self.find_msg_by_oid(&self.private_messages_collection, oid).await
    }

    pub async fn find_msg_by_oid<T>(&self, collection: &Option<Collection<T>>, oid: ObjectId) -> Result<T>
    where
        T: DeserializeOwned + Unpin + Send + Sync
    {
        if let Some(collection) = &collection {
            let resp = match collection.find_one(doc! {"_id": oid}, None).await {
                Ok(res) => {
                    match res {
                        Some(doc) => doc,
                        None => return Err(MyError::OwnError(String::from("Message not found")))
                    }
                },
                Err(e) => return Err(MyError::MongoError(e))
            };
            Ok(resp)
        } else {
            Err(MyError::OwnError(String::from("Messages collection not found")))
        }
    }
}