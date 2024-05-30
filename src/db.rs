use std::cell::RefCell;
use bson::oid::ObjectId;
use mongodb::bson::{doc, Document};
use mongodb::{bson, Collection, IndexModel};
use mongodb::options::FindOptions;
use serde::de::DeserializeOwned;
use tracing::info;
use crate::db_model::{MessageCollection, PrivateMessageCollection, SocketCollection};
use crate::errors::MyError;
use crate::model::{Message};

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
            private_messages_collection,
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

    pub async fn find_message_oid(&self, oid: ObjectId) -> Result<MessageCollection> {
        self.find_doc_by_oid(&self.messages_collection, oid).await
    }
    pub async fn find_private_message_oid(&self, oid: ObjectId) -> Result<PrivateMessageCollection> {
        self.find_doc_by_oid(&self.private_messages_collection, oid).await
    }

    pub async fn find_doc_by_oid<T>(&self, collection: &Option<Collection<T>>, oid: ObjectId) -> Result<T>
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
                }
                Err(e) => return Err(MyError::MongoError(e))
            };
            Ok(resp)
        } else {
            Err(MyError::OwnError(String::from("Messages collection not found")))
        }
    }

    /// Create a DB entry for socket id mapped to name
    ///
    pub async fn insert_socket_name(&self, username: String, socket: String) -> Result<SocketCollection> {
        if let Some(collection) = &self.sockets_collection {
            let doc = SocketCollection {
                id: ObjectId::new(),
                socket,
                username,
                created_at: chrono::Utc::now(),
                updated_at: chrono::Utc::now(),
            };
            let insert_res = match collection.insert_one(&doc, None).await {
                Ok(res) => res,
                Err(e) => return Err(MyError::MongoError(e))
            };

            let oid = insert_res.inserted_id.as_object_id().unwrap();
            let resp = self.find_doc_by_oid(&self.sockets_collection, oid).await?;

            Ok(resp)
        } else {
            Err(MyError::OwnError(String::from("Messages collection not found")))
        }
    }

    /// get the list of sockets
    pub async fn get_sockets(&self, limit: i64, page: i64) -> Result<Vec<SocketCollection>> {
        if let Some(collection) = &self.sockets_collection {
            let filter = FindOptions::builder()
                .limit(limit)
                .skip(u64::try_from((page - 1) * limit).unwrap())
                .build();

            let mut cursor = match collection.find(None, filter).await {
                Ok(res) => res,
                Err(e) => return Err(MyError::MongoError(e))
            };

            let mut sockets_list = Vec::new();

            // while cursor.advance().await? {
            //     let raw_doc = cursor.current().to_raw_document_buf();
            //     info!("Raw Doc: {:?}", raw_doc);
            //     let d:SocketCollection = bson::from_slice(raw_doc.as_bytes()).unwrap();
            //     info!("Doc: {:?}", d);
            //     // let socket: SocketCollection = bson::de::from_document(doc)?;
            //     // sockets_list.push(socket);
            // }

            while cursor.advance().await? {
                let doc = bson::from_slice::<SocketCollection>(cursor.current().to_raw_document_buf().as_bytes()).unwrap();
                sockets_list.push(SocketCollection {
                    id: doc.id.clone(),
                    socket: doc.socket.clone(),
                    username: doc.username.clone(),
                    created_at: doc.created_at.clone(),
                    updated_at: doc.updated_at.clone(),
                });
            }

            Ok(sockets_list)
        } else {
            Err(MyError::OwnError(String::from("Messages collection not found")))
        }
    }
}