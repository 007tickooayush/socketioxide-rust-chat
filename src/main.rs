use axum::{routing::get, Router};
// use tokio::net::TcpListener;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error + Send + Sync>>{
    const PORT: i32 = 4040;

    
    let app: Router = Router::new().route("/", get(|| async { "Server Running" }));
    
    let listener = tokio::net::TcpListener::bind(format!("0.0.0.0:{}",PORT)).await?;
    
    
    println!("Server running on port: {}", PORT);

    axum::serve(listener, app.into_make_service()).await.unwrap();
    Ok(())
}
