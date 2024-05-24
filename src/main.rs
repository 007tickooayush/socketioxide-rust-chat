use axum::{routing::get, Router};
use axum::http::{HeaderValue, Method};
use axum::http::header::{ACCEPT, AUTHORIZATION, CONTENT_TYPE};
use socketioxide::extract::{Data, SocketRef};
use socketioxide::SocketIo;
use tower::ServiceBuilder;
use tower_http::cors::CorsLayer;
use tracing::info;
use tracing_subscriber::fmt;
// use tracing::subscriber::set_global_default;
// use tokio::net::TcpListener;

pub async fn on_connect(socket: SocketRef) {
    info!("Socket Connected: {:?}", socket.id);

    // socket.on("message", |_socket: SocketRef, Data::<dyn Value>(data)| {
    //     info!("Message: {:?}", data);
    // });
    socket.on("message", |_socket: SocketRef, Data::<serde_json::Value>(data)| {
        info!("Message: {:?}", data);
    });
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
        .with_state(io)
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
