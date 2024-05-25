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