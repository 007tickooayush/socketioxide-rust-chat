use chrono::DateTime;
use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize)]
pub struct GeneralRequest {
    pub sender: String,
    pub room: String,
    pub message: String,
}

#[derive(Debug, Serialize, Clone)]
pub struct GeneralResponse {
    pub sender: String,
    pub room: String,
    pub message: String,
    pub date_time: DateTime<chrono::Utc>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct User {
    pub username: String,
    pub generated_username: String,
}

#[derive(Debug, Serialize, Clone)]
pub struct UserExists {
    pub exists: bool,
    pub username: String,
    pub generated_username: String,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct UserResp {
    pub owned_uname: String,
    pub cur_gen_uname: String,
    pub updated_at: DateTime<chrono::Utc>,
    pub created_at: DateTime<chrono::Utc>,
}

#[derive(Debug, Serialize, Clone)]
pub struct Message {
    pub sender: String,
    pub room: String,
    pub message: String,
    pub date_time: DateTime<chrono::Utc>,
}

#[derive(Serialize)]
pub struct Messages {
    pub messages: Vec<Message>,
}

/// Struct for handling the private messages <br/>
/// It utilizes the `sender` and `receiver` (socket IDs) fields to send the message to the respective user <br/>
/// For fetching the socket IDs the API endpoint can be used
///
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct PrivateMessage {
    pub sender: String,
    pub message: String,
    pub receiver: String,
    pub date_time: DateTime<chrono::Utc>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct PrivateMessageReq {
    pub sender: Option<String>,
    pub message: String,
    pub receiver: String,
}

#[derive(Deserialize, Debug, Default)]
pub struct Filter {
    pub page: Option<usize>,
    pub limit: Option<usize>,
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

#[derive(Serialize, Deserialize, Debug, Default)]
pub struct PaginationResponse<T> {
    pub data: Vec<T>,
    pub curr_page: i64,
    pub total_pages: i64,
    pub total_records: i64,
    pub next_page: Option<i64>,
    pub prev_page: Option<i64>,
}