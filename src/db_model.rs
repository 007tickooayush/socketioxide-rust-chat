use chrono::DateTime;
use mongodb::bson::oid::ObjectId;
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct SocketCollection {
    pub id: ObjectId,
    pub socket: String,
    pub created_at: DateTime<chrono::Utc>,
    pub updated_at: DateTime<chrono::Utc>

}

#[derive(Debug,Serialize,Deserialize)]
pub struct MessageCollection{
    pub id: ObjectId,
    pub room: String,
    pub message: String,
    pub created_at: DateTime<chrono::Utc>,
    pub updated_at: DateTime<chrono::Utc>
}