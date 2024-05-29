use std::sync::Arc;
use axum::Router;
use axum::routing::{get, post};
use crate::{AppState};
use crate::http_handlers::{http_socket_handler, http_socket_post_handler, http_sockets_list};

pub fn create_router(app_state: Arc<AppState>) -> Router {
    Router::new()
        .route("/api/", get(|| async { "Server Running" }))
        .route("/api/socket-test", get(http_socket_handler)) // handle GET request on /socket-test namespace
        // .route("/api/post", post(|| async { "POST Request"}))
        .route("/api/post", post(http_socket_post_handler))
        .route("/api/sockets-list", get(http_sockets_list))
        .with_state(app_state) // handle state and http events
}