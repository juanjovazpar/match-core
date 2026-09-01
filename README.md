# MATCH CORE

This project implements a **centralized matching engine**—the core of a trading platform or electronic exchange: it receives orders, maintains an order book, and (when fully implemented) matches compatible bids and asks into trades. **This repository** already provides **durable command/event logging**, **gRPC ingress**, and a **message-bus outbound path**; **full matching and cancel** are still being built toward that goal.

Our implementation is written in **Rust**, leveraging its safety guarantees and concurrency model to deliver **low latency, high throughput, and fault-tolerant order processing**. It is designed as a self-contained service that can run as a microservice in any system.

**This repository is an isolated component of the project [Exchain](https://github.com/juanjovazpar/exchain):**

## Key Features

- **Command ingress (gRPC):** `OrderCmdService` over HTTP/2 (Tonic); optional TLS/mTLS, Prometheus metrics, bounded async hand-off to the engine. See [Transport module](#transport-module-grpc-ingress).
- **Order Handling:** ingress accepts **limit and market** orders over gRPC; the engine stores them in the book and emits **accept** events—**full matching and cancel logic** are still evolving (see [Data structures](#data-structures-current-code)).
- **Order book:** `BTreeMap` price levels with `VecDeque` FIFO per level (`domain/orderbook.rs`).
- **Matching logic:** **roadmap**—the codebase is oriented toward full bid/ask crossing, but **trade execution and cancel handling are not finished** yet (see [Data structures](#data-structures-current-code)).
- **Persistence:** write-ahead style **command and event log** (`LogWriter` / WAL) under `storage.data_dir`; snapshot-related config exists for future recovery workflows.
- **Resilience:** foundation for replay/recovery from persisted commands and events (see core engine + WAL).
- **Real-time events:** the engine emits **`EventEnvelope`** values on a channel; a dedicated **publisher** thread forwards them to a configurable **message bus** (e.g. **NATS** via `event_bus` in `config.yaml`). Downstream services or gateways can fan out to WebSockets, HTTP, etc.
- **Concurrency:** **Tokio** for gRPC ingress and for the engine’s async command loop; **OS threads** separate gRPC, engine, and publisher; command path uses a **bounded** async channel for backpressure.

## Architecture Overview

This matching engine is built with a strong focus on efficiency, parallelism, and safety. To achieve these goals, each engine instance is designed to handle a `single trading pair independently`. This isolation minimizes contingencies between markets and allows the system to scale horizontally.
If bidirectional or cross-pair trading is required (for example, `USD/EUR` and `EUR/USD`), separated instances can be deployed — for instance, as two distinct Docker containers — each dedicated to one direction of the market.

The process uses **several OS threads** with a small, explicit pipeline: ingress, matching, and outbound events do not share a single blocking loop.

- **Ingress (gRPC):** a **Tokio multi-thread** runtime accepts **HTTP/2** connections and the **`OrderCmdService`** RPCs. Validated commands are sent asynchronously on a **bounded `tokio::sync::mpsc`** channel toward the engine (capacity from `engine.command_channel_capacity`). When the queue is full, ingress **awaits** (`send().await`) and exerts **backpressure** on callers instead of growing memory without bound. Details: [Transport module](#transport-module-grpc-ingress).
- **Matching engine core:** runs on its **own thread** with a **Tokio current-thread** runtime. It **`recv().await`s** commands in order, updates the in-memory book, writes the **WAL** (commands and derived events), and pushes **`EventEnvelope`** values to the event channel.
- **Event publisher:** a **third thread** blocks on **`std::sync::mpsc::Receiver`**, serializing events and publishing to the configured **event bus** (e.g. NATS). This path is **synchronous** and unbounded today; the command path is the one sized for load.
- **Persistence:** the engine persists **commands and events** through **`LogWriter`** (file under `storage.data_dir`). Snapshot settings in config anticipate fuller recovery tooling.

The diagram in `./assets/images/architecture-overview.png` may predate the gRPC + NATS layout; treat this section as the **source of truth** for the current binary.

### Data structures (current code)

The in-memory book is **`src/domain/orderbook.rs`**, driven by **`MatchingEngine`** in **`src/domain/matching.rs`**.

```rust
pub struct OrderBook {
    pub bids: BTreeMap<OrderedFloat<f64>, VecDeque<Order>>,
    pub asks: BTreeMap<OrderedFloat<f64>, VecDeque<Order>>,
}
```

- **`BTreeMap<OrderedFloat<f64>, …>`** — sorted **price levels** per side; `OrderedFloat` gives a total order on `f64`.
- **`VecDeque<Order>`** — **FIFO** queue at each price (`push_back`).

**Matching status:** `MatchingEngine::process` is still **simplified**: new orders are stored and an **`OrderAccepted`** event is emitted; **cancel** is a placeholder (no events). There is **no** full price–time priority matching loop in this repository yet—update this README when `domain/matching.rs` / `domain/orderbook.rs` evolve.

| Operation | Notes | Typical complexity |
| --------- | ----- | ------------------ |
| `add_order` | Insert at price level; creates level if missing | O(log P) map step + O(1) deque push |
| (future) match / cancel | To be documented when implemented | — |

**P** = number of distinct price levels on the side being updated.

### Matching flow (diagram)

The diagram below may describe a **target** end-to-end match path. The **current** Rust code only performs **accept + book insert** for new orders until matching and cancel are completed.

![Matching Overview](./assets/images/matching-overview.png)


## Transport module (gRPC ingress)

Order commands enter the process through **gRPC** (HTTP/2), implemented with **Tonic** + **Prost**. The `transport` crate module is responsible for everything from the wire up to handing a validated domain command to the matching engine thread.

### Responsibilities

| Piece | Role |
| ----- | ---- |
| **Proto / codegen** | `proto/orders.proto` is compiled in `build.rs`; generated types live under `transport::proto::pb` (`matchcore.v1`). |
| **`transport::grpc_serve::serve`** | Binds the Tonic server: TLS/plaintext, timeouts, concurrency per connection, request tracing spans, registers the order command service. |
| **`transport::orders_service`** | Tonic implementation of `OrderCmdService` (Submit / Cancel): validation, metrics, dispatch. |
| **`transport::orders_service::validator`** | Validates protobuf requests (enums, numeric fields, ids, shard symbol) before building `shared::command::Command`. |
| **`transport::dispatcher`** | Async `submit`: bounded wait (`grpc.command_enqueue_timeout_ms`) on `tokio::sync::mpsc::send`; `DispatchError::EnqueueTimeout` if the engine queue stays full; `ChannelClosed` if the receiver is gone. |
| **`transport::settings::GrpcServeOptions`** | Transport-owned snapshot of server tuning + TLS paths. **No dependency on `config`.** |
| **`transport::metrics`** | Prometheus-friendly counters for ingress outcomes (see below). |

### End-to-end flow

1. Client calls `Submit` or `Cancel` on `matchcore.v1.OrderCmdService`.
2. **`validator`** checks the request; on failure the RPC still returns **HTTP OK** with `OrderCmdReply` status **rejected** and a `CmdErr` code (application-level rejection, not gRPC `Status` for validation errors).
3. On success, a **`Command`** is built and **`dispatcher.submit(cmd).await`** tries to enqueue for the engine (waits up to **`grpc.command_enqueue_timeout_ms`** for a slot).
4. If the wait expires while the queue stays full, the reply is **rejected** with `CmdErr` **submit/cancel command queue timeout** and metric `rejected_command_queue_timeout`.
5. If the engine command channel is closed, the reply is rejected with an engine-unavailable error code and logged.

### Architectural decisions

- **gRPC instead of REST for commands** — binary, schema-first (`proto`), fits high-throughput internal ingress; clients need the `.proto` or compatible codegen.
- **Transport decoupled from `config`** — `GrpcServeOptions` / `GrpcTlsOptions` are defined in `transport::settings`. The binary maps YAML/env via `config::grpc_bridge::grpc_serve_options(&config.grpc)` so the transport layer does not import `config::types`.
- **Bounded async command queue** — `tokio::sync::mpsc` with capacity from `engine.command_channel_capacity` (default `65536`). **Transport admission**: `dispatcher` wraps `send` in `tokio::time::timeout` (`grpc.command_enqueue_timeout_ms`, default **5s**). A full queue blocks only until either a slot opens or the timeout fires (then RPC rejects to cap ingress latency). Event emission to NATS still uses **`std::sync::mpsc`** on a separate thread; only the **command** path is async Tokio.
- **Two Tokio runtimes** — the matching engine loop runs on a **current-thread** runtime in its own OS thread; gRPC runs on a **multi-thread** runtime. This isolates scheduling: network I/O does not share the engine’s single task queue.
- **Structured errors for dispatch** — `dispatcher::DispatchError` uses **`thiserror`**; ingress uses **`tracing`** with fields (`rpc`, `command_id`, `order_id`, etc.). These are complementary (types vs logs).
- **TLS and mTLS are optional** — omit `grpc.tls` for **plaintext**. With `grpc.tls`, set `cert_path` + `key_path` for server TLS. Add `client_ca_path` to require (or optionally allow) **client certificates** (`client_auth_optional`, default `false` when verifying clients).
- **Server hardening** — per-request timeout, per-connection concurrency limit, max encode/decode message sizes on the service, optional TLS/mTLS (rustls via Tonic’s `tls-ring` feature).
- **Observability** — optional **`observability.metrics_listen_addr`** starts a Prometheus scrape HTTP listener (`metrics` + `metrics-exporter-prometheus`). Counter: `match_core_grpc_requests_total` with labels `rpc` (`submit` \| `cancel`) and `outcome` (`queued`, `rejected_validation`, `rejected_engine_unavailable`, `rejected_command_queue_timeout`). If metrics are disabled, counter macros no-op until a global recorder is installed.

### Layout (source)

```
src/transport/
  mod.rs              # module tree
  proto.rs            # include_proto!("matchcore.v1")
  settings.rs         # GrpcServeOptions, validation
  grpc_serve.rs       # serve(...) — Tonic Server::builder
  dispatcher.rs
  metrics.rs
  orders_service/
    mod.rs            # pub use service::new
    service.rs        # OrderCmdService + tonic server wrapper
    validator.rs      # Validator
```

### Configuration reference (relevant keys)

| Key | Purpose |
| --- | ------- |
| `grpc.port` | Listen port (bind `0.0.0.0`). |
| `grpc.tls` | Optional. `cert_path`, `key_path`; optional `client_ca_path`, `client_auth_optional` for mTLS. |
| `grpc.request_timeout_secs` | Tonic server per-request timeout (default `30`). |
| `grpc.concurrency_limit_per_connection` | Max in-flight requests per HTTP/2 connection (default `256`). |
| `grpc.max_decoding_message_bytes` / `max_encoding_message_bytes` | gRPC message size limits (default 4 MiB). |
| `grpc.command_enqueue_timeout_ms` | Max wait for space in the engine command queue before rejecting the RPC (default `5000`). Must be greater than 0. |
| `engine.command_channel_capacity` | Tokio command channel depth (default `65536`). |
| `observability.metrics_listen_addr` | Optional `host:port` for Prometheus scrape (e.g. `0.0.0.0:9090`). |
| `shard.pair_symbols` | Symbol the shard accepts; must match incoming submit requests. |
| `storage.data_dir` | WAL / log directory for `LogWriter`. |
| `event_bus.type` / `event_bus.url` | Outbound bus (e.g. `nats` + `nats://…`) consumed by the publisher thread. |

Environment overrides use the existing `config` crate convention (e.g. `GRPC__TLS__CERT_PATH`, `EVENT_BUS__URL`, `OBSERVABILITY__METRICS_LISTEN_ADDR` with `__` nesting).


## Development 

### Dependencies

- **tokio:** Async runtime used for the **gRPC server** (multi-thread) and the **engine command loop** (current-thread worker inside a dedicated OS thread).
- **tonic / tonic-prost / prost:** gRPC over HTTP/2; `proto/orders.proto` is compiled in `build.rs`.
- **async-nats:** Client used by the event bus when `event_bus.type` is `nats` (see `src/events/`).
- **ordered-float:** `OrderedFloat<f64>` keys for price levels in `OrderBook`.
- **serde / serde_json:** (De)serialization for config and event payloads as needed.
- **config + dotenvy:** Load `config.yaml` and optional environment overrides (`__` nested keys).
- **tracing / tracing-subscriber:** Structured logs (e.g. ingress, dispatch failures).
- **anyhow:** Ergonomic errors at application boundaries (e.g. server startup).
- **thiserror:** Typed errors such as `transport::dispatcher::DispatchError`.
- **bincode:** Binary serialization for event payloads published to the bus.
- **metrics / metrics-exporter-prometheus:** Optional Prometheus HTTP scrape endpoint (`observability.metrics_listen_addr`).
- **axum / dashmap / uuid / chrono / exchain-commons, etc.:** Declared in `Cargo.toml` for shared utilities or future work; **ingress in `src/` is gRPC (Tonic), not Axum.**

### Commands

#### Install dependencies

````
cargo install cargo-watch
cargo install --path .
````

#### Start development mode

We must use `ssh` to authenticate our build and be able to download private repositories in Github.

First, ensure `ssh-agent` is running:
````
eval "$(ssh-agent -s)"
ssh-add ~/.ssh/{YOUR_SSH_KEY}
````

Then run the container:
````
DOCKER_BUILDKIT=1 docker-compose up --build
````

#### Start testing mode

````
cargo watch -x test
````



