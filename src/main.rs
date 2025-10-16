use std::{env};
use dotenvy::dotenv;

mod server;
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
    dotenv().ok();

    let host = env::var("HOST").unwrap_or_else(|_| "0.0.0.0".to_string());
    let port = env::var("PORT").unwrap_or_else(|_| "3000".to_string());

    let (tx, mut rx) = engine::start().await;

    let server_task = tokio::spawn(server::start(host, port, tx));
    let listener_task = tokio::spawn(async move {
        while let Some(msg) = rx.recv().await.or(Some(Message::new("noop".to_string()))) {
            println!("Message incoming from engine: {}", msg.content);
        }
    });

    tokio::select! {
        _ = server_task => {},
        _ = listener_task => {},
        _ = tokio::signal::ctrl_c() => {
            println!("Shutdown signal received");
        }
    }
}
