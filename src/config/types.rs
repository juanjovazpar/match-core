use serde::Deserialize;

use crate::shared::types::Symbol;

#[derive(Debug, Clone, Default, Deserialize)]
pub struct ObservabilityConfig {
    /// When set, exposes Prometheus scrape metrics on this `host:port` (e.g. `0.0.0.0:9090`).
    #[serde(default)]
    pub metrics_listen_addr: Option<String>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct Config {
    pub shard: ShardConfig,
    pub grpc: GrpcConfig,
    pub storage: StorageConfig,
    pub snapshot: SnapshotConfig,
    pub event_bus: EventBusConfig,
    pub engine: EngineConfig,
    #[serde(default)]
    pub observability: ObservabilityConfig,
}

#[derive(Debug, Clone, Deserialize)]
pub struct ShardConfig {
    pub id: String,
    pub pair_symbols: Symbol,
}

fn default_grpc_request_timeout_secs() -> u64 {
    30
}

fn default_grpc_concurrency_limit_per_connection() -> usize {
    256
}

fn default_grpc_max_decoding_message_bytes() -> usize {
    4 * 1024 * 1024
}

fn default_grpc_max_encoding_message_bytes() -> usize {
    4 * 1024 * 1024
}

fn default_grpc_command_enqueue_timeout_ms() -> u64 {
    5_000
}

#[derive(Debug, Clone, Deserialize)]
pub struct GrpcConfig {
    pub port: u16,
    /// When set, the gRPC server uses TLS (PEM cert + key). Omit for plaintext.
    #[serde(default)]
    pub tls: Option<GrpcTlsConfig>,
    /// Per-RPC timeout applied by tonic’s transport server.
    #[serde(default = "default_grpc_request_timeout_secs")]
    pub request_timeout_secs: u64,
    /// Max concurrent requests per HTTP/2 connection.
    #[serde(default = "default_grpc_concurrency_limit_per_connection")]
    pub concurrency_limit_per_connection: usize,
    /// Max decoded gRPC message size for ingress (per service).
    #[serde(default = "default_grpc_max_decoding_message_bytes")]
    pub max_decoding_message_bytes: usize,
    /// Max encoded gRPC message size for egress (per service).
    #[serde(default = "default_grpc_max_encoding_message_bytes")]
    pub max_encoding_message_bytes: usize,
    /// Max time to wait for a slot in the engine command queue before rejecting the RPC.
    #[serde(default = "default_grpc_command_enqueue_timeout_ms")]
    pub command_enqueue_timeout_ms: u64,
}

#[derive(Debug, Clone, Deserialize)]
pub struct GrpcTlsConfig {
    pub cert_path: String,
    pub key_path: String,
    /// PEM file of the CA used to verify **client** certificates (mTLS). Omit for TLS server-only.
    #[serde(default)]
    pub client_ca_path: Option<String>,
    /// If true, clients may connect without a client cert when `client_ca_path` is set (not typical for mTLS).
    #[serde(default)]
    pub client_auth_optional: bool,
}

#[derive(Debug, Clone, Deserialize)]
pub struct StorageConfig {
    pub data_dir: String,
    pub log_file_segment_size: String,
}

#[derive(Debug, Clone, Deserialize)]
pub struct SnapshotConfig {
    pub interval_seconds: u64,
    pub max_commands: u64,
}

#[derive(Debug, Clone, Deserialize)]
pub struct EventBusConfig {
    pub r#type: String, // "nats" o "kafka"
    pub url: String,
}

fn default_engine_command_channel_capacity() -> usize {
    65_536
}

#[derive(Debug, Clone, Deserialize)]
pub struct EngineConfig {
    pub max_inflight_commands: usize,
    /// Capacity of the async `tokio::sync::mpsc` queue from ingress to the engine (backpressure when full).
    #[serde(default = "default_engine_command_channel_capacity")]
    pub command_channel_capacity: usize,
}
