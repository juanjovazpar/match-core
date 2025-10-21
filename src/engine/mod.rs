
use std::env;
use dotenvy::dotenv;
use uuid::Uuid;
use tokio::sync::mpsc::{self, Sender, Receiver};
use crate::{engine::{order_book::OrderBook}, Message};
use exchain_commons::structs::{linked_hashmap, order::{self, Order, Side, Mode}, trade};

mod order_book;

pub async fn start() -> (Sender<Message>, Receiver<Message>) {
    dotenv().ok();

    let buffer = env::var("CHANNEL_BUFFER")
        .unwrap_or_else(|_| 100.to_string()) 
        .parse()
        .expect("❌ CHANNEL_BUFFER must be a number");
    let orderbook_pair = env::var("ORDERBOOK_PAIR_SYMBOLS")
        .expect("❌ Environment variable ORDERBOOK_PAIR_SYMBOLS not found");

    let mut order_book = OrderBook::new();

    println!("🧠 Order Book initialized for pair {}!", orderbook_pair);

    // Channel for the engine to receive message
    let (out_sender, mut out_receiver) = mpsc::channel::<Message>(buffer);

    // Channel for the engine to send message
    let (in_sender, in_receiver) = mpsc::channel::<Message>(buffer);

    tokio::spawn(async move {
        while let Some(msg) = out_receiver.recv().await {
            let _ = in_sender.send(Message::new(String::from("hello engine"))).await;
            println!("message incoming: {}", msg.content);
            order_book.execute(Order::new(Uuid::new_v4(), 10, 100, Side::Ask, Mode::Limit));
        }
    });


    (out_sender, in_receiver)
}