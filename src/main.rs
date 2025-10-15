use std::{env};
use dotenvy::dotenv;

// mod server;
mod engine;

pub struct Message {
    content: String
}
impl Message {
    pub fn new(content: String) -> Self {
        Self { content }
    }
}

#[tokio::main]
async fn main() {
    /* dotenv().ok();

    let host = env::var("HOST")
        .unwrap_or_else(|_| "127.0.0.1"
        .to_string());
    let port = env::var("PORT")
        .unwrap_or_else(|_| "3000"
        .to_string());

    let (tx, mut rx) = engine::start().await;
    
    server::start(host, port, tx).await;

    // Receiving messages from engine
    tokio::spawn(async move {
        while let Some(msg) = rx.recv().await {
            println!("Message incoming from engine: {}", msg.content);
        }
    }); */
}