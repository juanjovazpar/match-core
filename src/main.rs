use std::{env, net::SocketAddr};
use dotenvy::dotenv;

use tonic::{transport::Server, Request, Response, Status};
use exchain_commons::api::{order_service_server::{OrderService, OrderServiceServer}, Order, OrderAck}; 

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

#[derive(Default)]
pub struct MyOrderService;

#[tonic::async_trait]
impl OrderService for MyOrderService {
    async fn send_order(&self, request: Request<Order>) -> Result<Response<OrderAck>, Status> {
        let order = request.into_inner();
        Ok(Response::new(OrderAck { id: order.id, accepted: true }))
    }
}

#[tokio::main]
async fn main() {
    dotenv().ok();

    let host = env::var("HOST").unwrap_or_else(|_| "0.0.0.0".to_string());
    let port = env::var("PORT").unwrap_or_else(|_| "3000".to_string());

    let (tx, _) = engine::start().await;

    let server_task = tokio::spawn(server::start(host, port, tx));
    /* let listener_task = tokio::spawn(async move {
        while let Some(msg) = rx.recv().await.or(Some(Message::new("noop".to_string()))) {
            println!("Message incoming from engine: {}", msg.content);
        }
    }); */

    // gRPC connection to Gatekeeper
    let my_service = MyOrderService::default();
    let host = env::var("HOST").unwrap_or_else(|_| "0.0.0.0".to_string());
    let port = env::var("GRPC_PORT").unwrap_or_else(|_| "50051".to_string());
    let addr = format!("{}:{}", host, port).parse::<SocketAddr>().unwrap();


    let grpc_server = Server::builder()
        .add_service(OrderServiceServer::new(my_service))
        .serve(addr);
    //

    tokio::select! {
        _ = server_task => {},
        _ = tokio::signal::ctrl_c() => {
            println!("Shutdown signal received");
        }
        res = grpc_server => {
            if let Err(e) = res {
                eprintln!("gRPC server error: {}", e);
            }
        }
    }
}
