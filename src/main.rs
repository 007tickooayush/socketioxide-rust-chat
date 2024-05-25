mod model;

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
use crate::model::{GeneralRequest, GeneralResponse};
pub async fn on_connect(socket: SocketRef) {
    info!("Socket Connected: {:?}", socket.id);

    // socket.on("message", |_socket: SocketRef, Data::<dyn Value>(data)| {
    //     info!("Message: {:?}", data);
    // });

    socket.on("join_room", |_socket: SocketRef, Data::<GeneralRequest>(data) | async move {
        let general = GeneralRequest {
            room: data.room.clone(),
            message: data.message.clone()
        };
        info!("General: {:?}", &general);

        _socket.join(general.room.clone()).ok();

        let response = GeneralResponse {
            room: general.room.clone(),
            message: format!("Room joined by client: {}", _socket.id).to_owned(),
            date_time: chrono::Utc::now()
        };

        _socket.within(general.room.clone()).emit("response", response).ok();
    });

    socket.on("private", |_socket: SocketRef, Data::<Value>(data)| async move {
        info!("Private: {:?}", data);

    });

    socket.on("message", |_socket: SocketRef, Data::<serde_json::Value>(data)| {
        info!("Message: {:?}", data);
    });
}

/// ### In this handler, we are going to emit a message to the client using the HTTP request handler
/// *i.e, whenever the HTTP endpoint is hit, we are going to emit a message to the client and in this case we are broadcasting the message across all clients*
/// *it is achieved using State(io), passed to the axum server using with_state() function*
pub async fn http_socket_handler(State(io): State<SocketIo>) {
    let _ = io.emit("response", "Hello from server");
}


/// Handling the POST request from the client

pub async fn http_socket_post_handler(
    State(io): State<SocketIo>,
    Json(data): Json<GeneralRequest>
) -> Result<impl IntoResponse,(StatusCode, Json<Value>)> {
    let general = GeneralRequest {
        room: data.room.clone(),
        message: data.message.clone()
    };
    info!("General: {:?}", &general);

    let response = GeneralResponse {
        room: general.room.clone(),
        message: format!("Room joined by client: {}", "HTTP Request").to_owned(),
        date_time: chrono::Utc::now()
    };

    // not sending yet
    io.emit("response", response.clone()).ok();
    // io.within(general.room.clone()).emit("response", response.clone()).ok();

    Ok((StatusCode::OK, Json::<GeneralResponse>(response)))
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    const PORT: i32 = 4040;

    tracing::subscriber::set_global_default(fmt::Subscriber::default()).unwrap();

    let (layer, io) = SocketIo::builder().build_layer();

    io.ns("/", on_connect);

    // io.ns("/", |socket: SocketRef| {
    //     socket.on("message", |data, socket| {
    //         println!("Message: {:?}", data);
    //         socket.emit("message", data);
    //     });
    //
    //     socket.on("join", |data, socket| {
    //         println!("Join: {:?}", data);
    //         socket.emit("join", data);
    //     });
    //
    //     socket.on("leave", |data, socket| {
    //         println!("Leave: {:?}", data);
    //         socket.emit("leave", data);
    //     });
    //
    //     socket.on("disconnect", |data, socket| {
    //         println!("Disconnect: {:?}", data);
    //         socket.emit("disconnect", data);
    //     });
    // });

    let cors = CorsLayer::new()
        .allow_origin(["http:localhost:3000".parse::<HeaderValue>().unwrap()])
        .allow_methods([Method::GET, Method::POST, Method::PATCH, Method::DELETE])
        .allow_credentials(true)
        .allow_headers([AUTHORIZATION, ACCEPT, CONTENT_TYPE]);

    let app: Router = Router::new()
        .route("/", get(|| async { "Server Running" }))
        .route("/socket-test",get(http_socket_handler)) // handle GET request on /socket-test namespace
        // .route("/post", post(|| async { "POST Request"}))
        .route("/post", post(http_socket_post_handler))
        .with_state(io) // handle state and http events
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
