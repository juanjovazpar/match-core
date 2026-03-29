use std::{sync::Arc, thread};
use dotenvy::dotenv;
use tracing_subscriber::fmt::{init as tracingInit};

/*
use std::time::Duration;

use crate::domain::matching::MatchingEngine;
use crate::snapshot::snapshotter::Snapshotter;
use crate::storage::snapshot_store::SnapshotStore;
use crate::transport::grpc::GrpcServer;
 */

mod shared;
mod core;
mod config;
mod domain;
mod utils;
mod events;

use crate::config::load_config;
use crate::events::publisher::Publisher;
use crate::core::{engine::Engine, deduplicator::Deduplicator, log_writter::LogWriter, sequencer::Sequencer};
use crate::domain::matching::MatchingEngine;
use crate::shared::{
    channel::Channels
};
fn main() {
    // --------------------------------------------------
    // Set configuration
    // --------------------------------------------------
    tracingInit();
    dotenv().ok();

    let config = Arc::new(load_config());
    let log_path = format!("{}/wal.log", config.storage.data_dir);
    let segment_size = config.storage.log_file_segment_size.clone();
    let event_bus_config = config.event_bus.clone();

    print!("Match-core initiated with config: {:?}", config);
    
    // --------------------------------------------------
    // Create channels to allow communication between threads
    // --------------------------------------------------
    let channels = Channels::new();
    let command_rx = channels.command_rx;
    let event_rx = channels.event_rx;
    let event_tx = channels.event_tx.clone();

    // --------------------------------------------------
    // Instantiate the core thread to run the engine
    // --------------------------------------------------
    let mut engine = Engine {
        command_rx,
        event_tx,
        sequencer: Sequencer::new(),
        wal: LogWriter::new(&log_path, segment_size),
        dedup: Deduplicator::new(),
        matching: MatchingEngine::new(),
    };

    thread::spawn(move || {
        engine.run();
    });

    // --------------------------------------------------
    // Instantiate the core thread for the event publisher
    // --------------------------------------------------+
    thread::spawn(move || {
        let publisher = Publisher::new(event_rx, event_bus_config);
        publisher.run();
    });
}

/*
    {
        shard: ShardConfig { id: "shard-1",
        pair_symbols: "BTC-USD" },
        grpc: GrpcConfig { port: 50051 },
        storage: StorageConfig { data_dir: "/data", log_file_segment_size: "128MB" },
        snapshot: SnapshotConfig { interval_seconds: 30, max_commands: 10000 },
        event_bus: EventBusConfig { type: "nats", url: "nats://localhost:4222" },
        engine: EngineConfig { max_inflight_commands: 10000 }
    }
    
fn backup() {
    // --------------------------------------------------
    // 5. Spawn SNAPSHOT thread
    // --------------------------------------------------
    let snapshot_store = SnapshotStore::new("data/snapshots");
    let snapshotter = Snapshotter::new(
        || {
            // ⚠️ aquí deberías capturar estado real del engine
            vec![] // placeholder
        },
        snapshot_store,
        Duration::from_secs(10),
    );

    snapshotter.run();

    // --------------------------------------------------
    // 6. Start gRPC server (producer of commands)
    // --------------------------------------------------
    let grpc = GrpcServer::new(channels.command_tx.clone());

    // Simulación de requests
    loop {
        // aquí iría tonic::Server en producción
        thread::sleep(Duration::from_secs(60));

        // evita que main termine
        let _ = &grpc;
    }
} */