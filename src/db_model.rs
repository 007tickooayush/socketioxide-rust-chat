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
    #[serde(rename = "_id")]
    pub id: ObjectId,
    pub room: String,
    pub message: String,
    #[serde(with = "bson::serde_helpers::chrono_datetime_as_bson_datetime")]
    pub created_at: DateTime<chrono::Utc>,
    #[serde(with = "bson::serde_helpers::chrono_datetime_as_bson_datetime")]
    pub updated_at: DateTime<chrono::Utc>
}