//! Prost-generated types for gRPC ingress (`matchcore.v1`).
//!
//! Centralized here so validators and service handlers share one definition
//! without coupling validation to a specific service module.

pub mod pb {
    tonic::include_proto!("matchcore.v1");
}
