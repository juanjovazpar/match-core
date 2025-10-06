use std::net::SocketAddr;
use tokio::net::TcpListener;

use crate::server::router;

pub async fn start(host: String, port: String) {
    let addr = format!("{}:{}", host, port).parse::<SocketAddr>().unwrap();
    let listener = TcpListener::bind(addr).await.unwrap();
    let app = router::create();

    axum::serve(listener, app).await.unwrap();

    println!("🚀 Match Core Server listening at http://{}", addr);
}