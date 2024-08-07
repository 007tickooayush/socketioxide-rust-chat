use std::sync::Arc;
use axum::extract::{Query, State};
use axum::http::StatusCode;
use axum::Json;
use axum::response::IntoResponse;
use serde_json::Value;
use tracing::{error, info, warn};
use crate::AppState;
use crate::errors::MyError;
use crate::model::{Filter, GeneralRequest, GeneralResponse, InPrivate, PaginationResponse, SocketResponse, User, UserExists};

/// ### In this handler, we are going to emit a message to the client using the HTTP request handler
/// *i.e, whenever the HTTP endpoint is hit, we are going to emit a message to the client and in this case we are broadcasting the message across all clients*
/// *it is achieved using State(io), passed to the axum server using with_state() function*
pub async fn http_socket_handler(State(app_state): State<Arc<AppState>>) {
    let _ = app_state.io.emit("response", "Hello from server");
}

/// Handling the POST request from the client
pub async fn http_socket_post_handler(
    State(app_state): State<Arc<AppState>>,
    Json(data): Json<GeneralRequest>,
) -> Result<impl IntoResponse, (StatusCode, Json<Value>)> {
    let general = GeneralRequest {
        room: data.room.clone(),
        sender: data.sender.clone(),
        message: data.message.clone(),
    };
    info!("General: {:?}", &general);

    warn!("Sockets: {:?}", app_state.io.sockets());

    let response = GeneralResponse {
        sender: general.sender.clone(),
        room: general.room.clone(),
        message: format!("Message By Client: {}", "HTTP Request").to_owned(),
        date_time: chrono::Utc::now(),
    };
    info!("Response: {:?}", &response);

    // not sending yet
    app_state.io.emit("response", response.clone()).ok();
    // io.within(general.room.clone()).emit("response", response.clone()).ok();

    Ok((StatusCode::OK, Json::<GeneralResponse>(response)))
}

/// *NOTE:*
/// - The implementation is incomplete as is provided only for testing purposes, as List of Pair<String,String> is to be utilized rather than List of String.<br/>
/// - Providing the list of sockets connected to the server creates a form of vulnerability, and is not to be used in realtime applications.<br/>
/// <br/>
/// The function can be upgraded to fetch the socket details stored in any storage system like Redis, MongoDB, etc.
pub async fn http_sockets_list(
    filter: Option<Query<Filter>>,
    State(app_state): State<Arc<AppState>>,
) -> Result<impl IntoResponse, (StatusCode, Json<PaginationResponse<SocketResponse>>)> {

    // OLD IMPLEMENTATION
    /*let sockets: Vec<String> = app_state.io.sockets().unwrap().iter().map(|socket| {
        socket.id.clone().to_string()
    }).collect();

    return if sockets.clone().is_empty() {
        (StatusCode::NOT_FOUND, Json(vec![]))
    } else {
        (StatusCode::FOUND, Json(sockets))
    }*/

    // NEW IMPLEMENTATION
    let Query(filter) = filter.unwrap_or_default();

    // set the default values
    let limit = filter.limit.unwrap_or(10) as i64;
    let page = filter.page.unwrap_or(1) as i64;

    return match app_state.db.get_sockets(limit, page).await.map_err(MyError::from) {
        Ok(res) => {
            // info!("Sockets: {:?}", res);
            Ok((StatusCode::OK, Json(res)))
        }
        Err(e) => {
            error!("Error: {:?}", e);
            Ok((StatusCode::INTERNAL_SERVER_ERROR, Json(PaginationResponse {
                data: vec![],
                curr_page: 0,
                next_page: None,
                prev_page: None,
                total_records: 0,
                total_pages: 0,
            })))
        }
    };
}

pub async fn check_user_exists(
    State(state): State<Arc<AppState>>,
    Json(data): Json<User>,
) -> Result<impl IntoResponse, (StatusCode, Json<UserExists>)> {
    if let Some(res) = state.db.check_user_exists(data.username).await.unwrap() {
        Ok((StatusCode::FOUND, Json(UserExists {
            exists: true,
            username: res.username,
            generated_username: res.generated_username, // current generated username
        })))
    } else {
        Ok((StatusCode::OK, Json(UserExists {
            exists: false,
            username: "".to_owned(),
            generated_username: "".to_owned(),
        })))
    }
}

pub async fn check_user_in_private(
    Query(data): Query<User>,
    State(state): State<Arc<AppState>>,
) -> Result<impl IntoResponse, (StatusCode, Json<InPrivate>)> {
    info!("User in Private: {:?}", data);
    match state.db.check_private_exists(data).await.map_err(MyError::from) {
        Ok(res) => {
            let resp = InPrivate {
                username: res.owned_username,
                in_private: res.in_private,
            };

            Ok((StatusCode::FOUND, Json(resp)))
        }
        Err(e) => {
            error!("Error: {:?}", e);
            Ok((
                StatusCode::NOT_FOUND,
                Json(InPrivate {
                    username: format!("ERROR: {:?}", e).to_owned(),
                    in_private: false,
                })
            ))
        }
    }
}

pub async fn test_transaction(
    State(state): State<Arc<AppState>>
) -> Result<impl IntoResponse, (StatusCode, Json<Value>)> {
    let res = state.db.test_transaction().await.unwrap();
    Ok((StatusCode::OK, Json(res)))
}