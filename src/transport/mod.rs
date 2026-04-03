//! gRPC ingress: request validation, dispatch into the engine command channel,
//! and transport-level limits (message size, timeouts, tracing).

pub mod dispatcher;
mod metrics;
mod orders_service;
mod proto;
pub mod service;
pub mod settings;
