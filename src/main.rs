use std::env;
use dotenvy::dotenv;

mod engine;
mod server;

#[tokio::main]
async fn main() {
    dotenv().ok(); 

    let host = env::var("HOST")
        .unwrap_or_else(|_| "127.0.0.1"
        .to_string());
    let port = env::var("PORT")
        .unwrap_or_else(|_| "3000"
        .to_string());

    server::start(host, port).await;
}