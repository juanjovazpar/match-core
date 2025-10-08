use std::env;
use dotenvy::dotenv;
use uuid::Uuid;
use tokio::sync::mpsc::{self, Sender};

use crate::{engine::{order::{Order, Mode, Side}, order_book::OrderBook}, Message};

pub fn start() -> Sender<Message> {
    dotenv().ok();

    let buffer = env::var("CHANNEL_BUFFER")
        .unwrap_or_else(|_| 100.to_string()) 
        .parse()
        .expect("CHANNEL_BUFFER must be a number");
    
    let mut order_book = OrderBook::new();
    let (tx, mut rx) = mpsc::channel::<Message>(buffer);

    tokio::spawn(async move {
        while let Some(msg) = rx.recv().await {
            println!("message incoming: {}", msg.content);
            order_book.execute(Order::new(Uuid::new_v4(), 10, 100, Side::Ask, Mode::Limit));
        }
    });

    tx
}