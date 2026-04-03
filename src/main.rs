use std::net::SocketAddr;
use std::str::FromStr;
use std::{sync::Arc, thread};

use dotenvy::dotenv;
use metrics_exporter_prometheus::PrometheusBuilder;
use tokio::runtime::{Builder, Runtime};
use tracing::info;
use tracing_subscriber::fmt::init as tracingInit;

use crate::transport::service::serve;

mod config;
mod core;
mod domain;
mod events;
mod shared;
mod transport;
mod utils;

use crate::config::{grpc_serve_options, load_config};
use crate::core::{
    deduplicator::Deduplicator, engine::Engine, log_writter::LogWriter, sequencer::Sequencer,
};
use crate::domain::matching::MatchingEngine;
use crate::events::publisher::Publisher;
use crate::shared::channel::Channels;
use crate::transport::dispatcher::Dispatcher;

fn main() {
    // --------------------------------------------------
    // Set configuration
    // --------------------------------------------------
    tracingInit();
    dotenv().ok();

    let config = Arc::new(load_config());

    if let Some(ref addr_str) = config.observability.metrics_listen_addr {
        let addr = SocketAddr::from_str(addr_str.trim()).unwrap_or_else(|_| {
            panic!("invalid observability.metrics_listen_addr: {addr_str}");
        });
        PrometheusBuilder::new()
            .with_http_listener(addr)
            .install()
            .expect("install prometheus metrics exporter");
        info!(%addr, "Prometheus metrics endpoint");
    }
    let log_path = format!("{}/wal.log", config.storage.data_dir);
    let segment_size = config.storage.log_file_segment_size.clone();
    let event_bus_config = config.event_bus.clone();
    let grpc_port = config.grpc.port;
    let grpc_serve_opts = grpc_serve_options(&config.grpc);
    let shard_symbol = config.shard.pair_symbols.clone();
    let command_channel_capacity = config.engine.command_channel_capacity;
    if command_channel_capacity == 0 {
        panic!("engine.command_channel_capacity must be greater than 0");
    }

    // print!("Match-core initiated with config: {:?}", config);

    // --------------------------------------------------
    // Create channels to allow communication between threads
    // --------------------------------------------------
    let channels = Channels::new(command_channel_capacity);
    let command_rx = channels.command_rx;
    let command_tx = channels.command_tx.clone();
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
        let runtime = Builder::new_current_thread()
            .enable_all()
            .build()
            .expect("failed to create Tokio runtime for engine");
        runtime.block_on(engine.run());
    });

    // --------------------------------------------------
    // Instantiate gRPC server thread (command ingress)
    // --------------------------------------------------
    let dispatcher = Dispatcher::new(command_tx);
    thread::spawn(move || {
        let runtime = Runtime::new().expect("failed to create Tokio runtime for gRPC server");
        let addr = SocketAddr::from(([0, 0, 0, 0], grpc_port));

        runtime
            .block_on(serve(dispatcher, shard_symbol, addr, grpc_serve_opts))
            .unwrap_or_else(|e| panic!("gRPC server failed: {e}"));
    });

    // --------------------------------------------------
    // Event publisher (NATS connect inside thread so it cannot block or panic main before gRPC binds)
    // --------------------------------------------------
    let publisher = Publisher::new(event_rx, event_bus_config);
    thread::spawn(move || {
        publisher.run();
    });

    // If main returns, the whole process exits and all spawned threads are torn down — keep running.
    std::thread::park();
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
} */
