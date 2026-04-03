//! Maps application config into transport-owned [`GrpcServeOptions`](crate::transport::settings::GrpcServeOptions).

use std::time::Duration;

use crate::config::types::GrpcConfig;
use crate::transport::settings::{GrpcServeOptions, GrpcTlsOptions};

pub fn grpc_serve_options(cfg: &GrpcConfig) -> GrpcServeOptions {
    GrpcServeOptions {
        request_timeout_secs: cfg.request_timeout_secs,
        concurrency_limit_per_connection: cfg.concurrency_limit_per_connection,
        max_decoding_message_bytes: cfg.max_decoding_message_bytes,
        max_encoding_message_bytes: cfg.max_encoding_message_bytes,
        command_enqueue_timeout: Duration::from_millis(cfg.command_enqueue_timeout_ms),
        tls: cfg.tls.as_ref().map(|t| GrpcTlsOptions {
            cert_path: t.cert_path.clone(),
            key_path: t.key_path.clone(),
            client_ca_path: t.client_ca_path.clone(),
            client_auth_optional: t.client_auth_optional,
        }),
    }
}
