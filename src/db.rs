use bson::oid::ObjectId;
use chrono::Utc;
use mongodb::bson::{doc, Document};
use mongodb::{bson, ClientSession, Collection, IndexModel};
use mongodb::options::{CountOptions, FindOptions};
use serde::de::DeserializeOwned;
use tracing::info;
use crate::db_model::{MessageCollection, PrivateMessageCollection, SocketCollection, UserCollection};
use crate::errors::MyError;
use crate::model::{Message, PaginationResponse, SocketResponse, User};

/// override the standard result type for the module
type Result<T> = std::result::Result<T, MyError>;

#[derive(Clone, Debug)]
pub struct DB {
    pub sockets_collection: Option<Collection<SocketCollection>>,
    pub messages_collection: Option<Collection<MessageCollection>>,
    pub private_messages_collection: Option<Collection<PrivateMessageCollection>>,
    pub users_collection: Option<Collection<UserCollection>>,
}

impl DB {
    pub async fn connect_mongo() -> Result<DB> {
        let client = mongodb::Client::with_uri_str(std::env::var("MONGO_URI").unwrap_or("mongodb://localhost:27017".to_owned())).await?;
        let db = client.database("socketioxide");
        let sockets_collection = Some(db.collection("sockets"));
        let messages_collection = Some(db.collection("messages"));
        let private_messages_collection = Some(db.collection("private_messages"));
        let users_collection = Some(db.collection("users"));

        let id_idx = IndexModel::builder()
            .keys(doc! {"_id": 1})
            .build();

        let created_at_idx = IndexModel::builder()
            .keys(doc! {"created_at": 1})
            .build();

        if let Some(sockets_collection) = &sockets_collection {
            sockets_collection.create_index(id_idx.clone(), None).await?;
            sockets_collection.create_index(created_at_idx.clone(), None).await?;
            sockets_collection.create_index(
                IndexModel::builder().keys(doc! {"username": "text"}).build(),
                None,
            ).await?;
        }

        if let Some(messages_collection) = &messages_collection {
            messages_collection.create_index(id_idx.clone(), None).await?;
            messages_collection.create_index(created_at_idx.clone(), None).await?;
            messages_collection.create_index(
                IndexModel::builder().keys(doc! {"room": "text"}).build(),
                None,
            ).await?;
        }

        if let Some(private_messages_collection) = &private_messages_collection {
            private_messages_collection.create_index(id_idx.clone(), None).await?;
            private_messages_collection.create_index(created_at_idx.clone(), None).await?;
            private_messages_collection.create_index(
                IndexModel::builder().keys(doc! {"sender": "text"}).build(),
                None,
            ).await?;
        }

        if let Some(users_collection) = &users_collection {
            users_collection.create_index(id_idx.clone(), None).await?;
            users_collection.create_index(created_at_idx.clone(), None).await?;
            users_collection.create_index(
                IndexModel::builder().keys(doc! {"owned_uname": "text"}).build(),
                None,
            ).await?;
        }

        Ok(DB {
            sockets_collection,
            messages_collection,
            private_messages_collection,
            users_collection,
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
            Err(MyError::OwnError(String::from("Private Messages collection not found")))
        }
    }

    pub async fn find_message_oid(&self, oid: ObjectId) -> Result<MessageCollection> {
        self.find_doc_by_oid(&self.messages_collection, oid, None).await
    }
    pub async fn find_private_message_oid(&self, oid: ObjectId) -> Result<PrivateMessageCollection> {
        self.find_doc_by_oid(&self.private_messages_collection, oid, None).await
    }

    pub async fn find_doc_by_oid<T>(&self, collection: &Option<Collection<T>>, oid: ObjectId, session: Option<&mut ClientSession>) -> Result<T>
    where
        T: DeserializeOwned + Unpin + Send + Sync,
    {
        if let Some(collection) = &collection {
            let resp;
            if let Some(session) = session {
                resp = match collection.find_one_with_session(doc! {"_id": oid}, None, session).await {
                    Ok(res) => {
                        match res {
                            Some(doc) => doc,
                            None => return Err(MyError::OwnError(String::from("Message not found")))
                        }
                    }
                    Err(e) => return Err(MyError::MongoError(e))
                };
            } else {
                resp = match collection.find_one(doc! {"_id": oid}, None).await {
                    Ok(res) => {
                        match res {
                            Some(doc) => doc,
                            None => return Err(MyError::OwnError(String::from("Message not found")))
                        }
                    }
                    Err(e) => return Err(MyError::MongoError(e))
                };
            }

            Ok(resp)
        } else {
            Err(MyError::OwnError(String::from("Collection not found")))
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
            let resp = self.find_doc_by_oid(&self.sockets_collection, oid, None).await?;

            Ok(resp)
        } else {
            Err(MyError::OwnError(String::from("Sockets collection not found")))
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
            Err(MyError::OwnError(String::from("Sockets collection not found")))
        }
    }

    /// This is the function to read, validate and insert the data in form of a non-transaction <br/>
    /// Multiple re-rendering of the state in frontend or the operations in the function may cause multiple writes or retryable writing of data <br/>
    /// Those kind of operations might not be supported by the MongoDB driver provided<br/>
    pub async fn handle_user(&self, user: User) -> Result<UserCollection> {
        if let Some(collection) = &self.users_collection {

            // let mut session = collection.client().start_session(None).await?;
            // session.start_transaction(None).await?;

            let user = UserCollection {
                id: bson::oid::ObjectId::new(),
                owned_uname: user.username.clone(),
                cur_gen_uname: user.generated_username.clone(),
                last_username: "".to_string(),
                online: true,
                updated_at: chrono::Utc::now(),
                created_at: chrono::Utc::now(),
            };

            let found_res = collection.find_one(doc! {"owned_uname": user.owned_uname.clone()}, None).await.unwrap();
            let final_res;

            if let Some(res) = found_res {
                let last = res.cur_gen_uname.clone();

                collection.update_one(doc! {
                    "owned_uname": user.owned_uname.clone()
                }, doc! {
                    "$set": {
                        "cur_gen_uname": user.cur_gen_uname.clone(),
                        "online": user.online,
                        "last_username": last,
                        "updated_at": Utc::now()
                    }
                }, None).await?;

                final_res = self.find_doc_by_oid(&self.users_collection, res.id, None).await?;
            } else {
                let inserted = collection.insert_one(&user, None).await?;

                final_res = self.find_doc_by_oid(&self.users_collection, inserted.inserted_id.as_object_id().unwrap(), None).await?;
            }

            // session.commit_transaction().await?;

            Ok(final_res)

            // let insert_res = match collection.insert_one(&user, None).await {
            //     Ok(res) => res,
            //     Err(e) => return Err(MyError::MongoError(e))
            // };
            // let oid = insert_res.inserted_id.as_object_id().unwrap();
            //
            // let verified = self.find_doc_by_oid(&self.users_collection, oid).await?;
            //
            // Ok(verified)
        } else {
            Err(MyError::OwnError(String::from("Users collection not found")))
        }
    }

    pub async fn check_user_exists(&self, username: String) -> Result<Option<User>> {
        if let Some(collection) = &self.users_collection {
            collection.find_one(doc! {"owned_uname": username}, None).await
                .map(|res| {
                    match res {
                        Some(doc) => {
                            Some(User {
                                username: doc.owned_uname,
                                generated_username: doc.cur_gen_uname,
                            })
                        }
                        None => None
                    }
                })
                .map_err(MyError::from)
        } else {
            Err(MyError::OwnError(String::from("Users collection not found")))
        }
    }
    pub async fn remove_socket(&self, user: User) {
        if let Some(collection) = &self.sockets_collection {
            let res = collection.delete_one(doc! {"username": user.generated_username}, None).await.unwrap();
            info!("Removed: {:?}", res);
        }

        // mark the status as offline in users collection
        if let Some(collection) = &self.users_collection {
            collection.update_one(doc! {"owned_uname": user.username}, doc! {"$set": {"online": false, "updated_at": Utc::now()}}, None).await.unwrap();
        }
    }

    pub async fn test_transaction(&self) -> Result<User> {
        if let Some(collection) = &self.users_collection {
            let mut session = collection.client().start_session(None).await?;
            session.start_transaction(None).await?;

            let user = UserCollection {
                id: bson::oid::ObjectId::new(),
                owned_uname: "test".to_string(),
                cur_gen_uname: "test".to_string(),
                last_username: "".to_string(),
                online: true,
                updated_at: chrono::Utc::now(),
                created_at: chrono::Utc::now(),
            };

            let inserted = collection.insert_one_with_session(&user, None, &mut session).await?;
            let resp;
            if let Some(oid) = inserted.inserted_id.as_object_id() {
                if let Some(found) = collection.find_one_with_session(doc! {"_id": oid}, None, &mut session).await? {
                    resp = User {
                        username: found.owned_uname,
                        generated_username: found.cur_gen_uname,
                    };
                } else {
                    resp = User {
                        username: "".to_string(),
                        generated_username: "".to_string(),
                    };
                }

                collection.find_one_and_delete_with_session(doc! {"_id": oid}, None, &mut session).await?;
            } else {
                return Err(MyError::OwnError(String::from("Error in inserting the user")));
            }

            session.commit_transaction().await?;
            Ok(resp)
        } else {
            Err(MyError::OwnError(String::from("Users collection not found")))
        }
    }
}