use std::cell::RefCell;
use std::rc::Rc;
use mongodb::bson::doc;
use mongodb::{Collection, IndexModel};
use crate::db_model::{MessageCollection, SocketCollection};
use crate::errors::MyError;

/// override the standard result type for the module
type Result<T> = std::result::Result<T, MyError>;

#[derive(Clone, Debug)]
pub struct DB {
    pub sockets_collection: Collection<SocketCollection>,
    pub messages_collection: Collection<MessageCollection>,
}

impl DB {
    pub async fn connect_mongo() -> Result<DB> {
        let client = mongodb::Client::with_uri_str("mongodb://localhost:27017").await?;
        let db = client.database("socketioxide");
        let sockets_collection = db.collection("sockets");
        let messages_collection = db.collection("messages");

        let socket_idx = RefCell::new(IndexModel::builder()
            .keys(doc! {"_id": 1})
            .build());
        // sockets_collection.create_index(doc! {"id": 1}, None).await?;
        sockets_collection.create_index(socket_idx.borrow().clone(), None).await?;
        messages_collection.create_index(socket_idx.borrow().clone(), None).await?;

        Ok(DB {
            sockets_collection,
            messages_collection,
        })
    }
}