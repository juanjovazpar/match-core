# MATCH CORE

This project implements a **centralized off-chain matching engine**, the fundamental component at the core of any trading platform or electronic exchange. A matching engine is responsible for receiving buy and sell orders from multiple clients, storing them in an order book, and continuously matching compatible orders to generate trades. In practice, this is the same mechanism that powers real-world exchanges such as Binance or Coinbase: every time a trader places a limit or market order, the matching engine determines if there is a counterpart order available and executes the transaction.

Our implementation is written in **Rust**, leveraging its safety guarantees and concurrency model to deliver **low latency, high throughput, and fault-tolerant order processing**. It is designed as a self-contained service that can run locally on your machine, providing developers and students with a realistic environment to study and experiment with how exchanges operate under the hood.

### Key Features

- **Order Handling:** supports limit, market, and cancel orders.
- **Order Book:** efficient in-memory data structures for fast lookups and matching.
- **Matching Logic:** automatically executes trades when bid/ask prices overlap.
- **Persistence:** snapshots of the order book can be stored and restored from disk.
- **Resilience:** recovery after restarts or crashes.
- **Real-Time-Updates:** clients receive live notifications of book updates and trades via WebSocket.
- **Concurrency:** built with Rust’s async runtime to handle multiple clients safely and efficiently.
- **Metrics:** track trades, active orders, and basic performance stats.

## Why This Project?

The matching engine is the heart of any exchange (like Binance or Coinbase).
Building one from scratch demonstrates:

- Efficient data structures
- Concurrency and low latency
- Fault tolerance and durability
- Real-time system design

## Architecture Overview

- **Server Layer:** WebSocket server for client connections.
- **Matching Engine Core:** manages the order book and executes trades.
- **Event System:** broadcasts executions and updates to clients.
- **Persistence Layer:** stores snapshots of the order book for recovery.
- **Metrics:** collects runtime statistics.

### Dependencies
- **axum:** Web framework for building HTTP servers and WebSocket endpoints; handles routing, extractors, and middleware.
- **tokio:** Asynchronous runtime for handling multiple concurrent tasks efficiently, including WebSocket connections.
- **serde:** Serialization/deserialization library; allows converting Rust structs to JSON and back.
- **serde_json:** JSON support for serde, enabling sending/receiving JSON messages over WebSocket.
- **dashmap:** Thread-safe concurrent hashmap, useful for managing connected clients without locking the entire structure.
- **tracing:** Structured logging library to track events, useful for debugging and monitoring.
- **tracing-subscriber:** Subscriber implementation for tracing to collect and format logs.
- **anyhow:** Simple error handling library for Rust; allows returning and propagating errors easily.
- **bincode:** Efficient binary serialization library, useful for saving/loading snapshots of the order book quickly.

## Development 
### Start development mode

````
cargo watch -x run
````

### Install dependencies

````
cargo install cargo-watch
cargo install --path .
````

