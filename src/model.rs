use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct General {
    pub room : String,
    pub message: String
}