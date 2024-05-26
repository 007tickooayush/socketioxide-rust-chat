use std::sync::Arc;
use axum::extract::State;
use axum::http::StatusCode;
use axum::Json;
use axum::response::IntoResponse;
use serde_json::Value;
use socketioxide::SocketIo;
use tracing::{info, warn};
use crate::AppState;
use crate::model::{GeneralRequest, GeneralResponse};

/// ### In this handler, we are going to emit a message to the client using the HTTP request handler
/// *i.e, whenever the HTTP endpoint is hit, we are going to emit a message to the client and in this case we are broadcasting the message across all clients*
/// *it is achieved using State(io), passed to the axum server using with_state() function*
pub async fn http_socket_handler(State(app_state): State<Arc<AppState>>) {
    let _ = app_state.io.emit("response", "Hello from server");
}

/// Handling the POST request from the client
pub async fn http_socket_post_handler(
    State(app_state): State<Arc<AppState>>,
    Json(data): Json<GeneralRequest>
) -> Result<impl IntoResponse,(StatusCode, Json<Value>)> {
    let general = GeneralRequest {
        room: data.room.clone(),
        message: data.message.clone()
    };
    info!("General: {:?}", &general);

    warn!("Sockets: {:?}", app_state.io.sockets());

    let response = GeneralResponse {
        room: general.room.clone(),
        message: format!("Message By Client: {}", "HTTP Request").to_owned(),
        date_time: chrono::Utc::now()
    };

    // not sending yet
    app_state.io.emit("response", response.clone()).ok();
    // io.within(general.room.clone()).emit("response", response.clone()).ok();

    Ok((StatusCode::OK, Json::<GeneralResponse>(response)))
}