mod model;
mod socket;
mod http_routes;
mod db;
mod http_handlers;
mod errors;
mod db_model;

use std::sync::Arc;
use axum::{routing::get, Router, Json};
use axum::extract::State;
use axum::http::{HeaderValue, Method, StatusCode};
use axum::http::header::{ACCEPT, AUTHORIZATION, CONTENT_TYPE};
use axum::response::IntoResponse;
use axum::routing::post;
use serde_json::Value;
use socketioxide::extract::{Data, SocketRef};
use socketioxide::SocketIo;
use tower::ServiceBuilder;
use tower_http::cors::CorsLayer;
use tracing::info;
use tracing_subscriber::fmt;
use crate::db::DB;
use crate::http_routes::create_router;
use crate::model::{GeneralRequest, GeneralResponse};
use crate::socket::on_connect;

pub struct AppState {
    io: SocketIo,
    db: DB
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    const PORT: i32 = 4040;

    tracing::subscriber::set_global_default(fmt::Subscriber::default()).unwrap();

    let (layer, io) = SocketIo::builder().build_layer();

    io.ns("/", on_connect);


    let cors = CorsLayer::new()
        .allow_origin(["http:localhost:3000".parse::<HeaderValue>().unwrap()])
        .allow_methods([Method::GET, Method::POST, Method::PATCH, Method::DELETE])
        .allow_credentials(true)
        .allow_headers([AUTHORIZATION, ACCEPT, CONTENT_TYPE]);

    let db = DB::connect_mongo().await.unwrap();

    let app = create_router(Arc::new(AppState { io: io.clone(), db: db.clone() }))
        .layer(
            ServiceBuilder::new()
                .layer(cors)
                .layer(layer)
        );


    let listener = tokio::net::TcpListener::bind(format!("0.0.0.0:{}", PORT)).await?;

    println!("Server running on port: {}", PORT);

    axum::serve(listener, app.into_make_service()).await.unwrap();
    Ok(())
}
