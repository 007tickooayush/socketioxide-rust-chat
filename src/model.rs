use bson::oid::ObjectId;
use chrono::DateTime;
use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize)]
pub struct GeneralRequest {
    pub room : String,
    pub message: String
}

#[derive(Debug, Serialize, Clone)]
pub struct GeneralResponse {
    pub room: String,
    pub message: String,
    pub date_time: DateTime<chrono::Utc>
}

#[derive(Debug, Serialize, Clone)]
pub struct Message{
    pub room: String,
    pub message: String,
    pub date_time: DateTime<chrono::Utc>
}

#[derive(Serialize)]
pub struct Messages {
    pub messages: Vec<Message>
}

/// Struct for handling the private messages <br/>
/// It utilizes the `sender` and `receiver` (socket IDs) fields to send the message to the respective user <br/>
/// For fetching the socket IDs the API endpoint can be used
///
#[derive(Debug, Serialize, Deserialize)]
pub struct PrivateMessage {
    pub message: String,
    pub sender: String,
    pub receiver: String,
    pub date_time: DateTime<chrono::Utc>
}

#[derive(Clone,Debug, Serialize, Deserialize)]
pub struct PrivateMessageReq {
    pub message: String,
    pub sender: Option<String>,
    pub receiver: String
}

#[derive(Deserialize, Debug, Default)]
pub struct Filter {
    pub page: Option<usize>,
    pub limit: Option<usize>
}

#[derive(Serialize, Deserialize, Debug, Default)]
pub struct SocketResponse {
    #[serde(rename = "_id")]
    pub id: String,
    pub socket: String,
    pub username: String,
    pub created_at: DateTime<chrono::Utc>,
    pub updated_at: DateTime<chrono::Utc>,
}