use std::net::SocketAddr;
use tokio::net::TcpListener;
use tokio::sync::mpsc::{Sender};

use crate::server::router;
use crate::Message;
use crate::server::channel::set_sender;

pub async fn start(host: String, port: String, sender: Sender<Message>) {
    let addr = format!("{}:{}", host, port).parse::<SocketAddr>().unwrap();
    let listener = TcpListener::bind(addr).await.unwrap();
    let app = router::create();

    set_sender(sender);

    println!("🚀 Match Core Server listening at http://{}", addr);
    axum::serve(listener, app).await.unwrap();
}