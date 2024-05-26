use mongodb::Collection;
use crate::errors::MyError;

/// override the standard result type for the module
type Result<T> = std::result::Result<T, MyError>;

#[derive(Clone, Debug)]
pub struct DB {
    pub sockets_collection: Collection<serde_json::Value>,
    pub messages_collection: Collection<serde_json::Value>,
}

impl DB {
    pub async fn connect_mongo() -> Result<DB> {
        let client = mongodb::Client::with_uri_str("mongodb://localhost:27017").await?;
        let db = client.database("socketioxide");
        let sockets_collection = db.collection("sockets");
        let messages_collection = db.collection("messages");

        Ok(DB {
            sockets_collection,
            messages_collection,
        })
    }
}