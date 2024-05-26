use std::sync::Arc;
use axum::Router;
use axum::routing::{get, post};
use crate::{AppState};
use crate::http_handlers::{http_socket_handler, http_socket_post_handler};

pub fn create_router(app_state: Arc<AppState>) -> Router {
    Router::new()
        .route("/", get(|| async { "Server Running" }))
        .route("/socket-test", get(http_socket_handler)) // handle GET request on /socket-test namespace
        // .route("/post", post(|| async { "POST Request"}))
        .route("/post", post(http_socket_post_handler))
        .with_state(app_state) // handle state and http events
}