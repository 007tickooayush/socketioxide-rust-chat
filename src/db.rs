use bson::oid::ObjectId;
use mongodb::bson::{doc, Document};
use mongodb::{bson, Collection, IndexModel};
use mongodb::options::{CountOptions, FindOptions};
use serde::de::DeserializeOwned;
use tracing::info;
use crate::db_model::{MessageCollection, PrivateMessageCollection, SocketCollection};
use crate::errors::MyError;
use crate::model::{Message, PaginationResponse, SocketResponse};

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

        let id_idx = IndexModel::builder()
            .keys(doc! {"_id": 1})
            .build();

        let created_at_idx = IndexModel::builder()
            .keys(doc! {"created_at": 1})
            .build();

        if let Some(sockets_collection) = &sockets_collection {
            sockets_collection.create_index(id_idx.clone(), None).await?;
            sockets_collection.create_index(created_at_idx.clone(), None).await?;
        }

        if let Some(messages_collection) = &messages_collection {
            messages_collection.create_index(id_idx.clone(), None).await?;
            messages_collection.create_index(created_at_idx.clone(), None).await?;
        }

        if let Some(private_messages_collection) = &private_messages_collection {
            private_messages_collection.create_index(id_idx.clone(), None).await?;
            private_messages_collection.create_index(created_at_idx.clone(), None).await?;
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
            sender: String::from(&message.sender),
            room: String::from(&message.room),
            message: String::from(&message.message),
            created_at: message.date_time,
            updated_at: chrono::Utc::now(),
        })
    }

    #[allow(dead_code)]
    pub fn doc_to_message(&self, doc: Document) -> Result<Message> {
        Ok(Message {
            sender: doc.get_str("sender").unwrap().to_string(),
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

    pub async fn get_messages(&self, room: Option<String>) -> Result<Option<Vec<Message>>> {
        if let Some(collection) = &self.messages_collection {
            let filter = match room {
                Some(room) => doc! {"room": room},
                None => doc! {}
            };

            let find_options = FindOptions::builder()
                .sort(doc! {"created_at": -1})
                .limit(20)
                .build();

            let mut cursor = match collection.find(filter, find_options).await {
                Ok(res) => res,
                Err(e) => return Err(MyError::MongoError(e))
            };

            let mut results: Vec<Message> = Vec::new();

            while cursor.advance().await? {
                let doc = cursor.deserialize_current().unwrap();
                results.push(Message {
                    sender: doc.sender,
                    room: doc.room,
                    message: doc.message,
                    date_time: doc.created_at,
                });
            }

            if results.clone().is_empty() {
                Ok(None)
            } else {
                Ok(Some(results))
            }

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
    pub async fn get_sockets(&self, limit: i64, page: i64) -> Result<PaginationResponse<SocketResponse>> {
        if let Some(collection) = &self.sockets_collection {

            let filter = FindOptions::builder()
                .limit(limit)
                .skip(u64::try_from((page - 1) * limit).unwrap())
                .sort(doc! {"updated_at": -1, "created_at": -1})
                .build();

            let mut cursor = match collection.find(None, filter).await {
                Ok(res) => res,
                Err(e) => return Err(MyError::MongoError(e))
            };

            let mut sockets_list: Vec<SocketResponse> = Vec::new();

            while cursor.advance().await? {
                // let doc = cursor.current().to_owned().to_document().unwrap();
                // let socket = bson::from_document::<SocketCollection>(doc).unwrap();
                let socket = cursor.deserialize_current().unwrap();

                // info!("Socket: {:?}", &socket);
                sockets_list.push(SocketResponse {
                    id: socket.id.to_string(),
                    socket: socket.socket.to_string(),
                    username: socket.username.to_string(),
                    created_at: socket.created_at,
                    updated_at: socket.updated_at,
                });
            }

            let count_options = CountOptions::builder()
                .skip(u64::try_from(((page - 1) + 1) * limit).unwrap())
                .build();
            let next = if collection.count_documents(None, count_options).await? as i64 > 1 {
                Some(page + 1)
            } else {
                None
            };
            let prev = if page > 1 {
                Some(page - 1)
            } else {
                None
            };
            let pages = collection.estimated_document_count(None).await.unwrap() as i64 / limit;
            let total = collection.estimated_document_count(None).await.unwrap() as i64;

            let response = PaginationResponse {
                data: sockets_list,
                curr_page: page,
                next_page: next,
                prev_page: prev,
                total_pages: pages,
                total_records: total,
            };
            Ok(response)
        } else {
            Err(MyError::OwnError(String::from("Messages collection not found")))
        }
    }

    pub async fn remove_socket(&self, socket: String) {
        if let Some(collection) = &self.sockets_collection {
            let res = collection.delete_one(doc! {"socket": socket}, None).await.unwrap();
            info!("Removed: {:?}", res);
        }
    }
}