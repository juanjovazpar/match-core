use serde::Deserialize;

#[derive(Debug, Clone, Deserialize)]
pub struct Config {
    pub shard: ShardConfig,
    pub grpc: GrpcConfig,
    pub storage: StorageConfig,
    pub snapshot: SnapshotConfig,
    pub event_bus: EventBusConfig,
    pub engine: EngineConfig,
}

#[derive(Debug, Clone, Deserialize)]
pub struct ShardConfig {
    pub id: String,
    pub pair_symbols: String,
}

#[derive(Debug, Clone, Deserialize)]
pub struct GrpcConfig {
    pub port: u16,
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

#[derive(Debug, Clone, Deserialize)]
pub struct EngineConfig {
    pub max_inflight_commands: usize,
}